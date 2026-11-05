#![deny(dead_code, unused_imports, unused_variables)]

pub mod protocol;

fn main() {
    let hdr = protocol::FrameHeader {
        body_size: 0,
        flag: 0,
    };
    println!(
        "King Pirate Rust Server v0.1.0 initialized (header_size={}, mess_code={}, key_prefix={}, sample_hdr={:?})",
        protocol::HEAD_SIZE,
        String::from_utf8_lossy(protocol::MESS_CODE_PRIMARY),
        String::from_utf8_lossy(protocol::BABEL_CRYPT_KEY),
        hdr.encode(),
    );
    let _ = (
        protocol::AMF3_PREFIX,
        protocol::MESS_CODE_SECONDARY,
        protocol::BABEL_CRYPT_IV,
        protocol::PLAY_HASH_KEY,
        protocol::build_xor_checksum_buffer(protocol::MESS_CODE_PRIMARY, &[]),
        protocol::unpack_pid_bytes(&protocol::pack_pid_bytes(1)),
        hdr.is_compressed(),
        hdr.is_encrypted(),
        hdr.should_unwrap_ret(),
        protocol::FrameHeader::decode(&[0; 8]),
    );
}
