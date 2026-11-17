//! Empty-default host inputs for the Retail C_StoreSecure VAS surface.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct StoreVasState {
    pub realms: Vec<StoreVasRealmInput>,
    pub vas_realms: Vec<StoreVasRealmInput>,
    pub characters: Vec<StoreVasCharacterInput>,
    pub character_objects: Vec<StoreVasCharacterInput>,
    pub guild_character_guids: HashSet<String>,
    pub local_wow_account_guid: Option<String>,
    /// Exact account-name bytes map to an explicitly supplied WowAccount GUID.
    pub wow_account_guids: HashMap<Vec<u8>, String>,
    /// Computed client-side eligibility snapshots; not a server-supplied allow-list.
    pub race_eligibility: Vec<StoreVasRaceEligibilityInput>,
    pub guild_master_info: Vec<StoreVasGuildMasterInfoInput>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasRealmInput {
    pub virtual_realm_address: u32,
    pub realm_name: String,
    pub character_count: u32,
    pub pvp: Option<bool>,
    pub rp: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasCharacterInput {
    pub realm_address: u32,
    pub realm_name: Option<String>,
    pub guid: Option<String>,
    /// Host rank in the native full-GUID comparison order; absent means unknown.
    pub guid_order: Option<u64>,
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub race_name: Option<String>,
    pub level: u32,
    pub class_file_name: Option<String>,
    pub race_file_name: Option<String>,
    pub wow_account: u32,
    pub current_server: u32,
    pub faction: u32,
    pub sex: u32,
    pub create_screen_icon_atlas: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasRaceEligibilityInput {
    pub guid: String,
    pub service_type: u8,
    /// None means the required client-side race data is absent; Some(empty) is a result.
    pub results: Option<Vec<StoreVasEligibleRaceInput>>,
}

/// One copied result row from a host-computed client eligibility query.
#[derive(Clone, Debug, Default)]
pub struct StoreVasEligibleRaceInput {
    pub race_name: String,
    pub is_allied_race: bool,
    pub is_heritage_armor_unlocked: bool,
}

/// A copied cached guild-master result; no record is inserted by default.
#[derive(Clone, Debug, Default)]
pub struct StoreVasGuildMasterInfoInput {
    pub guid: String,
    pub guild_name: String,
    pub members: Vec<StoreVasGuildMemberInput>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasGuildMemberInput {
    pub guid: Option<String>,
    pub member_name: String,
}
