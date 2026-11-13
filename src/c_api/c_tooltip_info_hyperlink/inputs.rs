//! Explicit inputs for native helpers whose state or record-provider mapping is UNKNOWN.
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArtifactDisplay {
    pub normal_maximum: u32,
    pub rank_limit: u32,
    pub use_normal_maximum: bool,
}

#[derive(Clone, Debug)]
pub struct EssenceSpells {
    pub main_spell_id: u32,
    pub passive_spell_id: u32,
    pub main_heading: String,
    pub passive_heading: String,
}

#[derive(Clone, Debug, Default)]
pub struct HyperlinkTooltipInputs {
    /// R82/R82b: the maximum helper, rank limit and player setting are UNKNOWN.
    /// No entry means no supplied display context, not a native default.
    /// Link producer: Blizzard_ArtifactUI/Blizzard_ArtifactPowerButton.lua:62.
    pub artifact_display: HashMap<u32, ArtifactDisplay>,
    /// R82: the host confirms both records and selects main/passive spells.
    /// Key: (essence ID, unsigned rank). Headings are supplied localized text.
    /// Link producers: Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI.lua:617,1228.
    pub essence_spells: HashMap<(u32, u32), EssenceSpells>,
}
