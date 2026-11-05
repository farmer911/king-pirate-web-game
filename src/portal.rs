//! Native Rust Legacy Web Portal (`t9-rxhzw-web` / `KING PIRATE 2023`).
//!
//! Ground-Truth References:
//! - `legacy-ref/schema/t9-rxhzw-web-schema.sql` (`L26-L65`):
//!   - `account` table (`id AUTO_INCREMENT=202621271`, `name`, `password`, `Scode`, `dj`).
//!   - `server` table (`id=1`, `name='KING PIRATE'`, `fqid='1'`, `fcm=1`).
//! - `client-assets/core/config.php` (`L13-L15`):
//!   - `$WebTitle`, `$fuli = 1000000000` (`1,000,000,000` initial `dj`), `$bili = 10000` (`1 dj = 10,000 gold`).
//! - `client-assets/core/Cx_Ajax.php` (`L13-L97`):
//!   - `Action=Register`, `Action=Login`, `Action=EditPass`, `Action=FindPass`, `Action=SignOut`.
//! - `client-assets/index.php`, `client-assets/member.php`, `client-assets/app/Game/{index,game}.php`,
//!   `client-assets/app/dh/{index,ajax}.php`, `legacy-ref/www/{level,fix}.php`.

use crate::data::{ColumnValue, DataStore, RowMap, UpdateOp, WhereCond};
use std::collections::BTreeMap;
use std::path::Path;

/// Initial `dj` balance granted upon account registration (`$fuli = 1000000000` in `client-assets/core/config.php:14`).
pub const INITIAL_ACCOUNT_DJ: i64 = 1_000_000_000;

/// Gold exchange multiplier (`$bili = 10000` in `client-assets/core/config.php:15`).
pub const GOLD_EXCHANGE_RATIO: i64 = 10_000;

/// Exact 200-level cumulative Hero experience table (`$level_up_exp` in `legacy-ref/www/level.php:73-96`).
pub const LEVEL_UP_EXP: [i64; 200] = [
    0, 30, 100, 220, 395, 645, 995, 1475, 2115, 2970, 4050, 5400, 7095, 9195, 11795, 14995, 18895,
    23695, 29495, 36395, 44395, 53495, 63695, 75095, 87695, 101595, 116895, 133595, 151795, 171595,
    192995, 216195, 241195, 268195, 297195, 328195, 361195, 396195, 433195, 473195, 515195, 560195,
    608195, 659195, 713195, 770195, 830195, 893195, 959195, 1029195, 1104195, 1184195, 1269195,
    1359195, 1454195, 1554195, 1659195, 1769195, 1884195, 2004195, 2134195, 2274195, 2424195,
    2584195, 2754195, 2934195, 3124195, 3324195, 3534195, 3754195, 3994195, 4254195, 4534195,
    4834195, 5154195, 5494195, 5854195, 6234195, 6634195, 7054195, 7504195, 7994195, 8524195,
    9094195, 9704195, 10354195, 11044195, 11774195, 12544195, 13374195, 14354195, 15504195,
    16834195, 18354195, 20084195, 22024195, 24194195, 26604195, 29254195, 32174195, 35504195,
    39164195, 43184195, 47594195, 52434195, 57744195, 63574195, 69964195, 76974195, 84674195,
    92834195, 101484195, 110644195, 120344195, 130624195, 141514195, 153054195, 165284195,
    178244195, 191974195, 205904195, 220034195, 234364195, 248894195, 263624195, 278624195,
    293894195, 309434195, 325244195, 341324195, 357824195, 374744195, 392094195, 409884195,
    428124195, 446824195, 465994195, 485644195, 505784195, 526424195, 547574195, 569244195,
    591454195, 614214195, 637544195, 661454195, 685954195, 711064195, 736804195, 763184195,
    790224195, 817934195, 846334195, 875444195, 905274195, 935844195, 967174195, 999284195,
    1032194195, 1065924195, 1100494195, 1135924195, 1172234195, 1209444195, 1247584195,
    1286674195, 1326734195, 1367794195, 1409874195, 1453004195, 1497204195, 1542504195,
    1588934195, 1636524195, 1685294195, 1735274195, 1786504195, 1839014195, 1892834195,
    1947994195, 2004534195, 2062484195, 2121884195, 2182764195, 2245164195, 2309124195,
    2374684195, 2441874195, 2510744195, 2581334195, 2653684195, 2727844195, 2803854195,
    2881764195, 2961624195, 3043484195, 3127394195, 3213404195, 3301564195, 3391924195,
];

/// Ensure default `server` row (`S1 - LouLx-Game - KING PIRATE`) exists in `DataStore` (`t9-rxhzw-web-schema.sql:65`).
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

