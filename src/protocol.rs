//! Wire Protocol, 8-Byte Framing & XOR Buffer Preparation for Vua Hai Tac 2025
//!
//! Ground-Truth Citations:
//! - 8-Byte Header Framing (`body_size` BE u32 + `flag` BE u32):
//!   `legacy-ref/rpcfw/lib/PHPProxy.class.php:108-135`, `:235-326`
//! - Checksum XOR Stream (`MESS_CODE` + adjacent byte XOR):
//!   `legacy-ref/rpcfw/lib/PHPProxy.class.php:363-381`
//! - Gateway MessCode (`BabelTime` / `4bd4ace59`):
//!   `legacy-ref/lcserver/conf/game20002.xml:51-55`
//! - BabelCrypt Constants (`DES-CBC`, `KEY`, `IV`, `PlayHashKey`):
//!   `legacy-ref/rpcfw/conf/BabelCrypt.cfg.php:18-29`

pub const HEAD_SIZE: usize = 8; // legacy-ref/rpcfw/def/Framework.def.php:22
pub const AMF3_PREFIX: u8 = 0x11; // legacy-ref/rpcfw/def/Framework.def.php:24

pub const FLAG_COMPRESSED: u32 = 0x00ff_0000; // PHPProxy.class.php:124, 246
pub const FLAG_ENCRYPTED: u32 = 0xff00_0000; // PHPProxy.class.php:130, 240
pub const FLAG_UNWRAP_RET: u32 = 0x0000_ff00; // PHPProxy.class.php:253

pub const MESS_CODE_PRIMARY: &[u8] = b"BabelTime"; // game20002.xml:51
pub const MESS_CODE_SECONDARY: &[u8] = b"4bd4ace59"; // game20002.xml:54

pub const BABEL_CRYPT_KEY: &[u8; 8] = b"BabelTim"; // BabelCrypt.cfg.php:23 ('BabelTime777' 8-byte DES key)
pub const BABEL_CRYPT_IV: &[u8; 8] = b"32210967"; // BabelCrypt.cfg.php:25
pub const PLAY_HASH_KEY: &str = "2012#B@belPir@te#0410"; // BabelCrypt.cfg.php:28

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub body_size: u32,
    pub flag: u32,
}

impl FrameHeader {
    /// Encode 8-byte big-endian header (`PHPProxy::toU32` at L133, L317-326).
    #[inline]
    pub fn encode(self) -> [u8; HEAD_SIZE] {
        let mut out = [0u8; HEAD_SIZE];
        out[0..4].copy_from_slice(&self.body_size.to_be_bytes());
        out[4..8].copy_from_slice(&self.flag.to_be_bytes());
        out
    }

    /// Decode 8-byte big-endian header (`PHPProxy::readU32` at L238-239, L291-301).
    #[inline]
    pub fn decode(bytes: &[u8; HEAD_SIZE]) -> Self {
        let body_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let flag = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Self { body_size, flag }
    }

    #[inline]
    pub fn is_compressed(self) -> bool {
        (self.flag & FLAG_COMPRESSED) != 0
    }

    #[inline]
    pub fn is_encrypted(self) -> bool {
        (self.flag & FLAG_ENCRYPTED) != 0
    }

    #[inline]
    pub fn should_unwrap_ret(self) -> bool {
        (self.flag & FLAG_UNWRAP_RET) != 0
    }
}

/// Build the exact XOR buffer prior to MD5 hashing (`PHPProxy::encrypt` at L363-381).
/// Replaces O(N) PHP string concatenations (`$raw .= chr($a ^ $b)`) with a single pre-allocated `Vec<u8>`.
pub fn build_xor_checksum_buffer(mess_code: &[u8], data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut raw = Vec::with_capacity(mess_code.len() + len);
    raw.extend_from_slice(mess_code);
    for i in 0..len {
        let a = data[i];
        let b = if i + 1 == len {
            (len & 0xff) as u8
        } else {
            data[i + 1]
        };
        raw.push(a ^ b);
    }
    raw
}

