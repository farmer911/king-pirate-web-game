//! Native Rust Legacy Web Portal Backend + React.js Portal Host (`t9-rxhzw-web` / `VUA HAI TAC 2023`).
//!
//! Ground-Truth References:
//! - `legacy-ref/schema/t9-rxhzw-web-schema.sql` (`L26-L65`):
//!   - `account` table (`id AUTO_INCREMENT=202621271`, `name`, `password`, `Scode`, `dj`).
//!   - `server` table (`id=1`, `name='VUA HAI TAC'`, `fqid='1'`, `fcm=1`).
//! - `legacy-ref/www/core/config.php` (`L13-L15`):
//!   - `$fuli = 1000000000` (`1,000,000,000` initial `dj`), `$bili = 10000` (`1 dj = 10,000 gold`).
//! - `legacy-ref/www/core/Cx_Ajax.php` (`L13-L97`):
//!   - `Action=Register`, `Action=Login`, `Action=EditPass`, `Action=FindPass`, `Action=SignOut`.
//! - `legacy-ref/www/index.php`, `legacy-ref/www/member.php`, `legacy-ref/www/app/Game/{index,game}.php`,
//!   `legacy-ref/www/app/dh/{index,ajax}.php`, `legacy-ref/www/{level,fix}.php`.

use crate::data::{ColumnValue, DataStore, RowMap, UpdateOp, WhereCond};
use std::collections::BTreeMap;
use std::path::Path;

/// Initial `dj` balance granted upon account registration (`$fuli = 1000000000` in `legacy-ref/www/core/config.php:14`).
pub const INITIAL_ACCOUNT_DJ: i64 = 1_000_000_000;

/// Gold exchange multiplier (`$bili = 10000` in `legacy-ref/www/core/config.php:15`).
pub const GOLD_EXCHANGE_RATIO: i64 = 10_000;

/// Exact 199-level per-step experience table (`$level_up_exp[2..=200]` in `legacy-ref/www/level.php:17-39`).
pub const LEVEL_STEP_EXP: [i64; 199] = [
    1050, 2800, 5500, 9050, 13350, 18250, 23600, 29400, 35650, 42150, 49000, 56000, 63200, 70600,
    77950, 85350, 92850, 100200, 107550, 114650, 121700, 128500, 135100, 141450, 147550, 153400,
    158900, 164150, 169000, 199700, 233950, 272000, 314050, 360150, 368100, 375450, 382150, 380250,
    377300, 373450, 368750, 371650, 374050, 393650, 413700, 434100, 454750, 475700, 496950, 518350,
    539950, 573150, 607800, 643900, 681600, 720900, 761800, 788500, 815700, 843350, 871450, 900000,
    929050, 958550, 988500, 1018900, 1049800, 1081100, 1112900, 1145150, 1177900, 1211050,
    1244700, 1278800, 1313350, 1348400, 1383850, 1419800, 1456200, 2239550, 2417950, 2548200,
    2698700, 2878900, 2947450, 3016800, 3086950, 3157900, 3229700, 3302250, 3375650, 3449850,
    3524800, 3600650, 3677250, 3754650, 3832850, 3911900, 3991750, 4072350, 4153800, 4236050,
    4319150, 4403000, 4487650, 4573150, 4659450, 4746500, 4834400, 4923100, 5012650, 5102950,
    5194050, 5286000, 5378750, 5472300, 5566650, 5661800, 5757750, 5854500, 5952100, 6050450,
    6149650, 6249650, 6350450, 6452050, 6554450, 6657700, 6761700, 6866550, 6972200, 7078650,
    7185900, 7293950, 7402800, 7512450, 7622950, 7734250, 7846300, 7959200, 8072900, 8187450,
    8302750, 8418850, 8535800, 8653550, 8772050, 8891400, 9011550, 9132550, 9254300, 9376850,
    9500250, 9624450, 9749450, 9875200, 10001850, 10129250, 10257450, 10386500, 10516300,
    10646950, 10778400, 10910650, 11043700, 11177550, 11312250, 11447700, 11584000, 11721050,
    11858950, 11997650, 12137200, 12277500, 12418600, 12560550, 12703250, 12846800, 12991150,
    13136300, 13282250, 13429050, 13576600, 13725000, 13874150, 14024150, 14174950, 14326550,
    14478950, 14632200, 14786200, 14941050, 15096650, 15253100, 15410350, 15568400, 15727250,
    15886950, 16047400,
];

