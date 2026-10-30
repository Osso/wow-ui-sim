//! Authenticated acquisition identities for the profile-specific rarity table.
use crate::client_profile::{ACTIVE, ClientProfile};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pin {
    pub profile: String,
    pub product: String,
    pub version: String,
    pub build: String,
    pub fdid: u32,
    pub content_key: String,
    pub encoding_key: String,
    pub archive: String,
    pub offset: u64,
    pub size: usize,
    pub decoded_size: usize,
}
pub fn active() -> Result<Pin, String> {
    let bytes = match ACTIVE {
        ClientProfile::Retail => {
            include_str!("../data/blizzard-ui-builds/retail-dungeon-score-rarity.json")
        }
        ClientProfile::Ptr => {
            include_str!("../data/blizzard-ui-builds/ptr-dungeon-score-rarity.json")
        }
        _ => return Err("UIDungeonScoreRarity has no supported rows for this profile".into()),
    };
    serde_json::from_str(bytes).map_err(|e| format!("invalid dungeon score rarity pin: {e}"))
}
