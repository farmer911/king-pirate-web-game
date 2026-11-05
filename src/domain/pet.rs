//! Bounded Domain: `pet` (Pet & CoPet Breeding, Qualifications, Skills, Evolution, Ride).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L304, L363, L374, L375`):
//!   `pet.train`, `pet.evolution`, `pet.equip`, `copet.born`, `ride.mount`, `riding.train`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetRecord {
    pub pet_id: u32,
    pub pet_tmpl_id: u32,
    pub level: u16,
    pub exp: u32,
    pub evolved: bool,
}

/// Feed/train a Pet (`pet.feedingOnce` / `pet.train`, `Framework.cfg.php:363`).
pub fn train_pet(
    actor: &mut PlayerSessionActor,
    pet: &mut PetRecord,
    exp_gain: u32,
    belly_cost: i64,
) -> Amf3Value {
    if !actor.add_belly(-belly_cost) {
        return Amf3Value::String("no_belly".to_string());
    }
    pet.exp = pet.exp.saturating_add(exp_gain);
    while pet.exp >= 100 && pet.level < actor.level {
        pet.exp -= 100;
        pet.level += 1;
    }
    Amf3Value::assoc_map([
        ("ret", Amf3Value::String("ok".to_string())),
        ("level", Amf3Value::Integer(pet.level as i32)),
        ("exp", Amf3Value::Integer(pet.exp as i32)),
    ])
}
