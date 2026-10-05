//! Mutable outfit contents; catalog metadata stays in `OutfitCatalog`.

use std::collections::{BTreeMap, HashMap};

use super::ViewedOutfitSlotInfo;

pub type SlotKey = (i32, i32, i32);
pub type SituationKey = (i32, i32, i32, i32);

#[derive(Debug, Clone)]
pub struct SituationCategory {
    pub trigger_id: i32,
    pub name: String,
    pub description: String,
    pub is_radio_button: bool,
    pub groups: Vec<SituationGroup>,
}

#[derive(Debug, Clone)]
pub struct SituationGroup {
    pub group_id: i32,
    pub secondary_id: i32,
    pub options: Vec<(String, SituationKey)>,
}

#[derive(Debug, Clone, Default)]
pub struct OutfitContents {
    pub slots: HashMap<SlotKey, ViewedOutfitSlotInfo>,
    pub situations: BTreeMap<SituationKey, bool>,
    pub secondary_slots: HashMap<i32, bool>,
}

#[derive(Debug)]
pub struct OutfitState {
    pub saved: HashMap<i64, OutfitContents>,
    pub pending_slots: BTreeMap<SlotKey, ViewedOutfitSlotInfo>,
    pub pending_situations: BTreeMap<SituationKey, bool>,
    pub viewed_weapon_options: HashMap<i32, i32>,
    pub situation_categories: Vec<SituationCategory>,
    pub max_outfits_by_source: [u32; 3],
    pub unlocked_by_source: [u32; 3],
    pub next_outfit_cost: u64,
    pub slot_cost: u64,
}

impl Default for OutfitState {
    fn default() -> Self {
        // INFERRED simulator policy: twenty purchased slots available, no
        // stamped/automatic grants, free creation/application until host prices
        // are supplied. These are not claimed as native caps or prices.
        Self {
            saved: HashMap::new(),
            pending_slots: BTreeMap::new(),
            pending_situations: BTreeMap::new(),
            viewed_weapon_options: HashMap::new(),
            // INFERRED: no native category catalog is fabricated; host provides
            // trigger/group metadata, while selections read real outfit state.
            situation_categories: Vec::new(),
            max_outfits_by_source: [0, 0, 20],
            unlocked_by_source: [0, 0, 20],
            next_outfit_cost: 0,
            slot_cost: 0,
        }
    }
}

pub(crate) use super::super::c_transmog_collection::valid_name;
