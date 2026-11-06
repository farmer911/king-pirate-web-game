//! Traffic recorder: `king-pirate record [--upstream <host:port>]`.
//!
//! The server runs as usual (portal and client files on `:8081`), but every game connection on `:8003`
//! (WebSocket, used by Ruffle) or `:8002` (TCP) is forwarded to a legacy lcserver, and both directions
//! are saved frame by frame to `tests/fixtures/sessions/<unix time>-<n>.log`. These logs are the parity
//! fixtures for porting: a Rust handler is correct when it answers a recorded request with the recorded
//! response.
//!
//! Log format, one frame per line: `<ms since connect> <c2s|s2c> <frame as hex>`, preceded by a
//! `# <method or callback name>` comment for readers. The Flash policy exchange (NUL-terminated XML,
//! not a frame) is logged as `policy`.

use crate::amf3::{Amf3Value, amf3_decode};
use crate::crypt::hex_encode;
use crate::gateway::{accept_websocket, decode_framed_packet, frame_len, read_ws_frame, write_ws_frame};
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Where `king-pirate record` writes session logs.
pub const SESSION_DIR: &str = "tests/fixtures/sessions";
/// Default upstream: the legacy oracle container (`tools/legacy-oracle/compose.yml`).
pub const DEFAULT_UPSTREAM: &str = "127.0.0.1:18002";

#[derive(Clone, Copy)]
pub enum Dir {
    C2s,
    S2c,
}

/// Splits each direction's byte stream into frames and writes one log line per frame.
pub struct SessionLog<W: Write> {
    out: W,
    start: Instant,
    c2s: Vec<u8>,
    s2c: Vec<u8>,
}

impl<W: Write> SessionLog<W> {
    pub fn new(out: W) -> Self {
        Self { out, start: Instant::now(), c2s: Vec::new(), s2c: Vec::new() }
    }

    /// Append bytes seen in one direction and log every frame they complete.
    pub fn feed(&mut self, dir: Dir, bytes: &[u8]) -> io::Result<()> {
        let Self { out, start, c2s, s2c } = self;
        let (buf, tag) = match dir {
            Dir::C2s => (c2s, "c2s"),
            Dir::S2c => (s2c, "s2c"),
        };
        buf.extend_from_slice(bytes);
        loop {
            let ms = start.elapsed().as_millis();
            if buf.first() == Some(&b'<') {
                let Some(end) = buf.iter().position(|&b| b == 0) else { break };
                buf.drain(..=end);
                writeln!(out, "{ms} {tag} policy")?;
                continue;
            }
            let Some(n) = frame_len(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))? else {
                break;
            };
            let frame: Vec<u8> = buf.drain(..n).collect();
            let label = match decode_framed_packet(&frame) {
                Ok((_, v)) => frame_label(&v),
                Err(e) => format!("undecodable: {e}"),
            };
            writeln!(out, "# {label}")?;
            writeln!(out, "{ms} {tag} {}", hex_encode(&frame))?;
        }
        out.flush()
    }
}

/// Request method (`user.login`), or the callback names of a response. A response is
/// `{ ret: [ ByteArray, ... ] }`, each ByteArray holding `{ err, callback: { callbackName }, ret }`.
fn frame_label(v: &Amf3Value) -> String {
    if let Some(method) = v.get_field("method").and_then(|s| s.as_str()) {
        return method.to_string();
    }
    let names: Vec<String> = match v.get_field("ret") {
        Some(Amf3Value::Array { dense, .. }) => dense
            .iter()
            .filter_map(|r| {
                let inner = match r {
                    Amf3Value::ByteArray(bytes) => amf3_decode(bytes).ok()?,
                    other => other.clone(),
                };
                let name = inner.get_field("callback")?.get_field("callbackName")?.as_str()?.to_string();
                let err = inner.get_field("err").and_then(|e| e.as_str()).unwrap_or("?");
                Some(format!("{name} (err={err})"))
            })
            .collect(),
        _ => Vec::new(),
    };
    if names.is_empty() { "frame".to_string() } else { format!("reply {}", names.join(", ")) }
}

