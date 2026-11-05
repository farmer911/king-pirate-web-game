//! Bounded Domain: `user` (`user.login`, `user.getUsers`, `user.createUser`, `user.userLogin`, `user.getUser`).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/module/user/User.class.php` (`L34-L285`)
//! - `legacy-ref/rpcfw/module/user/UserLogic.class.php` (`L23-L250`)
//! - `legacy-ref/rpcfw/module/user/OtherUserObj.class.php` (`L108-L650`)

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::crypt::verify_login_request;
use crate::data::{ColumnValue, DataStore, RowMap, WhereCond};
use crate::gateway::ConnectionSession;
use std::collections::BTreeMap;

pub const PID_MAX_RETAIN: u64 = 10_000;
pub const LOGIN_DIFF_TIME: u64 = 900;

/// Execute `user.login($arrReq)` (`User.class.php:34-106` & `UserLogic.class.php:23-82`).
/// Strictly verifies `PlayHashKey` MD5 and DES-CBC `pid` (no `DEBUG` bypass).
pub fn rpc_user_login(
    session: &mut ConnectionSession,
    params: &BTreeMap<String, String>,
    expected_open_ymd: &str,
    now_unix: u64,
) -> Amf3Value {
    match verify_login_request(
        params,
        expected_open_ymd,
        now_unix,
        LOGIN_DIFF_TIME,
        PID_MAX_RETAIN,
    ) {
        Ok((pid, _ptype)) => {
            session.pid = Some(pid);
            Amf3Value::String("ok".to_string())
        }
        Err(err) if err.starts_with("timeout") => Amf3Value::String("timeout".to_string()),
        Err(_) => Amf3Value::String("fail".to_string()),
    }
}

/// Execute `user.getUsers()` (`User.class.php:111-157` & `UserLogic.class.php:84-125`).
pub fn rpc_get_users(session: &ConnectionSession, store: &DataStore) -> Result<Amf3Value, &'static str> {
    let pid = session.pid.ok_or("fake: missing global.pid")?;
    let rows = store.select(
        "t_user",
        &["uid", "utid", "uname", "level", "last_town_id", "last_xy"],
        &[("pid", WhereCond::Eq(ColumnValue::Int(pid as i64)))],
        Some(10),
    );
    let mut list = Vec::with_capacity(rows.len());
    for row in rows {
        let uid = row.get("uid").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let utid = row.get("utid").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let level = row.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let town_id = row.get("last_town_id").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let uname = match row.get("uname") {
            Some(ColumnValue::Text(s)) => s.clone(),
            _ => String::new(),
        };
        list.push(Amf3Value::assoc_map([
            ("uid", Amf3Value::Integer(uid)),
            ("utid", Amf3Value::Integer(utid)),
            ("uname", Amf3Value::String(uname)),
            ("level", Amf3Value::Integer(level)),
            ("last_town_id", Amf3Value::Integer(town_id)),
            ("dtime", Amf3Value::Integer(0)),
        ]));
    }
    Ok(Amf3Value::dense_list(list))
}

/// Execute `user.createUser($utid, $uname)` (`User.class.php:162-185` & `UserLogic.class.php:127-215`).
pub fn rpc_create_user(
    session: &ConnectionSession,
    store: &DataStore,
    utid: u8,
    uname: &str,
) -> Result<(u32, Amf3Value), &'static str> {
    let pid = session.pid.ok_or("fake: missing global.pid")?;
    let trimmed = uname.trim();
    if trimmed.is_empty() || trimmed.len() > 18 || !(1..=6).contains(&utid) {
        return Ok((0, Amf3Value::String("invalid_char".to_string())));
    }
    let existing_name = store.select_count(
        "t_user",
        &[("uname", WhereCond::Eq(ColumnValue::Text(trimmed.to_string())))],
    );
    if existing_name > 0 {
        return Ok((0, Amf3Value::String("name_used".to_string())));
    }

    let uid = store.id_gen.next_id("uid", 1) as u32;
    let mut row = RowMap::new();
    row.insert("uid".to_string(), ColumnValue::Int(uid as i64));
    row.insert("pid".to_string(), ColumnValue::Int(pid as i64));
    row.insert("utid".to_string(), ColumnValue::Int(utid as i64));
    row.insert("uname".to_string(), ColumnValue::Text(trimmed.to_string()));
    row.insert("level".to_string(), ColumnValue::Int(1));
    row.insert("vip".to_string(), ColumnValue::Int(0));
    row.insert("belly_num".to_string(), ColumnValue::Int(1000));
    row.insert("gold_num".to_string(), ColumnValue::Int(100));
    row.insert("prestige_num".to_string(), ColumnValue::Int(0));
    row.insert("cur_execution".to_string(), ColumnValue::Int(100));
    row.insert("last_town_id".to_string(), ColumnValue::Int(1));
    row.insert("last_xy".to_string(), ColumnValue::Int((10 << 16) | 10));
    store.insert_or_update("t_user", "uid", row, false);

    Ok((uid, Amf3Value::String("ok".to_string())))
}

/// Bind character `uid` to `ConnectionSession` and spawn in-memory `PlayerSessionActor` (`User.class.php:205-260`).
pub fn rpc_user_character_login(
    session: &mut ConnectionSession,
    store: &DataStore,
    uid: u32,
) -> Result<(PlayerSessionActor, Amf3Value), &'static str> {
    let pid = session.pid.ok_or("fake: missing global.pid")?;
    let rows = store.select(
        "t_user",
        &["uid", "pid", "utid", "uname", "gold_num", "belly_num"],
        &[
            ("uid", WhereCond::Eq(ColumnValue::Int(uid as i64))),
            ("pid", WhereCond::Eq(ColumnValue::Int(pid as i64))),
        ],
        Some(1),
    );
    let Some(row) = rows.first() else {
        return Err("fake: character uid does not belong to pid");
    };
    let utid = row.get("utid").and_then(|v| v.as_i64()).unwrap_or(1) as u8;
    let uname = match row.get("uname") {
        Some(ColumnValue::Text(s)) => s.clone(),
        _ => "Pirate".to_string(),
    };
    session.uid = Some(uid);
    session.utid = Some(utid);
    session.town_id = Some(1);

    let mut actor = PlayerSessionActor::new(uid, pid, utid, uname);
    actor.gold_num = row.get("gold_num").and_then(|v| v.as_i64()).unwrap_or(100);
    actor.belly_num = row.get("belly_num").and_then(|v| v.as_i64()).unwrap_or(1000);
    Ok((actor, Amf3Value::String("ok".to_string())))
}

/// Execute `user.buyExecution($num)` (`User.class.php:480-510`).
pub fn rpc_buy_execution(actor: &mut PlayerSessionActor, num: i32, gold_cost: i64) -> Amf3Value {
    if num <= 0 || !actor.add_gold(-gold_cost) {
        return Amf3Value::String("err".to_string());
    }
    let _ = actor.add_execution(num);
    Amf3Value::String("ok".to_string())
}
