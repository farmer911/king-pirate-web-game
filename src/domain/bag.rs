//! Bounded Domain: `bag` (Inventory Grids, Depot, Item Composite, Treasure, Element).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L289`): `bag.bagInfo`, `bag.openGrid`, `bag.useItem`, `bag.compositeItem`.
//! - `legacy-ref/schema/pirate20002-schema.sql`: `t_bag`, `t_item`.

use crate::actor::PlayerSessionActor;
use crate::amf3::Amf3Value;
use crate::data::DataStore;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagItem {
    pub item_id: u64,
    pub item_template_id: u32,
    pub item_num: u32,
    pub reinforce_level: u16,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerBag {
    pub max_grid_num: u16,
    pub grids: BTreeMap<u16, BagItem>,
}

impl PlayerBag {
    pub fn new(initial_grids: u16) -> Self {
        Self {
            max_grid_num: initial_grids.max(24),
            grids: BTreeMap::new(),
        }
    }

    /// Add an item stack into the first available grid (`bag.receiveItem` / `Bag::addItems`).
    pub fn add_item(
        &mut self,
        store: &DataStore,
        item_template_id: u32,
        item_num: u32,
    ) -> Result<u16, &'static str> {
        for gid in 1..=self.max_grid_num {
            if let Some(existing) = self.grids.get_mut(&gid) {
                if existing.item_template_id == item_template_id {
                    existing.item_num = existing.item_num.saturating_add(item_num);
                    return Ok(gid);
                }
            }
        }
        for gid in 1..=self.max_grid_num {
            if !self.grids.contains_key(&gid) {
                let item_id = store.id_gen.next_id("item_id", 1);
                self.grids.insert(
                    gid,
                    BagItem {
                        item_id,
                        item_template_id,
                        item_num,
                        reinforce_level: 0,
                    },
                );
                return Ok(gid);
            }
        }
        Err("bag full")
    }

    /// Unlock additional inventory slots using Gold (`bag.openGrid`, `Framework.cfg.php:289`).
    pub fn open_grid_by_gold(
        &mut self,
        actor: &mut PlayerSessionActor,
        slot_count: u16,
        gold_per_slot: i64,
    ) -> bool {
        let total_cost = (slot_count as i64) * gold_per_slot;
        if slot_count == 0 || !actor.add_gold(-total_cost) {
            return false;
        }
        self.max_grid_num = self.max_grid_num.saturating_add(slot_count);
        true
    }

    pub fn to_amf3(&self) -> Amf3Value {
        let mut grid_map = BTreeMap::new();
        for (&gid, item) in &self.grids {
            grid_map.insert(
                gid.to_string(),
                Amf3Value::assoc_map([
                    ("item_id", Amf3Value::Integer(item.item_id as i32)),
                    ("item_template_id", Amf3Value::Integer(item.item_template_id as i32)),
                    ("item_num", Amf3Value::Integer(item.item_num as i32)),
                    ("reinforce_level", Amf3Value::Integer(item.reinforce_level as i32)),
                ]),
            );
        }
        Amf3Value::assoc_map([
            ("max_grid_num", Amf3Value::Integer(self.max_grid_num as i32)),
            (
                "grid",
                Amf3Value::Array {
                    assoc: grid_map,
                    dense: Vec::new(),
                },
            ),
        ])
    }
}