/// Compute cumulative `all_exp` for `target_level` (`1..=200`) matching `calculateAllExp` (`legacy-ref/www/level.php:42-56`).
pub fn calculate_all_exp(target_level: usize) -> Option<i64> {
    if !(1..=200).contains(&target_level) {
        return None;
    }
    let mut total_exp = 0i64;
    for lvl in 2..=target_level {
        total_exp = total_exp.saturating_add(LEVEL_STEP_EXP[lvl - 2]);
    }
    Some(total_exp)
}

/// Ensure default `server` row (`S1 - LouLx-Game - VUA HAI TAC`) exists in `DataStore` (`t9-rxhzw-web-schema.sql:65`).
pub fn ensure_default_server_seeded(store: &DataStore) {
    if store.select_count("t_server", &[]) == 0 {
        let mut row = RowMap::new();
        row.insert("id".to_string(), ColumnValue::Int(1));
        row.insert(
            "name".to_string(),
            ColumnValue::Text("KING PIRATE".to_string()),
        );
        row.insert("fqid".to_string(), ColumnValue::Text("1".to_string()));
        row.insert("fcm".to_string(), ColumnValue::Int(1));
        row.insert(
            "opentime".to_string(),
            ColumnValue::Text("2023-01-01 00:00:00".to_string()),
        );
        store.insert_or_update("t_server", "id", row, true);
    }
}

/// HTTP response returned by a portal route handler.
pub struct PortalHttpResponse {
    pub status_line: &'static str,
    pub content_type: &'static str,
    pub extra_headers: Vec<String>,
    pub body: Vec<u8>,
}

