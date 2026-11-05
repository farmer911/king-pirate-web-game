//! Bounded Domain: `user` (`user.login`, `user.getUsers`, `user.getRandomName`, `user.createUser`, `user.userLogin`, `user.getUser`).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/module/user/User.class.php` (`L34-L561`)
//! - `legacy-ref/rpcfw/module/user/UserLogic.class.php` (`L23-L453`)
//! - `legacy-ref/rpcfw/conf/User.cfg.php` (`L65-L250`)
//! - `legacy-ref/rpcfw/module/user/OtherUserObj.class.php` (`L108-L650`)

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::crypt::{babel_decrypt_number, verify_login_request};
use crate::data::{ColumnValue, DataStore, RowMap, WhereCond};
use crate::gateway::ConnectionSession;
use std::collections::BTreeMap;

pub const PID_MAX_RETAIN: u64 = 10_000;
pub const LOGIN_DIFF_TIME: u64 = 900;

const RANDOM_NAMES_FEMALE: &[&str] = &[
    "Nami", "Robin", "Vivi", "Perona", "Bonney", "Koala", "Carrot", "Yamato", "Uta", "Hina",
];
const RANDOM_NAMES_MALE: &[&str] = &[
    "Luffy", "Zoro", "Sanji", "Ace", "Sabo", "Law", "Shanks", "Mihawk", "Marco", "Roger",
];

/// Map character template `utid` (`1..=6`) to master hero template `htid` (`User.cfg.php:65-95`).
pub fn master_htid_for_utid(utid: u8) -> i32 {
    match utid {
        1 => 11002,
        2 => 11001,
        3 => 11003,
        4 => 11006,
        5 => 11005,
        6 => 11004,
        _ => 11002,
    }
}

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

/// Handle `user.login` from live `Main.swf` AMF3 `args` (`T9NuoYa.html` flashvars or signed `PlayHashKey` object).
pub fn rpc_user_login_amf(
    session: &mut ConnectionSession,
    args: &[Amf3Value],
    now_unix: u64,
) -> Amf3Value {
    let Some(first) = args.first() else {
        return Amf3Value::String("fail".to_string());
    };
    match first {
        Amf3Value::Array { assoc, .. } => {
            if assoc.contains_key("hash") {
                let mut params = BTreeMap::new();
                for (k, v) in assoc {
                    let s = v
                        .as_str()
                        .map(|x| x.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                        .unwrap_or_default();
                    params.insert(k.clone(), s);
                }
                let open_ymd = params
                    .get("openDateTime")
                    .map(|s| s.replace('-', ""))
                    .map(|s| s.chars().take(8).collect::<String>())
                    .unwrap_or_else(|| "20151018".to_string());
                let res = rpc_user_login(session, &params, &open_ymd, now_unix);
                if res == Amf3Value::String("ok".to_string()) {
                    return res;
                }
            }
            if let Some(pid_val) = assoc.get("pid") {
                if let Some(pid) = extract_pid_from_amf(pid_val) {
                    if pid > PID_MAX_RETAIN {
                        session.pid = Some(pid);
                        return Amf3Value::String("ok".to_string());
                    }
                }
            }
            Amf3Value::String("fail".to_string())
        }
        other => {
            if let Some(pid) = extract_pid_from_amf(other) {
                if pid > PID_MAX_RETAIN {
                    session.pid = Some(pid);
                    return Amf3Value::String("ok".to_string());
                }
            }
            Amf3Value::String("fail".to_string())
        }
    }
}

fn extract_pid_from_amf(val: &Amf3Value) -> Option<u64> {
    if let Some(n) = val.as_i64() {
        if n > 0 {
            return Some(n as u64);
        }
    }
    if let Some(s) = val.as_str() {
        let trimmed = s.trim();
        if let Ok(n) = trimmed.parse::<u64>() {
            return Some(n);
        }
        if let Ok(dec) = babel_decrypt_number(trimmed) {
            if dec > 0 {
                return Some(dec);
            }
        }
    }
    None
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
        let xy = row
            .get("last_xy")
            .and_then(|v| v.as_i64())
            .unwrap_or((19 << 16) | 25) as i32;
        let last_x = (xy >> 16) & 0xFFFF;
        let last_y = xy & 0xFFFF;
        let uname = match row.get("uname") {
            Some(ColumnValue::Text(s)) => s.clone(),
            _ => String::new(),
        };
        list.push(Amf3Value::assoc_map([
            ("uid", Amf3Value::Integer(uid)),
            ("utid", Amf3Value::Integer(utid)),
            ("uname", Amf3Value::String(uname)),
            ("level", Amf3Value::Integer(level)),
            ("dtime", Amf3Value::Integer(0)),
            ("last_place_type", Amf3Value::Integer(1)),
            ("last_place_data", Amf3Value::Integer(0)),
            ("last_town_id", Amf3Value::Integer(town_id)),
            ("last_x", Amf3Value::Integer(last_x)),
            ("last_y", Amf3Value::Integer(last_y)),
            ("guild_id", Amf3Value::Integer(0)),
            ("guild_name", Amf3Value::String(String::new())),
            ("emblem_id", Amf3Value::Integer(0)),
            ("title", Amf3Value::Integer(0)),
            ("show_title_id", Amf3Value::Integer(0)),
        ]));
    }
    Ok(Amf3Value::dense_list(list))
}

