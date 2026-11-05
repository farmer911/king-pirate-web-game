//! Cryptographic primitives (`BabelCrypt` DES-CBC, RFC 1321 MD5, and `PlayHashKey` Auth Guard).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/lib/BabelCrypt.class.php` (`L18-L66`):
//!   - `encryptNumber($pid)` / `decryptNumber($hex)`: packs `$pid` into big-endian minimal bytes,
//!     encrypts/decrypts via OpenSSL `des-cbc` (`key = "BabelTim"`, `iv = "32210967"`, PKCS#7),
//!     and hex-encodes the 8-byte ciphertext block.
//! - `legacy-ref/rpcfw/conf/BabelCrypt.cfg.php` (`L18-L29`):
//!   - `PlayHashKey = "2012#B@belPir@te#0410"`.
//! - `legacy-ref/rpcfw/module/user/UserLogic.class.php` (`L23-L82`) & `User.class.php` (`L47-L64`):
//!   - Hardens `user.login` by permanently removing the `FrameworkConfig::DEBUG = true` bypass.
//! - `legacy-ref/rpcfw/lib/PHPProxy.class.php` (`L363-L381`):
//!   - `PHPProxy::encrypt($data)`: computes raw 16-byte MD5 over `build_xor_checksum_buffer`.

use crate::protocol::{
    BABEL_CRYPT_IV, BABEL_CRYPT_KEY, PLAY_HASH_KEY, build_xor_checksum_buffer, pack_pid_bytes,
    unpack_pid_bytes,
};
use std::collections::BTreeMap;

/// Compute a 16-byte raw RFC 1321 MD5 digest (`md5($data, true)` in PHP).
pub fn md5_digest(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let bit_len = (data.len() as u64) * 8;
    let mut msg = Vec::with_capacity((data.len() + 72) & !63);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            m[i] = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | (!b & d), i),
                16..=31 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let temp = d;
            d = c;
            c = b;
            let sum = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]);
            b = b.wrapping_add(sum.rotate_left(S[i]));
            a = temp;
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

/// Compute lowercase 32-char hex MD5 digest (`md5($data, false)` in PHP).
pub fn md5_hex(data: &[u8]) -> String {
    hex_encode(&md5_digest(data))
}

/// Compute the 16-byte `PHPProxy::encrypt($data)` checksum (`PHPProxy.class.php:363-381`).
pub fn phpproxy_checksum(payload: &[u8], mess_code: &[u8]) -> [u8; 16] {
    let xor_buf = build_xor_checksum_buffer(payload, mess_code);
    md5_digest(&xor_buf)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>, &'static str> {
    if hex.len() % 2 != 0 {
        return Err("odd hex string length");
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = from_hex_digit(bytes[i])?;
        let lo = from_hex_digit(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn from_hex_digit(b: u8) -> Result<u8, &'static str> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err("invalid hex character"),
    }
}

// ---------------------------------------------------------------------------
// FIPS 46-3 DES-CBC implementation for `BabelCrypt::encryptNumber/decryptNumber`
// ---------------------------------------------------------------------------

const IP: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2, 60, 52, 44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6,
    64, 56, 48, 40, 32, 24, 16, 8, 57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3,
    61, 53, 45, 37, 29, 21, 13, 5, 63, 55, 47, 39, 31, 23, 15, 7,
];

const FP: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32, 39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29, 36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26, 33, 1, 41, 9, 49, 17, 57, 25,
];

const PC1: [u8; 56] = [
    57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35, 27, 19, 11, 3,
    60, 52, 44, 36, 63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37,
    29, 21, 13, 5, 28, 20, 12, 4,
];

const PC2: [u8; 48] = [
    14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10, 23, 19, 12, 4, 26, 8, 16, 7, 27, 20, 13, 2, 41, 52,
    31, 37, 47, 55, 30, 40, 51, 45, 33, 48, 44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
];

