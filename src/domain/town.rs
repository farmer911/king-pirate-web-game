//! Bounded Domain: `town` (Real-Time Town Movement, Port Berths, Resource Plunder, Vassal).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L303, L366, L392, L401`):
//!   `city.enterTown`, `city.moveInTown`, `town.move`, `port.enterPort`, `vassal.conquer`.
//! - `legacy-ref/rpcfw/module/user/UserLogic.class.php` (`L91-L120`): `splitInt($ret['last_xy'])`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;

/// Split packed `last_xy` (`(x << 16) | y`) into `(x, y)` (`UserLogic.class.php:91`).
#[inline]
pub const fn unpack_town_xy(packed_xy: u32) -> (u16, u16) {
    (((packed_xy >> 16) & 0xFFFF) as u16, (packed_xy & 0xFFFF) as u16)
}

/// Handle `city.moveInTown` / `town.move` (`Framework.cfg.php:303, 392`).
pub fn move_in_town(
    actor: &mut PlayerSessionActor,
    town_id: u32,
    target_x: u16,
    target_y: u16,
) -> Amf3Value {
    actor.move_in_town(town_id, target_x, target_y);
    Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("town_id", Amf3Value::Integer(town_id as i32)),
        ("x", Amf3Value::Integer(target_x as i32)),
        ("y", Amf3Value::Integer(target_y as i32)),
    ])
}