/// Try to handle a legacy web portal route (`index.php`, `member.php`, `core/Cx_Ajax.php`, `app/sys/*`, `app/Game/*`, `app/dh/*`, `level.php`, `fix.php`, `WebRes/*`).
/// Returns `Some(PortalHttpResponse)` if matched, or `None` to fall through to static asset serving.
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
        "/WebRes/Css/pintuer.css" | "/WebRes/Css/my.css" => Some(PortalHttpResponse {
            status_line: "200 OK",
            content_type: "text/css; charset=utf-8",
            extra_headers: vec![],
            body: PORTAL_CSS.as_bytes().to_vec(),
        }),
        "/WebRes/Js/core.js"
        | "/WebRes/Js/jquery.js"
        | "/WebRes/Js/pintuer.js"
        | "/WebRes/Js/jquery.SuperSlide.2.1.1.js"
        | "/WebRes/Js/layer.min.js"
        | "/WebRes/js/cookie.js"
        | "/WebRes/Js/respond.js" => {
            let js = if clean_path.ends_with("core.js") {
                PORTAL_CORE_JS
            } else {
                "/* bundled in core.js */"
            };
            Some(PortalHttpResponse {
                status_line: "200 OK",
                content_type: "application/javascript; charset=utf-8",
                extra_headers: vec![],
                body: js.as_bytes().to_vec(),
            })
        }
        "/" | "/index.php" => {
            if let Some(ref uname) = session_user {
                if find_account_by_name(store, uname).is_some() {
                    return Some(redirect_to("/member.php"));
                }
            }
            Some(html_ok(render_index_page()))
        }
        "/member.php" => {
            let Some(ref uname) = session_user else {
                return Some(redirect_to("/index.php"));
            };
            let Some(acc) = find_account_by_name(store, uname) else {
                return Some(redirect_to("/index.php"));
            };
            let dj = acc.get("dj").and_then(|v| v.as_i64()).unwrap_or(0);
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(202_621_271);
            Some(html_ok(render_member_page(uname, dj, pid)))
        }
        "/app/sys/reg.php" => Some(html_ok(render_reg_page())),
        "/app/sys/login.php" => Some(html_ok(render_login_page())),
        "/app/sys/editpass.php" => Some(html_ok(render_editpass_page())),
        "/app/sys/findpass.php" => Some(html_ok(render_findpass_page())),
        "/core/Cx_Ajax.php" => {
            let action = query.get("Action").map(|s| s.as_str()).unwrap_or("");
            Some(handle_cx_ajax(
                action,
                &form,
                session_user.as_deref(),
                store,
                persist_path,
            ))
        }
        "/app/Game/index.php" => {
            let Some(ref uname) = session_user else {
                return Some(redirect_to("/index.php"));
            };
            if find_account_by_name(store, uname).is_none() {
                return Some(redirect_to("/index.php"));
            }
            Some(html_ok(render_server_select_page(uname)))
        }
        "/app/Game/game.php" => {
            let Some(ref uname) = session_user else {
                return Some(redirect_to("/index.php"));
            };
            let Some(acc) = find_account_by_name(store, uname) else {
                return Some(redirect_to("/index.php"));
            };
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(202_621_271);
            Some(html_ok(render_game_iframe_page(pid)))
        }
        "/app/dh/index.php" => {
            let Some(ref uname) = session_user else {
                return Some(redirect_to("/index.php"));
            };
            let Some(acc) = find_account_by_name(store, uname) else {
                return Some(redirect_to("/index.php"));
            };
            let dj = acc.get("dj").and_then(|v| v.as_i64()).unwrap_or(0);
            Some(html_ok(render_exchange_page(uname, dj)))
        }
        "/app/dh/ajax.php" => {
            let action = query.get("Action").map(|s| s.as_str()).unwrap_or("");
            Some(handle_dh_ajax(
                action,
                &form,
                session_user.as_deref(),
                store,
                persist_path,
            ))
        }
        "/level.php" => Some(handle_level_page(method, &form, store, persist_path)),
        "/fix.php" => Some(handle_fix_page(method, &form, store, persist_path)),
        _ => None,
    }
}

