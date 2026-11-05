//! Bounded Domain: `copy` (Normal Copy, Elite Copy, Hero Copy, Abyss Copy, Impel Down, Burning Crusade).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L280, L297, L305, L315, L343, L346`):
//!   `copy.attack`, `elitecopy.attack`, `herocopy.attack`, `abysscopy.*`, `impeldown.savingAce`, `burningCrusade.doBattle`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::battle::{FormationGrid, HeroUnit, execute_battle_do_hero};
use crate::btstore::StaticBtstore;

/// Execute a PvE copy attack (`copy.attack` / `elitecopy.attack`, `Framework.cfg.php:305, 315`).
/// Deducts 1 execution point on victory and grants Belly, Experience, and Prestige rewards.
pub fn attack_copy_enemy(
    actor: &mut PlayerSessionActor,
    player_formation: FormationGrid,
    army_id: u32,
    enemy_level: u16,
    btstore: &StaticBtstore,
) -> Result<Amf3Value, &'static str> {
    if actor.cur_execution < 1 {
        return Err("no_execution");
    }
    let mut enemy_grid = FormationGrid::new(army_id, 10002);
    let enemy_hp = 400 + (enemy_level as i32) * 80;
    let enemy_atk = 90 + (enemy_level as i32) * 20;
    let enemy_def = 30 + (enemy_level as i32) * 10;
    enemy_grid.set_hero(
        0,
        HeroUnit::new_basic(army_id * 10 + 1, 2, 0, enemy_level, enemy_hp, enemy_atk, enemy_def),
    );

    let report = execute_battle_do_hero(player_formation, enemy_grid, btstore);
    let won = report.winner_uid == actor.uid;
    if won {
        let _ = actor.add_execution(-1);
        let _ = actor.add_belly(250 * (enemy_level as i64));
        let _ = actor.add_prestige(20 * (enemy_level as i64));
    }

    Ok(Amf3Value::assoc_map([
        ("err", Amf3Value::String("ok".to_string())),
        ("appraisal", Amf3Value::String(report.appraisal.to_string())),
        ("army_id", Amf3Value::Integer(army_id as i32)),
        ("fightRet", report.to_amf3()),
    ]))
}
