//! Container metadata shared by C_Container queries and bag mutations.

use std::collections::HashMap;

use crate::lua_api::state::SimState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BagInfo {
    pub num_slots: i32,
    pub family: i32,
    pub name: Option<String>,
    pub inventory_slot: Option<i32>,
    pub item_id: Option<u32>,
    pub hyperlink: Option<String>,
}

impl BagInfo {
    pub fn default_bags() -> HashMap<i32, Self> {
        let mut bags = HashMap::new();
        for (bag, num_slots) in [
            (-4, 7),
            (-1, 28),
            (0, 16),
            (1, 16),
            (2, 16),
            (3, 16),
            (4, 16),
            (5, 0),
        ] {
            bags.insert(
                bag,
                Self {
                    num_slots,
                    family: 0,
                    name: (bag == 0).then(|| "Backpack".to_string()),
                    inventory_slot: Some((20 + bag).max(0)),
                    item_id: None,
                    hyperlink: None,
                },
            );
        }
        bags
    }
}

impl SimState {
    /// No fallback capacity: missing bags and explicitly empty bags both have zero slots.
    pub fn bag_num_slots(&self, bag: i32) -> i32 {
        self.bag_info.get(&bag).map_or(0, |info| info.num_slots)
    }

    pub fn bag_free_slots(&self, bag: i32) -> i32 {
        (1..=self.bag_num_slots(bag))
            .filter(|slot| !self.bag_items.contains_key(&(bag, *slot)))
            .count() as i32
    }
}