/// Parse URL-encoded query string or `application/x-www-form-urlencoded` body into a `BTreeMap`.
pub fn parse_urlencoded(input: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for pair in input.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        map.insert(url_decode(k), url_decode(v));
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let raw = s.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'+' {
            bytes.push(b' ');
            i += 1;
        } else if raw[i] == b'%' && i + 2 < raw.len() {
            if let (Some(h), Some(l)) = (hex_val(raw[i + 1]), hex_val(raw[i + 2])) {
                bytes.push((h << 4) | l);
                i += 3;
                continue;
            }
            bytes.push(raw[i]);
            i += 1;
        } else {
            bytes.push(raw[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Extract the logged-in `kp_account` session cookie from raw HTTP headers.
pub fn extract_session_account(raw_req: &str) -> Option<String> {
    for line in raw_req.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("cookie:") {
            let cookie_str = line.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
            for part in cookie_str.split(';') {
                let trimmed = part.trim();
                if let Some(val) = trimmed.strip_prefix("kp_account=") {
                    let clean = url_decode(val.trim());
                    if !clean.is_empty() {
                        return Some(clean);
                    }
                }
            }
        }
    }
    None
}

/// Look up an account row in `t_account` by `name`.
pub fn find_account_by_name(store: &DataStore, name: &str) -> Option<RowMap> {
    let rows = store.select(
        "t_account",
        &["id", "name", "password", "Scode", "dj"],
        &[("name", WhereCond::Eq(ColumnValue::Text(name.to_string())))],
        Some(1),
    );
    rows.into_iter().next()
}

/// Embedded React.js Portal Entry HTML (`client-assets/portal/index.html`).
const REACT_PORTAL_SHELL_HTML: &str = r#"<!DOCTYPE html>
<html lang="vi">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate" />
  <meta http-equiv="Pragma" content="no-cache" />
  <meta http-equiv="Expires" content="0" />
  <title>VUA HAI TAC - LOULX GAME</title>
  <link rel="stylesheet" href="/portal/legacy.css" />
</head>
<body class="body_2">
  <div id="root"></div>
  <script type="module" src="/portal/app.js"></script>
</body>
</html>
"#;

/// Serve the React.js portal HTML with cache-clearing headers so Chrome never replays stale `.php` downloads.
fn portal_react_shell(clear_cache: bool, extra_comment: &str) -> PortalHttpResponse {
    let mut extra_headers = Vec::new();
    if clear_cache {
        extra_headers.push(r#"Clear-Site-Data: "cache""#.to_string());
    }
    let html = if extra_comment.is_empty() {
        REACT_PORTAL_SHELL_HTML.to_string()
    } else {
        format!("{REACT_PORTAL_SHELL_HTML}<!-- {extra_comment} -->")
    };
    PortalHttpResponse {
        status_line: "200 OK",
        content_type: "text/html; charset=utf-8",
        extra_headers,
        body: html.into_bytes(),
    }
}

/// Try to handle a legacy web portal or React API route.
pub fn try_handle_portal_route(
    method: &str,
    clean_path: &str,
    query_str: &str,
    raw_req: &str,
    body_str: &str,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> Option<PortalHttpResponse> {
    ensure_default_server_seeded(store);
    let query = parse_urlencoded(query_str);
    let form = parse_urlencoded(body_str);
    let session_user = extract_session_account(raw_req);

    match clean_path {
        "/api/session" => Some(handle_api_session(session_user.as_deref(), store)),
        "/api/level" => Some(handle_level_api(&form, store, persist_path)),
        "/api/fix" => Some(handle_fix_api(&form, store, persist_path)),
        "/core/Cx_Ajax.php" => {
            let action = query
                .get("Action")
                .or_else(|| query.get("type"))
                .map(|s| s.as_str())
                .unwrap_or("");
            Some(handle_cx_ajax(
                action,
                &form,
                session_user.as_deref(),
                store,
                persist_path,
            ))
        }
        "/app/dh/ajax.php" => {
            let action = query
                .get("Action")
                .or_else(|| query.get("type"))
                .map(|s| s.as_str())
                .unwrap_or("");
            Some(handle_dh_ajax(
                action,
                &form,
                session_user.as_deref(),
                store,
                persist_path,
            ))
        }
        "/level.php" => {
            if method.eq_ignore_ascii_case("POST") {
                Some(handle_level_api(&form, store, persist_path))
            } else {
                Some(portal_react_shell(true, "level.php"))
            }
        }
        "/fix.php" => {
            if method.eq_ignore_ascii_case("POST") {
                Some(handle_fix_api(&form, store, persist_path))
            } else {
                Some(portal_react_shell(true, "fix.php"))
            }
        }
        "/app/Game/game.php" => {
            let pid = session_user
                .as_deref()
                .and_then(|u| find_account_by_name(store, u))
                .and_then(|acc| acc.get("id").and_then(|v| v.as_i64()))
                .unwrap_or(202_621_271);
            let comment = format!("iframe:/T9NuoYa/T9NuoYa.html?sid={pid}");
            Some(portal_react_shell(false, &comment))
        }
        "/"
        | "/index.php"
        | "/member.php"
        | "/app/sys/reg.php"
        | "/app/sys/login.php"
        | "/app/sys/editpass.php"
        | "/app/sys/findpass.php"
        | "/app/Game/index.php"
        | "/app/dh/index.php" => Some(portal_react_shell(true, clean_path)),
        _ => None,
    }
}

fn json_ok(json: String, extra_headers: Vec<String>) -> PortalHttpResponse {
    PortalHttpResponse {
        status_line: "200 OK",
        content_type: "application/json; charset=utf-8",
        extra_headers,
        body: json.into_bytes(),
    }
}

/// Return current logged-in session and character list for the React.js portal (`GET /api/session`).
pub fn handle_api_session(session_user: Option<&str>, store: &DataStore) -> PortalHttpResponse {
    let Some(uname) = session_user else {
        return json_ok(
            r#"{"loggedIn":false,"id":0,"accountName":"","dj":0,"characters":[]}"#.to_string(),
            vec![],
        );
    };
    let Some(acc) = find_account_by_name(store, uname) else {
        return json_ok(
            r#"{"loggedIn":false,"id":0,"accountName":"","dj":0,"characters":[]}"#.to_string(),
            vec![],
        );
    };
    let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let dj = acc.get("dj").and_then(|v| v.as_i64()).unwrap_or(0);
    let users = store.select(
        "t_user",
        &[
            "uid",
            "uname",
            "utid",
            "level",
            "gold_num",
            "belly_num",
            "vip",
        ],
        &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
        Some(10),
    );
    let mut chars_json = Vec::with_capacity(users.len());
    for u in users {
        let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);
        let char_name = match u.get("uname") {
            Some(ColumnValue::Text(s)) => s.as_str(),
            _ => "",
        };
        let utid = u.get("utid").and_then(|v| v.as_i64()).unwrap_or(1);
        let level = u.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
        let gold_num = u.get("gold_num").and_then(|v| v.as_i64()).unwrap_or(0);
        let belly_num = u.get("belly_num").and_then(|v| v.as_i64()).unwrap_or(0);
        let vip = u.get("vip").and_then(|v| v.as_i64()).unwrap_or(0);
        chars_json.push(format!(
            r#"{{"uid":{uid},"uname":"{}","utid":{utid},"level":{level},"gold_num":{gold_num},"belly_num":{belly_num},"vip":{vip}}}"#,
            json_escape(char_name)
        ));
    }
    let body = format!(
        r#"{{"loggedIn":true,"id":{pid},"accountName":"{}","dj":{dj},"characters":[{}]}}"#,
        json_escape(uname),
        chars_json.join(",")
    );
    json_ok(body, vec![])
}

/// Handle `legacy-ref/www/core/Cx_Ajax.php` actions (`Register`, `Login`, `EditPass`, `FindPass`, `SignOut`).
pub fn handle_cx_ajax(
    action: &str,
    form: &BTreeMap<String, String>,
    session_user: Option<&str>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    match action {
        "Register" | "reg" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            if uname.len() < 4 || upass.len() < 4 || scode.len() < 4 {
                return json_ok(
                    r#"{"info":"Vui l\u00f2ng nh\u1eadp t\u00e0i kho\u1ea3n, m\u1eadt kh\u1ea9u v\u00e0 m\u00e3 b\u1ea3o m\u1eadt (t\u1ed1i thi\u1ec3u 4 k\u00fd t\u1ef1)!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            if find_account_by_name(store, uname).is_some() {
                return json_ok(
                    r#"{"info":"T\u00e0i kho\u1ea3n \u0111\u00e3 \u0111\u01b0\u1ee3c \u0111\u0103ng k\u00fd, vui l\u00f2ng \u0111\u1ed5i sang ng\u01b0\u1eddi d\u00f9ng kh\u00e1c","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            let pid = store.id_gen.next_id("pid", 1) as i64;
            let mut row = RowMap::new();
            row.insert("id".to_string(), ColumnValue::Int(pid));
            row.insert("name".to_string(), ColumnValue::Text(uname.to_string()));
            row.insert("password".to_string(), ColumnValue::Text(upass.to_string()));
            row.insert("Scode".to_string(), ColumnValue::Text(scode.to_string()));
            row.insert("dj".to_string(), ColumnValue::Int(INITIAL_ACCOUNT_DJ));
            store.insert_or_update("t_account", "id", row, false);
            if let Some(p) = persist_path {
                let _ = store.save_to_disk(p);
            }
            json_ok(
                r#"{"info":"\u0110\u0103ng k\u00fd th\u00e0nh c\u00f4ng, chuy\u1ec3n sang trung t\u00e2m th\u00e0nh vi\u00ean!!","status":"y","url":"/member.php"}"#.to_string(),
                vec![
                    format!("Set-Cookie: kp_account={uname}; Path=/; SameSite=Lax"),
                    r#"Clear-Site-Data: "cache""#.to_string(),
                ],
            )
        }
        "Login" | "login" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"T\u00e0i kho\u1ea3n kh\u00f4ng t\u1ed3n t\u1ea1i ho\u1eb7c m\u1eadt kh\u1ea9u sai","status":"n","url":"index.php"}"#.to_string(),
                    vec![],
                );
            };
            let db_pass = match acc.get("password") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_pass != upass {
                return json_ok(
                    r#"{"info":"T\u00e0i kho\u1ea3n kh\u00f4ng t\u1ed3n t\u1ea1i ho\u1eb7c m\u1eadt kh\u1ea9u sai","status":"n","url":"index.php"}"#.to_string(),
                    vec![],
                );
            }
            json_ok(
                r#"{"info":"\u0110\u0103ng nh\u1eadp th\u00e0nh c\u00f4ng, chuy\u1ec3n \u0111\u1ebfn trung t\u00e2m th\u00e0nh vi\u00ean!!","status":"y","url":"/member.php"}"#.to_string(),
                vec![
                    format!("Set-Cookie: kp_account={uname}; Path=/; SameSite=Lax"),
                    r#"Clear-Site-Data: "cache""#.to_string(),
                ],
            )
        }
        "EditPass" | "editpass" => {
            let uname = session_user
                .or_else(|| form.get("Uname").map(|s| s.trim()))
                .unwrap_or("");
            if uname.is_empty() {
                return json_ok(
                    r#"{"info":"Phi\u00ean \u0111\u0103ng nh\u1eadp \u0111\u00e3 h\u1ebft h\u1ea1n!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"T\u00e0i kho\u1ea3n kh\u00f4ng t\u1ed3n t\u1ea1i!","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let db_scode = match acc.get("Scode") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_scode != scode {
                return json_ok(
                    r#"{"info":"M\u00e3 b\u1ea3o m\u1eadt kh\u00f4ng ch\u00ednh x\u00e1c v\u00e0 kh\u00f4ng th\u1ec3 s\u1eeda \u0111\u1ed5i \u0111\u01b0\u1ee3c.","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            store.update(
                "t_account",
                &[("password", UpdateOp::Set(ColumnValue::Text(upass.to_string())))],
                &[("name", WhereCond::Eq(ColumnValue::Text(uname.to_string())))],
            );
            if let Some(p) = persist_path {
                let _ = store.save_to_disk(p);
            }
            json_ok(
                r#"{"info":"\u0110\u00e3 \u0111\u1ed5i m\u1eadt kh\u1ea9u th\u00e0nh c\u00f4ng, vui l\u00f2ng \u0111\u0103ng nh\u1eadp l\u1ea1i","status":"y","url":"/member.php"}"#.to_string(),
                vec![],
            )
        }
        "FindPass" | "findpass" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"M\u00e3 b\u1ea3o m\u1eadt kh\u00f4ng ch\u00ednh x\u00e1c v\u00e0 kh\u00f4ng th\u1ec3 s\u1eeda \u0111\u1ed5i \u0111\u01b0\u1ee3c.","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let db_scode = match acc.get("Scode") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_scode != scode {
                return json_ok(
                    r#"{"info":"M\u00e3 b\u1ea3o m\u1eadt kh\u00f4ng ch\u00ednh x\u00e1c v\u00e0 kh\u00f4ng th\u1ec3 s\u1eeda \u0111\u1ed5i \u0111\u01b0\u1ee3c.","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            store.update(
                "t_account",
                &[("password", UpdateOp::Set(ColumnValue::Text(upass.to_string())))],
                &[("name", WhereCond::Eq(ColumnValue::Text(uname.to_string())))],
            );
            if let Some(p) = persist_path {
                let _ = store.save_to_disk(p);
            }
            json_ok(
                r#"{"info":"\u0110\u00e3 \u0111\u1ed5i m\u1eadt kh\u1ea9u th\u00e0nh c\u00f4ng, vui l\u00f2ng \u0111\u0103ng nh\u1eadp l\u1ea1i","status":"y","url":"/member.php"}"#.to_string(),
                vec![],
            )
        }
        "SignOut" | "signout" => json_ok(
            r#"{"info":"Ng\u01b0\u1eddi d\u00f9ng \u0111\u00e3 tho\u00e1t kh\u1ecfi h\u1ec7 th\u1ed1ng th\u00e0nh c\u00f4ng!","status":"y","url":"/index.php"}"#.to_string(),
            vec![
                "Set-Cookie: kp_account=; Path=/; Max-Age=0".to_string(),
                r#"Clear-Site-Data: "cache""#.to_string(),
            ],
        ),
        _ => json_ok(
            r#"{"info":"Invalid action!","status":"n"}"#.to_string(),
            vec![],
        ),
    }
}

/// Handle `legacy-ref/www/app/dh/ajax.php` (`Get_GameUser`, `GoShop`, `DHYB`).
pub fn handle_dh_ajax(
    action: &str,
    form: &BTreeMap<String, String>,
    session_user: Option<&str>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let Some(uname) = session_user else {
        return json_ok(
            r#"{"info":"Vui l\u00f2ng \u0111\u0103ng nh\u1eadp tr\u01b0\u1edbc!","status":"n"}"#.to_string(),
            vec![],
        );
    };
    let Some(acc) = find_account_by_name(store, uname) else {
        return json_ok(
            r#"{"info":"T\u00e0i kho\u1ea3n kh\u00f4ng t\u1ed3n t\u1ea1i!","status":"n"}"#.to_string(),
            vec![],
        );
    };
    let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let dj = acc.get("dj").and_then(|v| v.as_i64()).unwrap_or(0);

    match action {
        "Get_GameUser" | "GoShop" => {
            let users = store.select(
                "t_user",
                &["uid", "uname"],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                Some(10),
            );
            if users.is_empty() {
                return json_ok(
                    r#"{"info":"Ch\u01b0a c\u00f3 nh\u00e2n v\u1eadt trong m\u00e1y ch\u1ee7 n\u00e0y. Vui l\u00f2ng v\u00e0o game t\u1ea1o nh\u00e2n v\u1eadt tr\u01b0\u1edbc!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            let mut html = String::new();
            for u in users {
                let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);
                let char_name = match u.get("uname") {
                    Some(ColumnValue::Text(s)) => s.as_str(),
                    _ => uname,
                };
                html.push_str(&format!(
                    "<label class='button margin-top'><input name='Rid' value='{uid}' type='radio'> {}</label>",
                    json_escape(char_name)
                ));
            }
            json_ok(
                format!(
                    r#"{{"info":"T\u00ecm th\u1ea5y nh\u00e2n v\u1eadt!","status":"y","html":"{}"}}"#,
                    json_escape(&html)
                ),
                vec![],
            )
        }
        "DHYB" => {
            let dh_num = form
                .get("U_DH_Num")
                .or_else(|| form.get("DH_Num"))
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);
            if dh_num <= 0 {
                return json_ok(
                    r#"{"info":"\u0110\u00e3 nh\u1eadp sai s\u1ed1","status":"n","url":"member.php"}"#.to_string(),
                    vec![],
                );
            }
            if dh_num > dj {
                return json_ok(
                    r#"{"info":"Trao \u0111\u1ed5i kh\u00f4ng th\u00e0nh c\u00f4ng do kh\u00f4ng \u0111\u1ee7 ti\u1ec1n v\u00e0ng!!","status":"n","url":"member.php"}"#.to_string(),
                    vec![],
                );
            }
            let requested_uid = form
                .get("Rid")
                .or_else(|| form.get("User_ID"))
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);

            let add_gold = dh_num.saturating_mul(GOLD_EXCHANGE_RATIO);
            let mut updated = 0;
            if requested_uid > 0 {
                updated = store.update(
                    "t_user",
                    &[("gold_num", UpdateOp::Inc(add_gold))],
                    &[
                        ("uid", WhereCond::Eq(ColumnValue::Int(requested_uid))),
                        ("pid", WhereCond::Eq(ColumnValue::Int(pid))),
                    ],
                );
            }
            if updated == 0 {
                updated = store.update(
                    "t_user",
                    &[("gold_num", UpdateOp::Inc(add_gold))],
                    &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                );
            }
            if updated == 0 {
                return json_ok(
                    r#"{"info":"Kh\u00f4ng t\u00ecm th\u1ea5y nh\u00e2n v\u1eadt trong m\u00e1y ch\u1ee7!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            store.update(
                "t_account",
                &[("dj", UpdateOp::Dec(dh_num))],
                &[("id", WhereCond::Eq(ColumnValue::Int(pid)))],
            );
            if let Some(p) = persist_path {
                let _ = store.save_to_disk(p);
            }
            json_ok(
                format!(
                    r#"{{"info":"\u0110\u1ed5i th\u01b0\u1edfng th\u00e0nh c\u00f4ng (+{add_gold} V\u00e0ng)!","status":"y","url":"/member.php"}}"#
                ),
                vec![],
            )
        }
        _ => json_ok(
            r#"{"info":"Invalid action!","status":"n"}"#.to_string(),
            vec![],
        ),
    }
}

/// Handle `POST /api/level` & `POST /level.php` (`legacy-ref/www/level.php:79-145`).
pub fn handle_level_api(
    form: &BTreeMap<String, String>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let char_or_acc = form
        .get("uname")
        .or_else(|| form.get("account"))
        .map(|s| s.trim())
        .unwrap_or("");
    let lv = form
        .get("level")
        .or_else(|| form.get("lv"))
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let logout_confirmed = form
        .get("logout_confirm")
        .map(|s| s == "on" || s == "true" || s == "1")
        .unwrap_or(true);

    if char_or_acc.is_empty() {
        return json_ok(
            r#"{"info":"Vui l\u00f2ng nh\u1eadp t\u00ean nh\u00e2n v\u1eadt","status":"n"}"#.to_string(),
            vec![],
        );
    }
    let Some(all_exp) = calculate_all_exp(lv) else {
        return json_ok(
            r#"{"info":"Vui l\u00f2ng nh\u1eadp level h\u1ee3p l\u1ec7 (1-200)","status":"n"}"#.to_string(),
            vec![],
        );
    };
    if !logout_confirmed {
        return json_ok(
            r#"{"info":"Vui l\u00f2ng x\u00e1c nh\u1eadn \u0111\u00e3 tho\u00e1t game","status":"n"}"#.to_string(),
            vec![],
        );
    }

    // Lookup character by `uname` first (`legacy-ref/www/level.php:102`), or fallback to account `name`.
    let mut users = store.select(
        "t_user",
        &["uid", "uname"],
        &[(
            "uname",
            WhereCond::Eq(ColumnValue::Text(char_or_acc.to_string())),
        )],
        Some(1),
    );
    if users.is_empty() {
        if let Some(acc) = find_account_by_name(store, char_or_acc) {
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            users = store.select(
                "t_user",
                &["uid", "uname"],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                Some(1),
            );
        }
    }

    let Some(u) = users.first() else {
        return json_ok(
            format!(
                r#"{{"info":"Kh\u00f4ng t\u00ecm th\u1ea5y nh\u00e2n v\u1eadt {}","status":"n"}}"#,
                json_escape(char_or_acc)
            ),
            vec![],
        );
    };
    let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);

    store.update(
        "t_user",
        &[("level", UpdateOp::Set(ColumnValue::Int(lv as i64)))],
        &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
    );
    store.update(
        "t_hero",
        &[
            ("level", UpdateOp::Set(ColumnValue::Int(lv as i64))),
            ("all_exp", UpdateOp::Set(ColumnValue::Int(all_exp))),
        ],
        &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
    );
    if let Some(p) = persist_path {
        let _ = store.save_to_disk(p);
    }
    json_ok(
        format!(
            r#"{{"info":"C\u1eadp nh\u1eadt th\u00e0nh c\u00f4ng cho nh\u00e2n v\u1eadt {} (UID: {uid}, Level: {lv}, all_exp: {all_exp})","status":"y"}}"#,
            json_escape(char_or_acc)
        ),
        vec![],
    )
}

/// Handle `POST /api/fix` & `POST /fix.php` (`legacy-ref/www/fix.php:22-104`).
pub fn handle_fix_api(
    form: &BTreeMap<String, String>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let form_type = form
        .get("form_type")
        .map(|s| s.trim())
        .unwrap_or("blood");

    let target_name = if form_type == "hero" {
        form.get("uname_hero")
            .or_else(|| form.get("account"))
            .map(|s| s.trim())
            .unwrap_or("")
    } else {
        form.get("uname_blood")
            .or_else(|| form.get("account"))
            .map(|s| s.trim())
            .unwrap_or("")
    };

    if target_name.is_empty() {
        return json_ok(
            r#"{"info":"Vui l\u00f2ng nh\u1eadp t\u00ean nh\u00e2n v\u1eadt","status":"n"}"#.to_string(),
            vec![],
        );
    }

    let mut users = store.select(
        "t_user",
        &["uid", "uname"],
        &[(
            "uname",
            WhereCond::Eq(ColumnValue::Text(target_name.to_string())),
        )],
        Some(1),
    );
    if users.is_empty() {
        if let Some(acc) = find_account_by_name(store, target_name) {
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            users = store.select(
                "t_user",
                &["uid", "uname"],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                Some(1),
            );
        }
    }

    let Some(u) = users.first() else {
        return json_ok(
            format!(
                r#"{{"info":"Kh\u00f4ng t\u00ecm th\u1ea5y nh\u00e2n v\u1eadt {}","status":"n"}}"#,
                json_escape(target_name)
            ),
            vec![],
        );
    };
    let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);

    if form_type == "hero" {
        store.update(
            "t_user",
            &[("level", UpdateOp::Set(ColumnValue::Int(10)))],
            &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
        );
        store.update(
            "t_hero",
            &[
                ("level", UpdateOp::Set(ColumnValue::Int(10))),
                ("exp", UpdateOp::Set(ColumnValue::Int(0))),
                ("all_exp", UpdateOp::Set(ColumnValue::Int(0))),
            ],
            &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
        );
        if let Some(p) = persist_path {
            let _ = store.save_to_disk(p);
        }
        json_ok(
            format!(
                r#"{{"info":"C\u1eadp nh\u1eadt Hero th\u00e0nh c\u00f4ng cho {} (UID: {uid})","status":"y"}}"#,
                json_escape(target_name)
            ),
            vec![],
        )
    } else {
        store.update(
            "t_user",
            &[("blood_package", UpdateOp::Set(ColumnValue::Int(0)))],
            &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
        );
        if let Some(p) = persist_path {
            let _ = store.save_to_disk(p);
        }
        json_ok(
            format!(
                r#"{{"info":"C\u1eadp nh\u1eadt Blood Package th\u00e0nh c\u00f4ng cho {} (UID: {uid})","status":"y"}}"#,
                json_escape(target_name)
            ),
            vec![],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::SqlDialect;

    #[test]
    fn test_legacy_portal_register_login_member_server_and_gold_exchange() {
        let store = DataStore::new(SqlDialect::SqliteWal);
        ensure_default_server_seeded(&store);

        // 1. Register a new account via `Cx_Ajax.php?Action=Register`
        let mut reg_form = BTreeMap::new();
        reg_form.insert("Uname".to_string(), "luffy2026".to_string());
        reg_form.insert("Upass".to_string(), "secret123".to_string());
        reg_form.insert("Scode".to_string(), "654321".to_string());
        let reg_resp = handle_cx_ajax("Register", &reg_form, None, &store, None);
        let reg_body = String::from_utf8(reg_resp.body).unwrap();
        assert!(reg_body.contains(r#""status":"y""#));
        assert!(reg_resp
            .extra_headers
            .iter()
            .any(|h| h.contains("kp_account=luffy2026")));
        assert!(reg_resp
            .extra_headers
            .iter()
            .any(|h| h.contains(r#"Clear-Site-Data: "cache""#)));

        // Verify `account.id` starts at `202621271` (`t9-rxhzw-web-schema.sql:37`) and `dj = 1_000_000_000`
        let acc = find_account_by_name(&store, "luffy2026").expect("account must exist");
        let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap();
        assert_eq!(pid, 202_621_271);
        assert_eq!(
            acc.get("dj").and_then(|v| v.as_i64()),
            Some(INITIAL_ACCOUNT_DJ)
        );

        // 2. Login via `Cx_Ajax.php?Action=Login`
        let login_resp = handle_cx_ajax("Login", &reg_form, None, &store, None);
        let login_body = String::from_utf8(login_resp.body).unwrap();
        assert!(login_body.contains(r#""status":"y""#));

        // 3. `/app/Game/game.php?FQ=1` includes the player's `sid=202621271`
        let game_page = try_handle_portal_route(
            "GET",
            "/app/Game/game.php",
            "FQ=1",
            "GET /app/Game/game.php?FQ=1 HTTP/1.1\r\nCookie: kp_account=luffy2026\r\n\r\n",
            "",
            &store,
            None,
        )
        .expect("expected game.php response");
        let game_html = String::from_utf8(game_page.body).unwrap();
        assert!(game_html.contains("/T9NuoYa/T9NuoYa.html?sid=202621271"));

        // 4. Verify `calculate_all_exp` matches `legacy-ref/www/level.php`
        assert_eq!(calculate_all_exp(1), Some(0));
        assert_eq!(calculate_all_exp(2), Some(1050));
        assert_eq!(calculate_all_exp(3), Some(3850));
        assert!(calculate_all_exp(200).unwrap() > 1_000_000_000);
    }
}
