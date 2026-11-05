//! Zero-copy AMF3 wire serializer and deserializer for `King Pirate (v298)`.
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/lib/Util.class.php` (`L492-L538`):
//!   - `Util::amfDecode`: prepends `0x11` (`FrameworkDef::AMF_AMF3`) if absent before decoding.
//!   - `Util::amfEncode`: strips leading `0x11` after encoding when `AMF_AMF3` flag is set.
//! - `legacy-ref/rpcfw/lib/PHPProxy.class.php` (`L185-L269`):
//!   - Encodes PHP associative arrays (`err`, `ret`, `callback`, `session`, `method`, `args`)
//!     using AMF3 Array (`0x09`) and ByteArray (`0x0C`) markers.

use crate::protocol::AMF3_PREFIX;
use std::collections::BTreeMap;

pub const MARKER_UNDEFINED: u8 = 0x00;
pub const MARKER_NULL: u8 = 0x01;
pub const MARKER_FALSE: u8 = 0x02;
pub const MARKER_TRUE: u8 = 0x03;
pub const MARKER_INTEGER: u8 = 0x04;
pub const MARKER_DOUBLE: u8 = 0x05;
pub const MARKER_STRING: u8 = 0x06;
pub const MARKER_ARRAY: u8 = 0x09;
pub const MARKER_OBJECT: u8 = 0x0A;
pub const MARKER_BYTE_ARRAY: u8 = 0x0C;

/// Represents an AMF3 value exchanged between the ActionScript 3 client (`Main.swf`)
/// and the `King Pirate` server (`Util::amfEncode` / `Util::amfDecode`).
#[derive(Debug, Clone, PartialEq)]
pub enum Amf3Value {
    Undefined,
    Null,
    Bool(bool),
    Integer(i32),
    Double(f64),
    String(String),
    ByteArray(Vec<u8>),
    /// AMF3 Array (`0x09`) containing both associative (`key => val`) and dense (`0..N`) entries.
    Array {
        assoc: BTreeMap<String, Amf3Value>,
        dense: Vec<Amf3Value>,
    },
}

impl Amf3Value {
    /// Helper to construct a pure associative AMF3 Array (equivalent to a PHP keyed `array(...)`).
    pub fn assoc_map(entries: impl IntoIterator<Item = (impl Into<String>, Amf3Value)>) -> Self {
        let mut assoc = BTreeMap::new();
        for (k, v) in entries {
            assoc.insert(k.into(), v);
        }
        Self::Array {
            assoc,
            dense: Vec::new(),
        }
    }

    /// Helper to construct a dense AMF3 Array (equivalent to a PHP 0-indexed list `array(a, b)`).
    pub fn dense_list(items: impl IntoIterator<Item = Amf3Value>) -> Self {
        Self::Array {
            assoc: BTreeMap::new(),
            dense: items.into_iter().collect(),
        }
    }

    /// Look up a field inside an associative AMF3 Array.
    pub fn get_field(&self, key: &str) -> Option<&Amf3Value> {
        match self {
            Self::Array { assoc, .. } => assoc.get(key),
            _ => None,
        }
    }

    /// Extract `&str` if this value is `Amf3Value::String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract `i64` from either `Amf3Value::Integer` or numeric `Amf3Value::String`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v as i64),
            Self::Double(v) => Some(*v as i64),
            Self::String(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }
}

/// Encode a 29-bit unsigned integer (`U29`) in big-endian variable-length AMF3 format (1..4 bytes).
pub fn write_u29(out: &mut Vec<u8>, val: u32) {
    let v = val & 0x1FFF_FFFF;
    if v < 0x80 {
        out.push(v as u8);
    } else if v < 0x4000 {
        out.push(((v >> 7) as u8) | 0x80);
        out.push((v & 0x7F) as u8);
    } else if v < 0x20_0000 {
        out.push(((v >> 14) as u8) | 0x80);
        out.push(((v >> 7) as u8) | 0x80);
        out.push((v & 0x7F) as u8);
    } else {
        out.push(((v >> 22) as u8) | 0x80);
        out.push(((v >> 15) as u8) | 0x80);
        out.push(((v >> 8) as u8) | 0x80);
        out.push((v & 0xFF) as u8);
    }
}

