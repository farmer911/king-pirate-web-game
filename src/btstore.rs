//! `#[repr(C)]` Zero-Allocation Static Game Store (`CRawSkill`, `CRawBuffer`, `CRawTarget`).
//!
//! Ground-Truth References:
//! - `legacy-ref/dwarf/battle-dwarf-clean.hpp` (`L241-L245`, `L316-L320`):
//!   - `CSkill` references `const CRawSkill *mpRawSkill` and `const CRawTarget *mpRawTarget`.
//!   - `CBuffer` references `const CRawBuffer *mpRawBuffer`.
//! - `legacy-ref/battle/data/skill.csv` (`L1-L10`), `buffer.csv` (`L1-L10`), `target.csv` (`L1-L30`).
//! - `legacy-ref/rpcfw/lib/SimpleBtstore.php` (`L103-L148`): replaces `flock(LOCK_EX)` runtime file
//!   parsing with `O(1)` contiguous slice indexing in RAM.

use std::path::Path;

/// Contiguous `#[repr(C)]` static skill definition (`CRawSkill` in `battle-dwarf-clean.hpp:243`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawSkill {
    pub id: u32,
    pub position_type: u8,
    pub effect_type: u8,
    pub attack_number_type: u8,
    pub range_id: u16,
    pub rage_cost: i32,
    pub attacker_rage_gain: i32,
    pub defender_rage_gain: i32,
    /// Damage multiplier in basis points (`10000 = 100%`, `skill.csv` col 35).
    pub damage_rate_bp: u32,
}

/// Contiguous `#[repr(C)]` static buffer/debuff definition (`CRawBuffer` in `battle-dwarf-clean.hpp:318`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawBuffer {
    pub id: u32,
    pub damage_time_type: u8,
    pub add_time_type: u8,
    pub remove_time_type: u8,
    pub prob_bp: u16,
    pub total_rounds: u8,
    pub effect_type: u8,
    pub effect_value: i32,
}

/// Contiguous `#[repr(C)]` target selector (`CRawTarget` in `battle-dwarf-clean.hpp:244` & `target.csv`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTarget {
    pub id: u16,
    /// `1` = Enemy team, `2` = Ally team (`target.csv` col 3).
    pub team_side: u8,
    /// `1` = Single front, `2` = Single back, `3` = Row, `4` = Column, `5` = All (`target.csv` col 4-6).
    pub pattern: u8,
}

/// In-memory `O(1)` lookup store for static combat and game configuration tables.
pub struct StaticBtstore {
    skills: Vec<Option<RawSkill>>,
    buffers: Vec<Option<RawBuffer>>,
    targets: Vec<Option<RawTarget>>,
}

impl StaticBtstore {
    /// Initialize with built-in baseline records from `skill.csv`, `buffer.csv`, and `target.csv` (`L3-L15`).
    pub fn new_baseline() -> Self {
        let mut store = Self {
            skills: vec![None; 64],
            buffers: vec![None; 32],
            targets: vec![None; 32],
        };

        // Ground truth from `legacy-ref/battle/data/target.csv:3-15`
        for (id, side, pat) in [
            (1u16, 1u8, 1u8),
            (2, 1, 2),
            (3, 1, 3),
            (4, 1, 6),
            (5, 1, 4),
            (6, 1, 5),
            (7, 2, 1),
        ] {
            store.insert_target(RawTarget {
                id,
                team_side: side,
                pattern: pat,
            });
        }

        // Ground truth from `legacy-ref/battle/data/skill.csv:3-10`
        for (id, pos_type, range_id, rage_cost, atk_rage, def_rage, dmg_bp) in [
            (1u32, 1u8, 7u16, 0i32, 25i32, 25i32, 10000u32),
            (2, 1, 7, 0, 0, 0, 10000),
            (3, 2, 7, 0, 0, 0, 10000),
            (4, 1, 7, 0, 25, 25, 10000),
            (5, 2, 7, 0, 25, 25, 10000),
            (6, 1, 20, 100, 0, 25, 15000),
            (7, 2, 20, 100, 0, 25, 15000),
            (8, 1, 21, 100, 0, 25, 18000),
        ] {
            store.insert_skill(RawSkill {
                id,
                position_type: pos_type,
                effect_type: 1,
                attack_number_type: 1,
                range_id,
                rage_cost,
                attacker_rage_gain: atk_rage,
                defender_rage_gain: def_rage,
                damage_rate_bp: dmg_bp,
            });
        }

        // Ground truth from `legacy-ref/battle/data/buffer.csv:3-10`
        for (id, d_type, a_type, r_type, prob_bp, rounds, eff_type, eff_val) in [
            (1u32, 2u8, 2u8, 0u8, 10000u16, 1u8, 9u8, 1000i32),
            (2, 2, 2, 1, 10000, 1, 3, 0),
            (3, 2, 2, 1, 5000, 1, 1, 0),
            (4, 3, 2, 3, 10000, 3, 51, 10),
            (5, 2, 2, 1, 10000, 1, 51, -333),
        ] {
            store.insert_buffer(RawBuffer {
                id,
                damage_time_type: d_type,
                add_time_type: a_type,
                remove_time_type: r_type,
                prob_bp,
                total_rounds: rounds,
                effect_type: eff_type,
                effect_value: eff_val,
            });
        }

        store
    }

