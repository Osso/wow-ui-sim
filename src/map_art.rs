//! Map art (UiMapArt layers and tiles) for the active client.
//!
//! WoW Forever uses its own uiMap IDs (1411-1458 and so on), which the retail
//! tables do not carry, so that client reads the table generated from its own
//! build's exports.

pub use sim_data::map_art::{MapArtInfo, MapArtLayer};

pub fn get_map_art(map_id: u32) -> Option<&'static MapArtInfo> {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::WowForever => {
            sim_data::map_art_forever::get_map_art(map_id)
        }
        _ => sim_data::map_art::get_map_art(map_id),
    }
}

#[cfg(all(test, feature = "client-wowforever"))]
mod tests {
    #[test]
    fn forever_maps_have_their_own_art() {
        // Stormwind City and Elwynn Forest, uiMap IDs only the Forever client uses.
        assert_eq!(super::get_map_art(1453).map(|a| a.art_id), Some(2146));
        assert_eq!(super::get_map_art(1429).map(|a| a.art_id), Some(2153));
    }
}
