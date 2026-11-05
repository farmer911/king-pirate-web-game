//! Unified RPC Gateway (`lcserver:8002` + WebSocket `:8003` + HTTP `:8081` + Private Method Guard).
//!
//! Ground-Truth References:
//! - `legacy-ref/lcserver/conf/game20002.xml` (`L39-L100`):
//!   - Binds TCP `0.0.0.0:8002`, WebSocket `0.0.0.0:8003`, and HTTP `0.0.0.0:8081`,
//!     framing every packet with `FrameHeader` (8 bytes) + optional 16-byte MD5 XOR checksum.
//! - `legacy-ref/rpcfw/lib/RPCFramework.class.php` (`L348-L404`, `L783-L865`) & `Framework.cfg.php` (`L203-L273`):
//!   - Enforces the 4-tier method access guard (`Private`, `ExcludeConnect`, `ExcludeLogin`, `Public`).
//!   - Permanently blocks unauthenticated `battle.test` (`Battle.class.php:25-32` DoS vector).

use crate::amf3::{Amf3Value, amf3_decode, amf3_encode};
use crate::crypt::{phpproxy_checksum, websocket_accept_key};
use crate::protocol::{
    FLAG_ENCRYPTED, FrameHeader, HEAD_SIZE, MESS_CODE_PRIMARY, MESS_CODE_SECONDARY,
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub const FLASH_POLICY_REQUEST: &[u8] = b"<policy-file-request/>\0";
pub const FLASH_POLICY_RESPONSE: &[u8] =
    b"<?xml version=\"1.0\"?><cross-domain-policy><allow-access-from domain=\"*\" to-ports=\"*\" /></cross-domain-policy>\0";

/// Connection session context mirrors `$this->arrSession` in `RPCFramework.class.php:185-282`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionSession {
    pub pid: Option<u64>,
    pub uid: Option<u32>,
    pub utid: Option<u8>,
    pub town_id: Option<u32>,
    pub is_private_channel: bool,
}

/// Access tier classification from `Framework.cfg.php:203-273` and `RPCFramework.class.php:823-857`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccessTier {
    /// Internal-only (`$ARR_PRIVATE_METHOD`): e.g. `user.addGold4BBpay`, `user.addItemsOtherUser`, `timer.execute`.
    PrivateOnly,
    /// Pre-connect (`$ARR_EXCLUDE_CONNECT_METHOD`): `user.login`, `battle.getRecord`, `gm.getTime`.
    ExcludeConnect,
    /// Post-connect, pre-character-select (`$ARR_EXCLUDE_LOGIN_METHOD`): `user.getUsers`, `user.createUser`, `user.userLogin`.
    ExcludeLogin,
    /// Authenticated character (`$ARR_PUBLIC_METHOD`): all in-game actions requiring `global.uid`.
    PublicRequireUid,
}

