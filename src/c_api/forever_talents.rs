//! EMULATED character inputs for WM90's ordinary seven-by-three talent grid.
//! No populated Forever 70707 database export is available. Callers supply rows
//! in provider order through WowLuaEnv::state(); these are not Retail fixtures.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct TalentRow {
    pub id: u32,
    pub class_id: i32,
    /// Database coordinates, zero-based.
    pub tier_id: u32,
    pub column_index: u32,
    pub spec_id: u32,
    pub spell_id: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TalentGroup {
    pub spec_id: u32,
    pub selected_ids: HashSet<u32>,
    pub tier_mask: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SpellIcons {
    /// Already resolved visual/difficulty row; provider selection is UNKNOWN.
    pub visual_icon: u32,
    pub misc_icon: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Inspection {
    pub guid: String,
    pub class_id: i32,
    pub groups: Vec<TalentGroup>,
    pub current_group: usize,
    /// Talent IDs extracted from enabled type-224 aura effects.
    pub aura_grants: HashSet<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct LevelAllowance {
    pub num_talents: u32,
    pub death_knight: u32,
    pub demon_hunter: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ForeverTalentState {
    /// Whether this supplied snapshot has a normal player.
    pub player_present: bool,
    pub rows: Vec<TalentRow>,
    pub spell_names: HashMap<u32, String>,
    pub spell_icons: HashMap<u32, SpellIcons>,
    pub groups: Vec<TalentGroup>,
    /// Zero-based character group, independent of Retail trait loadouts.
    pub current_group: usize,
    /// Direct stored allowance when supplied; otherwise use level records.
    pub allowance: Option<u32>,
    pub level_allowances: HashMap<u32, LevelAllowance>,
    /// EMULATED lookup inputs: native effective/base-level lifecycle is unknown.
    pub effective_level: u32,
    pub base_level: u32,
    pub aura_grants: HashSet<u32>,
    pub inspection: Option<Inspection>,
}
