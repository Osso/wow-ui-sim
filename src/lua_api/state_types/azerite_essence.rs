//! `C_AzeriteEssence` sim-state types.
//!
//! Backs the surface read by `Blizzard_AzeriteEssenceUI`. The addon walks
//! `GetMilestones()` to build the radial slot frames, `GetEssences()` to
//! populate the right-side scroll list, and a handful of single-id probes
//! (`GetEssenceInfo`, `GetMilestoneInfo`, `GetMilestoneSpell`) when
//! refreshing per-frame state.

use std::collections::HashMap;

/// One azerite milestone (a slot or stat-bonus node on the Heart of
/// Azeroth's radial layout). The addon's `SetupMilestones` chooses an
/// XML template per row by checking `slot`, `rank`, and the
/// `Enum.AzeriteEssenceSlot.MainSlot` shortcut; mirror those fields
/// 1:1 so the simulator's table builder can hand back the canonical
/// shape.
#[derive(Clone, Debug)]
pub struct AzeriteEssenceMilestoneInfo {
    /// Server-assigned milestone id. Used as the lookup key for
    /// `GetMilestoneInfo` and `GetMilestoneSpell`.
    pub id: i32,
    /// Heart-of-Azeroth power level required to unlock this milestone.
    pub required_level: i32,
    /// `Enum.AzeriteEssenceSlot` value (`MainSlot=0`, `PassiveOneSlot=1`,
    /// `PassiveTwoSlot=2`, `PassiveThreeSlot=3`). `None` for the
    /// stat-bonus stamina row, which the addon detects by the absence
    /// of both `slot` and `rank`.
    pub slot: Option<i32>,
    /// Whether the player has unlocked this milestone.
    pub unlocked: bool,
    /// Whether the player meets the requirements to unlock now (the
    /// addon shows a button-press prompt when this is true).
    pub can_unlock: bool,
    /// Convenience flag — true when `slot == Some(MainSlot)`. Mirrors
    /// the field the addon's `IsMajorSlot()` method reads.
    pub is_major_slot: bool,
    /// Scale factor applied to the milestone's reveal swirl.
    pub swirl_scale: f32,
    /// True when activating this milestone only applies an aura
    /// (rather than a click-to-activate spell).
    pub requires_only_aura: bool,
    /// Spell granted when this milestone unlocks. Returned by
    /// `GetMilestoneSpell`.
    pub spell_id: i32,
    /// Rank shown next to the row label for ranked milestones (the
    /// stamina-tier rows). `None` for slot milestones.
    pub rank: Option<i32>,
    /// Currently-active essence id sitting in this milestone's slot.
    /// Returned by `GetMilestoneEssence`. `None` when empty.
    pub active_essence_id: Option<i32>,
}

/// One azerite essence (a power that can be slotted into a milestone).
/// `valid` is the role/spec gate — invalid essences still appear in the
/// list under a collapsible header. `accessRank` is the highest rank
/// the player has unlocked across all sources for this essence.
#[derive(Clone, Debug)]
pub struct AzeriteEssenceInfo {
    /// Essence id (lookup key for `GetEssenceInfo`).
    pub id: i32,
    /// Display name shown in the list.
    pub name: String,
    /// Currently-equipped rank (1-4).
    pub rank: i32,
    /// File-data id for the essence icon. Stored as a number to match
    /// what `SetTexture` consumes when it receives a fileDataID.
    pub icon: i32,
    /// True when the player has unlocked this essence at any rank.
    pub unlocked: bool,
    /// True when the essence is usable by the current spec/role.
    pub valid: bool,
    /// Highest rank the player has access to.
    pub access_rank: i32,
    /// True when the player has never activated this essence (drives
    /// the "new" highlight in the list).
    pub has_never_activated: bool,
}

/// `C_AzeriteEssence` backing state. Default values reflect a fresh
/// character with no Heart of Azeroth equipped: the panel can't open
/// (`CanOpenUI` returns false), nothing is unlocked, no pending
/// activation. Tests seed the lists they need.
#[derive(Clone, Debug)]
pub struct AzeriteEssenceState {
    /// Milestones in canonical order — the addon's `SetupMilestones`
    /// walks this list 1-based and pairs each entry with a row in
    /// `MILESTONE_LOCATIONS`.
    pub milestones: Vec<AzeriteEssenceMilestoneInfo>,
    /// Essence entries keyed by id. `GetEssences()` returns them in
    /// `essence_order` order so list rendering is deterministic.
    pub essences: HashMap<i32, AzeriteEssenceInfo>,
    /// Iteration order for `GetEssences`. Tests can re-order this
    /// without touching the keyed map.
    pub essence_order: Vec<i32>,
    /// Host-supplied result of the tooltip source faction-group selector.
    /// None uses the first source because race-to-selector mapping is unknown.
    pub source_faction_selector_result: Option<i32>,
    /// Source-field overrides keyed by catalog `(essence_id, rank)`.
    /// Tests use them to exercise empty-source rows without adding catalog records.
    pub tooltip_source_overrides: HashMap<(i32, u32), (String, String)>,
    /// Currently-pending-activation essence id. `None` when no
    /// activation is queued.
    pub pending_activation_essence: Option<i32>,
    /// Player-visible "X essences unlocked" counter. Returned by
    /// `GetNumUnlockedEssences`; the addon uses it to decide whether
    /// `ShouldOpenBagsOnShow` returns true.
    pub num_unlocked: i32,
    /// True while the player is standing at an Azerite forge NPC.
    /// Drives `IsAtForge`/`CloseForge` and the `AZERITE_ESSENCE_FORGE_*`
    /// event branches.
    pub is_at_forge: bool,
    /// True when the player has never activated any essence — drives
    /// the panel's first-time reveal animation.
    pub has_never_activated: bool,
    /// Whether the player currently has a Heart of Azeroth equipped.
    /// `CanOpenUI()` returns this; alts without a neck see the panel
    /// stay hidden.
    pub has_neck_equipped: bool,
    /// Heart-of-Azeroth power level, mirrored here for callers that
    /// don't want to reach through `state.azerite_item`.
    pub neck_power_level: i32,
}

impl Default for AzeriteEssenceState {
    fn default() -> Self {
        Self {
            milestones: Vec::new(),
            essences: HashMap::new(),
            essence_order: Vec::new(),
            source_faction_selector_result: None,
            tooltip_source_overrides: HashMap::new(),
            pending_activation_essence: None,
            num_unlocked: 0,
            is_at_forge: false,
            has_never_activated: false,
            has_neck_equipped: false,
            neck_power_level: 0,
        }
    }
}
