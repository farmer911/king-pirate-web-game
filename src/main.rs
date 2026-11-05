#![deny(dead_code, unused_imports, unused_variables)]

pub mod actor;
pub mod amf3;
pub mod crypt;
pub mod data;
pub mod gateway;
pub mod protocol;

fn main() {
    let hdr = protocol::FrameHeader {
        body_size: 0,
        flag: protocol::FLAG_ENCRYPTED | protocol::FLAG_UNWRAP_RET,
    };
    let enc_pid = crypt::babel_encrypt_number(202621270);
    let dec_pid = crypt::babel_decrypt_number(&enc_pid).unwrap_or(0);
    let mut actor = actor::PlayerSessionActor::new(20001, dec_pid, 1, "Luffy".to_string());
    let _ = actor.add_gold(50);
    let store = data::DataStore::new(data::SqlDialect::SqliteWal);
    if let Some(delta) = actor.take_dirty_delta() {
        let _ = store.apply_write_behind_delta(&delta);
    }
    let sample_pkt = gateway::encode_framed_packet(
        &amf3::Amf3Value::String("ok".to_string()),
        hdr.flag,
    );
    println!(
        "King Pirate Rust Server v0.2.0 ready (header_size={}, pid={}, pkt_bytes={}, policy_len={})",
        protocol::HEAD_SIZE,
        dec_pid,
        sample_pkt.len(),
        gateway::FLASH_POLICY_RESPONSE.len()
    );
}
