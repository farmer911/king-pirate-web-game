//! Bounded Domain: `economy` (Sailboat, Kitchen, Talks, Train, Practice, AllBlue, Active, Task, Reward, Mail, Chat).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L282, L283, L302, L349, L352, L367, L373, L379, L388, L389, L394`):
//!   `kitchen.cook`, `sailboat.upgradeCabinLv`, `talks.startTalks`, `train.rapid`, `active.fetchPrize`, `reward.dailySign`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;

/// Execute `kitchen.cook` (`Framework.cfg.php:349`).
/// Converts cooking action into Belly profit on the server.
pub fn kitchen_cook(actor: &mut PlayerSessionActor, base_belly_gain: i64) -> Amf3Value {
    let gain = base_belly_gain.max(100) * (actor.level as i64).max(1);
    let _ = actor.add_belly(gain);
    Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("belly_gain", Amf3Value::Integer(gain as i32)),
        ("total_belly", Amf3Value::Integer(actor.belly_num as i32)),
    ])
}

/// Execute `reward.dailySign` (`Framework.cfg.php:373`).
pub fn daily_sign_reward(actor: &mut PlayerSessionActor, gold_bonus: i64, belly_bonus: i64) -> Amf3Value {
    let _ = actor.add_gold(gold_bonus.max(0));
    let _ = actor.add_belly(belly_bonus.max(0));
    Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("gold_num", Amf3Value::Integer(actor.gold_num as i32)),
        ("belly_num", Amf3Value::Integer(actor.belly_num as i32)),
    ])
}
