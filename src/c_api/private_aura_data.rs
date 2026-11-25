//! Host-supplied private-aura entries used by the selected mainline UI consumers.

use crate::lua_api::game_data::AuraInfo;
use std::collections::HashMap;

/// INFERRED: a private aura uses the documented AuraData fields. The additional
/// fields below are read by the selected Blizzard consumers.
#[derive(Clone, Debug)]
pub struct PrivateAuraEntry {
    pub aura: AuraInfo,
    /// INFERRED from Blizzard_PrivateAurasUI.lua:424-428.
    pub can_active_player_dispel: bool,
    /// INFERRED from Blizzard_PrivateAurasUI.lua:443.
    pub hide_on_party_frames: bool,
    /// INFERRED from Blizzard_AuraContainerSources.lua:73 and the temporary shim.
    pub source_guid: Option<String>,
}

/// Explicit private-aura input, separate from ordinary unit aura state.
#[derive(Debug, Default)]
pub struct PrivateAuraData {
    pub private_auras_by_unit: HashMap<String, Vec<PrivateAuraEntry>>,
    pub aura_data_by_unit: HashMap<String, HashMap<i32, PrivateAuraEntry>>,
}