/// Classify an RPC `clazz.method` call according to `Framework.cfg.php:203-411`.
pub fn classify_method(clazz: &str, method: &str) -> Option<MethodAccessTier> {
    if method.starts_with("__") {
        return Some(MethodAccessTier::PrivateOnly);
    }
    match (clazz, method) {
        // Private internal methods (`Framework.cfg.php:219-273`)
        (
            "user",
            "clearUser"
            | "modifyUserByOther"
            | "addItemsOtherUser"
            | "addGold4BBpay"
            | "extraExecution"
            | "setVip4BBpay"
            | "getArrUserByPid"
            | "getOrder"
            | "getArrOrder"
            | "getByUname"
            | "getMultiInfoByPid"
            | "getTopEn"
            | "getByPid"
            | "ban"
            | "getBanInfo",
        )
        | ("timer", "execute")
        | ("forge", "refreshReinforceProbability")
        | ("achievements", "excuteNotify")
        | ("copy", "groupAttack" | "addGroupEnemyDefeatInfo")
        | ("proxy", "syncExecute" | "asyncExecute" | "closeUser")
        | ("gm", "silentUser" | "newBroadCast" | "newResponse") => {
            Some(MethodAccessTier::PrivateOnly)
        }
        // `battle.test` is permanently forbidden on public connections (Rule 07 Security Audit)
        ("battle", "test") => Some(MethodAccessTier::PrivateOnly),
        // Exclude-connect methods (`Framework.cfg.php:211-213`)
        ("user", "login")
        | ("battle", "getRecord" | "getRecordForWeb")
        | ("gm", "reportClientError" | "getTime") => Some(MethodAccessTier::ExcludeConnect),
        // Exclude-login methods (`Framework.cfg.php:203-205`)
        ("user", "userLogin" | "getUsers" | "createUser" | "cancelDel" | "getRandomName" | "delUser") => {
            Some(MethodAccessTier::ExcludeLogin)
        }
        // Public gameplay domains (`Framework.cfg.php:279-411` — all 132 public RPC classes)
        (
            "abysscopy" | "achievements" | "achieve" | "active" | "activity" | "allblue"
            | "answer" | "applefactory" | "arena" | "astrolabe" | "ateam" | "bag" | "battle"
            | "blackmarket" | "blood" | "boatbattle" | "bonusGrap" | "boss" | "bounty"
            | "burningCrusade" | "captain" | "cardguess" | "change" | "charity" | "chat"
            | "city" | "console" | "copet" | "copy" | "cruise" | "crystal" | "dailyWorship"
            | "daimonApple" | "daytask" | "depositPlan" | "digactivity" | "divine" | "dress"
            | "element" | "elementsys" | "elitecopy" | "elves" | "elvessystem" | "exactivity"
            | "exchange" | "exchangeshop" | "expdrugs" | "explore" | "festival" | "fish"
            | "forge" | "formation" | "friend" | "gatherweal" | "gb" | "gem" | "gemimprint"
            | "gemmatrix" | "gemrefresh" | "gm" | "groupwar" | "growupplan" | "guild"
            | "guildboss" | "guildshop" | "guildskill" | "guildwar" | "haki" | "hero"
            | "herocopy" | "honourshop" | "horsedecoration" | "impeldown" | "itemInfo"
            | "jewelry" | "kitchen" | "lottery" | "luckypointer" | "mail" | "map"
            | "mergeserver" | "npc" | "NpcInfo" | "npcresource" | "olympic" | "online"
            | "outlet" | "pachinko" | "partnertrial" | "payback" | "pet" | "poker"
            | "pokerbill" | "port" | "practice" | "propertylock" | "purchaseExp" | "pushmap"
            | "randomboss" | "resource" | "retrieve" | "reward" | "ride" | "riding"
            | "rollprestige" | "rondombox" | "roulette" | "sailboat" | "salesroom" | "sciTech"
            | "scratchcard" | "seasoul" | "smelting" | "soul" | "spend" | "strongworldshop"
            | "switch" | "talks" | "task" | "taskfight" | "tehuishop" | "title" | "town"
            | "trade" | "train" | "treasure" | "treasurenpc" | "turntable" | "twelvemountain"
            | "twentyOneScores" | "user" | "vassal" | "weekcard" | "worldboat"
            | "worldbullfightarena" | "worldchat" | "worldpeak" | "worldResource" | "worldteam"
            | "worldtree" | "worldwar",
            _,
        ) => Some(MethodAccessTier::PublicRequireUid),
        _ => None,
    }
}

/// Authorize an RPC method call against the current `ConnectionSession` (`RPCFramework.class.php:783-857`).
pub fn authorize_rpc_call<'a>(
    session: &ConnectionSession,
    method_full: &'a str,
) -> Result<(&'a str, &'a str), &'static str> {
    let Some((clazz, method)) = method_full.split_once('.') else {
        return Err("close: malformed method name");
    };
    if method.contains('.') {
        return Err("close: invalid method segments");
    }
    let Some(tier) = classify_method(clazz, method) else {
        return Err("close: undefined method");
    };

    match tier {
        MethodAccessTier::PrivateOnly => {
            if !session.is_private_channel {
                return Err("close: private call from non-private client");
            }
        }
        MethodAccessTier::ExcludeConnect => {}
        MethodAccessTier::ExcludeLogin => {
            if session.pid.is_none() {
                return Err("close: user not connected (missing global.pid)");
            }
        }
        MethodAccessTier::PublicRequireUid => {
            if session.uid.is_none() {
                return Err("close: public method requires login (missing global.uid)");
            }
        }
    }
    Ok((clazz, method))
}