/// Decode a 29-bit unsigned integer (`U29`) from a byte slice, advancing `pos`.
pub fn read_u29(buf: &[u8], pos: &mut usize) -> Result<u32, &'static str> {
    if *pos >= buf.len() {
        return Err("unexpected EOF reading U29");
    }
    let mut result: u32 = 0;
    for i in 0..4 {
        if *pos >= buf.len() {
            return Err("truncated U29 sequence");
        }
        let b = buf[*pos];
        *pos += 1;
        if i < 3 {
            result = (result << 7) | ((b & 0x7F) as u32);
            if (b & 0x80) == 0 {
                return Ok(result);
            }
        } else {
            result = (result << 8) | (b as u32);
        }
    }
    Ok(result)
}

/// Write an inline AMF3 UTF-8 string (`U29S-value`: `(byte_len << 1) | 1` followed by UTF-8 bytes).
pub fn write_amf3_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u29(out, ((bytes.len() as u32) << 1) | 1);
    out.extend_from_slice(bytes);
}

/// Read an AMF3 UTF-8 string using the active string reference table.
fn read_amf3_str(
    buf: &[u8],
    pos: &mut usize,
    str_refs: &mut Vec<String>,
) -> Result<String, &'static str> {
    let header = read_u29(buf, pos)?;
    if (header & 1) == 0 {
        let idx = (header >> 1) as usize;
        return str_refs
            .get(idx)
            .cloned()
            .ok_or("invalid AMF3 string reference index");
    }
    let len = (header >> 1) as usize;
    if len == 0 {
        return Ok(String::new());
    }
    if *pos + len > buf.len() {
        return Err("truncated AMF3 string payload");
    }
    let s = std::str::from_utf8(&buf[*pos..*pos + len])
        .map_err(|_| "invalid UTF-8 in AMF3 string")?
        .to_string();
    *pos += len;
    str_refs.push(s.clone());
    Ok(s)
}

/// Encode an `Amf3Value` into raw AMF3 bytes.
///
/// Mirrors `Util::amfEncode` (`legacy-ref/rpcfw/lib/Util.class.php:515-538`):
/// - When `strip_amf3_prefix` is `true`, omits the outer `0x11` marker (`L525-L531`).
/// - When `strip_amf3_prefix` is `false`, prepends `0x11` (`AMF3_PREFIX`).
pub fn amf3_encode(val: &Amf3Value, strip_amf3_prefix: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    if !strip_amf3_prefix {
        out.push(AMF3_PREFIX);
    }
    encode_value(&mut out, val);
    out
}

fn encode_value(out: &mut Vec<u8>, val: &Amf3Value) {
    match val {
        Amf3Value::Undefined => out.push(MARKER_UNDEFINED),
        Amf3Value::Null => out.push(MARKER_NULL),
        Amf3Value::Bool(false) => out.push(MARKER_FALSE),
        Amf3Value::Bool(true) => out.push(MARKER_TRUE),
        Amf3Value::Integer(n) => {
            if *n >= -(1 << 28) && *n < (1 << 28) {
                out.push(MARKER_INTEGER);
                let u29 = (*n as u32) & 0x1FFF_FFFF;
                write_u29(out, u29);
            } else {
                out.push(MARKER_DOUBLE);
                out.extend_from_slice(&(*n as f64).to_be_bytes());
            }
        }
        Amf3Value::Double(d) => {
            out.push(MARKER_DOUBLE);
            out.extend_from_slice(&d.to_be_bytes());
        }
        Amf3Value::String(s) => {
            out.push(MARKER_STRING);
            write_amf3_str(out, s);
        }
        Amf3Value::ByteArray(bytes) => {
            out.push(MARKER_BYTE_ARRAY);
            write_u29(out, ((bytes.len() as u32) << 1) | 1);
            out.extend_from_slice(bytes);
        }
        Amf3Value::Array { assoc, dense } => {
            if !assoc.is_empty() && dense.is_empty() {
                // Encode pure associative map as anonymous dynamic AS3 Object (`0x0A 0x0B 0x01 ... 0x01`)
                // so ActionScript 3 (`Main.swf`) receives a native `Object` instead of an `Array`.
                out.push(MARKER_OBJECT);
                write_u29(out, 0x0B);
                write_amf3_str(out, "");
                for (k, v) in assoc {
                    if !k.is_empty() {
                        write_amf3_str(out, k);
                        encode_value(out, v);
                    }
                }
                write_amf3_str(out, "");
            } else {
                out.push(MARKER_ARRAY);
                write_u29(out, ((dense.len() as u32) << 1) | 1);
                for (k, v) in assoc {
                    if !k.is_empty() {
                        write_amf3_str(out, k);
                        encode_value(out, v);
                    }
                }
                // Empty string (`0x01`) terminates the associative key-value section.
                write_amf3_str(out, "");
                for item in dense {
                    encode_value(out, item);
                }
            }
        }
    }
}

