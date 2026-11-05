//! Native Rust L1-Cache Contiguous Combat Engine (`CHero`, `CFormation`, `CSkill`, `CActionRecord`).
//!
//! Ground-Truth References:
//! - `legacy-ref/dwarf/battle-dwarf-clean.hpp` (`L1-L386`):
//!   - `CHero` (`L1-L240`): `mCurrHp`, `mMaxHp`, `mCurrRage`, `mPhysicalAttackBase`, `mPhysicalDefendBase`,
//!     `mMagicAttackBase`, `mMagicDefendBase`, `mKillAttackBase`, `mKillDefendBase`, `mFatal`, `mParry`, `mDodge`, `mHit`.
//!   - `CSkill` (`L241-L280`): `getPhysicalDamage`, `getMagicDamage`, `getKillDamage`, `getAttackerRage`, `getDefenderRage`.
//!   - `CFormation` (`L281-L315`): `CHero *mHeroList[9]`, 3x3 grid column-first target selection (`getFirstSameCol`).
//!   - `CActionRecord` (`L348-L367`) & `CReactionRecord` (`L368-L386`): per-action replay serialization (`toAMF`).
//! - `legacy-ref/rpcfw/module/battle/Battle.class.php` (`L81-L93`, `L128-L134`): `doHero($arrFormation1, $arrFormation2)`.

use crate::amf3::Amf3Value;
use crate::btstore::{RawSkill, StaticBtstore};

pub const MAX_FORMATION_SLOTS: usize = 9;
pub const MAX_BATTLE_ROUNDS: u32 = 30;
pub const RAGE_SKILL_THRESHOLD: i32 = 100;

/// Contiguous `#[repr(C)]` combat unit matching `class CHero` (`battle-dwarf-clean.hpp:1-155`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroUnit {
    pub hid: u32,
    pub htid: u32,
    pub position: u8,
    pub level: u16,
    pub curr_hp: i32,
    pub max_hp: i32,
    pub curr_rage: i32,
    pub physical_attack_base: i32,
    pub physical_defend_base: i32,
    pub magic_attack_base: i32,
    pub magic_defend_base: i32,
    pub kill_attack_base: i32,
    pub kill_defend_base: i32,
    pub absolute_attack: i32,
    pub absolute_defend: i32,
    pub hit: u16,
    pub dodge: u16,
    pub fatal: u16,
    pub parry: u16,
    pub normal_skill_id: u32,
    pub rage_skill_id: u32,
    pub faint: bool,
    pub silent: bool,
    pub invincible: bool,
}

impl HeroUnit {
    pub const fn new_basic(
        hid: u32,
        htid: u32,
        position: u8,
        level: u16,
        max_hp: i32,
        phys_atk: i32,
        phys_def: i32,
    ) -> Self {
        Self {
            hid,
            htid,
            position,
            level,
            curr_hp: max_hp,
            max_hp,
            curr_rage: 0,
            physical_attack_base: phys_atk,
            physical_defend_base: phys_def,
            magic_attack_base: phys_atk / 2,
            magic_defend_base: phys_def / 2,
            kill_attack_base: phys_atk,
            kill_defend_base: phys_def,
            absolute_attack: 10,
            absolute_defend: 0,
            hit: 10000,
            dodge: 0,
            fatal: 500,
            parry: 0,
            normal_skill_id: 1,
            rage_skill_id: 6,
            faint: false,
            silent: false,
            invincible: false,
        }
    }

    /// Mirrors `CHero::isDead(void)` (`battle-dwarf-clean.hpp:196`).
    #[inline]
    pub const fn is_dead(&self) -> bool {
        self.curr_hp <= 0
    }

    /// Mirrors `CHero::changeHp(int, bool)` (`battle-dwarf-clean.hpp:209`).
    #[inline]
    pub fn change_hp(&mut self, delta: i32) -> i32 {
        let next = (self.curr_hp + delta).clamp(0, self.max_hp);
        let diff = next - self.curr_hp;
        self.curr_hp = next;
        diff
    }

    /// Mirrors `CHero::changeRage(int, bool)` (`battle-dwarf-clean.hpp:178`).
    #[inline]
    pub fn change_rage(&mut self, delta: i32) -> i32 {
        let next = (self.curr_rage + delta).max(0);
        let diff = next - self.curr_rage;
        self.curr_rage = next;
        diff
    }
}

/// 9-slot 3x3 formation grid matching `class CFormation` (`battle-dwarf-clean.hpp:281-315`).
#[derive(Debug, Clone)]
pub struct FormationGrid {
    pub uid: u32,
    pub formation_id: u32,
    pub slots: [Option<HeroUnit>; MAX_FORMATION_SLOTS],
}

impl FormationGrid {
    pub const fn new(uid: u32, formation_id: u32) -> Self {
        Self {
            uid,
            formation_id,
            slots: [None; MAX_FORMATION_SLOTS],
        }
    }