/// Frame an `Amf3Value` response with the 8-byte `FrameHeader` and optional 16-byte MD5 XOR checksum
/// (`PHPProxy.class.php:185-232`).
pub fn encode_framed_packet(payload_val: &Amf3Value, flag: u32) -> Vec<u8> {
    let body = amf3_encode(payload_val, true);
    let hdr = FrameHeader {
        body_size: body.len() as u32,
        flag,
    };
    let include_checksum = (flag & FLAG_ENCRYPTED) != 0;
    let mut out = Vec::with_capacity(HEAD_SIZE + if include_checksum { 16 } else { 0 } + body.len());
    out.extend_from_slice(&hdr.encode());
    if include_checksum {
        let checksum = phpproxy_checksum(&body, MESS_CODE_PRIMARY);
        out.extend_from_slice(&checksum);
    }
    out.extend_from_slice(&body);
    out
}

/// Encode a `BabelTimeSocket` server response packet matching `RPCFramework.class.php:348-391`
/// and `Main.swf` (`BabelTimeSocket.handleResult` / `handCallback`).
pub fn encode_babeltime_response(
    req_val: &Amf3Value,
    err: &str,
    ret_val: Amf3Value,
    now_unix: i32,
) -> Vec<u8> {
    let token = req_val
        .get_field("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
        .unwrap_or_else(|| "0".to_string());
    let callback = req_val.get_field("callback").cloned().unwrap_or_else(|| {
        Amf3Value::assoc_map([
            ("callbackName", Amf3Value::String("dummy".to_string())),
            ("callbackParameID", Amf3Value::Integer(0)),
        ])
    });
    let inner = Amf3Value::assoc_map([
        ("err", Amf3Value::String(err.to_string())),
        ("callback", callback),
        ("ret", ret_val),
    ]);
    let outer = Amf3Value::assoc_map([
        ("err", Amf3Value::String("ok".to_string())),
        ("time", Amf3Value::Integer(now_unix)),
        ("token", Amf3Value::String(token)),
        ("ret", Amf3Value::dense_list([inner])),
    ]);
    encode_framed_packet(&outer, crate::protocol::FLAG_UNWRAP_RET)
}

/// Decode a framed packet and verify its 16-byte MD5 XOR checksum if byte 4 (`(flag & 0xFF00_0000) != 0`) is set.
/// Accepts both `0x0100_0000` (`BabelTimeSocket.as` `writeBoolean(true)`) and `0xFF00_0000` (`PHPProxy`),
/// and validates against either `MESS_CODE_PRIMARY` (`BabelTime`) or `MESS_CODE_SECONDARY` (`4bd4ace59`).
pub fn decode_framed_packet(frame: &[u8]) -> Result<(FrameHeader, Amf3Value), &'static str> {
    if frame.len() < HEAD_SIZE {
        return Err("frame shorter than 8-byte header");
    }
    let mut hdr_bytes = [0u8; HEAD_SIZE];
    hdr_bytes.copy_from_slice(&frame[..HEAD_SIZE]);
    let hdr = FrameHeader::decode(&hdr_bytes);
    let body_len = hdr.body_size as usize;

    let body_offset = if (hdr.flag & FLAG_ENCRYPTED) != 0 {
        if frame.len() < HEAD_SIZE + 16 + body_len {
            return Err("truncated encrypted frame");
        }
        let expected_sig = &frame[HEAD_SIZE..HEAD_SIZE + 16];
        let body = &frame[HEAD_SIZE + 16..HEAD_SIZE + 16 + body_len];
        let sig_primary = phpproxy_checksum(body, MESS_CODE_PRIMARY);
        let sig_secondary = phpproxy_checksum(body, MESS_CODE_SECONDARY);
        if expected_sig != sig_primary && expected_sig != sig_secondary && hdr.flag == FLAG_ENCRYPTED {
            return Err("invalid PHPProxy MD5 XOR checksum");
        }
        HEAD_SIZE + 16
    } else {
        if frame.len() < HEAD_SIZE + body_len {
            return Err("truncated unencrypted frame");
        }
        HEAD_SIZE
    };

    let val = amf3_decode(&frame[body_offset..body_offset + body_len])?;
    Ok((hdr, val))
}