const E_TABLE: [u8; 48] = [
    32, 1, 2, 3, 4, 5, 4, 5, 6, 7, 8, 9, 8, 9, 10, 11, 12, 13, 12, 13, 14, 15, 16, 17, 16, 17, 18,
    19, 20, 21, 20, 21, 22, 23, 24, 25, 24, 25, 26, 27, 28, 29, 28, 29, 30, 31, 32, 1,
];

const P_TABLE: [u8; 32] = [
    16, 7, 20, 21, 29, 12, 28, 17, 1, 15, 23, 26, 5, 18, 31, 10, 2, 8, 24, 14, 32, 27, 3, 9, 19,
    13, 30, 6, 22, 11, 4, 25,
];

const S_BOXES: [[[u8; 16]; 4]; 8] = [
    [
        [14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7],
        [0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8],
        [4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0],
        [15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13],
    ],
    [
        [15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10],
        [3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5],
        [0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15],
        [13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9],
    ],
    [
        [10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8],
        [13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1],
        [13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7],
        [1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12],
    ],
    [
        [7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15],
        [13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9],
        [10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4],
        [3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14],
    ],
    [
        [2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9],
        [14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6],
        [4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14],
        [11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3],
    ],
    [
        [12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11],
        [10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8],
        [9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6],
        [4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13],
    ],
    [
        [4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1],
        [13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6],
        [1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2],
        [6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12],
    ],
    [
        [13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7],
        [1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2],
        [7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8],
        [2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11],
    ],
];

const SHIFTS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

fn permute(src: u64, table: &[u8], in_bits: u8) -> u64 {
    let mut out = 0u64;
    for (i, &pos) in table.iter().enumerate() {
        let bit = (src >> (in_bits - pos)) & 1;
        out |= bit << (table.len() - 1 - i);
    }
    out
}

fn des_subkeys(key_bytes: &[u8; 8]) -> [u64; 16] {
    let key = u64::from_be_bytes(*key_bytes);
    let perm56 = permute(key, &PC1, 64);
    let mut c = ((perm56 >> 28) & 0x0FFF_FFFF) as u32;
    let mut d = (perm56 & 0x0FFF_FFFF) as u32;
    let mut subkeys = [0u64; 16];
    for i in 0..16 {
        let s = SHIFTS[i];
        c = ((c << s) | (c >> (28 - s))) & 0x0FFF_FFFF;
        d = ((d << s) | (d >> (28 - s))) & 0x0FFF_FFFF;
        let cd = ((c as u64) << 28) | (d as u64);
        subkeys[i] = permute(cd, &PC2, 56);
    }
    subkeys
}

fn des_block(block: u64, subkeys: &[u64; 16], decrypt: bool) -> u64 {
    let ip = permute(block, &IP, 64);
    let mut left = ((ip >> 32) & 0xFFFF_FFFF) as u32;
    let mut right = (ip & 0xFFFF_FFFF) as u32;

    for round in 0..16 {
        let k = if decrypt {
            subkeys[15 - round]
        } else {
            subkeys[round]
        };
        let expanded = permute(right as u64, &E_TABLE, 32) ^ k;
        let mut s_out = 0u32;
        for s_idx in 0..8 {
            let six = ((expanded >> (42 - s_idx * 6)) & 0x3F) as u8;
            let row = (((six & 0x20) >> 4) | (six & 0x01)) as usize;
            let col = ((six >> 1) & 0x0F) as usize;
            s_out = (s_out << 4) | (S_BOXES[s_idx][row][col] as u32);
        }
        let f = permute(s_out as u64, &P_TABLE, 32) as u32;
        let next_right = left ^ f;
        left = right;
        right = next_right;
    }

    let pre_fp = ((right as u64) << 32) | (left as u64);
    permute(pre_fp, &FP, 64)
}