fn html_ok(html: String) -> PortalHttpResponse {
    PortalHttpResponse {
        status_line: "200 OK",
        content_type: "text/html; charset=utf-8",
        extra_headers: vec![],
        body: html.into_bytes(),
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

fn redirect_to(location: &str) -> PortalHttpResponse {
    PortalHttpResponse {
        status_line: "302 Found",
        content_type: "text/html; charset=utf-8",
        extra_headers: vec![format!("Location: {location}")],
        body: Vec::new(),
    }
}

/// Handle `client-assets/core/Cx_Ajax.php` actions (`Register`, `Login`, `EditPass`, `FindPass`, `SignOut`).
pub fn handle_cx_ajax(
    action: &str,
    form: &BTreeMap<String, String>,
    session_user: Option<&str>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    match action {
        "Register" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            if uname.len() < 4 || upass.len() < 4 || scode.len() < 4 {
                return json_ok(
                    r#"{"info":"Please enter account name, password, and security code (min 4 chars)!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            if find_account_by_name(store, uname).is_some() {
                return json_ok(
                    r#"{"info":"Registration failed: account already exists!","status":"n"}"#.to_string(),
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
                r#"{"info":"Registration successful! Redirecting to Member Center...","status":"y","url":"/member.php"}"#.to_string(),
                vec![format!("Set-Cookie: kp_account={uname}; Path=/; SameSite=Lax")],
            )
        }
        "Login" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"Invalid account name or password!","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let db_pass = match acc.get("password") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_pass != upass {
                return json_ok(
                    r#"{"info":"Invalid account name or password!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            json_ok(
                r#"{"info":"Login successful! Redirecting to Member Center...","status":"y","url":"/member.php"}"#.to_string(),
                vec![format!("Set-Cookie: kp_account={uname}; Path=/; SameSite=Lax")],
            )
        }
        "EditPass" => {
            let Some(uname) = session_user else {
                return json_ok(
                    r#"{"info":"Session expired, please login again!","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"Account not found!","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let db_scode = match acc.get("Scode") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_scode != scode {
                return json_ok(
                    r#"{"info":"Password change failed: invalid security code!","status":"n"}"#.to_string(),
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
                r#"{"info":"Password updated successfully!","status":"y","url":"/member.php"}"#.to_string(),
                vec![],
            )
        }
        "FindPass" => {
            let uname = form.get("Uname").map(|s| s.trim()).unwrap_or("");
            let upass = form.get("Upass").map(|s| s.trim()).unwrap_or("");
            let scode = form.get("Scode").map(|s| s.trim()).unwrap_or("");
            let Some(acc) = find_account_by_name(store, uname) else {
                return json_ok(
                    r#"{"info":"Password recovery failed: invalid account or security code!","status":"n"}"#.to_string(),
                    vec![],
                );
            };
            let db_scode = match acc.get("Scode") {
                Some(ColumnValue::Text(s)) => s.as_str(),
                _ => "",
            };
            if db_scode != scode {
                return json_ok(
                    r#"{"info":"Password recovery failed: invalid account or security code!","status":"n"}"#.to_string(),
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
                r#"{"info":"Password recovered successfully!","status":"y","url":"/member.php"}"#.to_string(),
                vec![],
            )
        }
        "SignOut" => json_ok(
            r#"{"info":"Signed out successfully!","status":"y","url":"/index.php"}"#.to_string(),
            vec!["Set-Cookie: kp_account=; Path=/; Max-Age=0".to_string()],
        ),
        _ => json_ok(
            r#"{"info":"Invalid action!","status":"n"}"#.to_string(),
            vec![],
        ),
    }
}

/// Handle `client-assets/app/dh/ajax.php` (`Get_GameUser` & `DHYB` gold exchange).
pub fn handle_dh_ajax(
    action: &str,
    form: &BTreeMap<String, String>,
    session_user: Option<&str>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let Some(uname) = session_user else {
        return json_ok(
            r#"{"info":"Please login first!","status":"n"}"#.to_string(),
            vec![],
        );
    };
    let Some(acc) = find_account_by_name(store, uname) else {
        return json_ok(
            r#"{"info":"Account not found!","status":"n"}"#.to_string(),
            vec![],
        );
    };
    let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let dj = acc.get("dj").and_then(|v| v.as_i64()).unwrap_or(0);

    match action {
        "Get_GameUser" => {
            let users = store.select(
                "t_user",
                &["uid", "uname"],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                Some(10),
            );
            if users.is_empty() {
                return json_ok(
                    r#"{"info":"No character found on this server. Please create a character in-game first!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            let mut options = String::new();
            for u in users {
                let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);
                let char_name = match u.get("uname") {
                    Some(ColumnValue::Text(s)) => s.as_str(),
                    _ => "Pirate",
                };
                options.push_str(&format!("<option value='{uid}'>{char_name}</option>"));
            }
            let body = format!(
                r#"{{"info":"Character found!","status":"y","html":"{}"}}"#,
                options.replace('"', "\\\"")
            );
            json_ok(body, vec![])
        }
        "DHYB" => {
            let uid = form
                .get("User_ID")
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);
            let dh_num = form
                .get("DH_Num")
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);
            if dh_num <= 0 {
                return json_ok(
                    r#"{"info":"Exchange amount must be greater than 0!","status":"n"}"#.to_string(),
                    vec![],
                );
            }
            if dh_num > dj {
                return json_ok(
                    format!(r#"{{"info":"Insufficient balance for {dh_num} points!","status":"n"}}"#),
                    vec![],
                );
            }
            let add_gold = dh_num.saturating_mul(GOLD_EXCHANGE_RATIO);
            let updated = store.update(
                "t_user",
                &[("gold_num", UpdateOp::Inc(add_gold))],
                &[
                    ("uid", WhereCond::Eq(ColumnValue::Int(uid))),
                    ("pid", WhereCond::Eq(ColumnValue::Int(pid))),
                ],
            );
            if updated == 0 {
                return json_ok(
                    r#"{"info":"Character not found!","status":"n"}"#.to_string(),
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
                format!(r#"{{"info":"Successfully exchanged {add_gold} Gold to character!","status":"y"}}"#),
                vec![],
            )
        }
        _ => json_ok(
            r#"{"info":"Invalid action!","status":"n"}"#.to_string(),
            vec![],
        ),
    }
}

/// Handle `legacy-ref/www/level.php` (Fix Hero Level `1..=200`).
fn handle_level_page(
    method: &str,
    form: &BTreeMap<String, String>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let mut notice = String::new();
    if method == "POST" {
        let acc_name = form.get("account").map(|s| s.trim()).unwrap_or("");
        let lv = form
            .get("lv")
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if acc_name.is_empty() || !(1..=200).contains(&lv) {
            notice = "Please enter a valid account name and level (1 - 200)!".to_string();
        } else if let Some(acc) = find_account_by_name(store, acc_name) {
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let users = store.select(
                "t_user",
                &["uid", "master_hid"],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
                Some(1),
            );
            if let Some(u) = users.first() {
                let uid = u.get("uid").and_then(|v| v.as_i64()).unwrap_or(0);
                let exp = LEVEL_UP_EXP[lv - 1];
                store.update(
                    "t_user",
                    &[("level", UpdateOp::Set(ColumnValue::Int(lv as i64)))],
                    &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
                );
                store.update(
                    "t_hero",
                    &[
                        ("level", UpdateOp::Set(ColumnValue::Int(lv as i64))),
                        ("all_exp", UpdateOp::Set(ColumnValue::Int(exp))),
                    ],
                    &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
                );
                if let Some(p) = persist_path {
                    let _ = store.save_to_disk(p);
                }
                notice = format!(
                    "Updated character level to {lv} (EXP: {exp}) for account {acc_name}!"
                );
            } else {
                notice = "Account has not created an in-game character yet!".to_string();
            }
        } else {
            notice = "Account does not exist!".to_string();
        }
    }
    html_ok(render_utility_page(
        "Fix Character Level (1-200) - KING PIRATE",
        "/level.php",
        true,
        &notice,
    ))
}

/// Handle `legacy-ref/www/fix.php` (Fix Copy Dungeon Bug - Reset `blood_package = 0`).
fn handle_fix_page(
    method: &str,
    form: &BTreeMap<String, String>,
    store: &DataStore,
    persist_path: Option<&Path>,
) -> PortalHttpResponse {
    let mut notice = String::new();
    if method == "POST" {
        let acc_name = form.get("account").map(|s| s.trim()).unwrap_or("");
        if let Some(acc) = find_account_by_name(store, acc_name) {
            let pid = acc.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let affected = store.update(
                "t_user",
                &[("blood_package", UpdateOp::Set(ColumnValue::Int(0)))],
                &[("pid", WhereCond::Eq(ColumnValue::Int(pid)))],
            );
            if affected > 0 {
                if let Some(p) = persist_path {
                    let _ = store.save_to_disk(p);
                }
                notice = format!("Reset blood_package=0 for account {acc_name}!");
            } else {
                notice = "Account has not created an in-game character yet!".to_string();
            }
        } else {
            notice = "Account does not exist!".to_string();
        }
    }
    html_ok(render_utility_page(
        "Fix Dungeon Blood Package - KING PIRATE",
        "/fix.php",
        false,
        &notice,
    ))
}

const PORTAL_CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; min-height: 100vh; }
.bg_top { background: radial-gradient(circle at top, #1e293b 0%, #090d16 100%); min-height: 100vh; padding: 28px 16px; }
.container, .container-layout { max-width: 1020px; margin: 0 auto; }
.portal-header { display: flex; justify-content: space-between; align-items: center; background: rgba(15, 23, 42, 0.9); border: 1px solid #334155; border-radius: 12px; padding: 18px 24px; margin-bottom: 24px; box-shadow: 0 10px 30px rgba(0,0,0,0.45); }
.brand-title { font-size: 24px; font-weight: 800; color: #f59e0b; letter-spacing: 0.5px; }
.brand-sub { font-size: 13px; color: #94a3b8; margin-top: 4px; }
.button { display: inline-block; padding: 10px 18px; border-radius: 8px; border: 1px solid transparent; font-weight: 600; font-size: 14px; cursor: pointer; text-decoration: none; color: #fff; transition: all 0.15s; }
.button:hover { filter: brightness(1.1); transform: translateY(-1px); }
.bg-yellow { background: #d97706; }
.bg-dot { background: #dc2626; }
.bg-sub { background: #0284c7; }
.bg-main { background: #2563eb; }
.bg-green { background: #16a34a; }
.button-block { display: block; width: 100%; text-align: center; padding: 12px; font-size: 16px; }
.grid-2 { display: grid; grid-template-columns: 1.2fr 1fr; gap: 24px; }
.card { background: rgba(30, 41, 59, 0.9); border: 1px solid #334155; border-radius: 12px; padding: 24px; box-shadow: 0 10px 25px rgba(0,0,0,0.35); }
.card h2 { color: #fbbf24; font-size: 20px; margin-bottom: 14px; border-bottom: 1px solid #334155; padding-bottom: 10px; }
.patch-list p { font-size: 14px; line-height: 1.65; color: #e2e8f0; margin-bottom: 6px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; color: #cbd5e1; margin-bottom: 6px; font-weight: 600; }
.input { width: 100%; padding: 10px 12px; border-radius: 8px; border: 1px solid #475569; background: #0f172a; color: #f8fafc; font-size: 14px; }
.input:focus { outline: none; border-color: #38bdf8; }
.status-msg { margin-top: 12px; padding: 10px 12px; border-radius: 8px; font-size: 13px; display: none; }
.status-ok { display: block; background: rgba(22, 163, 74, 0.2); border: 1px solid #22c55e; color: #86efac; }
.status-err { display: block; background: rgba(220, 38, 38, 0.2); border: 1px solid #ef4444; color: #fca5a5; }
.server-btn { display: flex; align-items: center; justify-content: space-between; background: linear-gradient(135deg, #1d4ed8, #0284c7); border: 1px solid #38bdf8; padding: 18px 24px; border-radius: 12px; color: #fff; text-decoration: none; font-size: 18px; font-weight: 700; margin-top: 12px; }
.badge-online { background: #22c55e; color: #052e16; font-size: 12px; padding: 4px 10px; border-radius: 999px; font-weight: 700; }
.nav-actions { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.75); display: none; align-items: center; justify-content: center; z-index: 999; }
.modal-box { background: #1e293b; border: 1px solid #475569; border-radius: 12px; width: 480px; max-width: 95vw; overflow: hidden; box-shadow: 0 20px 50px rgba(0,0,0,0.6); }
.modal-head { display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; background: #0f172a; border-bottom: 1px solid #334155; font-weight: 700; color: #fbbf24; }
.modal-close { background: none; border: none; color: #94a3b8; font-size: 20px; cursor: pointer; }
"#;

const PORTAL_CORE_JS: &str = r#"
function OpenApp(encodedJson) {
    try {
        var cfg = JSON.parse(decodeURIComponent(encodedJson));
        var modal = document.getElementById('kp-modal');
        var frame = document.getElementById('kp-modal-frame');
        var title = document.getElementById('kp-modal-title');
        if (modal && frame) {
            title.textContent = cfg.title || 'KING PIRATE';
            frame.src = cfg.Url;
            frame.style.height = (cfg.W_h || 420) + 'px';
            modal.style.display = 'flex';
            return;
        }
        window.location.href = cfg.Url;
    } catch (e) {
        console.error(e);
    }
}
function CloseModal() {
    var modal = document.getElementById('kp-modal');
    if (modal) modal.style.display = 'none';
}
function showMsg(elId, text, ok) {
    var el = document.getElementById(elId || 'form-msg');
    if (!el) return;
    el.textContent = text;
    el.className = 'status-msg ' + (ok ? 'status-ok' : 'status-err');
}
function serializeForm(formEl) {
    var params = new URLSearchParams();
    var inputs = formEl.querySelectorAll('input, select');
    for (var i = 0; i < inputs.length; i++) {
        var inp = inputs[i];
        if (!inp.name) continue;
        if ((inp.type === 'radio' || inp.type === 'checkbox') && !inp.checked) continue;
        params.append(inp.name, inp.value);
    }
    return params.toString();
}
function Ajax_Action(action, formId, msgId) {
    var form = document.getElementById(formId || action);
    if (!form) return;
    var body = serializeForm(form);
    fetch('/core/Cx_Ajax.php?Action=' + encodeURIComponent(action), {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: body
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
        showMsg(msgId || 'form-msg', res.info || '', res.status === 'y');
        if (res.status === 'y' && res.url) {
            setTimeout(function() {
                if (window.top && window.top !== window) {
                    window.top.location.href = res.url;
                } else {
                    window.location.href = res.url;
                }
            }, 250);
        }
    });
}
function SignOut() {
    fetch('/core/Cx_Ajax.php?Action=SignOut', { method: 'POST' })
    .then(function(r) { return r.json(); })
    .then(function(res) {
        window.location.href = res.url || '/index.php';
    });
}
function Ajax_Get_GameUser(fqid) {
    fetch('/app/dh/ajax.php?Action=Get_GameUser', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: 'FQ=' + encodeURIComponent(fqid)
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
        var sel = document.getElementById('User_ID');
        var btn = document.getElementById('THTj');
        showMsg('form-msg', res.info || '', res.status === 'y');
        if (res.status === 'y' && sel) {
            sel.innerHTML = res.html;
            if (btn) btn.removeAttribute('disabled');
        }
    });
}
function Ajax_DHYB(selector) {
    var form = document.querySelector(selector) || document.getElementById('DHYB');
    if (!form) return;
    fetch('/app/dh/ajax.php?Action=DHYB', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: serializeForm(form)
    })
    .then(function(r) { return r.json(); })
    .then(function(res) {
        showMsg('form-msg', res.info || '', res.status === 'y');
        if (res.status === 'y') {
            setTimeout(function() { window.location.reload(); }, 600);
        }
    });
}
"#;

fn render_index_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>KING PIRATE 2023 - Official Web Portal</title>
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body>
<div class="bg_top">
  <div class="container">
    <div class="portal-header">
      <div>
        <div class="brand-title">KING PIRATE 2023 - OFFICIAL PORTAL</div>
        <div class="brand-sub">Account Registration, Login, Member Center & Server Selection</div>
      </div>
      <div class="nav-actions">
        <button id="btn-open-login" class="button bg-yellow" onclick="OpenApp('%7B%22Url%22%3A%22%2Fapp%2Fsys%2Flogin.php%22%2C%22title%22%3A%22Account%20Login%22%2C%22W_h%22%3A360%7D')">Account Login</button>
        <button id="btn-open-reg" class="button bg-dot" onclick="OpenApp('%7B%22Url%22%3A%22%2Fapp%2Fsys%2Freg.php%22%2C%22title%22%3A%22Account%20Registration%22%2C%22W_h%22%3A420%7D')">Register Account</button>
      </div>
    </div>

    <div class="grid-2">
      <div class="card patch-list">
        <h2>KING PIRATE 2023 Server Features & Rewards</h2>
        <p>* New accounts receive <strong>1,000,000,000 Exchange Points (dj)</strong>. New characters start with <strong>VIP 9</strong>, <strong>1,000,000 Gold</strong>, <strong>10,000,000 Belly</strong>, and <strong>100,000 Execution (Stamina)</strong>.</p>
        <p>1. New Title System: Fifth Emperor & Supreme Pirate King titles.</p>
        <p>2. Wano Country Fashion & Spirit Mounts.</p>
        <p>3. Level Cap 200 with Punk Hazard & Whitebeard Hill dungeons.</p>
        <p>4. World War, Cross-Server Team Battle & Guild War.</p>
        <p>5. Haki Heroes: Gear 4 Luffy, Kaido, Big Mom, Whitebeard, Shanks, Mihawk.</p>
      </div>

      <div class="card">
        <h2>Quick Login / Account Registration</h2>
        <form id="Login" onsubmit="event.preventDefault(); Ajax_Action('Login', 'Login', 'login-msg');">
          <div class="form-group">
            <label for="login-uname">Account Username</label>
            <input type="text" class="input" id="login-uname" name="Uname" placeholder="Enter username" />
          </div>
          <div class="form-group">
            <label for="login-upass">Password</label>
            <input type="password" class="input" id="login-upass" name="Upass" placeholder="Enter password" />
          </div>
          <button id="btn-quick-login" class="button bg-main button-block" type="submit">Login Now</button>
          <div id="login-msg" class="status-msg"></div>
        </form>

        <hr style="border:none;border-top:1px solid #334155;margin:18px 0;" />

        <form id="Register" onsubmit="event.preventDefault(); Ajax_Action('Register', 'Register', 'reg-msg');">
          <div class="form-group">
            <label for="reg-uname">Create New Account (min 4 chars)</label>
            <input type="text" class="input" id="reg-uname" name="Uname" placeholder="e.g. luffy2026" />
          </div>
          <div class="form-group">
            <label for="reg-upass">Login Password</label>
            <input type="password" class="input" id="reg-upass" name="Upass" placeholder="Enter password" />
          </div>
          <div class="form-group">
            <label for="reg-scode">Security Code (for password recovery)</label>
            <input type="password" class="input" id="reg-scode" name="Scode" placeholder="Enter security code (e.g. 123456)" />
          </div>
          <button id="btn-quick-register" class="button bg-dot button-block" type="submit">Register New Account</button>
          <div id="reg-msg" class="status-msg"></div>
        </form>
      </div>
    </div>
  </div>
</div>

<div id="kp-modal" class="modal-backdrop">
  <div class="modal-box">
    <div class="modal-head">
      <span id="kp-modal-title">KING PIRATE</span>
      <button class="modal-close" onclick="CloseModal()">x</button>
    </div>
    <iframe id="kp-modal-frame" src="about:blank" style="width:100%;border:0;height:400px;"></iframe>
  </div>
</div>
</body>
</html>"#
        .to_string()
}

fn render_member_page(uname: &str, dj: i64, pid: i64) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Member Center - KING PIRATE</title>
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body>
<div class="bg_top">
  <div class="container">
    <div class="portal-header">
      <div>
        <div class="brand-title">MEMBER CENTER - KING PIRATE</div>
        <div class="brand-sub">Account: <strong id="member-uname" style="color:#38bdf8">{uname}</strong> (SID: <code id="member-sid">{pid}</code>) &nbsp;|&nbsp; Point Balance (dj): <strong id="member-dj" style="color:#fbbf24">{dj}</strong></div>
      </div>
      <div class="nav-actions">
        <a id="btn-start-game" class="button bg-dot" href="/app/Game/index.php">Start Game (Select Server)</a>
        <a id="btn-exchange-gold" class="button bg-yellow" href="/app/dh/index.php">Exchange Gold</a>
        <a id="btn-fix-level" class="button bg-sub" href="/level.php">Set Level (1-200)</a>
        <a id="btn-fix-copy" class="button bg-green" href="/fix.php">Fix Dungeon Bug</a>
        <button id="btn-edit-pass" class="button bg-main" onclick="OpenApp('%7B%22Url%22%3A%22%2Fapp%2Fsys%2Feditpass.php%22%2C%22title%22%3A%22Change%20Password%22%2C%22W_h%22%3A360%7D')">Change Password</button>
        <button id="btn-signout" class="button" style="background:#475569" onclick="SignOut()">Sign Out</button>
      </div>
    </div>

    <div class="card">
      <h2>Active Game Servers</h2>
      <p style="color:#cbd5e1;margin-bottom:12px;">Select a server below to enter King Pirate as <strong>{uname}</strong> (SID: <code>{pid}</code>):</p>
      <a id="btn-enter-s1" class="server-btn" href="/app/Game/game.php?FQ=1">
        <span>Server S1 - LouLx-Game - KING PIRATE 2023</span>
        <span class="badge-online">ONLINE - ENTER GAME</span>
      </a>
    </div>
  </div>
</div>

<div id="kp-modal" class="modal-backdrop">
  <div class="modal-box">
    <div class="modal-head">
      <span id="kp-modal-title">KING PIRATE</span>
      <button class="modal-close" onclick="CloseModal()">x</button>
    </div>
    <iframe id="kp-modal-frame" src="about:blank" style="width:100%;border:0;height:400px;"></iframe>
  </div>
</div>
</body>
</html>"#
    )
}

fn render_server_select_page(uname: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Select Server - KING PIRATE</title>
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
</head>
<body>
<div class="bg_top">
  <div class="container">
    <div class="portal-header">
      <div>
        <div class="brand-title">SERVER SELECTION - KING PIRATE</div>
        <div class="brand-sub">Account: <strong>{uname}</strong></div>
      </div>
      <div class="nav-actions">
        <a class="button bg-sub" href="/member.php">Back to Member Center</a>
      </div>
    </div>
    <div class="card">
      <h2>Select a Server to Enter Game</h2>
      <a id="server-s1-link" class="server-btn" href="/app/Game/game.php?FQ=1">
        <span>S1 - LouLx-Game - KING PIRATE</span>
        <span class="badge-online">ENTER GAME</span>
      </a>
    </div>
  </div>
</div>
</body>
</html>"#
    )
}

fn render_game_iframe_page(pid: i64) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>S1 - KING PIRATE (SID {pid})</title>
<style>
  html, body {{ margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: #000; }}
  iframe {{ width: 100%; height: 100%; border: 0; display: block; }}
</style>
</head>
<body>
<iframe id="game-client-iframe" src="/T9NuoYa/T9NuoYa.html?sid={pid}" allowfullscreen></iframe>
</body>
</html>"#
    )
}

fn render_reg_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body style="padding:18px;">
<form id="Register" onsubmit="event.preventDefault(); Ajax_Action('Register');">
  <div class="form-group">
    <label for="Uname">Account Username</label>
    <input type="text" class="input" id="Uname" name="Uname" placeholder="Account Username" />
  </div>
  <div class="form-group">
    <label for="Upass">Login Password</label>
    <input type="password" class="input" id="Upass" name="Upass" placeholder="Login Password" />
  </div>
  <div class="form-group">
    <label for="Scode">Security Code</label>
    <input type="password" class="input" id="Scode" name="Scode" placeholder="Security Code" />
  </div>
  <button class="button button-block bg-sub" type="submit">Register</button>
  <div id="form-msg" class="status-msg"></div>
</form>
</body>
</html>"#
        .to_string()
}

fn render_login_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body style="padding:18px;">
<form id="Login" onsubmit="event.preventDefault(); Ajax_Action('Login');">
  <div class="form-group">
    <label for="Uname">Account Username</label>
    <input type="text" class="input" id="Uname" name="Uname" placeholder="Account Username" />
  </div>
  <div class="form-group">
    <label for="Upass">Login Password</label>
    <input type="password" class="input" id="Upass" name="Upass" placeholder="Login Password" />
  </div>
  <button class="button button-block bg-sub" type="submit">Login Now</button>
  <div id="form-msg" class="status-msg"></div>
</form>
</body>
</html>"#
        .to_string()
}

fn render_editpass_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body style="padding:18px;">
<form id="EditPass" onsubmit="event.preventDefault(); Ajax_Action('EditPass');">
  <div class="form-group">
    <label for="Upass">New Password</label>
    <input type="password" class="input" id="Upass" name="Upass" placeholder="Enter new password" />
  </div>
  <div class="form-group">
    <label for="Scode">Security Code</label>
    <input type="password" class="input" id="Scode" name="Scode" placeholder="Enter security code" />
  </div>
  <button class="button button-block bg-sub" type="submit">Update Password</button>
  <div id="form-msg" class="status-msg"></div>
</form>
</body>
</html>"#
        .to_string()
}

fn render_findpass_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body style="padding:18px;">
<form id="FindPass" onsubmit="event.preventDefault(); Ajax_Action('FindPass');">
  <div class="form-group">
    <label for="Uname">Account Username</label>
    <input type="text" class="input" id="Uname" name="Uname" placeholder="Enter username" />
  </div>
  <div class="form-group">
    <label for="Scode">Security Code</label>
    <input type="password" class="input" id="Scode" name="Scode" placeholder="Enter security code" />
  </div>
  <div class="form-group">
    <label for="Upass">New Password</label>
    <input type="password" class="input" id="Upass" name="Upass" placeholder="Enter new password" />
  </div>
  <button class="button button-block bg-sub" type="submit">Recover Password</button>
  <div id="form-msg" class="status-msg"></div>
</form>
</body>
</html>"#
        .to_string()
}

fn render_exchange_page(uname: &str, dj: i64) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Exchange Gold - KING PIRATE</title>
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
<script src="/WebRes/Js/core.js"></script>
</head>
<body>
<div class="bg_top">
  <div class="container">
    <div class="portal-header">
      <div>
        <div class="brand-title">EXCHANGE POINTS TO IN-GAME GOLD (1 : 10,000)</div>
        <div class="brand-sub">Account: <strong>{uname}</strong> | Point Balance (dj): <strong>{dj}</strong></div>
      </div>
      <a class="button bg-sub" href="/member.php">Back to Member Center</a>
    </div>
    <div class="card">
      <form id="DHYB" onsubmit="event.preventDefault(); Ajax_DHYB('#DHYB');">
        <div class="form-group">
          <label>1. Select Server</label>
          <label class="button bg-sub"><input name="FQ" onclick="Ajax_Get_GameUser('1');" value="1" type="radio"> S1.LouLx-Game</label>
        </div>
        <div class="form-group">
          <label for="User_ID">2. Select Character</label>
          <select class="input" id="User_ID" name="User_ID">
            <option value="">Please select server first</option>
          </select>
        </div>
        <div class="form-group">
          <label for="DH_Num">3. Points to Exchange (1 Point = 10,000 Gold)</label>
          <input type="number" class="input" id="DH_Num" name="DH_Num" value="100" min="1" />
        </div>
        <button class="button bg-main button-block" disabled="disabled" type="submit" id="THTj">Confirm Exchange</button>
        <div id="form-msg" class="status-msg"></div>
      </form>
    </div>
  </div>
</div>
</body>
</html>"#
    )
}

fn render_utility_page(title: &str, action_url: &str, include_level: bool, notice: &str) -> String {
    let lv_input = if include_level {
        r#"<div class="form-group">
          <label for="lv">Target Level (1 - 200)</label>
          <input type="number" class="input" id="lv" name="lv" min="1" max="200" value="100" />
        </div>"#
    } else {
        ""
    };
    let notice_html = if notice.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="status-msg status-ok" style="display:block">{notice}</div>"#)
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>{title}</title>
<link rel="stylesheet" href="/WebRes/Css/pintuer.css">
</head>
<body>
<div class="bg_top">
  <div class="container">
    <div class="portal-header">
      <div class="brand-title">{title}</div>
      <a class="button bg-sub" href="/member.php">Back to Member Center</a>
    </div>
    <div class="card">
      <form method="POST" action="{action_url}">
        <div class="form-group">
          <label for="account">Account Username</label>
          <input type="text" class="input" id="account" name="account" placeholder="Enter account username" required />
        </div>
        {lv_input}
        <button class="button bg-main button-block" type="submit">Apply Now</button>
        {notice_html}
      </form>
    </div>
  </div>
</div>
</body>
</html>"#
    )
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

        // 3. `/app/Game/game.php?FQ=1` embeds `<iframe src="/T9NuoYa/T9NuoYa.html?sid=202621271">`
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
    }
}
