#![deny(warnings, dead_code, unused_imports, unused_variables)]

pub mod actor;
pub mod amf3;
pub mod battle;
pub mod btstore;
pub mod crypt;
pub mod data;
pub mod domain;
pub mod gateway;
pub mod portal;
pub mod protocol;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_700_000_000)
}

/// Process accumulated stream bytes from either a WebSocket (`:8003`) or TCP (`:8002`) client,
/// extracting complete `BabelTimeSocket` packets (or Flash policy requests) and returning response packets.
pub fn process_stream_buffer(
    recv_buf: &mut Vec<u8>,
    session: &mut gateway::ConnectionSession,
    actor_slot: &mut Option<actor::PlayerSessionActor>,
    store: &data::DataStore,
    persist_path: Option<&Path>,
) -> Result<Vec<Vec<u8>>, &'static str> {
    let mut responses = Vec::new();
    loop {
        if recv_buf.starts_with(gateway::FLASH_POLICY_REQUEST) {
            recv_buf.drain(..gateway::FLASH_POLICY_REQUEST.len());
            responses.push(gateway::FLASH_POLICY_RESPONSE.to_vec());
            continue;
        }
        if recv_buf.len() < protocol::HEAD_SIZE {
            break;
        }
        let mut hdr_bytes = [0u8; protocol::HEAD_SIZE];
        hdr_bytes.copy_from_slice(&recv_buf[..protocol::HEAD_SIZE]);
        let hdr = protocol::FrameHeader::decode(&hdr_bytes);
        let body_len = hdr.body_size as usize;
        if body_len > 4 * 1024 * 1024 {
            return Err("frame body_size exceeds 4MB safety limit");
        }
        let sig_len = if (hdr.flag & protocol::FLAG_ENCRYPTED) != 0 {
            16
        } else {
            0
        };
        let total_len = protocol::HEAD_SIZE + sig_len + body_len;
        if recv_buf.len() < total_len {
            break;
        }
        let frame: Vec<u8> = recv_buf.drain(..total_len).collect();
        let (_hdr, req_val) = match gateway::decode_framed_packet(&frame) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[RPC-ERR] decode_framed_packet failed: {} (len={})", e, frame.len());
                return Err(e);
            }
        };
        let method_name = req_val
            .get_field("method")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>")
            .to_string();
        let resp = match domain::dispatch_client_rpc(
            session,
            actor_slot,
            store,
            persist_path,
            &req_val,
            current_unix_seconds(),
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "[RPC-ERR] dispatch_client_rpc failed for method={}: {}",
                    method_name, e
                );
                return Err(e);
            }
        };
        println!(
            "[RPC] method={} pid={:?} uid={:?} req_bytes={} resp_bytes={}",
            method_name,
            session.pid,
            session.uid,
            frame.len(),
            resp.len()
        );
        let _ = std::io::stdout().flush();
        responses.push(resp);
    }
    Ok(responses)
}

fn handle_ws_client(
    mut stream: TcpStream,
    store: Arc<data::DataStore>,
    persist_path: PathBuf,
) {
    let mut hs_buf = [0u8; 4096];
    let n = match stream.read(&mut hs_buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let req_str = String::from_utf8_lossy(&hs_buf[..n]);
    let Some(ws_key) = gateway::parse_websocket_key(&req_str) else {
        return;
    };
    let upgrade = gateway::build_websocket_upgrade_response(&ws_key);
    if stream.write_all(&upgrade).is_err() || stream.flush().is_err() {
        return;
    }

    let mut session = gateway::ConnectionSession::default();
    let mut actor_slot = None;
    let mut recv_buf = Vec::with_capacity(1024);

    while let Ok(Some((opcode, payload))) = gateway::read_ws_frame(&mut stream) {
        if opcode == 0x9 {
            let _ = gateway::write_ws_frame(&mut stream, 0xA, &payload);
            continue;
        }
        if opcode != 0x1 && opcode != 0x2 {
            continue;
        }
        recv_buf.extend_from_slice(&payload);
        match process_stream_buffer(
            &mut recv_buf,
            &mut session,
            &mut actor_slot,
            &store,
            Some(&persist_path),
        ) {
            Ok(resps) => {
                for resp in resps {
                    if gateway::write_ws_frame(&mut stream, 0x2, &resp).is_err() {
                        return;
                    }
                }
            }
            Err(_) => return,
        }
    }
}

fn handle_tcp_client(
    mut stream: TcpStream,
    store: Arc<data::DataStore>,
    persist_path: PathBuf,
) {
    let mut session = gateway::ConnectionSession::default();
    let mut actor_slot = None;
    let mut recv_buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 4096];

    loop {
        let n = match stream.read(&mut chunk) {
            Ok(n) if n > 0 => n,
            _ => return,
        };
        recv_buf.extend_from_slice(&chunk[..n]);
        match process_stream_buffer(
            &mut recv_buf,
            &mut session,
            &mut actor_slot,
            &store,
            Some(&persist_path),
        ) {
            Ok(resps) => {
                for resp in resps {
                    if stream.write_all(&resp).is_err() || stream.flush().is_err() {
                        return;
                    }
                }
            }
            Err(_) => return,
        }
    }
}

pub fn run_live_daemon(
    http_addr: &str,
    ws_addr: &str,
    tcp_addr: &str,
    assets_root: PathBuf,
    persist_path: PathBuf,
) -> std::io::Result<()> {
    let store = Arc::new(data::DataStore::new(data::SqlDialect::SqliteWal));
    if persist_path.is_file() {
        let _ = store.load_from_disk(&persist_path);
    }

    let http_listener = TcpListener::bind(http_addr)?;
    let ws_listener = TcpListener::bind(ws_addr)?;
    let tcp_listener = TcpListener::bind(tcp_addr)?;

    println!(
        "King Pirate Rust Daemon listening: HTTP={} WS={} TCP={} (assets={}, store={})",
        http_addr,
        ws_addr,
        tcp_addr,
        assets_root.display(),
        persist_path.display()
    );

    let store_ws = Arc::clone(&store);
    let persist_ws = persist_path.clone();
    std::thread::spawn(move || {
        for incoming in ws_listener.incoming().flatten() {
            let st = Arc::clone(&store_ws);
            let pp = persist_ws.clone();
            std::thread::spawn(move || handle_ws_client(incoming, st, pp));
        }
    });

    let store_tcp = Arc::clone(&store);
    let persist_tcp = persist_path.clone();
    std::thread::spawn(move || {
        for incoming in tcp_listener.incoming().flatten() {
            let st = Arc::clone(&store_tcp);
            let pp = persist_tcp.clone();
            std::thread::spawn(move || handle_tcp_client(incoming, st, pp));
        }
    });

    for incoming in http_listener.incoming().flatten() {
        let root = assets_root.clone();
        let st = Arc::clone(&store);
        let pp = persist_path.clone();
        std::thread::spawn(move || {
            let mut stream = incoming;
            let _ = gateway::handle_http_stream_with_store(&mut stream, &root, Some(&st), Some(&pp));
        });
    }
    Ok(())
}

/// Always starts the server. Arguments are ignored, so the old `king-pirate serve` still works.
fn main() {
    let assets_root = std::env::var("KING_PIRATE_ASSETS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("client-assets"));
    let persist_path = std::env::var("KING_PIRATE_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/pirate_store.amf3"));
    if let Err(err) = run_live_daemon(
        "0.0.0.0:8081",
        "0.0.0.0:8003",
        "0.0.0.0:8002",
        assets_root,
        persist_path,
    ) {
        eprintln!("Fatal daemon error: {}", err);
        std::process::exit(1);
    }
}
