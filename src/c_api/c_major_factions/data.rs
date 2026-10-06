/// `MajorFactionData` row returned by `C_MajorFactions.GetMajorFactionData`
/// for a single faction. Drives `ReputationStatusBarMixin:Update` when the
/// watched faction is a major faction (Dragonflight Renown style bar).
#[derive(Clone, Debug, Default)]
pub struct MajorFactionData {
    pub description: String,
    pub highlights: Vec<RenownHighlightInfo>,
    pub bounty_set_id: i32,
    pub use_journey_unlock_toast: bool,
    pub renown_track_level_effect_id: Option<i32>,
    pub player_companion_id: Option<i32>,
    pub faction_id: i64,
    pub name: String,
    pub expansion_filter: i32,
    pub max_level: i32,
    pub renown_level: i32,
    pub renown_reputation_earned: i32,
    pub renown_level_threshold: i32,
    pub ui_priority: i32,
    pub is_unlocked: bool,
    pub unlock_description: Option<String>,
    pub celebration_sound_kit: i32,
    pub renown_fanfare_sound_kit_id: i32,
    pub texture_kit: String,
    /// `DBColorExport.color` (RGB in 0..1 range). The simulator wraps it in
    /// `CreateColor` so `factionFontColor.color:GetRGB()` works in
    /// `JourneysProgressBarMixin:RefreshBar`.
    pub faction_font_color: (f32, f32, f32),
}

#[derive(Clone, Debug, Default)]
pub struct RenownHighlightInfo {
    pub title: String,
    pub description: String,
    pub level: i32,
}