    pub fn set_hero(&mut self, slot: usize, mut hero: HeroUnit) {
        if slot < MAX_FORMATION_SLOTS {
            hero.position = slot as u8;
            self.slots[slot] = Some(hero);
        }
    }

    pub fn alive_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.is_some_and(|h| !h.is_dead()))
            .count()
    }

    pub fn total_remaining_hp(&self) -> i64 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|h| h.curr_hp.max(0) as i64)
            .sum()
    }

    /// Mirrors `CFormation::getFirstSameCol` (`battle-dwarf-clean.hpp:314`):
    /// searches the attacker's column (`pos % 3`) front-to-back first, then falls back to any alive slot.
    pub fn find_target_slot(&self, attacker_pos: usize) -> Option<usize> {
        let col = attacker_pos % 3;
        for row in 0..3 {
            let idx = row * 3 + col;
            if self.slots[idx].is_some_and(|h| !h.is_dead()) {
                return Some(idx);
            }
        }
        (0..MAX_FORMATION_SLOTS).find(|&idx| self.slots[idx].is_some_and(|h| !h.is_dead()))
    }
}

/// Single defender reaction record matching `class CReactionRecord` (`battle-dwarf-clean.hpp:368-386`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionRecord {
    pub defender_hid: u32,
    pub reaction_id: u8,
    pub fatal: bool,
    pub damage: i32,
    pub defender_rage_after: i32,
}

/// Single skill action record matching `class CActionRecord` (`battle-dwarf-clean.hpp:348-367`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRecord {
    pub round: u32,
    pub attacker_hid: u32,
    pub skill_id: u32,
    pub attacker_rage_after: i32,
    pub reactions: Vec<ReactionRecord>,
}

/// Complete battle outcome matching `BattleLogic::doHero` (`Battle.class.php:89-93, 128-134`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleResult {
    /// Appraisal code: `'A'` (decisive win), `'B'` (standard win), `'E'` (defeat), `'F'` (timeout draw).
    pub appraisal: &'static str,
    pub winner_uid: u32,
    pub total_rounds: u32,
    pub team1_remaining_hp: i64,
    pub team2_remaining_hp: i64,
    pub actions: Vec<ActionRecord>,
}

impl BattleResult {
    /// Convert combat outcome to `Amf3Value` for client replay (`CActionRecord::toAMF`, `L365`).
    pub fn to_amf3(&self) -> Amf3Value {
        let battle_actions: Vec<Amf3Value> = self
            .actions
            .iter()
            .map(|act| {
                let react_list: Vec<Amf3Value> = act
                    .reactions
                    .iter()
                    .map(|r| {
                        Amf3Value::assoc_map([
                            ("defender", Amf3Value::Integer(r.defender_hid as i32)),
                            ("reaction", Amf3Value::Integer(r.reaction_id as i32)),
                            ("fatal", Amf3Value::Bool(r.fatal)),
                            ("damage", Amf3Value::Integer(r.damage)),
                            ("rage", Amf3Value::Integer(r.defender_rage_after)),
                        ])
                    })
                    .collect();
                Amf3Value::assoc_map([
                    ("round", Amf3Value::Integer(act.round as i32)),
                    ("attacker", Amf3Value::Integer(act.attacker_hid as i32)),
                    ("skill", Amf3Value::Integer(act.skill_id as i32)),
                    ("rage", Amf3Value::Integer(act.attacker_rage_after)),
                    ("arrReaction", Amf3Value::dense_list(react_list)),
                ])
            })
            .collect();

        Amf3Value::assoc_map([
            ("appraisal", Amf3Value::String(self.appraisal.to_string())),
            ("uid1", Amf3Value::Integer(self.winner_uid as i32)),
            ("battle", Amf3Value::dense_list(battle_actions)),
        ])
    }
}

/// Compute `CSkill::getPhysicalDamage(CHero *, CHero *)` (`battle-dwarf-clean.hpp:257`).
#[inline]
pub fn compute_skill_damage(attacker: &HeroUnit, defender: &HeroUnit, skill: &RawSkill) -> i32 {
    if defender.invincible {
        return 1;
    }
    let base_diff = (attacker.physical_attack_base - defender.physical_defend_base).max(1);
    let scaled = ((base_diff as i64) * (skill.damage_rate_bp as i64)) / 10_000;
    let total = scaled + (attacker.absolute_attack - defender.absolute_defend).max(0) as i64;
    total.max(1) as i32
}