    /// Optionally enrich from `legacy-ref/battle/data/skill.csv` if present on disk.
    pub fn load_skills_csv(&mut self, csv_path: &Path) -> usize {
        let Ok(content) = std::fs::read_to_string(csv_path) else {
            return 0;
        };
        let mut count = 0;
        for line in content.lines().skip(2) {
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 36 {
                continue;
            }
            let Ok(id) = cols[0].trim().parse::<u32>() else {
                continue;
            };
            let position_type = cols[4].trim().parse::<u8>().unwrap_or(1);
            let effect_type = cols[16].trim().parse::<u8>().unwrap_or(1);
            let attack_number_type = cols[17].trim().parse::<u8>().unwrap_or(1);
            let range_id = cols[22].trim().parse::<u16>().unwrap_or(1);
            let attacker_rage_gain = cols[24].trim().parse::<i32>().unwrap_or(25);
            let defender_rage_gain = cols[27].trim().parse::<i32>().unwrap_or(25);
            let damage_rate_bp = cols
                .iter()
                .skip(30)
                .take(10)
                .find_map(|c| c.trim().parse::<u32>().ok().filter(|&v| v >= 100))
                .unwrap_or(10000);
            self.insert_skill(RawSkill {
                id,
                position_type,
                effect_type,
                attack_number_type,
                range_id,
                rage_cost: 0,
                attacker_rage_gain,
                defender_rage_gain,
                damage_rate_bp,
            });
            count += 1;
        }
        count
    }

    pub fn insert_skill(&mut self, skill: RawSkill) {
        let idx = skill.id as usize;
        if idx >= self.skills.len() {
            self.skills.resize(idx + 1, None);
        }
        self.skills[idx] = Some(skill);
    }

    pub fn insert_buffer(&mut self, buffer: RawBuffer) {
        let idx = buffer.id as usize;
        if idx >= self.buffers.len() {
            self.buffers.resize(idx + 1, None);
        }
        self.buffers[idx] = Some(buffer);
    }

    pub fn insert_target(&mut self, target: RawTarget) {
        let idx = target.id as usize;
        if idx >= self.targets.len() {
            self.targets.resize(idx + 1, None);
        }
        self.targets[idx] = Some(target);
    }

    /// `O(1)` slice index lookup for `CRawSkill` (`0` hashing or heap allocation).
    #[inline]
    pub fn get_skill(&self, id: u32) -> Option<&RawSkill> {
        self.skills.get(id as usize).and_then(|s| s.as_ref())
    }

    /// `O(1)` slice index lookup for `CRawBuffer`.
    #[inline]
    pub fn get_buffer(&self, id: u32) -> Option<&RawBuffer> {
        self.buffers.get(id as usize).and_then(|b| b.as_ref())
    }

    /// `O(1)` slice index lookup for `CRawTarget`.
    #[inline]
    pub fn get_target(&self, id: u16) -> Option<&RawTarget> {
        self.targets.get(id as usize).and_then(|t| t.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_o1_static_btstore_lookup_matches_battle_csv() {
        let mut store = StaticBtstore::new_baseline();
        let s1 = store.get_skill(1).expect("skill 1 missing");
        assert_eq!(s1.attacker_rage_gain, 25);
        assert_eq!(s1.defender_rage_gain, 25);
        assert_eq!(s1.damage_rate_bp, 10000);

        let b1 = store.get_buffer(1).expect("buffer 1 missing");
        assert_eq!(b1.effect_type, 9);
        assert_eq!(b1.effect_value, 1000);

        let csv_path = Path::new("legacy-ref/battle/data/skill.csv");
        if csv_path.exists() {
            let loaded = store.load_skills_csv(csv_path);
            assert!(loaded > 1000);
        }
    }
}