/// Pack big-endian integer bytes for `BabelCrypt::encryptNumber` (`BabelCrypt.class.php:24-31`).
pub fn pack_pid_bytes(mut pid: u64) -> Vec<u8> {
    if pid == 0 {
        return Vec::new();
    }
    let mut bytes = Vec::with_capacity(8);
    while pid > 0 {
        bytes.push((pid % 256) as u8);
        pid /= 256;
    }
    bytes.reverse();
    bytes
}

/// Unpack big-endian integer bytes for `BabelCrypt::decryptNumber` (`BabelCrypt.class.php:46-52`).
pub fn unpack_pid_bytes(data: &[u8]) -> u64 {
    let mut pid = 0u64;
    for &b in data {
        pid = (pid << 8) + u64::from(b);
    }
    pid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header_roundtrip() {
        let hdr = FrameHeader {
            body_size: 4096,
            flag: FLAG_COMPRESSED | FLAG_ENCRYPTED,
        };
        let encoded = hdr.encode();
        let decoded = FrameHeader::decode(&encoded);
        assert_eq!(hdr, decoded);
        assert!(decoded.is_compressed());
        assert!(decoded.is_encrypted());
        assert!(!decoded.should_unwrap_ret());
    }

    #[test]
    fn test_xor_checksum_buffer_matches_phpproxy_encrypt() {
        // Ground-truth trace of PHPProxy.class.php:366-379 for data = [0x10, 0x20, 0x30] (len = 3):
        // i=0: 0x10 ^ 0x20 = 0x30
        // i=1: 0x20 ^ 0x30 = 0x10
        // i=2: 0x30 ^ 3    = 0x33
        let buf = build_xor_checksum_buffer(MESS_CODE_PRIMARY, &[0x10, 0x20, 0x30]);
        assert_eq!(&buf[..9], b"BabelTime");
        assert_eq!(&buf[9..], &[0x30, 0x10, 0x33]);
    }

    #[test]
    fn test_babel_crypt_pid_packing_matches_php() {
        let pid = 202621270u64;
        let packed = pack_pid_bytes(pid);
        assert_eq!(packed, vec![0x0c, 0x13, 0xc1, 0x56]);
        assert_eq!(unpack_pid_bytes(&packed), pid);
    }

    #[test]
    fn test_zero_chinese_and_english_only_policy() {
        use std::fs;
        use std::path::Path;

        fn has_cjk(s: &str) -> Option<char> {
            s.chars().find(|&c| {
                ('\u{3400}'..='\u{4DBF}').contains(&c)
                    || ('\u{4E00}'..='\u{9FFF}').contains(&c)
                    || ('\u{F900}'..='\u{FAFF}').contains(&c)
            })
        }

        fn has_vietnamese(s: &str) -> Option<char> {
            s.chars().find(|&c| {
                ('\u{00C0}'..='\u{024F}').contains(&c) || ('\u{1E00}'..='\u{1EFF}').contains(&c)
            })
        }

        fn check_dir(dir: &Path, forbid_vietnamese: bool) {
            if !dir.exists() {
                return;
            }
            for entry in fs::read_dir(dir).expect("read_dir failed") {
                let entry = entry.expect("dir entry failed");
                let path = entry.path();
                if path.is_dir() {
                    check_dir(&path, forbid_vietnamese);
                } else if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(ch) = has_cjk(&content) {
                        panic!("Forbidden Chinese/CJK character '{}' found in {:?}", ch, path);
                    }
                    if forbid_vietnamese {
                        if let Some(ch) = has_vietnamese(&content) {
                            panic!(
                                "Non-English character '{}' found in English-only file {:?}",
                                ch, path
                            );
                        }
                    }
                }
            }
        }

        check_dir(Path::new("src"), true);
        check_dir(Path::new(".agents/rules"), true);
        check_dir(Path::new(".agents/skills"), true);
        check_dir(Path::new("legacy-ref"), false);

        for doc in ["GEMINI.md", "REWRITE_PLAN.md", ".agents/STATE.md", "Cargo.toml"] {
            if let Ok(content) = fs::read_to_string(doc) {
                if let Some(ch) = has_cjk(&content) {
                    panic!("Forbidden Chinese/CJK character '{}' found in {}", ch, doc);
                }
                if let Some(ch) = has_vietnamese(&content) {
                    panic!("Non-English character '{}' found in {}", ch, doc);
                }
            }
        }
    }

    #[test]
    fn test_gitignore_blocks_plan_and_enforces_checkbox_guardrail() {
        use std::fs;
        let gitignore = fs::read_to_string(".gitignore").expect("read .gitignore failed");
        for must_ignore in [
            "REWRITE_PLAN.md",
            "docs/",
            ".archify/",
            ".agents/STATE.md",
            "graphify-out/",
            "legacy-ref/",
        ] {
            assert!(
                gitignore.lines().any(|line| line.trim() == must_ignore),
                ".gitignore must block '{}' from being pushed to Git",
                must_ignore
            );
        }

        for plan_file in ["REWRITE_PLAN.md", ".agents/STATE.md"] {
            if let Ok(content) = fs::read_to_string(plan_file) {
                let checkbox_count = content
                    .lines()
                    .filter(|l| {
                        let t = l.trim_start();
                        t.starts_with("- [ ] ") || t.starts_with("- [x] ")
                    })
                    .count();
                assert!(
                    checkbox_count >= 10,
                    "{} must contain granular '- [ ]' / '- [x]' checkboxes (found {})",
                    plan_file,
                    checkbox_count
                );
            }
        }
    }

    #[test]
    fn test_mandatory_guardrail_and_zero_php_leak() {
        use std::fs;
        use std::path::Path;

        // 1. Verify Rule 09 guardrail exists and is always_on
        let rule_09 = fs::read_to_string(
            ".agents/rules/09-mandatory-planning-and-zero-fabrication-guardrail.md",
        )
        .expect("Rule 09 guardrail file must exist");
        assert!(
            rule_09.contains("trigger: always_on"),
            "Rule 09 must have trigger: always_on"
        );

        // 2. Verify React.js web portal source (`web/src/App.tsx`) and built bundle exist
        let react_app =
            fs::read_to_string("web/src/App.tsx").expect("web/src/App.tsx React portal must exist");
        for required_class in ["Cx_Nav", "DH_Panel", "User_Panel", "Main_Content"] {
            assert!(
                react_app.contains(required_class),
                "web/src/App.tsx must preserve legacy layout class {}",
                required_class
            );
        }
        assert!(
            Path::new("client-assets/portal/app.js").is_file(),
            "Built React bundle client-assets/portal/app.js must exist"
        );

        // 3. Verify zero `.php` files exist inside `client-assets/` (prevents raw .php downloads)
        fn assert_no_php_files(dir: &Path) {
            if !dir.exists() {
                return;
            }
            for entry in fs::read_dir(dir).expect("read_dir failed") {
                let entry = entry.expect("entry failed");
                let path = entry.path();
                if path.is_dir() {
                    assert_no_php_files(&path);
                } else if let Some(ext) = path.extension() {
                    assert_ne!(
                        ext.to_string_lossy().to_ascii_lowercase(),
                        "php",
                        "Raw .php file found in client-assets/: {:?}",
                        path
                    );
                }
            }
        }
        assert_no_php_files(Path::new("client-assets"));
    }

    #[test]
    fn test_honesty_no_fake_metrics_no_test_only_domains_no_junk() {
        use std::fs;
        use std::path::Path;

        // 1. Verify `.agents/skills/honesty/SKILL.md` exists, is <= 200 lines, and enforces Graphify MCP
        let honesty_skill = fs::read_to_string(".agents/skills/honesty/SKILL.md")
            .expect("Honesty skill (.agents/skills/honesty/SKILL.md) must exist");
        assert!(
            honesty_skill.lines().count() <= 200,
            "honesty/SKILL.md must be <= 200 lines"
        );
        assert!(
            honesty_skill.contains("description: Use when"),
            "honesty/SKILL.md must have third-person 'description: Use when...'"
        );
        assert!(
            honesty_skill.contains("query_graph") && honesty_skill.contains("get_neighbors"),
            "honesty/SKILL.md must enforce Graphify MCP tools (query_graph, get_neighbors)"
        );

        // 2. Verify zero phantom crates or unmeasured ~ns/~us claims in docs & source
        let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");
        for path in [
            "REWRITE_PLAN.md",
            "GEMINI.md",
            ".agents/STATE.md",
            "src/actor.rs",
            "src/data.rs",
        ] {
            let content = fs::read_to_string(path).expect("file must exist");
            for phantom in ["DashMap", "rusqlite", "rust-embed", "tokio::mpsc"] {
                if !cargo_toml.contains(phantom) {
                    assert!(
                        !content.contains(phantom),
                        "HONESTY VIOLATION: {} claims phantom crate `{}` not in Cargo.toml",
                        path,
                        phantom
                    );
                }
            }
            for fake_metric in [
                "~20-50 ns",
                "~20–50 ns",
                "~50 ns",
                "~50ns",
                "5–15 µs",
                "15–50 µs",
                "0.1 µs",
            ] {
                assert!(
                    !content.contains(fake_metric),
                    "HONESTY VIOLATION: {} contains unmeasured fake metric `{}`",
                    path,
                    fake_metric
                );
            }
        }

        // 3. Verify all 11 domain modules + StaticBtstore are wired in production `dispatch_client_rpc`
        let mod_rs = fs::read_to_string("src/domain/mod.rs").expect("src/domain/mod.rs must exist");
        let prod_code = mod_rs
            .split("#[cfg(test)]")
            .next()
            .expect("production section must exist");
        for required_domain in [
            "user::",
            "hero::",
            "bag::",
            "forge::",
            "copy::",
            "arena::",
            "boss::",
            "guild::",
            "town::",
            "pet::",
            "economy::",
            "StaticBtstore",
        ] {
            assert!(
                prod_code.contains(required_domain),
                "HONESTY VIOLATION: `{}` is only called in #[cfg(test)] and not wired into production `dispatch_client_rpc`",
                required_domain
            );
        }

        // 4. Verify `web/src/App.tsx` contains verbatim `legacy-ref/www/index.php:61-65` text
        let app_tsx = fs::read_to_string("web/src/App.tsx").expect("web/src/App.tsx must exist");
        assert!(
            app_tsx.contains("Monkey D Dragon, Tezoro, CP0 Lucci, CP0 Kaku")
                && app_tsx.contains("id=\"spa-game-shell\""),
            "HONESTY VIOLATION: web/src/App.tsx does not match verbatim legacy-ref/www/index.php:61 or Option B SPA shell"
        );

        // 5. Verify zero backup/junk files (.orig, .bk, .bak, .jsfl, .2017*-.2020*) in client-assets/
        fn assert_no_junk_files(dir: &Path) {
            if !dir.exists() {
                return;
            }
            for entry in fs::read_dir(dir).expect("read_dir failed") {
                let entry = entry.expect("entry failed");
                let path = entry.path();
                if path.is_dir() {
                    assert_no_junk_files(&path);
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_ascii_lowercase();
                    let is_dated_backup = ext_str.len() == 8
                        && (ext_str.starts_with("201") || ext_str.starts_with("202"))
                        && ext_str.chars().all(|c| c.is_ascii_digit());
                    assert!(
                        !matches!(ext_str.as_str(), "orig" | "bk" | "bak" | "jsfl" | "1")
                            && !is_dated_backup,
                        "HONESTY VIOLATION: Junk/backup file left in client-assets/: {:?}",
                        path
                    );
                }
            }
        }
        assert_no_junk_files(Path::new("client-assets"));
    }
}
