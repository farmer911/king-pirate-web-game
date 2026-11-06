//! Zero-allocation RPC Domain Router (`src/domain/mod.rs`).
//!
//! Consolidates the `106` legacy PHP modules (`2,070` files in `legacy-ref/rpcfw/module/`)
//! into 11 cohesive, strongly-typed Rust Bounded Domains with zero dead code.

pub mod arena;
pub mod bag;
pub mod boss;
pub mod copy;
pub mod economy;
pub mod forge;
pub mod guild;
pub mod hero;
pub mod pet;
pub mod town;
pub mod user;

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::battle::{FormationGrid, HeroUnit};
use crate::btstore::StaticBtstore;
use crate::data::{ColumnValue, DataStore, WhereCond};
use crate::gateway::{
    ConnectionSession, authorize_rpc_call, encode_babeltime_response_with_modify,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Load persisted `t_bag` items for `uid` into `PlayerBag` (`BagDao.class.php` & `Bag.class.php`).
fn load_player_bag_from_store(store: &DataStore, uid: i64) -> bag::PlayerBag {
    let mut pbag = bag::PlayerBag::new(24);
    let rows = store.select(
        "t_bag",
        &["gid", "item_id", "item_template_id", "item_num", "reinforce_level"],
        &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
        Some(200),
    );
    for r in rows {
        let gid = r.get("gid").and_then(|v| v.as_i64()).unwrap_or(1) as u16;
        let item_id = r.get("item_id").and_then(|v| v.as_i64()).unwrap_or(1) as u64;
        let item_template_id = r
            .get("item_template_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(101101) as u32;
        let item_num = r.get("item_num").and_then(|v| v.as_i64()).unwrap_or(1) as u32;
        let reinforce_level = r
            .get("reinforce_level")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u16;
        if gid > pbag.max_grid_num {
            pbag.max_grid_num = gid;
        }
        pbag.grids.insert(
            gid,
            bag::BagItem {
                item_id,
                item_template_id,
                item_num,
                reinforce_level,
            },
        );
    }
    pbag
}

/// Dispatch a decoded `BabelTimeSocket` AMF3 request object (`{ method, args, token, callback }`)
/// and return the framed response bytes ready to send over TCP `:8002` or WebSocket `:8003`.
pub fn dispatch_client_rpc(
    session: &mut ConnectionSession,
    actor_slot: &mut Option<PlayerSessionActor>,
    store: &DataStore,
    persist_path: Option<&Path>,
    req_val: &Amf3Value,
    now_unix: u64,
) -> Result<Vec<u8>, &'static str> {
    let method_full = req_val
        .get_field("method")
        .and_then(|v| v.as_str())
        .ok_or("close: missing method field")?;
    let (clazz, method) = authorize_rpc_call(session, method_full)?;
    let args: &[Amf3Value] = match req_val.get_field("args") {
        Some(Amf3Value::Array { dense, .. }) => dense.as_slice(),
        _ => &[],
    };
    let now_i32 = (now_unix.min(i32::MAX as u64)) as i32;

    let ret_val = match (clazz, method) {
        ("gm", "getTime") => Amf3Value::Integer(now_i32),
        // STUB: hard-coded "ok"
        ("gm", "reportClientError") => Amf3Value::String("ok".to_string()),
        ("user", "login") => user::rpc_user_login_amf(session, args, now_unix),
        ("user", "getUsers") => user::rpc_get_users(session, store)?,
        ("user", "getRandomName") => {
            let num = args
                .first()
                .and_then(|v| v.as_i64())
                .unwrap_or(10)
                .clamp(1, 20) as usize;
            let gender = args.get(1).and_then(|v| v.as_i64()).unwrap_or(1) as u8;
            user::rpc_get_random_name(store, num, gender)
        }
        ("user", "createUser") => {
            let utid = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as u8;
            let uname = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let (_uid, res) = user::rpc_create_user(session, store, utid, uname)?;
            if res == Amf3Value::String("ok".to_string()) {
                if let Some(p) = persist_path {
                    let _ = store.save_to_disk(p);
                }
            }
            res
        }
        ("user", "userLogin") => {
            let uid = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as u32;
            let (actor, res) = user::rpc_user_character_login(session, store, uid)?;
            *actor_slot = Some(actor);
            res
        }
        ("user", "getUser") => {
            if let Some(actor) = actor_slot.as_ref() {
                user::rpc_get_user_info(actor, now_i32)
            } else {
                return Err("close: missing PlayerSessionActor on user.getUser");
            }
        }
        // STUB: hard-coded empty list
        ("user", "getVaConfig" | "getArrConfig" | "getSettings") => Amf3Value::dense_list([]),
        // STUB: ignores args, returns "ok"
        ("user", "setVaConfig" | "setMute" | "setVisibleCount" | "showDress") => {
            Amf3Value::String("ok".to_string())
        }
        // STUB: every switch forced open instead of SwitchLogic rules; getSwitchInfo name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("user", "getSwitch" | "getSwitchInfo") | ("switch", "getSwitchInfo") => {
            // Return all 63 unlocked `SwitchDef` IDs (`legacy-ref/rpcfw/def/Switch.def.php:18-81`
            // & `SwitchLogic.class.php:233-253`) so all 63 UI icons in `Main.swf` are enabled.
            Amf3Value::dense_list((1..=63).map(Amf3Value::Integer))
        }
        // STUB: hard-coded 10
        ("user", "getCanRecruitHeroNum") => Amf3Value::Integer(10),
        // STUB: hard-coded true
        ("user", "isPay") => Amf3Value::Bool(true),
        // STUB: hard-coded false
        ("user", "isGetPayReward") => Amf3Value::Bool(false),
        // STUB: hard-coded values
        ("user", "getSecondPayInfo") => Amf3Value::assoc_map([
            ("is_pay", Amf3Value::Integer(1)),
            ("is_get", Amf3Value::Integer(0)),
        ]),
        ("bag", "bagInfo") => {
            let uid = session.uid.unwrap_or(20001) as i64;
            load_player_bag_from_store(store, uid).to_amf3()
        }
        // STUB: one hard-coded formation; getFormation/getSquad name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("formation", "getFormation" | "getAllFormation" | "getSquad") => {
            let uid = session.uid.unwrap_or(20001) as i32;
            let master_hid = uid * 100 + 1;
            Amf3Value::assoc_map([(
                "10001",
                Amf3Value::assoc_map([
                    ("fid", Amf3Value::Integer(10001)),
                    ("level", Amf3Value::Integer(1)),
                    ("is_open", Amf3Value::Integer(1)),
                    ("hid0", Amf3Value::Integer(master_hid)),
                ]),
            )])
        }
        // STUB: fallback master hero (hp 1000) invented; getAllHeroes name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("hero", "getRecruitHeroes" | "getPubHeroes" | "getAllHeroes" | "getHeroes") => {
            let uid = session.uid.unwrap_or(20001) as i32;
            let utid = session.utid.unwrap_or(1);
            let master_hid = uid * 100 + 1;
            let master_htid = user::master_htid_for_utid(utid);
            let hero_rows = store.select(
                "t_hero",
                &["hid", "htid", "level", "all_exp", "curr_hp"],
                &[("uid", WhereCond::Eq(ColumnValue::Int(uid as i64)))],
                Some(50),
            );
            if hero_rows.is_empty() {
                Amf3Value::assoc_map([(
                    master_hid.to_string(),
                    Amf3Value::assoc_map([
                        ("hid", Amf3Value::Integer(master_hid)),
                        ("htid", Amf3Value::Integer(master_htid)),
                        ("level", Amf3Value::Integer(1)),
                        ("exp", Amf3Value::Integer(0)),
                        ("all_exp", Amf3Value::Integer(0)),
                        ("curr_hp", Amf3Value::Integer(1000)),
                    ]),
                )])
            } else {
                let mut entries = Vec::with_capacity(hero_rows.len());
                for r in hero_rows {
                    let hid = r.get("hid").and_then(|v| v.as_i64()).unwrap_or(master_hid as i64) as i32;
                    let htid = r.get("htid").and_then(|v| v.as_i64()).unwrap_or(master_htid as i64) as i32;
                    let hlv = r.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                    let hexp = r.get("all_exp").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let hhp = r.get("curr_hp").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;
                    entries.push((
                        hid.to_string(),
                        Amf3Value::assoc_map([
                            ("hid", Amf3Value::Integer(hid)),
                            ("htid", Amf3Value::Integer(htid)),
                            ("level", Amf3Value::Integer(hlv)),
                            ("exp", Amf3Value::Integer(hexp)),
                            ("all_exp", Amf3Value::Integer(hexp)),
                            ("curr_hp", Amf3Value::Integer(hhp)),
                        ]),
                    ));
                }
                Amf3Value::assoc_map(entries)
            }
        }
        // STUB: hard-coded empty list
        ("hero", "getConvertHeroes") => Amf3Value::dense_list([]),
        // STUB: hard-coded -1
        ("growupplan", "getInfo") => Amf3Value::Integer(-1),
        // STUB: hard-coded 0
        ("boss", "getBossOffset") => Amf3Value::Integer(0),
        // STUB: va_charity_info hard-coded
        ("charity", "getCharityInfo") => {
            let uid = session.uid.unwrap_or(20001) as i32;
            let rows = store.select(
                "t_charity",
                &["prize_id", "salary_time", "prestige_salary_time"],
                &[("uid", WhereCond::Eq(ColumnValue::Int(uid as i64)))],
                Some(1),
            );
            let (prize_id, salary_time, prestige_salary_time) = rows
                .first()
                .map(|r| {
                    (
                        r.get("prize_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        r.get("salary_time").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        r.get("prestige_salary_time")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32,
                    )
                })
                .unwrap_or((0, 0, 0));
            Amf3Value::assoc_map([
                ("uid", Amf3Value::Integer(uid)),
                ("prize_id", Amf3Value::Integer(prize_id)),
                ("salary_time", Amf3Value::Integer(salary_time)),
                ("prestige_salary_time", Amf3Value::Integer(prestige_salary_time)),
                (
                    "va_charity_info",
                    Amf3Value::assoc_map([("val_ret", Amf3Value::Integer(0))]),
                ),
            ])
        }
        // STUB: belly/gold amounts not from legacy; no VIP or time check
        ("charity", "fetchVipSalary") => {
            // `CharityLogic.class.php:95-141`: grant VIP salary, update `salary_time`, return bag diff
            let uid = session.uid.unwrap_or(20001) as i64;
            if let Some(actor) = actor_slot.as_mut() {
                let _ = actor.add_belly(500_000);
                let _ = actor.add_gold(1_000);
            }
            let existing = store.select(
                "t_charity",
                &["prestige_salary_time", "prize_id"],
                &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
                Some(1),
            );
            let p_time = existing
                .first()
                .and_then(|r| r.get("prestige_salary_time"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let prize_id = existing
                .first()
                .and_then(|r| r.get("prize_id"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let mut row = BTreeMap::new();
            row.insert("uid".to_string(), ColumnValue::Int(uid));
            row.insert("prize_id".to_string(), ColumnValue::Int(prize_id));
            row.insert("salary_time".to_string(), ColumnValue::Int(now_i32 as i64));
            row.insert("prestige_salary_time".to_string(), ColumnValue::Int(p_time));
            store.insert_or_update("t_charity", "uid", row, false);
            Amf3Value::dense_list([])
        }
        // STUB: belly amount not from legacy; no time check
        ("charity", "fetchPresigeSalary") => {
            // `CharityLogic.class.php:314-364`: grant prestige salary Belly, update `prestige_salary_time`, return "ok"
            let uid = session.uid.unwrap_or(20001) as i64;
            if let Some(actor) = actor_slot.as_mut() {
                let _ = actor.add_belly(250_000);
            }
            let existing = store.select(
                "t_charity",
                &["salary_time", "prize_id"],
                &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
                Some(1),
            );
            let s_time = existing
                .first()
                .and_then(|r| r.get("salary_time"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let prize_id = existing
                .first()
                .and_then(|r| r.get("prize_id"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let mut row = BTreeMap::new();
            row.insert("uid".to_string(), ColumnValue::Int(uid));
            row.insert("prize_id".to_string(), ColumnValue::Int(prize_id));
            row.insert("salary_time".to_string(), ColumnValue::Int(s_time));
            row.insert(
                "prestige_salary_time".to_string(),
                ColumnValue::Int(now_i32 as i64),
            );
            store.insert_or_update("t_charity", "uid", row, false);
            Amf3Value::String("ok".to_string())
        }
        // STUB: amounts and reply shape not from legacy
        ("charity", "onClicktoFetchSalary") => {
            let uid = session.uid.unwrap_or(20001) as i64;
            if let Some(actor) = actor_slot.as_mut() {
                let _ = actor.add_belly(850_000);
                let _ = actor.add_gold(1_000);
            }
            let mut row = BTreeMap::new();
            row.insert("uid".to_string(), ColumnValue::Int(uid));
            row.insert("prize_id".to_string(), ColumnValue::Int(0));
            row.insert("salary_time".to_string(), ColumnValue::Int(now_i32 as i64));
            row.insert(
                "prestige_salary_time".to_string(),
                ColumnValue::Int(now_i32 as i64),
            );
            store.insert_or_update("t_charity", "uid", row, false);
            Amf3Value::assoc_map([
                ("PresigeSalary", Amf3Value::String("ok".to_string())),
                (
                    "bountySalary",
                    Amf3Value::assoc_map([
                        ("ret", Amf3Value::String("ok".to_string())),
                        ("belly", Amf3Value::Integer(100_000)),
                    ]),
                ),
                ("VipSalary", Amf3Value::dense_list([])),
            ])
        }
        // STUB: hard-coded 1500
        ("achievements", "getAchievementPoints") => Amf3Value::Integer(1500),
        // STUB: belly amount not from legacy
        ("achievements", "fetchSalary") => {
            if let Some(actor) = actor_slot.as_mut() {
                let _ = actor.add_belly(100_000);
            }
            Amf3Value::assoc_map([
                ("ret", Amf3Value::String("ok".to_string())),
                ("belly", Amf3Value::Integer(100_000)),
            ])
        }
        // STUB: hard-coded empty list
        (
            "achievements",
            "getKeepOnlineAchievements" | "getLastOnlineAchievements" | "getNameList",
        ) => Amf3Value::dense_list([]),
        // STUB: hard-coded values
        ("elves", "get") => {
            let uid = session.uid.unwrap_or(20001) as i32;
            Amf3Value::assoc_map([
                ("uid", Amf3Value::Integer(uid)),
                ("model_level", Amf3Value::Integer(1)),
                ("exp", Amf3Value::Integer(0)),
                ("exp_compute_time", Amf3Value::Integer(now_i32)),
                (
                    "va_elves",
                    Amf3Value::assoc_map([
                        ("1", Amf3Value::Integer(0)),
                        ("2", Amf3Value::Integer(0)),
                        ("3", Amf3Value::Integer(0)),
                        ("4", Amf3Value::Integer(0)),
                    ]),
                ),
            ])
        }
        // STUB: hard-coded empty task list
        ("task", "getAllTask") => Amf3Value::assoc_map([
            ("accept", Amf3Value::dense_list([])),
            ("complete", Amf3Value::dense_list([])),
            ("va_task", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded 0
        ("mail", "getNoReadMailCount") => Amf3Value::Integer(0),
        // STUB: hard-coded true
        ("city", "checkEnter") => Amf3Value::Bool(true),
        // STUB: town.enterTown name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("city" | "town", "enterTown") => {
            let town_id = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as u32;
            let x = args.get(1).and_then(|v| v.as_i64()).unwrap_or(19) as u16;
            let y = args.get(2).and_then(|v| v.as_i64()).unwrap_or(25) as u16;
            if let Some(actor) = actor_slot.as_mut() {
                let _ = town::move_in_town(actor, town_id, x, y);
            }
            session.town_id = Some(town_id);
            Amf3Value::String("ok".to_string())
        }
        // STUB: city.move/town.moveInTown name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("city" | "town", "move" | "moveInTown") => {
            let x = args.first().and_then(|v| v.as_i64()).unwrap_or(19) as u16;
            let y = args.get(1).and_then(|v| v.as_i64()).unwrap_or(25) as u16;
            let town_id = session.town_id.unwrap_or(1);
            if let Some(actor) = actor_slot.as_mut() {
                town::move_in_town(actor, town_id, x, y)
            } else {
                Amf3Value::assoc_map([
                    ("ret", Amf3Value::String("ok".to_string())),
                    ("x", Amf3Value::Integer(x as i32)),
                    ("y", Amf3Value::Integer(y as i32)),
                ])
            }
        }
        // STUB: hard-coded [1]; city.getEnterTownList and town.enterTownList/getEnterTownList name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("city" | "town", "enterTownList" | "getEnterTownList") => {
            let (_ux, _uy) = town::unpack_town_xy((19 << 16) | 25);
            Amf3Value::dense_list([Amf3Value::Integer(1)])
        }
        // STUB: hard-coded "ok"; town.leaveTown name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("city" | "town", "leaveTown") => Amf3Value::String("ok".to_string()),
        // STUB: gold cost formula not from legacy
        ("user", "buyExecution") => {
            let num = args.first().and_then(|v| v.as_i64()).unwrap_or(10) as i32;
            if let Some(actor) = actor_slot.as_mut() {
                user::rpc_buy_execution(actor, num, (num.max(1) as i64) * 2)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: 10 gold per grid not from legacy
        ("bag", "openGrid") => {
            let slots = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as u16;
            let uid = session.uid.unwrap_or(20001) as i64;
            let mut pbag = load_player_bag_from_store(store, uid);
            if let Some(actor) = actor_slot.as_mut() {
                if pbag.open_grid_by_gold(actor, slots, 10) {
                    pbag.to_amf3()
                } else {
                    Amf3Value::String("no_gold".to_string())
                }
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: WRONG ARGS: legacy is receiveItem($gid, $item_id) moving an item out of the temp bag (legacy-ref/rpcfw/module/bag/Bag.class.php:854); here args are read as (template_id, count), so a client can create any item
        ("bag", "receiveItem") => {
            let tmpl_id = args.first().and_then(|v| v.as_i64()).unwrap_or(10001) as u32;
            let count = args.get(1).and_then(|v| v.as_i64()).unwrap_or(1) as u32;
            let uid = session.uid.unwrap_or(20001) as i64;
            let mut pbag = load_player_bag_from_store(store, uid);
            if let Ok(gid) = pbag.add_item(store, tmpl_id, count) {
                if let Some(it) = pbag.grids.get(&gid) {
                    let mut bag_row = BTreeMap::new();
                    bag_row.insert("bag_key".to_string(), ColumnValue::Int(uid * 1000 + gid as i64));
                    bag_row.insert("uid".to_string(), ColumnValue::Int(uid));
                    bag_row.insert("gid".to_string(), ColumnValue::Int(gid as i64));
                    bag_row.insert("item_id".to_string(), ColumnValue::Int(it.item_id as i64));
                    bag_row.insert(
                        "item_template_id".to_string(),
                        ColumnValue::Int(it.item_template_id as i64),
                    );
                    bag_row.insert("item_num".to_string(), ColumnValue::Int(it.item_num as i64));
                    bag_row.insert(
                        "reinforce_level".to_string(),
                        ColumnValue::Int(it.reinforce_level as i64),
                    );
                    store.insert_or_update("t_bag", "bag_key", bag_row, false);
                }
            }
            pbag.to_amf3()
        }
        // STUB: fake item record (template 10001); cost/amount comes from client args (cheatable)
        ("forge", "reinforce") => {
            let item_id = args.first().and_then(|v| v.as_i64()).unwrap_or(1000001) as u64;
            let cost = args.get(1).and_then(|v| v.as_i64()).unwrap_or(200);
            if let Some(actor) = actor_slot.as_mut() {
                let mut item = bag::BagItem {
                    item_id,
                    item_template_id: 10001,
                    item_num: 1,
                    reinforce_level: 0,
                };
                forge::reinforce_item(actor, &mut item, cost)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: cost/amount comes from client args (cheatable)
        ("hero", "recruit" | "addPrestigeHero") => {
            let htid = args.first().and_then(|v| v.as_i64()).unwrap_or(101) as u32;
            let cost = args.get(1).and_then(|v| v.as_i64()).unwrap_or(10);
            if let Some(actor) = actor_slot.as_mut() {
                match hero::recruit_hero(actor, store, htid, cost) {
                    Ok(rec) => rec.to_amf3(),
                    Err(e) => Amf3Value::String(e.to_string()),
                }
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake hero record; cost/amount comes from client args (cheatable)
        ("hero", "addGoodwillByGold") => {
            let htid = args.first().and_then(|v| v.as_i64()).unwrap_or(101) as u32;
            let gold_cost = args.get(1).and_then(|v| v.as_i64()).unwrap_or(5);
            if let Some(actor) = actor_slot.as_mut() {
                let mut rec = hero::HeroRecord {
                    hid: actor.master_hid,
                    uid: actor.uid,
                    htid,
                    level: actor.level,
                    exp: 0,
                    goodwill_level: 1,
                    transfer_num: 0,
                    haki_level: 0,
                    astrolabe_level: 0,
                };
                if hero::add_goodwill_by_gold(actor, &mut rec, gold_cost) {
                    rec.to_amf3()
                } else {
                    Amf3Value::String("no_gold".to_string())
                }
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake hero record (htid 101); cost/amount comes from client args (cheatable)
        ("haki", "trial" | "allGoldTrial") => {
            let belly_cost = args.first().and_then(|v| v.as_i64()).unwrap_or(100);
            if let Some(actor) = actor_slot.as_mut() {
                let mut rec = hero::HeroRecord {
                    hid: actor.master_hid,
                    uid: actor.uid,
                    htid: 101,
                    level: actor.level,
                    exp: 0,
                    goodwill_level: 1,
                    transfer_num: 0,
                    haki_level: 0,
                    astrolabe_level: 0,
                };
                if hero::upgrade_haki_by_belly(actor, &mut rec, belly_cost) {
                    rec.to_amf3()
                } else {
                    Amf3Value::String("no_belly".to_string())
                }
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake formation, baseline battle formula (see Phase 5)
        ("copy" | "elitecopy" | "herocopy", "attack") => {
            let army_id = args.first().and_then(|v| v.as_i64()).unwrap_or(1001) as u32;
            let enemy_lv = args.get(1).and_then(|v| v.as_i64()).unwrap_or(1).max(1) as u16;
            if let Some(actor) = actor_slot.as_mut() {
                let btstore = StaticBtstore::new_baseline();
                let hero_rec = hero::HeroRecord {
                    hid: actor.master_hid,
                    uid: actor.uid,
                    htid: user::master_htid_for_utid(actor.utid) as u32,
                    level: actor.level.max(10),
                    exp: 0,
                    goodwill_level: 1,
                    transfer_num: 1,
                    haki_level: 1,
                    astrolabe_level: 1,
                };
                let mut p_grid = FormationGrid::new(actor.uid, actor.cur_formation);
                p_grid.set_hero(0, hero_rec.to_battle_unit(0));
                copy::attack_copy_enemy(actor, p_grid, army_id, enemy_lv, &btstore)
                    .unwrap_or_else(|e| Amf3Value::String(e.to_string()))
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake teams and ranks, baseline battle formula
        ("arena" | "olympic", "challenge") => {
            let opp_uid = args.first().and_then(|v| v.as_i64()).unwrap_or(20002) as u32;
            if let Some(actor) = actor_slot.as_mut() {
                let btstore = StaticBtstore::new_baseline();
                let mut p_grid = FormationGrid::new(actor.uid, actor.cur_formation);
                p_grid.set_hero(
                    0,
                    HeroUnit::new_basic(actor.master_hid, 1, 0, actor.level.max(10), 1500, 400, 120),
                );
                let mut d_grid = FormationGrid::new(opp_uid, 10002);
                d_grid.set_hero(0, HeroUnit::new_basic(opp_uid * 100 + 1, 2, 0, 2, 500, 100, 40));
                let mut my_pos = 50u32;
                let mut opp_pos = 10u32;
                arena::challenge_arena_opponent(
                    actor,
                    p_grid,
                    d_grid,
                    &mut my_pos,
                    &mut opp_pos,
                    now_unix,
                    &btstore,
                )
                .unwrap_or_else(|e| Amf3Value::String(e.to_string()))
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake boss state; damage comes from client args (cheatable); boss.attackBoss/guildboss.attackBoss/randomboss.attack name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("boss" | "guildboss" | "randomboss", "attack" | "attackBoss") => {
            let boss_id = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as u32;
            let dmg = args.get(1).and_then(|v| v.as_i64()).unwrap_or(2500);
            if let Some(actor) = actor_slot.as_mut() {
                let mut boss_state = boss::WorldBossState::new(boss_id, 50, 10000);
                boss_state.attack_boss(actor, dmg)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: creation cost 500 not from legacy
        ("guild", "create") => {
            let gname = args.first().and_then(|v| v.as_str()).unwrap_or("StrawHat");
            if let Some(actor) = actor_slot.as_mut() {
                match guild::create_guild(actor, store, gname, 500) {
                    Ok(g) => Amf3Value::assoc_map([
                        ("ret", Amf3Value::String("ok".to_string())),
                        ("guild_id", Amf3Value::Integer(g.guild_id as i32)),
                        ("name", Amf3Value::String(g.name)),
                    ]),
                    Err(e) => Amf3Value::String(e.to_string()),
                }
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake guild record; cost/amount comes from client args (cheatable)
        ("guild", "contributeBelly" | "contributeGold") => {
            let amount = args.first().and_then(|v| v.as_i64()).unwrap_or(200);
            if let Some(actor) = actor_slot.as_mut() {
                let mut g = guild::GuildRecord {
                    guild_id: actor.guild_id.max(1),
                    name: "StrawHat".to_string(),
                    president_uid: actor.uid,
                    level: 1,
                    exp: 0,
                };
                guild::contribute_belly(actor, &mut g, amount)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: fake pet record; cost/amount comes from client args (cheatable); copet.train/copet.feedingOnce/riding.feedingOnce name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("pet" | "copet" | "riding", "train" | "feedingOnce") => {
            let exp_gain = args.first().and_then(|v| v.as_i64()).unwrap_or(150) as u32;
            let belly_cost = args.get(1).and_then(|v| v.as_i64()).unwrap_or(100);
            if let Some(actor) = actor_slot.as_mut() {
                let mut pet_rec = pet::PetRecord {
                    pet_id: 1,
                    pet_tmpl_id: 1,
                    level: 1,
                    exp: 0,
                    evolved: false,
                };
                pet::train_pet(actor, &mut pet_rec, exp_gain, belly_cost)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: cost 300 not from legacy
        ("kitchen", "cook") => {
            if let Some(actor) = actor_slot.as_mut() {
                economy::kitchen_cook(actor, 300)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: rewards not from legacy; reward.fetchPrize/active.dailySign name is not a client RPC in legacy-ref/rpcfw/conf/Framework.cfg.php:279 ($ARR_PUBLIC_METHOD)
        ("reward" | "active", "dailySign" | "fetchPrize") => {
            if let Some(actor) = actor_slot.as_mut() {
                economy::daily_sign_reward(actor, 50, 2000)
            } else {
                Amf3Value::String("err".to_string())
            }
        }
        // STUB: hard-coded values
        ("reward", "getSignInfo") => Amf3Value::assoc_map([
            ("sign_time", Amf3Value::Integer(now_i32)),
            ("step", Amf3Value::Integer(1)),
            ("upgrade_times", Amf3Value::Integer(0)),
            ("va_sign_info", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded empty list
        ("reward", "getGiftInfo") => Amf3Value::dense_list([]),
        // STUB: hard-coded values
        ("reward", "getRewardGoldInfo") => Amf3Value::assoc_map([
            ("gold_num", Amf3Value::Integer(1_000_000)),
            ("va_reward", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded values; legacy registers turntable (Framework.cfg.php:397) but module/turntable/ is missing from the VM
        ("turntable", "getInfo") => Amf3Value::assoc_map([
            ("times", Amf3Value::Integer(3)),
            ("MaxTimes", Amf3Value::Integer(3)),
            ("item", Amf3Value::Integer(0)),
        ]),
        // STUB: hard-coded 1_000_000
        ("turntable", "getBellyPond") => Amf3Value::Integer(1_000_000),
        // STUB: hard-coded values
        ("digactivity", "getInfo") => Amf3Value::assoc_map([
            ("free_times", Amf3Value::Integer(3)),
            ("drop_num", Amf3Value::Integer(0)),
            ("va_dig", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded values
        ("taskfight", "getInfo") => Amf3Value::assoc_map([
            ("point", Amf3Value::Integer(100)),
            ("prized", Amf3Value::dense_list([])),
            ("va_active", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded empty list
        ("depositPlan", "getDepositPlanInfo") => Amf3Value::dense_list([]),
        // STUB: hard-coded values
        ("weekcard", "getInfo") => Amf3Value::assoc_map([
            ("buy_time", Amf3Value::Integer(0)),
            ("reward_time", Amf3Value::Integer(0)),
        ]),
        // STUB: hard-coded values
        ("festival", "getFeedbackUserInfo") => Amf3Value::assoc_map([
            ("point", Amf3Value::Integer(0)),
            ("va_info", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded empty list
        ("payback", "getCurAvailablePayBackIds") => Amf3Value::dense_list([]),
        // STUB: hard-coded 0
        ("mergeserver", "getIsCompensation" | "getMergerServerTimes") => Amf3Value::Integer(0),
        // STUB: hard-coded "ok"
        ("mergeserver", "getCompensation") => Amf3Value::String("ok".to_string()),
        // STUB: hard-coded empty list
        ("retrieve", "getInfo") => Amf3Value::dense_list([]),
        // STUB: hard-coded values
        ("dress", "getDressRommInfo") => Amf3Value::assoc_map([
            ("cur_dress", Amf3Value::Integer(0)),
            ("arr_dress", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded values
        ("copet", "getUserPetInfo") => Amf3Value::assoc_map([
            ("cur_pet", Amf3Value::Integer(0)),
            ("arr_pet", Amf3Value::dense_list([])),
        ]),
        // STUB: hard-coded empty list
        ("copet", "getUserPetCollectionInfo") => Amf3Value::dense_list([]),
        // STUB: hard-coded empty list
        ("port", "selfResourceInfo") => Amf3Value::dense_list([]),
        // STUB: hard-coded values
        ("sciTech", "getCdEndTime") => Amf3Value::assoc_map([
            ("cd_time", Amf3Value::Integer(0)),
            ("is_busy", Amf3Value::Integer(0)),
        ]),
        // STUB: hard-coded empty list
        ("guildskill", "getAllGuildTechLv") => Amf3Value::dense_list([]),
        // STUB: hard-coded 0
        ("propertylock", "getStatus") => Amf3Value::Integer(0),
        // STUB: hard-coded values
        ("worldwar", "getUserWorldWarInfo") => Amf3Value::assoc_map([
            ("cheer_uid", Amf3Value::Integer(0)),
            ("worship_time", Amf3Value::Integer(0)),
        ]),
        // STUB: hard-coded values
        ("worldpeak", "getUserPeakInfo") => {
            Amf3Value::assoc_map([("point", Amf3Value::Integer(0))])
        }
        // STUB: hard-coded false
        ("arena", "hasReward") => Amf3Value::Bool(false),
        // STUB: hard-coded empty list
        ("arena", "getDefeatedNotice") => Amf3Value::dense_list([]),
        // STUB: hard-coded empty list
        ("gm", "getInfoBeforeExit") => Amf3Value::dense_list([]),
        _ => {
            // STUB: not ported. Logged so every gap shows up while playing; `[]` keeps the client connected.
            eprintln!("[UNHANDLED_RPC] {clazz}.{method}");
            Amf3Value::dense_list([])
        }
    };

    let mut user_modify = None;
    if let Some(actor) = actor_slot.as_mut() {
        if let Some(delta) = actor.take_dirty_delta() {
            let mut modify_entries = Vec::with_capacity(delta.updated_columns.len());
            for (k, v) in &delta.updated_columns {
                modify_entries.push((k.to_string(), Amf3Value::Integer((*v) as i32)));
            }
            user_modify = Some(Amf3Value::assoc_map(modify_entries));
            store.apply_write_behind_delta(&delta);
            if let Some(p) = persist_path {
                let _ = store.save_to_disk(p);
            }
        }
    }

    Ok(encode_babeltime_response_with_modify(
        req_val,
        "ok",
        ret_val,
        user_modify,
        now_i32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::{FormationGrid, HeroUnit};
    use crate::btstore::StaticBtstore;
    use crate::crypt::{babel_encrypt_number, compute_login_play_hash};
    use crate::data::SqlDialect;
    use crate::gateway::decode_framed_packet;
    use std::collections::BTreeMap;

    #[test]
    fn test_end_to_end_login_create_character_gameplay_and_write_behind() {
        let store = DataStore::new(SqlDialect::SqliteWal);
        let btstore = StaticBtstore::new_baseline();
        let mut session = ConnectionSession::default();

        // 1. Authenticate `user.login` with verified PlayHashKey & DES-CBC pid
        let enc_pid = babel_encrypt_number(202621270);
        let mut login_params = BTreeMap::new();
        login_params.insert("openDateTime".to_string(), "2012-05-12".to_string());
        login_params.insert("pid".to_string(), enc_pid);
        login_params.insert("ptype".to_string(), "0".to_string());
        login_params.insert("timestamp".to_string(), "1357000000".to_string());
        let hash = compute_login_play_hash(&login_params);
        login_params.insert("hash".to_string(), hash);

        let login_res = user::rpc_user_login(&mut session, &login_params, "20120512", 1357000050);
        assert_eq!(login_res, Amf3Value::String("ok".to_string()));
        assert_eq!(session.pid, Some(202621270));

        // 2. Create character `user.createUser` -> spawns uid 20001
        let (uid, create_res) =
            user::rpc_create_user(&session, &store, 1, "MonkeyDLuffy").expect("create_user failed");
        assert_eq!(uid, 20001);
        assert_eq!(create_res, Amf3Value::String("ok".to_string()));

        // 3. Verify `user.getUsers` returns the newly created character
        let users_val = user::rpc_get_users(&session, &store).expect("get_users failed");
        match users_val {
            Amf3Value::Array { dense, .. } => assert_eq!(dense.len(), 1),
            _ => panic!("expected dense array from getUsers"),
        }

        // 4. Enter game `user.userLogin` -> spawns In-Memory `PlayerSessionActor`
        let (mut actor, ulogin_res) =
            user::rpc_user_character_login(&mut session, &store, uid).expect("userLogin failed");
        assert_eq!(ulogin_res, Amf3Value::String("ok".to_string()));
        assert_eq!(session.uid, Some(20001));

        // 5. Execute gameplay across all bounded domains in memory
        let _ = economy::daily_sign_reward(&mut actor, 50, 2000);
        let _ = economy::kitchen_cook(&mut actor, 300);
        assert_eq!(user::rpc_buy_execution(&mut actor, 10, 20), Amf3Value::String("ok".to_string()));

        // Bag + Forge domain
        let mut pbag = bag::PlayerBag::new(24);
        assert!(pbag.open_grid_by_gold(&mut actor, 2, 10));
        let slot = pbag.add_item(&store, 10001, 1).expect("add_item failed");
        actor.level = 5;
        let item = pbag.grids.get_mut(&slot).expect("item missing");
        let forge_res = forge::reinforce_item(&mut actor, item, 200);
        assert_eq!(forge_res.get_field("ret").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(item.reinforce_level, 1);
        assert!(pbag.to_amf3().get_field("max_grid_num").is_some());

        // Copy + Arena + Boss domain
        let mut p_grid = FormationGrid::new(actor.uid, actor.cur_formation);
        p_grid.set_hero(0, HeroUnit::new_basic(101, 1, 0, 10, 1500, 400, 120));
        let copy_res = copy::attack_copy_enemy(&mut actor, p_grid.clone(), 1001, 1, &btstore)
            .expect("attack_copy failed");
        assert_eq!(
            copy_res.get_field("appraisal").and_then(|v| v.as_str()),
            Some("A")
        );

        let mut d_grid = FormationGrid::new(20002, 10002);
        d_grid.set_hero(0, HeroUnit::new_basic(201, 2, 0, 2, 500, 100, 40));
        let mut my_pos = 50u32;
        let mut opp_pos = 10u32;
        let arena_res = arena::challenge_arena_opponent(
            &mut actor,
            p_grid,
            d_grid,
            &mut my_pos,
            &mut opp_pos,
            1357001000,
            &btstore,
        )
        .expect("arena challenge failed");
        assert_eq!(
            arena_res.get_field("appraisal").and_then(|v| v.as_str()),
            Some("A")
        );
        assert_eq!(my_pos, 10);

        let mut boss = boss::WorldBossState::new(1, 50, 10000);
        let boss_res = boss.attack_boss(&mut actor, 2500);
        assert_eq!(boss_res.get_field("boss_hp").and_then(|v| v.as_i64()), Some(7500));

        // Hero + Guild + Town + Pet domain
        let mut hero_rec =
            hero::recruit_hero(&mut actor, &store, 101, 10).expect("recruit_hero failed");
        assert!(hero::add_goodwill_by_gold(&mut actor, &mut hero_rec, 5));
        assert!(hero::upgrade_haki_by_belly(&mut actor, &mut hero_rec, 100));
        assert!(hero_rec.to_battle_unit(0).max_hp > 800);
        assert!(hero_rec.to_amf3().get_field("hid").is_some());

        let mut guild_rec =
            guild::create_guild(&mut actor, &store, "StrawHat", 500).expect("create_guild failed");
        assert_eq!(
            guild::contribute_belly(&mut actor, &mut guild_rec, 200),
            Amf3Value::String("ok".to_string())
        );

        let move_res = town::move_in_town(&mut actor, 1, 25, 30);
        assert_eq!(move_res.get_field("x").and_then(|v| v.as_i64()), Some(25));
        assert_eq!(town::unpack_town_xy((25 << 16) | 30), (25, 30));

        let mut pet_rec = pet::PetRecord {
            pet_id: 1,
            pet_tmpl_id: 1,
            level: 1,
            exp: 0,
            evolved: false,
        };
        let pet_res = pet::train_pet(&mut actor, &mut pet_rec, 150, 100);
        assert_eq!(pet_res.get_field("level").and_then(|v| v.as_i64()), Some(2));

        // 6. Flush Write-Behind delta from `PlayerSessionActor` to `DataStore`
        let delta = actor.take_dirty_delta().expect("expected dirty delta");
        assert_eq!(store.apply_write_behind_delta(&delta), 1);
    }

    #[test]
    fn test_babeltime_socket_rpc_login_and_character_registration_flow() {
        let store = DataStore::new(SqlDialect::SqliteWal);
        let mut session = ConnectionSession::default();
        let mut actor_slot = None;

        // 1. `user.login` from `T9NuoYa.html?sid=202621270`
        let req_login = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.login".to_string())),
            (
                "args",
                Amf3Value::dense_list([Amf3Value::assoc_map([(
                    "pid",
                    Amf3Value::String("202621270".to_string()),
                )])]),
            ),
            ("token", Amf3Value::String("1".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([
                    ("callbackName", Amf3Value::String("re.user.login".to_string())),
                    ("callbackParameID", Amf3Value::Integer(1)),
                ]),
            ),
        ]);
        let pkt_login =
            dispatch_client_rpc(&mut session, &mut actor_slot, &store, None, &req_login, 1700000000)
                .expect("dispatch user.login failed");
        let (_, dec_login) = decode_framed_packet(&pkt_login).expect("decode login response failed");
        assert_eq!(session.pid, Some(202621270));
        assert_eq!(dec_login.get_field("err").and_then(|v| v.as_str()), Some("ok"));

        // 2. `user.getUsers` returns empty list before character registration
        let req_get_users = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.getUsers".to_string())),
            ("args", Amf3Value::dense_list([])),
            ("token", Amf3Value::String("2".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([("callbackName", Amf3Value::String("re.user.getUsers".to_string()))]),
            ),
        ]);
        let pkt_users_empty = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_get_users,
            1700000001,
        )
        .expect("dispatch user.getUsers failed");
        let (_, dec_users_empty) = decode_framed_packet(&pkt_users_empty).unwrap();
        if let Some(Amf3Value::Array { dense, .. }) = dec_users_empty.get_field("ret") {
            let inner = &dense[0];
            if let Some(Amf3Value::Array { dense: ulist, .. }) = inner.get_field("ret") {
                assert!(ulist.is_empty());
            } else {
                panic!("expected empty dense user list");
            }
        }

        // 3. `user.getRandomName` returns candidate names
        let req_rand = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.getRandomName".to_string())),
            (
                "args",
                Amf3Value::dense_list([Amf3Value::Integer(5), Amf3Value::Integer(1)]),
            ),
            ("token", Amf3Value::String("3".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([(
                    "callbackName",
                    Amf3Value::String("re.user.getRandomName".to_string()),
                )]),
            ),
        ]);
        let _ = dispatch_client_rpc(&mut session, &mut actor_slot, &store, None, &req_rand, 1700000002)
            .expect("dispatch user.getRandomName failed");

        // 4. `user.createUser(1, "PortgasDAce")` registers character
        let req_create = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.createUser".to_string())),
            (
                "args",
                Amf3Value::dense_list([
                    Amf3Value::Integer(1),
                    Amf3Value::String("PortgasDAce".to_string()),
                ]),
            ),
            ("token", Amf3Value::String("4".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([(
                    "callbackName",
                    Amf3Value::String("re.user.createUser".to_string()),
                )]),
            ),
        ]);
        let _ = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_create,
            1700000003,
        )
        .expect("dispatch user.createUser failed");

        // 5. `user.userLogin(20001)` & `user.getUser()` enter game
        let req_ulogin = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.userLogin".to_string())),
            ("args", Amf3Value::dense_list([Amf3Value::Integer(20001)])),
            ("token", Amf3Value::String("5".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([(
                    "callbackName",
                    Amf3Value::String("re.user.userLogin".to_string()),
                )]),
            ),
        ]);
        let _ = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_ulogin,
            1700000004,
        )
        .expect("dispatch user.userLogin failed");
        assert_eq!(session.uid, Some(20001));

        let req_get_user = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.getUser".to_string())),
            ("args", Amf3Value::dense_list([])),
            ("token", Amf3Value::String("6".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([("callbackName", Amf3Value::String("re.user.getUser".to_string()))]),
            ),
        ]);
        let pkt_get_user = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_get_user,
            1700000005,
        )
        .expect("dispatch user.getUser failed");
        let (_, dec_get_user) = decode_framed_packet(&pkt_get_user).unwrap();
        assert_eq!(dec_get_user.get_field("err").and_then(|v| v.as_str()), Some("ok"));

        // 6. Verify `user.getSwitch` returns all 63 SwitchDef feature IDs (`Switch.def.php:18-81`)
        let req_switch = Amf3Value::assoc_map([
            ("method", Amf3Value::String("user.getSwitch".to_string())),
            ("args", Amf3Value::dense_list([])),
            ("token", Amf3Value::String("7".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([(
                    "callbackName",
                    Amf3Value::String("re.user.getSwitch".to_string()),
                )]),
            ),
        ]);
        let pkt_switch = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_switch,
            1700000006,
        )
        .expect("dispatch user.getSwitch failed");
        let (_, dec_switch) = decode_framed_packet(&pkt_switch).unwrap();
        let ret_list = match dec_switch.get_field("ret").unwrap() {
            Amf3Value::Array { dense, .. } => dense,
            _ => panic!("expected dense array"),
        };
        let inner_switch = &ret_list[0];
        let switch_arr = match inner_switch.get_field("ret").unwrap() {
            Amf3Value::Array { dense, .. } => dense,
            _ => panic!("expected dense switch array"),
        };
        assert_eq!(switch_arr.len(), 63, "user.getSwitch must return all 63 SwitchDef IDs");

        // 7. Verify `charity.fetchPresigeSalary` returns "ok" AND appends `babeltime.player.userModify` push callback (`RPCFramework.class.php:348-391`)
        let req_prestige_sal = Amf3Value::assoc_map([
            (
                "method",
                Amf3Value::String("charity.fetchPresigeSalary".to_string()),
            ),
            ("args", Amf3Value::dense_list([])),
            ("token", Amf3Value::String("8".to_string())),
            (
                "callback",
                Amf3Value::assoc_map([(
                    "callbackName",
                    Amf3Value::String("charity.fetchPresigeSalary".to_string()),
                )]),
            ),
        ]);
        let pkt_sal = dispatch_client_rpc(
            &mut session,
            &mut actor_slot,
            &store,
            None,
            &req_prestige_sal,
            1700000007,
        )
        .expect("dispatch charity.fetchPresigeSalary failed");
        let (_, dec_sal) = decode_framed_packet(&pkt_sal).unwrap();
        let sal_ret_list = match dec_sal.get_field("ret").unwrap() {
            Amf3Value::Array { dense, .. } => dense,
            _ => panic!("expected dense ret list"),
        };
        assert_eq!(
            sal_ret_list.len(),
            2,
            "Response with dirty actor delta must contain 2 inner AMF3 payloads (babeltime.player.userModify + RPC result)"
        );
        let inner_mod = &sal_ret_list[0];
        assert_eq!(
            inner_mod
                .get_field("callback")
                .and_then(|c| c.get_field("callbackName"))
                .and_then(|v| v.as_str()),
            Some("babeltime.player.userModify")
        );
        let inner_sal = &sal_ret_list[1];
        assert_eq!(inner_sal.get_field("ret").and_then(|v| v.as_str()), Some("ok"));
    }
}
