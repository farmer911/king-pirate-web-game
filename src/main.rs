#![deny(dead_code, unused_imports, unused_variables)]

pub mod actor;
pub mod amf3;
pub mod battle;
pub mod btstore;
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
    let static_store = btstore::StaticBtstore::new_baseline();
    let mut t1 = battle::FormationGrid::new(20001, 10001);
    let mut t2 = battle::FormationGrid::new(20002, 10002);
    t1.set_hero(0, battle::HeroUnit::new_basic(101, 1, 0, 10, 1000, 350, 100));
    t2.set_hero(0, battle::HeroUnit::new_basic(201, 2, 0, 5, 600, 150, 50));
    let combat_res = battle::execute_battle_do_hero(t1, t2, &static_store);
    let sample_pkt = gateway::encode_framed_packet(&combat_res.to_amf3(), hdr.flag);
    println!(
        "King Pirate Rust Server v0.3.0 ready (header_size={}, pid={}, appraisal={}, pkt_bytes={})",
        protocol::HEAD_SIZE,
        dec_pid,
        combat_res.appraisal,
        sample_pkt.len()
    );
}