/// Forward one game connection to `upstream` and record it into a new file in `log_dir`.
/// `ws`: the client speaks WebSocket (Ruffle on `:8003`); otherwise raw TCP (`:8002`).
pub fn proxy_connection(mut client: TcpStream, upstream: &str, ws: bool, log_dir: &Path) {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    if ws && !accept_websocket(&mut client) {
        return;
    }
    let mut up = match TcpStream::connect(upstream) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[RECORD] cannot reach upstream {upstream}: {e}");
            return;
        }
    };
    let unix = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let path = log_dir.join(format!("{unix}-{}.log", NEXT.fetch_add(1, Ordering::Relaxed)));
    let file = match std::fs::create_dir_all(log_dir).and_then(|_| File::create(&path)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[RECORD] cannot create {}: {e}", path.display());
            return;
        }
    };
    println!("[RECORD] {} -> {}", upstream, path.display());
    let log = Arc::new(Mutex::new(SessionLog::new(file)));
    let (Ok(mut up_read), Ok(mut client_write)) = (up.try_clone(), client.try_clone()) else {
        return;
    };

    let s2c_log = Arc::clone(&log);
    let s2c = std::thread::spawn(move || {
        let mut chunk = [0u8; 16 * 1024];
        while let Ok(n @ 1..) = up_read.read(&mut chunk) {
            let data = &chunk[..n];
            let sent = if ws { write_ws_frame(&mut client_write, 0x2, data) } else { client_write.write_all(data) };
            if sent.is_err() {
                break;
            }
            if let Ok(mut l) = s2c_log.lock() {
                let _ = l.feed(Dir::S2c, data);
            }
        }
        let _ = client_write.shutdown(Shutdown::Both);
    });

    let mut chunk = [0u8; 16 * 1024];
    loop {
        let data = if ws {
            match read_ws_frame(&mut client) {
                Ok(Some((0x1 | 0x2, payload))) => payload,
                Ok(Some((0x8, _))) | Ok(None) | Err(_) => break,
                Ok(Some(_)) => continue,
            }
        } else {
            match client.read(&mut chunk) {
                Ok(n @ 1..) => chunk[..n].to_vec(),
                _ => break,
            }
        };
        if up.write_all(&data).is_err() {
            break;
        }
        if let Ok(mut l) = log.lock() {
            let _ = l.feed(Dir::C2s, &data);
        }
    }
    let _ = up.shutdown(Shutdown::Both);
    let _ = s2c.join();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::encode_framed_packet;
    use std::net::TcpListener;

    fn request(method: &str) -> Vec<u8> {
        encode_framed_packet(&Amf3Value::assoc_map([("method", Amf3Value::String(method.to_string()))]), 0)
    }

    #[test]
    fn test_session_log_splits_frames_across_reads() {
        let mut log = SessionLog::new(Vec::new());
        let mut stream = b"<policy-file-request/>\0".to_vec();
        stream.extend(request("user.login"));
        stream.extend(request("user.getUsers"));
        let (a, b) = stream.split_at(30); // second read starts mid-frame
        log.feed(Dir::C2s, a).unwrap();
        log.feed(Dir::C2s, b).unwrap();
        let text = String::from_utf8(log.out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 5, "{text}");
        assert!(lines[0].ends_with("c2s policy"));
        assert_eq!(lines[1], "# user.login");
        assert_eq!(lines[3], "# user.getUsers");
        let hex = lines[4].rsplit(' ').next().unwrap();
        assert_eq!(crate::crypt::hex_decode(hex).unwrap(), request("user.getUsers"));
    }

    /// Every recorded legacy frame must decode with our framing and AMF3 code, or it cannot serve as a fixture.
    #[test]
    fn test_recorded_sessions_decode() {
        let Ok(dir) = std::fs::read_dir(SESSION_DIR) else { return };
        for entry in dir.flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            for line in text.lines().filter(|l| !l.starts_with('#') && !l.ends_with(" policy")) {
                let hex = line.rsplit(' ').next().unwrap();
                let frame = crate::crypt::hex_decode(hex).unwrap();
                let decoded = decode_framed_packet(&frame);
                assert!(decoded.is_ok(), "{}: {:?} in {}", entry.path().display(), decoded.err(), &line[..40.min(line.len())]);
            }
        }
    }

    #[test]
    fn test_proxy_forwards_and_records_both_directions() {
        let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
        let up_addr = upstream.local_addr().unwrap().to_string();
        std::thread::spawn(move || {
            let (mut s, _) = upstream.accept().unwrap();
            let mut buf = vec![0u8; request("user.login").len()];
            s.read_exact(&mut buf).unwrap();
            s.write_all(&request("re.user.login")).unwrap();
        });
        let front = TcpListener::bind("127.0.0.1:0").unwrap();
        let front_addr = front.local_addr().unwrap();
        let dir = std::env::temp_dir().join(format!("kp-record-test-{}", std::process::id()));
        let dir2 = dir.clone();
        let proxy = std::thread::spawn(move || {
            let (c, _) = front.accept().unwrap();
            proxy_connection(c, &up_addr, false, &dir2);
        });

        let mut client = TcpStream::connect(front_addr).unwrap();
        client.write_all(&request("user.login")).unwrap();
        let mut reply = vec![0u8; request("re.user.login").len()];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(reply, request("re.user.login"));
        drop(client);
        proxy.join().unwrap();

        let log_file = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let text = std::fs::read_to_string(&log_file).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(text.contains("# user.login") && text.contains(" c2s "), "{text}");
        assert!(text.contains("# re.user.login") && text.contains(" s2c "), "{text}");
    }
}
