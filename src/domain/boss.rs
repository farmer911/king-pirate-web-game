//! Bounded Domain: `boss` (World Boss, Guild Boss, Random Boss).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L295, L337, L371`):
//!   `boss.attack`, `boss.inspire`, `boss.inspireByGold`, `guildboss.attack`, `randomboss.attackBoss`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldBossState {
    pub boss_id: u32,
    pub level: u16,
    pub max_hp: i64,
    pub curr_hp: i64,
}

impl WorldBossState {
    pub const fn new(boss_id: u32, level: u16, max_hp: i64) -> Self {
        Self {
            boss_id,
            level,
            max_hp,
            curr_hp: max_hp,
        }
    }

    /// Execute `boss.attack` (`Framework.cfg.php:295`).
    pub fn attack_boss(
        &mut self,
        actor: &mut PlayerSessionActor,
        damage_dealt: i64,
    ) -> Amf3Value {
        let actual_dmg = damage_dealt.max(1).min(self.curr_hp);
        self.curr_hp -= actual_dmg;
        let belly_reward = (actual_dmg / 10).max(100);
        let _ = actor.add_belly(belly_reward);
        let _ = actor.add_prestige(15);

        Amf3Value::assoc_map([
            ("ret", Amf3Value::String("ok".to_string())),
            ("damage", Amf3Value::Integer(actual_dmg as i32)),
            ("boss_hp", Amf3Value::Integer(self.curr_hp as i32)),
            ("killed", Amf3Value::Bool(self.curr_hp == 0)),
        ])
    }
}
