//! In-Memory `PlayerSessionActor` and Write-Behind `userModify` Delta Queue (`Rule 08`).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/module/user/OtherUserObj.class.php` (`L53-L55`, `L108-L130`, `L447-L650`):
//!   - Tracks baseline `$this->user` and mutated `$this->userModify` in memory during an RPC turn.
//!   - Enforces server-authoritative bounds on `belly_num` (`0..=10_000_000_000`), `gold_num` (`>= 0`),
//!     `prestige_num` (`>= 0`), `blood_package` (`>= 0`), and `fight_cdtime`.
//! - `legacy-ref/rpcfw/module/user/User.class.php` (`L1573-L1576`):
//!   - Guarantees a single active connection per `uid` (`closeConnection` on duplicate login),
//!     allowing lock-free in-memory reads and mutations inside `PlayerSessionActor`.

use std::collections::BTreeMap;

/// Maximum Belly cap enforced by `OtherUserObj::getUpperLimitBelly()` (`OtherUserObj.class.php:521-524`).
pub const UPPER_LIMIT_BELLY: i64 = 10_000_000_000;

/// Dirty state diff (`$this->userModify` equivalent) queued for Write-Behind SQL batching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDirtyDelta {
    pub uid: u32,
    pub updated_columns: BTreeMap<&'static str, i64>,
}

/// Hot in-memory state for an online player (`uid`), residing in process RAM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSessionActor {
    pub uid: u32,
    pub pid: u64,
    pub utid: u8,
    pub uname: String,
    pub level: u16,
    pub vip: u8,
    pub master_hid: u32,
    pub cur_formation: u32,
    pub guild_id: u32,
    pub town_id: u32,
    pub town_x: u16,
    pub town_y: u16,
    pub belly_num: i64,
    pub gold_num: i64,
    pub prestige_num: i64,
    pub experience_num: i64,
    pub food_num: i64,
    pub blood_package: i64,
    pub cur_execution: i32,
    pub fight_cdtime: u64,
    pub protect_cdtime: u64,
    dirty_columns: BTreeMap<&'static str, i64>,
}

impl PlayerSessionActor {
    pub fn new(uid: u32, pid: u64, utid: u8, uname: String) -> Self {
        Self {
            uid,
            pid,
            utid,
            uname,
            level: 1,
            vip: 9,
            master_hid: uid * 100 + 1,
            cur_formation: 10001,
            guild_id: 0,
            town_id: 1,
            town_x: 19,
            town_y: 25,
            belly_num: 10_000_000,
            gold_num: 1_000_000,
            prestige_num: 0,
            experience_num: 1_000_000,
            food_num: 0,
            blood_package: 0,
            cur_execution: 100_000,
            fight_cdtime: 0,
            protect_cdtime: 0,
            dirty_columns: BTreeMap::new(),
        }
    }

    /// Server-authoritative Belly mutation (`OtherUserObj::addBelly` / `subBelly`, `L526-L549`).
    /// Rejects negative final balances (`return false`) and clamps to `UPPER_LIMIT_BELLY`.
    pub fn add_belly(&mut self, delta: i64) -> bool {
        let next = self.belly_num.saturating_add(delta);
        if next < 0 {
            return false;
        }
        self.belly_num = next.min(UPPER_LIMIT_BELLY);
        self.dirty_columns.insert("belly_num", self.belly_num);
        true
    }

    /// Server-authoritative Gold mutation (`OtherUserObj::addGold`, `L551-L562`).
    /// Prevents client memory scanners (e.g., Cheat Engine) from spending non-existent Gold.
    pub fn add_gold(&mut self, delta: i64) -> bool {
        let next = self.gold_num.saturating_add(delta);
        if next < 0 {
            return false;
        }
        self.gold_num = next;
        self.dirty_columns.insert("gold_num", self.gold_num);
        true
    }

    /// Server-authoritative Prestige mutation (`OtherUserObj::addPrestige`, `L635-L650`).
    pub fn add_prestige(&mut self, delta: i64) -> bool {
        let next = self.prestige_num.saturating_add(delta);
        if next < 0 {
            return false;
        }
        self.prestige_num = next;
        self.dirty_columns.insert("prestige_num", self.prestige_num);
        true
    }

    /// Server-authoritative Execution (stamina) mutation (`UserObj::addExecution`).
    pub fn add_execution(&mut self, delta: i32) -> bool {
        let next = self.cur_execution.saturating_add(delta);
        if next < 0 {
            return false;
        }
        self.cur_execution = next;
        self.dirty_columns
            .insert("cur_execution", self.cur_execution as i64);
        true
    }

    /// Server-authoritative combat cooldown check & update (`OtherUserObj::addFightCDTime`, `L447-L458`).
    pub fn add_fight_cdtime(&mut self, now_unix: u64, cd_secs: u64) -> bool {
        if self.fight_cdtime > now_unix {
            return false;
        }
        self.fight_cdtime = now_unix + cd_secs;
        self.dirty_columns
            .insert("fight_cdtime", self.fight_cdtime as i64);
        true
    }

    /// Update town position in L1 RAM (`City::moveInTown`).
    pub fn move_in_town(&mut self, town_id: u32, x: u16, y: u16) {
        self.town_id = town_id;
        self.town_x = x;
        self.town_y = y;
        let packed_xy = (((x as u32) << 16) | (y as u32)) as i64;
        self.dirty_columns.insert("last_town_id", town_id as i64);
        self.dirty_columns.insert("last_xy", packed_xy);
    }

    /// Drain dirty `userModify` columns into a `PlayerDirtyDelta` for Write-Behind persistence.
    pub fn take_dirty_delta(&mut self) -> Option<PlayerDirtyDelta> {
        if self.dirty_columns.is_empty() {
            return None;
        }
        let mut updated = BTreeMap::new();
        std::mem::swap(&mut updated, &mut self.dirty_columns);
        Some(PlayerDirtyDelta {
            uid: self.uid,
            updated_columns: updated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_authoritative_gold_and_belly_rejects_underflow() {
        let mut actor = PlayerSessionActor::new(20001, 202621270, 1, "Luffy".to_string());
        assert_eq!(actor.gold_num, 1_000_000);
        assert_eq!(actor.cur_execution, 100_000);

        // Attempting to spend 2,000,000 gold when server balance is 1,000,000 must fail
        assert!(!actor.add_gold(-2_000_000));
        assert_eq!(actor.gold_num, 1_000_000);
        assert!(actor.take_dirty_delta().is_none());

        // Spending 40 gold succeeds and records a dirty Write-Behind delta
        assert!(actor.add_gold(-40));
        assert_eq!(actor.gold_num, 999_960);

        // Adding belly above UPPER_LIMIT_BELLY clamps at 10,000,000,000
        assert!(actor.add_belly(UPPER_LIMIT_BELLY + 5000));
        assert_eq!(actor.belly_num, UPPER_LIMIT_BELLY);

        let delta = actor.take_dirty_delta().expect("expected dirty delta");
        assert_eq!(delta.uid, 20001);
        assert_eq!(delta.updated_columns.get("gold_num"), Some(&999_960));
        assert_eq!(
            delta.updated_columns.get("belly_num"),
            Some(&UPPER_LIMIT_BELLY)
        );
        assert!(actor.take_dirty_delta().is_none());
    }
}
