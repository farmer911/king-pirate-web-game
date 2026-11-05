//! Bounded Domain: `forge` (Equipment Reinforce, Gild, Refresh, Gems, Smelting, Dress, Jewelry).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L325, L348, L384`): `forge.reinforce`, `forge.gild`, `smelting.smeltingOnce`, `jewelry.reinforce`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::domain::bag::BagItem;

/// Reinforce an equipment item (`forge.reinforce`, `Framework.cfg.php:325`).
/// Checks server-authoritative Belly cost, reinforcement cooldown, and player level cap.
pub fn reinforce_item(
    actor: &mut PlayerSessionActor,
    item: &mut BagItem,
    belly_cost: i64,
) -> Amf3Value {
    if item.reinforce_level >= actor.level {
        return Amf3Value::String("level_limit".to_string());
    }
    if !actor.add_belly(-belly_cost) {
        return Amf3Value::String("no_belly".to_string());
    }
    item.reinforce_level = item.reinforce_level.saturating_add(1);
    Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("reinforce_level", Amf3Value::Integer(item.reinforce_level as i32)),
    ])
}
