//! Bounded Domain: `guild` (Guild Management, Contributions, Banquet, Tech, Guild Shop, Guild War).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L336-L340`):
//!   `guild.create`, `guild.contributeBelly`, `guild.contributeGold`, `guildshop.buyShopItems`, `guildwar.*`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::data::DataStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildRecord {
    pub guild_id: u32,
    pub name: String,
    pub president_uid: u32,
    pub level: u16,
    pub exp: i64,
}

/// Create a new Guild (`guild.create`, `Framework.cfg.php:336`).
pub fn create_guild(
    actor: &mut PlayerSessionActor,
    store: &DataStore,
    name: &str,
    belly_cost: i64,
) -> Result<GuildRecord, &'static str> {
    if actor.guild_id != 0 {
        return Err("already_in_guild");
    }
    if !actor.add_belly(-belly_cost) {
        return Err("no_belly");
    }
    let guild_id = store.id_gen.next_id("guild_id", 1) as u32;
    actor.guild_id = guild_id;
    Ok(GuildRecord {
        guild_id,
        name: name.trim().to_string(),
        president_uid: actor.uid,
        level: 1,
        exp: 0,
    })
}

/// Contribute Belly to Guild (`guild.contributeBelly`, `Framework.cfg.php:336`).
pub fn contribute_belly(
    actor: &mut PlayerSessionActor,
    guild: &mut GuildRecord,
    belly_amount: i64,
) -> Amf3Value {
    // A negative amount would pay the player instead of charging them.
    if belly_amount <= 0 {
        return Amf3Value::String("err".to_string());
    }
    if !actor.add_belly(-belly_amount) {
        return Amf3Value::String("no_belly".to_string());
    }
    guild.exp = guild.exp.saturating_add(belly_amount / 100);
    let _ = actor.add_prestige(belly_amount / 200);
    Amf3Value::String("ok".to_string())
}