/// Encrypt a player ID (`pid`) into a 16-char lowercase hex string, matching
/// `BabelCrypt::encryptNumber($pid)` (`legacy-ref/rpcfw/lib/BabelCrypt.class.php:21-34`).
pub fn babel_encrypt_number(pid: u64) -> String {
    let raw = pack_pid_bytes(pid);
    let subkeys = des_subkeys(BABEL_CRYPT_KEY);
    let mut prev = u64::from_be_bytes(*BABEL_CRYPT_IV);

    // PKCS#7 padding to 8-byte DES block boundary
    let pad_len = 8 - (raw.len() % 8);
    let mut padded = Vec::with_capacity(raw.len() + pad_len);
    padded.extend_from_slice(&raw);
    padded.resize(raw.len() + pad_len, pad_len as u8);

    let mut out = Vec::with_capacity(padded.len());
    for chunk in padded.chunks_exact(8) {
        let mut blk = [0u8; 8];
        blk.copy_from_slice(chunk);
        let plain = u64::from_be_bytes(blk) ^ prev;
        let cipher = des_block(plain, &subkeys, false);
        prev = cipher;
        out.extend_from_slice(&cipher.to_be_bytes());
    }
    hex_encode(&out)
}

/// Decrypt a hex-encoded `pid` string back into `u64`, matching
/// `BabelCrypt::decryptNumber($data)` (`legacy-ref/rpcfw/lib/BabelCrypt.class.php:36-53`).
pub fn babel_decrypt_number(hex_cipher: &str) -> Result<u64, &'static str> {
    let cipher_bytes = hex_decode(hex_cipher)?;
    if cipher_bytes.is_empty() || (cipher_bytes.len() % 8) != 0 {
        return Err("invalid DES ciphertext length");
    }
    let subkeys = des_subkeys(BABEL_CRYPT_KEY);
    let mut prev = u64::from_be_bytes(*BABEL_CRYPT_IV);
    let mut plain = Vec::with_capacity(cipher_bytes.len());

    for chunk in cipher_bytes.chunks_exact(8) {
        let mut blk = [0u8; 8];
        blk.copy_from_slice(chunk);
        let c = u64::from_be_bytes(blk);
        let p = des_block(c, &subkeys, true) ^ prev;
        prev = c;
        plain.extend_from_slice(&p.to_be_bytes());
    }

    let pad = *plain.last().ok_or("empty decrypted block")? as usize;
    if pad == 0 || pad > 8 || pad > plain.len() {
        return Err("invalid PKCS#7 padding in BabelCrypt");
    }
    for &b in &plain[plain.len() - pad..] {
        if (b as usize) != pad {
            return Err("corrupted PKCS#7 padding bytes in BabelCrypt");
        }
    }
    plain.truncate(plain.len() - pad);
    Ok(unpack_pid_bytes(&plain))
}

/// Compute the expected `PlayHashKey` MD5 login signature (`UserLogic.class.php:27-37`).
///
/// Sorts all request parameters alphabetically (excluding `"hash"`), concatenates `key + val`,
/// appends `PLAY_HASH_KEY` (`"2012#B@belPir@te#0410"`), and returns the 32-char MD5 hex string.
pub fn compute_login_play_hash(params: &BTreeMap<String, String>) -> String {
    let mut buf = String::with_capacity(128);
    for (k, v) in params {
        if k != "hash" {
            buf.push_str(k);
            buf.push_str(v);
        }
    }
    buf.push_str(PLAY_HASH_KEY);
    md5_hex(buf.as_bytes())
}

