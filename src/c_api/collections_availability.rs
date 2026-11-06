//! Collections journal subsystem availability.
//!
//! PTR 12.1.5 and WowForever 1.60.1 declare both getters in their pinned
//! `Blizzard_APIDocumentationGenerated` addon:
//!
//! - `HeirloomInfoDocumentation.lua:29` — `C_HeirloomInfo.HeirloomsAvailable`.
//! - `MountJournalDocumentation.lua:498` — `C_MountJournal.MountEquipmentAvailable`.
//!
//! Neither name is declared in the Retail 12.1.5 pin, so both stay unpublished
//! there. The original consumers are
//! `Blizzard_Collections/Shared/Blizzard_Collections.lua:25` (PTR) / `:21`
//! (WowForever), which decides whether the Heirlooms tab exists, and
//! `Blizzard_Collections/Mainline/Blizzard_MountCollection.lua:161`, which wires
//! or hides the mount-equipment inset.
//!
//! Native behaviour is UNKNOWN: no decompiled implementation has been consulted.
//! Both getters therefore read supplied scenario state instead of a fixed
//! constant. The defaults are best-guess scenario defaults for the two clients
//! that ship the respective collection surface.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionsAvailability {
    /// Whether the heirloom journal is available to the current character.
    pub heirlooms: bool,
    /// Whether the mount journal offers mount equipment.
    pub mount_equipment: bool,
}

impl Default for CollectionsAvailability {
    fn default() -> Self {
        Self {
            heirlooms: true,
            mount_equipment: true,
        }
    }
}