/// Decode an `Amf3Value` from raw bytes.
///
/// Mirrors `Util::amfDecode` (`legacy-ref/rpcfw/lib/Util.class.php:492-513`):
/// automatically accepts payloads with or without the leading `0x11` (`AMF3_PREFIX`) byte,
/// and maintains full AMF3 string, object, and trait reference tables for ActionScript 3 compatibility.
pub fn amf3_decode(buf: &[u8]) -> Result<Amf3Value, &'static str> {
    if buf.is_empty() {
        return Err("empty AMF3 payload");
    }
    let mut pos = if buf[0] == AMF3_PREFIX { 1 } else { 0 };
    let mut str_refs = Vec::new();
    let mut obj_refs = Vec::new();
    let mut trait_refs = Vec::new();
    decode_value(buf, &mut pos, &mut str_refs, &mut obj_refs, &mut trait_refs)
}

fn decode_value(
    buf: &[u8],
    pos: &mut usize,
    str_refs: &mut Vec<String>,
    obj_refs: &mut Vec<Amf3Value>,
    trait_refs: &mut Vec<(bool, Vec<String>)>,
) -> Result<Amf3Value, &'static str> {
    if *pos >= buf.len() {
        return Err("unexpected EOF reading AMF3 marker");
    }
    let marker = buf[*pos];
    *pos += 1;
    match marker {
        MARKER_UNDEFINED => Ok(Amf3Value::Undefined),
        MARKER_NULL => Ok(Amf3Value::Null),
        MARKER_FALSE => Ok(Amf3Value::Bool(false)),
        MARKER_TRUE => Ok(Amf3Value::Bool(true)),
        MARKER_INTEGER => {
            let raw = read_u29(buf, pos)?;
            // Sign-extend 29-bit integer if bit 28 is set.
            let signed = if (raw & 0x1000_0000) != 0 {
                (raw | 0xE000_0000) as i32
            } else {
                raw as i32
            };
            Ok(Amf3Value::Integer(signed))
        }
        MARKER_DOUBLE => {
            if *pos + 8 > buf.len() {
                return Err("truncated AMF3 double");
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&buf[*pos..*pos + 8]);
            *pos += 8;
            Ok(Amf3Value::Double(f64::from_be_bytes(bytes)))
        }
        MARKER_STRING => {
            let s = read_amf3_str(buf, pos, str_refs)?;
            Ok(Amf3Value::String(s))
        }
        MARKER_BYTE_ARRAY => {
            let header = read_u29(buf, pos)?;
            if (header & 1) == 0 {
                let idx = (header >> 1) as usize;
                return obj_refs
                    .get(idx)
                    .cloned()
                    .ok_or("invalid AMF3 ByteArray reference index");
            }
            let len = (header >> 1) as usize;
            if *pos + len > buf.len() {
                return Err("truncated AMF3 ByteArray");
            }
            let val = Amf3Value::ByteArray(buf[*pos..*pos + len].to_vec());
            *pos += len;
            obj_refs.push(val.clone());
            Ok(val)
        }
        MARKER_ARRAY => {
            let header = read_u29(buf, pos)?;
            if (header & 1) == 0 {
                let idx = (header >> 1) as usize;
                return obj_refs
                    .get(idx)
                    .cloned()
                    .ok_or("invalid AMF3 Array reference index");
            }
            let dense_len = (header >> 1) as usize;
            let obj_slot = obj_refs.len();
            obj_refs.push(Amf3Value::Null);
            let mut assoc = BTreeMap::new();
            loop {
                let key = read_amf3_str(buf, pos, str_refs)?;
                if key.is_empty() {
                    break;
                }
                let val = decode_value(buf, pos, str_refs, obj_refs, trait_refs)?;
                assoc.insert(key, val);
            }
            let mut dense = Vec::with_capacity(dense_len.min(4096));
            for _ in 0..dense_len {
                dense.push(decode_value(buf, pos, str_refs, obj_refs, trait_refs)?);
            }
            let result = Amf3Value::Array { assoc, dense };
            obj_refs[obj_slot] = result.clone();
            Ok(result)
        }
        MARKER_OBJECT => {
            let header = read_u29(buf, pos)?;
            if (header & 1) == 0 {
                let idx = (header >> 1) as usize;
                return obj_refs
                    .get(idx)
                    .cloned()
                    .ok_or("invalid AMF3 Object reference index");
            }
            let obj_slot = obj_refs.len();
            obj_refs.push(Amf3Value::Null);

            let (is_dynamic, prop_names) = if (header & 2) == 0 {
                let trait_idx = (header >> 2) as usize;
                trait_refs
                    .get(trait_idx)
                    .cloned()
                    .ok_or("invalid AMF3 Object trait reference index")?
            } else {
                if (header & 4) != 0 {
                    return Err("unsupported externalizable AMF3 Object");
                }
                let is_dyn = (header & 8) != 0;
                let prop_count = (header >> 4) as usize;
                let _class_name = read_amf3_str(buf, pos, str_refs)?;
                let mut names = Vec::with_capacity(prop_count.min(256));
                for _ in 0..prop_count {
                    names.push(read_amf3_str(buf, pos, str_refs)?);
                }
                trait_refs.push((is_dyn, names.clone()));
                (is_dyn, names)
            };

            let mut assoc = BTreeMap::new();
            for name in prop_names {
                let val = decode_value(buf, pos, str_refs, obj_refs, trait_refs)?;
                assoc.insert(name, val);
            }
            if is_dynamic {
                loop {
                    let key = read_amf3_str(buf, pos, str_refs)?;
                    if key.is_empty() {
                        break;
                    }
                    let val = decode_value(buf, pos, str_refs, obj_refs, trait_refs)?;
                    assoc.insert(key, val);
                }
            }
            let result = Amf3Value::Array {
                assoc,
                dense: Vec::new(),
            };
            obj_refs[obj_slot] = result.clone();
            Ok(result)
        }
        _ => Err("unsupported AMF3 marker"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u29_and_signed_integer_roundtrip() {
        for &val in &[0i32, 1, 127, 128, 16383, 16384, 20002, -1, -20000] {
            let encoded = amf3_encode(&Amf3Value::Integer(val), true);
            let decoded = amf3_decode(&encoded).expect("decode integer failed");
            assert_eq!(decoded, Amf3Value::Integer(val));
        }
    }

    #[test]
    fn test_rpc_request_and_double_amf3_response_roundtrip() {
        // Mirrors RPCFramework.class.php:357-391 (outer response wrapping inner AMF3-encoded ret)
        let inner_ret = Amf3Value::assoc_map([
            ("err", Amf3Value::String("ok".to_string())),
            ("ret", Amf3Value::String("ok".to_string())),
        ]);
        let inner_bytes = amf3_encode(&inner_ret, true);

        let outer_resp = Amf3Value::assoc_map([
            ("err", Amf3Value::String("ok".to_string())),
            ("response", Amf3Value::ByteArray(inner_bytes.clone())),
            ("session", Amf3Value::Bool(false)),
        ]);

        let wire = amf3_encode(&outer_resp, false);
        assert_eq!(wire[0], AMF3_PREFIX);

        let decoded_outer = amf3_decode(&wire).expect("decode outer failed");
        assert_eq!(
            decoded_outer.get_field("err").and_then(|v| v.as_str()),
            Some("ok")
        );
        if let Some(Amf3Value::ByteArray(raw_inner)) = decoded_outer.get_field("response") {
            let decoded_inner = amf3_decode(raw_inner).expect("decode inner failed");
            assert_eq!(decoded_inner, inner_ret);
        } else {
            panic!("expected ByteArray in response field");
        }
    }
}