/// Strictly verify a `user.login` request (`UserLogic.class.php:23-82` & `User.class.php:47-82`).
///
/// Permanently eliminates the `FrameworkConfig::DEBUG = true` account-takeover vulnerability
/// where an attacker could pass an unverified `pid` without a valid `PlayHashKey` signature.
pub fn verify_login_request(
    params: &BTreeMap<String, String>,
    expected_open_ymd: &str,
    now_unix: u64,
    max_skew_secs: u64,
    min_retained_pid: u64,
) -> Result<(u64, u32), &'static str> {
    let provided_hash = params.get("hash").ok_or("missing login hash")?;
    let expected_hash = compute_login_play_hash(params);
    if provided_hash != &expected_hash {
        return Err("fake: invalid PlayHashKey signature");
    }

    let open_date_raw = params.get("openDateTime").ok_or("missing openDateTime")?;
    let normalized_date: String = open_date_raw.chars().filter(|&c| c != '-').collect();
    if !normalized_date.starts_with(expected_open_ymd) {
        return Err("sys: server openDateTime mismatch");
    }

    let ts: u64 = params
        .get("timestamp")
        .ok_or("missing timestamp")?
        .parse()
        .map_err(|_| "invalid timestamp")?;
    if now_unix.saturating_sub(ts) > max_skew_secs {
        return Err("timeout: login timestamp expired");
    }

    let encrypted_pid = params.get("pid").ok_or("missing encrypted pid")?;
    let pid = babel_decrypt_number(encrypted_pid)?;
    if pid <= min_retained_pid {
        return Err("fake: pid must be greater than PID_MAX_RETAIN");
    }

    let ptype: u32 = params
        .get("ptype")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Ok((pid, ptype))
}

/// Pure-Rust FIPS 180-4 SHA-1 160-bit (20-byte) digest for RFC 6455 WebSocket handshakes
/// and `Util::signVa` (`legacy-ref/rpcfw/lib/Util.class.php:485-490`).
pub fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x6745_2301;
    let mut h1: u32 = 0xEFCD_AB89;
    let mut h2: u32 = 0x98BA_DCFE;
    let mut h3: u32 = 0x1032_5476;
    let mut h4: u32 = 0xC3D2_E1F0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            let off = i * 4;
            w[i] = u32::from_be_bytes([chunk[off], chunk[off + 1], chunk[off + 2], chunk[off + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = if i < 20 {
                ((b & c) | ((!b) & d), 0x5A82_7999u32)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ED9_EBA1u32)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1B_BCDCu32)
            } else {
                (b ^ c ^ d, 0xCA62_C1D6u32)
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// RFC 4648 standard Base64 encoder.
pub fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((data.len() + 2) / 3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Compute the RFC 6455 `Sec-WebSocket-Accept` header value for a client `Sec-WebSocket-Key`.
pub fn websocket_accept_key(sec_websocket_key: &str) -> String {
    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut input = Vec::with_capacity(sec_websocket_key.len() + WS_GUID.len());
    input.extend_from_slice(sec_websocket_key.trim().as_bytes());
    input.extend_from_slice(WS_GUID.as_bytes());
    let digest = sha1_digest(&input);
    base64_encode(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_babel_crypt_des_cbc_matches_openssl_ground_truth() {
        // Verified against OpenSSL 3 legacy DES-CBC (`BabelTim`, `32210967`)
        assert_eq!(babel_encrypt_number(1), "45038311beb65fc6");
        assert_eq!(babel_decrypt_number("45038311beb65fc6"), Ok(1));

        assert_eq!(babel_encrypt_number(202621270), "efa1f17ed91e79f8");
        assert_eq!(babel_decrypt_number("efa1f17ed91e79f8"), Ok(202621270));
    }

    #[test]
    fn test_strict_login_verification_and_debug_backdoor_blocked() {
        let enc_pid = babel_encrypt_number(202621270);
        let mut req = BTreeMap::new();
        req.insert("openDateTime".to_string(), "2012-05-12".to_string());
        req.insert("pid".to_string(), enc_pid);
        req.insert("ptype".to_string(), "0".to_string());
        req.insert("timestamp".to_string(), "1357000000".to_string());

        // 1. Forged hash (attacker attempting User.class.php:61-64 DEBUG bypass) MUST fail
        req.insert("hash".to_string(), "00000000000000000000000000000000".to_string());
        assert_eq!(
            verify_login_request(&req, "20120512", 1357000100, 900, 10000),
            Err("fake: invalid PlayHashKey signature")
        );

        // 2. Authentic PlayHashKey signature succeeds and returns decrypted pid
        let valid_hash = compute_login_play_hash(&req);
        req.insert("hash".to_string(), valid_hash);
        assert_eq!(
            verify_login_request(&req, "20120512", 1357000100, 900, 10000),
            Ok((202621270, 0))
        );
    }
}
