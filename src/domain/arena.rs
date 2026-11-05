//! Bounded Domain: `arena` (Arena PvP Rankings, Olympic, WorldWar, GroupWar, BoatBattle).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L286, L293, L334, L358, L410`):
//!   `arena.challenge`, `arena.getInfo`, `olympic.challenge`, `worldwar.*`, `groupwar.*`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::battle::{FormationGrid, execute_battle_do_hero};
use crate::btstore::StaticBtstore;

/// Execute an Arena PvP challenge (`arena.challenge`, `Framework.cfg.php:286`).
pub fn challenge_arena_opponent(
    challenger: &mut PlayerSessionActor,
    challenger_grid: FormationGrid,
    defender_grid: FormationGrid,
    challenger_pos: &mut u32,
    defender_pos: &mut u32,
    now_unix: u64,
    btstore: &StaticBtstore,
) -> Result<Amf3Value, &'static str> {
    if !challenger.add_fight_cdtime(now_unix, 600) {
        return Err("cd_not_ready");
    }
    let result = execute_battle_do_hero(challenger_grid, defender_grid, btstore);
    let won = result.winner_uid == challenger.uid;
    if won {
        let _ = challenger.add_prestige(100);
        let _ = challenger.add_belly(500);
        if *defender_pos < *challenger_pos {
            std::mem::swap(challenger_pos, defender_pos);
        }
    } else {
        let _ = challenger.add_prestige(20);
    }

    Ok(Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("appraisal", Amf3Value::String(result.appraisal.to_string())),
        ("position", Amf3Value::Integer(*challenger_pos as i32)),
    ]))
}