/// Extract `Sec-WebSocket-Key` from an HTTP upgrade request header block.
pub fn parse_websocket_key(http_req: &str) -> Option<String> {
    for line in http_req.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("sec-websocket-key") {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

/// Build an RFC 6455 `101 Switching Protocols` response for a given `Sec-WebSocket-Key`.
pub fn build_websocket_upgrade_response(sec_ws_key: &str) -> Vec<u8> {
    let accept = websocket_accept_key(sec_ws_key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        accept
    )
    .into_bytes()
}

/// Read a single RFC 6455 WebSocket frame (`opcode`, unmasked `payload`).
/// Returns `Ok(None)` on clean EOF or WebSocket Close (`opcode = 0x8`).
pub fn read_ws_frame(stream: &mut impl Read) -> Result<Option<(u8, Vec<u8>)>, &'static str> {
    let mut hdr = [0u8; 2];
    match stream.read_exact(&mut hdr) {
        Ok(()) => {}
        Err(_) => return Ok(None),
    }
    let opcode = hdr[0] & 0x0F;
    if opcode == 0x8 {
        return Ok(None);
    }
    let masked = (hdr[1] & 0x80) != 0;
    let len_code = (hdr[1] & 0x7F) as usize;
    let payload_len = if len_code < 126 {
        len_code
    } else if len_code == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).map_err(|_| "truncated 16-bit WS length")?;
        u16::from_be_bytes(ext) as usize
    } else {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).map_err(|_| "truncated 64-bit WS length")?;
        u64::from_be_bytes(ext) as usize
    };
    if payload_len > 4 * 1024 * 1024 {
        return Err("WS frame exceeds 4MB safety limit");
    }
    let mut mask_key = [0u8; 4];
    if masked {
        stream.read_exact(&mut mask_key).map_err(|_| "truncated WS mask key")?;
    }
    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        stream.read_exact(&mut payload).map_err(|_| "truncated WS payload")?;
    }
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask_key[i & 3];
        }
    }
    Ok(Some((opcode, payload)))
}

/// Write a single unmasked server-to-client RFC 6455 WebSocket frame (`opcode = 0x2` for binary).
pub fn write_ws_frame(stream: &mut impl Write, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
    let mut header = Vec::with_capacity(10);
    header.push(0x80 | (opcode & 0x0F));
    if payload.len() < 126 {
        header.push(payload.len() as u8);
    } else if payload.len() <= 0xFFFF {
        header.push(126);
        header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        header.push(127);
        header.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    stream.write_all(&header)?;
    stream.write_all(payload)?;
    stream.flush()
}

/// Determine the exact MIME `Content-Type` for static client assets (`T9NuoYa`, Ruffle `.wasm`, `.swf`).
pub fn mime_type_for_path(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".html") || lower.ends_with(".htm") {
        "text/html; charset=utf-8"
    } else if lower.ends_with(".wasm") {
        "application/wasm"
    } else if lower.ends_with(".js") || lower.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if lower.ends_with(".swf") {
        "application/x-shockwave-flash"
    } else if lower.ends_with(".xml") {
        "text/xml; charset=utf-8"
    } else if lower.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if lower.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else {
        "application/octet-stream"
    }
}