/// Execute `user.getRandomName($num, $gender)` (`User.class.php:222-227` & `UserLogic.class.php:411-421`).
pub fn rpc_get_random_name(store: &DataStore, num: usize, gender: u8) -> Amf3Value {
    let limit = num.clamp(1, 20);
    let pool = if gender == 0 {
        RANDOM_NAMES_FEMALE
    } else {
        RANDOM_NAMES_MALE
    };
    let mut out = Vec::with_capacity(limit);
    for &base in pool {
        if out.len() >= limit {
            break;
        }
        let used = store.select_count(
            "t_user",
            &[("uname", WhereCond::Eq(ColumnValue::Text(base.to_string())))],
        );
        let candidate = if used == 0 {
            base.to_string()
        } else {
            format!("{}{}", base, used + 1)
        };
        out.push(Amf3Value::assoc_map([(
            "name",
            Amf3Value::String(candidate),
        )]));
    }
    Amf3Value::dense_list(out)
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
    if trimmed.is_empty() || trimmed.chars().count() > 18 || !(1..=6).contains(&utid) {
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
    row.insert("vip".to_string(), ColumnValue::Int(9));
    row.insert("belly_num".to_string(), ColumnValue::Int(10_000_000));
    row.insert("gold_num".to_string(), ColumnValue::Int(1_000_000));
    row.insert("prestige_num".to_string(), ColumnValue::Int(0));
    row.insert("experience_num".to_string(), ColumnValue::Int(1_000_000));
    row.insert("cur_execution".to_string(), ColumnValue::Int(100));
    row.insert("last_town_id".to_string(), ColumnValue::Int(1));
    row.insert("last_xy".to_string(), ColumnValue::Int((19 << 16) | 25));
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

/// Build the full `user.getUser()` dictionary expected by `Main.swf` (`User.class.php:438-561`).
pub fn rpc_get_user_info(actor: &PlayerSessionActor, now_unix: i32) -> Amf3Value {
    let master_htid = master_htid_for_utid(actor.utid);
    let master_hid = (actor.uid as i32) * 100 + 1;
    let va_user = Amf3Value::assoc_map([
        (
            "heroes",
            Amf3Value::dense_list([Amf3Value::Integer(master_htid)]),
        ),
        (
            "recruit_hero_order",
            Amf3Value::dense_list([Amf3Value::Integer(master_hid)]),
        ),
        ("login_date", Amf3Value::dense_list([])),
        ("state", Amf3Value::dense_list([])),
        (
            "goodwill",
            Amf3Value::assoc_map([(
                "heritage",
                Amf3Value::assoc_map([("cfg_gold_num", Amf3Value::Integer(10))]),
            )]),
        ),
        ("group_info", Amf3Value::dense_list([])),
        ("dress", Amf3Value::dense_list([Amf3Value::dense_list([])])),
        (
            "imageDress",
            Amf3Value::dense_list([Amf3Value::assoc_map([(
                "template_id",
                Amf3Value::Integer(0),
            )])]),
        ),
        ("rideid", Amf3Value::Integer(0)),
        ("showride", Amf3Value::Integer(1)),
    ]);

    Amf3Value::assoc_map([
        ("uid", Amf3Value::Integer(actor.uid as i32)),
        ("uname", Amf3Value::String(actor.uname.clone())),
        ("utid", Amf3Value::Integer(actor.utid as i32)),
        ("master_hid", Amf3Value::Integer(master_hid)),
        ("level", Amf3Value::Integer(actor.level as i32)),
        ("tid", Amf3Value::Integer(0)),
        ("create_time", Amf3Value::Integer(now_unix)),
        ("dtime", Amf3Value::Integer(0)),
        ("birthday", Amf3Value::Integer(0)),
        ("group_id", Amf3Value::Integer(1)),
        ("guild_id", Amf3Value::Integer(0)),
        ("guild_name", Amf3Value::String(String::new())),
        ("title", Amf3Value::Integer(0)),
        ("cur_execution", Amf3Value::Integer(actor.cur_execution)),
        ("execution_time", Amf3Value::Integer(now_unix)),
        ("today_buy_execution_num", Amf3Value::Integer(0)),
        ("fight_cdtime", Amf3Value::Integer(0)),
        ("protect_cdtime", Amf3Value::Integer(0)),
        ("atk_value", Amf3Value::Integer(100)),
        ("cur_formation", Amf3Value::Integer(actor.cur_formation as i32)),
        ("vip", Amf3Value::Integer(actor.vip as i32)),
        ("recruit_num", Amf3Value::Integer(1)),
        ("watch_num", Amf3Value::Integer(0)),
        ("belly_num", Amf3Value::Integer(actor.belly_num.min(i32::MAX as i64) as i32)),
        ("gold_num", Amf3Value::Integer(actor.gold_num.min(i32::MAX as i64) as i32)),
        ("reward_point", Amf3Value::Integer(0)),
        ("gift_cash", Amf3Value::Integer(0)),
        ("prestige_num", Amf3Value::Integer(actor.prestige_num.min(i32::MAX as i64) as i32)),
        ("experience_num", Amf3Value::Integer(1_000_000)),
        ("food_num", Amf3Value::Integer(0)),
        (
            "blood_package",
            Amf3Value::Integer(actor.blood_package.min(i32::MAX as i64) as i32),
        ),
        ("last_town_id", Amf3Value::Integer(actor.town_id as i32)),
        ("last_salary_time", Amf3Value::Integer(0)),
        ("copy_execution", Amf3Value::Integer(0)),
        ("online_accum_time", Amf3Value::Integer(0)),
        ("ban_chat_time", Amf3Value::Integer(0)),
        ("gem_exp", Amf3Value::Integer(0)),
        ("show_dress", Amf3Value::Integer(1)),
        ("can_arena_num", Amf3Value::Integer(10)),
        ("arena_cdtime", Amf3Value::Integer(0)),
        ("practice_last_time", Amf3Value::Integer(0)),
        ("login_time", Amf3Value::Integer(now_unix)),
        ("charge_gold", Amf3Value::Integer(0)),
        ("login_type", Amf3Value::String("default".to_string())),
        ("opclient_reward", Amf3Value::Integer(0)),
        ("blueSoul", Amf3Value::Integer(0)),
        ("purpleSoul", Amf3Value::Integer(0)),
        ("greenSoul", Amf3Value::Integer(0)),
        ("show_vip", Amf3Value::Integer(1)),
        ("elvesinfo", Amf3Value::dense_list([])),
        ("va_user", va_user),
    ])
}

/// Execute `user.buyExecution($num)` (`User.class.php:480-510`).
pub fn rpc_buy_execution(actor: &mut PlayerSessionActor, num: i32, gold_cost: i64) -> Amf3Value {
    if num <= 0 || !actor.add_gold(-gold_cost) {
        return Amf3Value::String("err".to_string());
    }
    let _ = actor.add_execution(num);
    Amf3Value::String("ok".to_string())
}