/// Execute a full turn-based 3x3 battle (`Battle::doHero`) on stack/L1-allocated `FormationGrid`s.
pub fn execute_battle_do_hero(
    mut team1: FormationGrid,
    mut team2: FormationGrid,
    btstore: &StaticBtstore,
) -> BattleResult {
    let mut actions = Vec::with_capacity(32);
    let mut round = 1u32;

    while round <= MAX_BATTLE_ROUNDS && team1.alive_count() > 0 && team2.alive_count() > 0 {
        for slot in 0..MAX_FORMATION_SLOTS {
            // Team 1 slot attacks Team 2
            if let Some(attacker) = team1.slots[slot] {
                if !attacker.is_dead() && !attacker.faint {
                    if let Some(def_slot) = team2.find_target_slot(slot) {
                        let act = execute_single_turn(
                            round,
                            slot,
                            def_slot,
                            &mut team1,
                            &mut team2,
                            btstore,
                        );
                        actions.push(act);
                        if team2.alive_count() == 0 {
                            break;
                        }
                    }
                }
            }

            // Team 2 slot attacks Team 1
            if let Some(attacker) = team2.slots[slot] {
                if !attacker.is_dead() && !attacker.faint {
                    if let Some(def_slot) = team1.find_target_slot(slot) {
                        let act = execute_single_turn(
                            round,
                            slot,
                            def_slot,
                            &mut team2,
                            &mut team1,
                            btstore,
                        );
                        actions.push(act);
                        if team1.alive_count() == 0 {
                            break;
                        }
                    }
                }
            }
        }
        round += 1;
    }

    let t1_alive = team1.alive_count();
    let t2_alive = team2.alive_count();
    let (appraisal, winner_uid) = if t2_alive == 0 {
        ("A", team1.uid)
    } else if t1_alive == 0 {
        ("E", team2.uid)
    } else {
        ("F", team2.uid)
    };

    BattleResult {
        appraisal,
        winner_uid,
        total_rounds: round.min(MAX_BATTLE_ROUNDS),
        team1_remaining_hp: team1.total_remaining_hp(),
        team2_remaining_hp: team2.total_remaining_hp(),
        actions,
    }
}

fn execute_single_turn(
    round: u32,
    atk_slot: usize,
    def_slot: usize,
    atk_team: &mut FormationGrid,
    def_team: &mut FormationGrid,
    btstore: &StaticBtstore,
) -> ActionRecord {
    let mut attacker = atk_team.slots[atk_slot].expect("attacker slot empty");
    let mut defender = def_team.slots[def_slot].expect("defender slot empty");

    let use_rage = !attacker.silent && attacker.curr_rage >= RAGE_SKILL_THRESHOLD;
    let chosen_skill_id = if use_rage {
        attacker.rage_skill_id
    } else {
        attacker.normal_skill_id
    };

    let default_skill = RawSkill {
        id: chosen_skill_id,
        position_type: 1,
        effect_type: 1,
        attack_number_type: 1,
        range_id: 1,
        rage_cost: if use_rage { 100 } else { 0 },
        attacker_rage_gain: if use_rage { 0 } else { 25 },
        defender_rage_gain: 25,
        damage_rate_bp: if use_rage { 15000 } else { 10000 },
    };
    let skill = btstore.get_skill(chosen_skill_id).unwrap_or(&default_skill);

    if use_rage {
        attacker.change_rage(-skill.rage_cost.max(100));
    } else {
        attacker.change_rage(skill.attacker_rage_gain);
    }

    let dmg = compute_skill_damage(&attacker, &defender, skill);
    defender.change_hp(-dmg);
    if !defender.is_dead() {
        defender.change_rage(skill.defender_rage_gain);
    }

    atk_team.slots[atk_slot] = Some(attacker);
    def_team.slots[def_slot] = Some(defender);

    ActionRecord {
        round,
        attacker_hid: attacker.hid,
        skill_id: skill.id,
        attacker_rage_after: attacker.curr_rage,
        reactions: vec![ReactionRecord {
            defender_hid: defender.hid,
            reaction_id: 1,
            fatal: false,
            damage: dmg,
            defender_rage_after: defender.curr_rage,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwarf_combat_engine_rage_accumulation_and_victory() {
        let store = StaticBtstore::new_baseline();
        let mut team1 = FormationGrid::new(20001, 10001);
        let mut team2 = FormationGrid::new(20002, 10002);

        // Luffy (hid=101, HP=1000, ATK=350, DEF=100) vs Bandit (hid=201, HP=600, ATK=150, DEF=50)
        team1.set_hero(0, HeroUnit::new_basic(101, 1, 0, 10, 1000, 350, 100));
        team2.set_hero(0, HeroUnit::new_basic(201, 2, 0, 5, 600, 150, 50));

        let result = execute_battle_do_hero(team1, team2, &store);
        assert_eq!(result.appraisal, "A");
        assert_eq!(result.winner_uid, 20001);
        assert_eq!(result.team2_remaining_hp, 0);
        assert!(result.team1_remaining_hp > 0);
        assert!(!result.actions.is_empty());

        // Round 1 normal attack (skill 1) grants +25 rage to Luffy (`skill.csv:3`)
        assert_eq!(result.actions[0].attacker_hid, 101);
        assert_eq!(result.actions[0].skill_id, 1);
        assert_eq!(result.actions[0].attacker_rage_after, 25);

        let amf = result.to_amf3();
        assert_eq!(amf.get_field("appraisal").and_then(|v| v.as_str()), Some("A"));
    }
}