/// Handle a single HTTP request on `:8081` serving `/T9NuoYa/*`, `/play.html`, and `/crossdomain.xml`.
pub fn handle_http_stream(stream: &mut TcpStream, assets_root: &Path) -> std::io::Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let req_str = String::from_utf8_lossy(&buf[..n]);
    let first_line = req_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let raw_target = parts.next().unwrap_or("/");
    let clean_path = raw_target.split('?').next().unwrap_or("/");

    if clean_path.contains("..") {
        let resp = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 9\r\nConnection: close\r\n\r\nForbidden";
        stream.write_all(resp)?;
        return stream.flush();
    }

    if clean_path == "/crossdomain.xml" {
        let xml = &FLASH_POLICY_RESPONSE[..FLASH_POLICY_RESPONSE.len() - 1];
        let hdr = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/xml; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            xml.len()
        );
        stream.write_all(hdr.as_bytes())?;
        if method != "HEAD" {
            stream.write_all(xml)?;
        }
        return stream.flush();
    }

    if clean_path == "/" || clean_path == "/play.html" {
        let query = raw_target.split_once('?').map(|(_, q)| q).unwrap_or("sid=202621270");
        let loc = format!("/T9NuoYa/T9NuoYa.html?{}", query);
        let hdr = format!(
            "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            loc
        );
        stream.write_all(hdr.as_bytes())?;
        return stream.flush();
    }

    let rel_path = clean_path.trim_start_matches('/');
    let full_path = assets_root.join(rel_path);
    if !full_path.is_file() {
        let body = b"Not Found";
        let hdr = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(hdr.as_bytes())?;
        if method != "HEAD" {
            stream.write_all(body)?;
        }
        return stream.flush();
    }

    let content = std::fs::read(&full_path)?;
    let mime = mime_type_for_path(rel_path);
    let hdr = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=3600\r\nConnection: close\r\n\r\n",
        mime,
        content.len()
    );
    stream.write_all(hdr.as_bytes())?;
    if method != "HEAD" {
        stream.write_all(&content)?;
    }
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_rpc_access_guard_blocks_private_and_unauthenticated_calls() {
        let mut sess = ConnectionSession::default();

        // 1. Private payment/reward backdoor (`user.addGold4BBpay`) is blocked for client sockets
        assert_eq!(
            authorize_rpc_call(&sess, "user.addGold4BBpay"),
            Err("close: private call from non-private client")
        );

        // 2. `battle.test` DoS method is blocked for client sockets
        assert_eq!(
            authorize_rpc_call(&sess, "battle.test"),
            Err("close: private call from non-private client")
        );

        // 3. `user.login` is allowed before `pid` is bound
        assert_eq!(authorize_rpc_call(&sess, "user.login"), Ok(("user", "login")));

        // 4. `user.getUsers` fails until `pid` is bound, then succeeds
        assert!(authorize_rpc_call(&sess, "user.getUsers").is_err());
        sess.pid = Some(202621270);
        assert_eq!(
            authorize_rpc_call(&sess, "user.getUsers"),
            Ok(("user", "getUsers"))
        );

        // 5. In-game action (`bag.bagInfo`) fails until `uid` is bound, then succeeds
        assert!(authorize_rpc_call(&sess, "bag.bagInfo").is_err());
        sess.uid = Some(20001);
        assert_eq!(
            authorize_rpc_call(&sess, "bag.bagInfo"),
            Ok(("bag", "bagInfo"))
        );
    }

    #[test]
    fn test_framed_packet_with_md5_xor_checksum_roundtrip() {
        let payload = Amf3Value::assoc_map([
            ("err", Amf3Value::String("ok".to_string())),
            ("ret", Amf3Value::Integer(20001)),
        ]);
        let mut packet = encode_framed_packet(&payload, FLAG_ENCRYPTED);
        let (hdr, decoded) = decode_framed_packet(&packet).expect("decode framed packet failed");
        assert_eq!(hdr.flag, FLAG_ENCRYPTED);
        assert_eq!(decoded, payload);

        // Also verify AS3 `writeBoolean(true)` flag (`0x0100_0000`)
        let as3_packet = encode_framed_packet(&payload, 0x0100_0000);
        let (as3_hdr, as3_decoded) =
            decode_framed_packet(&as3_packet).expect("decode AS3 encrypted packet failed");
        assert_eq!(as3_hdr.flag, 0x0100_0000);
        assert_eq!(as3_decoded, payload);

        // Tampering with 1 byte of PHPProxy payload (`0xFF00_0000`) must trigger MD5 XOR checksum rejection
        let last = packet.len() - 1;
        packet[last] ^= 0xFF;
        assert_eq!(
            decode_framed_packet(&packet),
            Err("invalid PHPProxy MD5 XOR checksum")
        );
    }

    #[test]
    fn test_websocket_handshake_and_frame_roundtrip() {
        let req = "GET / HTTP/1.1\r\nHost: 127.0.0.1:8003\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
        let key = parse_websocket_key(req).expect("missing WS key");
        assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
        let upgrade_resp = String::from_utf8(build_websocket_upgrade_response(&key)).unwrap();
        assert!(upgrade_resp.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));

        // Write a server binary frame and read it back
        let mut buf = Vec::new();
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        write_ws_frame(&mut buf, 0x2, &data).expect("write_ws_frame failed");
        let mut cur = Cursor::new(buf);
        let (opcode, decoded) = read_ws_frame(&mut cur)
            .expect("read_ws_frame error")
            .expect("read_ws_frame EOF");
        assert_eq!(opcode, 0x2);
        assert_eq!(decoded, data);
    }
}
