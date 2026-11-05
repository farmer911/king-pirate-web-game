//! Bounded Domain: `hero` (Recruitment, Attributes, Goodwill, Transfer, Haki, Astrolabe).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L287, L341, L342`): `hero.*`, `haki.*`, `astrolabe.*`.
//! - `legacy-ref/schema/pirate20002-schema.sql`: `t_hero`, `t_haki`, `t_astrolabe_info`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::battle::HeroUnit;
use crate::data::DataStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroRecord {
    pub hid: u32,
    pub uid: u32,
    pub htid: u32,
    pub level: u16,
    pub exp: u32,
    pub goodwill_level: u16,
    pub transfer_num: u8,
    pub haki_level: u16,
    pub astrolabe_level: u16,
}

impl HeroRecord {
    pub fn to_battle_unit(&self, position: u8) -> HeroUnit {
        let base_hp = 800 + (self.level as i32) * 120 + (self.transfer_num as i32) * 300;
        let base_atk = 150 + (self.level as i32) * 35 + (self.haki_level as i32) * 25;
        let base_def = 60 + (self.level as i32) * 15 + (self.astrolabe_level as i32) * 15;
        HeroUnit::new_basic(
            self.hid,
            self.htid,
            position,
            self.level,
            base_hp,
            base_atk,
            base_def,
        )
    }

    pub fn to_amf3(&self) -> Amf3Value {
        Amf3Value::assoc_map([
            ("hid", Amf3Value::Integer(self.hid as i32)),
            ("htid", Amf3Value::Integer(self.htid as i32)),
            ("level", Amf3Value::Integer(self.level as i32)),
            ("exp", Amf3Value::Integer(self.exp as i32)),
            ("goodwill_lv", Amf3Value::Integer(self.goodwill_level as i32)),
            ("transfer_num", Amf3Value::Integer(self.transfer_num as i32)),
            ("haki_lv", Amf3Value::Integer(self.haki_level as i32)),
            ("astrolabe_lv", Amf3Value::Integer(self.astrolabe_level as i32)),
        ])
    }
}

/// Recruit a new hero (`hero.recruit` / `hero.addPrestigeHero`, `Framework.cfg.php:342`).
pub fn recruit_hero(
    actor: &mut PlayerSessionActor,
    store: &DataStore,
    htid: u32,
    prestige_cost: i64,
) -> Result<HeroRecord, &'static str> {
    if !actor.add_prestige(-prestige_cost) {
        return Err("insufficient prestige to recruit hero");
    }
    let hid = store.id_gen.next_id("hid", 1) as u32;
    Ok(HeroRecord {
        hid,
        uid: actor.uid,
        htid,
        level: 1,
        exp: 0,
        goodwill_level: 1,
        transfer_num: 0,
        haki_level: 0,
        astrolabe_level: 0,
    })
}

/// Upgrade Hero Goodwill by Gold (`hero.addGoodwillByGold`, `Framework.cfg.php:342`).
pub fn add_goodwill_by_gold(
    actor: &mut PlayerSessionActor,
    hero: &mut HeroRecord,
    gold_cost: i64,
) -> bool {
    if !actor.add_gold(-gold_cost) {
        return false;
    }
    hero.goodwill_level = hero.goodwill_level.saturating_add(1);
    true
}

/// Upgrade Hero Haki (`haki.trial` / `haki.allGoldTrial`, `Framework.cfg.php:341`).
pub fn upgrade_haki_by_belly(
    actor: &mut PlayerSessionActor,
    hero: &mut HeroRecord,
    belly_cost: i64,
) -> bool {
    if !actor.add_belly(-belly_cost) {
        return false;
    }
    hero.haki_level = hero.haki_level.saturating_add(1);
    true
}
