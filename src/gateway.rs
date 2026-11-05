//! Unified RPC Gateway (`lcserver:8002` + `phpproxy` + Private Method Access Control).
//!
//! Ground-Truth References:
//! - `legacy-ref/lcserver/conf/game20002.xml` (`L39-L100`):
//!   - Binds `0.0.0.0:8002` (fixing the legacy hardcoded `192.168.1.234`), Flash policy XML on `:843`,
//!     and frames every packet with `FrameHeader` (8 bytes) + optional 16-byte MD5 XOR checksum.
//! - `legacy-ref/rpcfw/lib/RPCFramework.class.php` (`L348-L404`, `L783-L865`) & `Framework.cfg.php` (`L203-L273`):
//!   - Enforces the 4-tier method access guard (`Private`, `ExcludeConnect`, `ExcludeLogin`, `Public`).
//!   - Permanently blocks unauthenticated `battle.test` (`Battle.class.php:25-32` DoS vector).

use crate::amf3::{Amf3Value, amf3_decode, amf3_encode};
use crate::crypt::phpproxy_checksum;
use crate::protocol::{FLAG_ENCRYPTED, FrameHeader, HEAD_SIZE, MESS_CODE_PRIMARY};

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
            | "ban"
            | "getBanInfo",
        )
        | ("timer", "execute")
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
        ("user", "userLogin" | "getUsers" | "createUser" | "cancelDel" | "getRandomName") => {
            Some(MethodAccessTier::ExcludeLogin)
        }
        // Public gameplay domains (`Framework.cfg.php:279-411`)
        (
            "user" | "hero" | "bag" | "forge" | "copy" | "elitecopy" | "herocopy" | "abysscopy"
            | "arena" | "boss" | "guild" | "town" | "city" | "port" | "pet" | "copet"
            | "sailboat" | "kitchen" | "talks" | "train" | "practice" | "task" | "active"
            | "reward" | "mail" | "chat" | "formation" | "captain" | "explore" | "smelting"
            | "crystal" | "soul" | "treasure" | "allblue" | "impeldown" | "haki" | "astrolabe",
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

/// Decode a framed packet and verify its 16-byte MD5 XOR checksum if `FLAG_ENCRYPTED` is set.
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
        let computed_sig = phpproxy_checksum(body, MESS_CODE_PRIMARY);
        if expected_sig != computed_sig {
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

#[cfg(test)]
mod tests {
    use super::*;

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

        // Tampering with 1 byte of payload must trigger MD5 XOR checksum rejection
        let last = packet.len() - 1;
        packet[last] ^= 0xFF;
        assert_eq!(
            decode_framed_packet(&packet),
            Err("invalid PHPProxy MD5 XOR checksum")
        );
    }
}
