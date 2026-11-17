//! Empty-default host inputs for the Retail C_StoreSecure VAS surface.

use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct StoreVasState {
    pub realms: Vec<StoreVasRealmInput>,
    pub vas_realms: Vec<StoreVasRealmInput>,
    pub characters: Vec<StoreVasCharacterInput>,
    pub character_objects: Vec<StoreVasCharacterInput>,
    pub guild_character_guids: HashSet<String>,
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
