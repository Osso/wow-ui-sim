//! Backing state for the panel-side `C_ArtifactUI` surface consumed by
//! `Blizzard_ArtifactUI` (the LoD artifact panel) plus the sibling
//! probes `C_ArtifactRelicForgeUI.IsAtForge` and
//! `C_ItemSocketInfo.IsArtifactRelicItem`.
//!
//! The action-bar subset of `C_ArtifactUI` (already wired in
//! `c_artifact_ui.rs`) reads from `state.equipped_artifact`; the panel
//! reads from a parallel `state.viewed_artifact` because retail draws a
//! distinction between the *equipped* artifact (always known if any) and
//! the *viewed* artifact (the one the panel is currently inspecting at
//! a forge or in the bag overlay). `None` means the panel has no
//! artifact data — every `MayReturnNothing` getter returns 0 values in
//! that case, matching the canonical `ArtifactUIDocumentation.lua`
//! shape.
//!
//! Field shapes are 1:1 with the structures documented in
//! `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/ArtifactUIDocumentation.lua`:
//! `ArtifactArtInfo`, `ArtifactPowerInfo`, `ArtifactAppearanceInfo`,
//! `ArtifactAppearanceSetInfo`, `ArtifactRelicInfo`, and
//! `ArtifactMetaPowerInfo`.

use super::super::state::ArtifactInfo;
use std::collections::{HashMap, HashSet};

/// `ArtifactArtInfo` returned by `GetArtifactArtInfo`. Drives the panel's
/// header label and the bar-fill colors at
/// `Blizzard_ArtifactUI.lua:225-227`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArtifactArtInfo {
    pub texture_kit: String,
    pub title_name: String,
    pub title_color: ColorRgb,
    pub bar_connected_color: ColorRgb,
    pub bar_disconnected_color: ColorRgb,
    pub ui_model_scene_id: i32,
    pub spell_visual_kit_id: i32,
}

/// RGB color (`[0.0, 1.0]`) used by the `colorRGB`/`ColorMixin` returns
/// from `GetArtifactArtInfo`. Stored with named fields rather than a
/// 3-tuple so callers can read individual channels without index magic.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// `ArtifactPowerInfo` returned by `GetPowerInfo(powerID)`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArtifactPowerInfo {
    pub spell_id: i32,
    pub cost: i32,
    pub current_rank: i32,
    pub max_rank: i32,
    pub bonus_ranks: i32,
    pub num_max_rank_bonus_from_tier: i32,
    pub prereqs_met: bool,
    pub is_start: bool,
    pub is_gold_medal: bool,
    pub is_final: bool,
    pub tier: i32,
    pub position: (f32, f32),
    pub offset: Option<(f32, f32)>,
    pub linear_index: Option<i32>,
}

/// `ArtifactAppearanceInfo` row returned by both `GetAppearanceInfo`
/// (indexed by `(setIndex, appearanceIndex)`) and `GetAppearanceInfoByID`
/// (indexed by `artifactAppearanceID`). The `set_id` field carries the
/// parent `artifactAppearanceSetID` because `GetAppearanceInfoByID` is
/// the only call site that returns it.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArtifactAppearanceInfo {
    pub set_id: i32,
    pub appearance_id: i32,
    pub name: String,
    pub display_index: i32,
    pub unlocked: bool,
    pub failure_description: Option<String>,
    pub ui_camera_id: i32,
    pub alt_hand_camera_id: Option<i32>,
    pub swatch_color: ColorRgb,
    pub model_opacity: f32,
    pub model_saturation: f32,
    pub obtainable: bool,
}

/// `ArtifactAppearanceSetInfo` returned by `GetAppearanceSetInfo`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArtifactAppearanceSetInfo {
    pub set_id: i32,
    pub name: String,
    pub description: String,
    pub num_appearances: i32,
}

/// `ArtifactRelicInfo` returned by `GetRelicInfo` /
/// `GetRelicInfoByItemID`. The same record is used for both relic-slot
/// state on the equipped artifact and the relic-by-item-id lookup.
/// `slot_type` matches the socket identifiers used by the socketing
/// system (e.g. `"Iron"`, `"Blood"`, `"Fel"`).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RelicSlotInfo {
    pub slot_type: String,
    pub locked_reason: Option<String>,
    pub name: String,
    pub icon: String,
    pub link: String,
    /// WM97 DQ4: the live socket's item descriptor (item ID, context and bonus
    /// lists). Supplied by the host; a socket without one cannot build a tooltip.
    pub socket: Option<crate::c_api::item_descriptor::ItemDescriptor>,
}

/// `ArtifactMetaPowerInfo` row returned (one per stride of 3) by
/// `GetMetaPowerInfo`. Stored as a struct so the simulator side can name
/// the fields; Lua callers consume them as a flat `(spellID, cost, rank)`
/// tuple via `select("#", ...)` strides.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MetaPowerEntry {
    pub spell_id: i32,
    pub cost: i32,
    pub current_rank: i32,
}

/// Backing record for the panel-side `C_ArtifactUI` surface. Every panel
/// getter reads from a single `ViewedArtifactState`; the action-bar's
/// `Equipped*` getters keep using `state.equipped_artifact` instead.
#[derive(Debug, Default, Clone)]
pub struct ViewedArtifactState {
    /// Core 13-tuple returned by `GetArtifactInfo`. `None` puts every
    /// `MayReturnNothing` panel getter into the no-values branch.
    pub info: Option<ArtifactInfo>,
    /// `ArtifactArtInfo` returned by `GetArtifactArtInfo`. Always
    /// populated even when `info` is `Some` — the addon reads the title
    /// name independently in `MetaPowerTooltipHelper`.
    pub art_info: ArtifactArtInfo,
    /// Trait points the player can still spend (`GetPointsRemaining`).
    pub points_remaining: i32,
    /// Sum of ranks across every owned trait
    /// (`GetTotalPurchasedRanks`). Drives the
    /// `ARTIFACTS_NUM_PURCHASED_RANKS` tooltip line.
    pub total_purchased_ranks: i32,
    /// Whether the player is currently at the artifact forge — mirrors
    /// `IsAtForge`. Distinct from
    /// `state.relic_forge_at_forge`, which is the relic-forge probe.
    pub is_at_forge: bool,
    /// Whether the panel-side artifact is disabled (`IsArtifactDisabled`).
    pub is_disabled: bool,
    /// Whether the panel-side artifact has been maxed by tier rules or
    /// an external effect (`IsMaxedByRulesOrEffect`).
    pub is_maxed_by_rules: bool,
    /// Whether the viewed artifact is the same item the player has
    /// equipped (`IsViewedArtifactEquipped`).
    pub is_viewed_equipped: bool,
    /// XP cost lookup keyed by `(startingTrait, numTraits, tier)` for
    /// `GetTotalPowerCost`. A missing entry returns 0.
    pub total_power_cost_table: HashMap<(i32, i32, i32), i64>,
    /// Meta powers returned (3 values per entry) by `GetMetaPowerInfo`.
    pub meta_powers: Vec<MetaPowerEntry>,
    /// All powers indexed by `powerID`. `GetPowerInfo` reads from this
    /// map; `GetPowers` returns the keys as a 1-based array.
    pub powers: HashMap<i32, ArtifactPowerInfo>,
    /// Power-link adjacency: `powers_links[powerID]` is the list of
    /// neighbouring `powerID`s (`GetPowerLinks`).
    pub power_links: HashMap<i32, Vec<i32>>,
    /// Powers affected by a relic at `relicSlotIndex` — backs
    /// `GetPowersAffectedByRelic`.
    pub powers_affected_by_relic_slot: HashMap<i32, Vec<i32>>,
    /// Powers affected by a relic identified by its item link — backs
    /// `GetPowersAffectedByRelicItemLink`.
    pub powers_affected_by_relic_item: HashMap<String, Vec<i32>>,
    /// Powers the player currently knows. Backs `IsPowerKnown` and
    /// flips on `AddPower`. Cleared by `ConfirmRespec`.
    pub power_known: HashSet<i32>,
    /// Appearance sets in display order. Backs `GetNumAppearanceSets`
    /// and `GetAppearanceSetInfo`.
    pub appearance_sets: Vec<ArtifactAppearanceSetInfo>,
    /// Per-appearance lookup keyed by `(setIndex, appearanceIndex)`.
    /// Backs `GetAppearanceInfo`.
    pub appearances: HashMap<(i32, i32), ArtifactAppearanceInfo>,
    /// Appearance lookup keyed by `artifactAppearanceID`. Backs
    /// `GetAppearanceInfoByID`. The 14-return shape includes the parent
    /// set id — stored on the entry's `set_id` field.
    pub appearances_by_id: HashMap<i32, ArtifactAppearanceInfo>,
    /// Currently previewed appearance id, or `None`. Backs
    /// `GetPreviewAppearance`; `SetPreviewAppearance(nil)` clears it.
    pub preview_appearance: Option<i32>,
    /// Number of artifacts the player has obtained. Backs
    /// `GetNumObtainedArtifacts`.
    pub num_obtained_artifacts: i32,
    /// Relic slots on the viewed artifact in slot-index order. Backs
    /// `GetNumRelicSlots`, `GetRelicInfo`, `GetRelicSlotType`,
    /// `GetRelicLockedReason`. `name`/`icon`/`link` empty + `slot_type`
    /// non-empty represents a slot with no relic socketed.
    pub relic_slots: Vec<RelicSlotInfo>,
    /// Relic info keyed by item id. Backs `GetRelicInfoByItemID`.
    pub relic_info_by_item_id: HashMap<i32, RelicSlotInfo>,
    /// WM104 2: item-state flags word for the viewed artifact instance. Bit
    /// 8192 suppresses enchant-derived scans, including the extra socket count.
    /// The native field name is UNKNOWN; the host supplies the value.
    pub instance_flags: u32,
    /// WM104 2: enchantment entry index 6 of the instance's thirteen entries,
    /// used only by the extra socket count. The game-facing slot name is UNKNOWN.
    pub instance_enchantment_slot6: i32,
    /// Whether a respec NPC is currently interactable. Backs
    /// `CheckRespecNPC`.
    pub respec_npc_active: bool,
    /// Forge rotation `(x, y, z)`. Backs `GetForgeRotation` /
    /// `SetForgeRotation`.
    pub forge_rotation: (f32, f32, f32),
    /// Whether the rotation OnUpdate must skip its rotation tick. Backs
    /// `ShouldSuppressForgeRotation`.
    pub suppress_forge_rotation: bool,
}

impl ViewedArtifactState {
    /// Apply an `AddPower` mutation: mark the trait as known, debit a
    /// trait point if any remain, increment the purchased-ranks counter.
    /// Returns true on success (the value `C_ArtifactUI.AddPower` returns
    /// to Lua).
    pub fn add_power(&mut self, power_id: i32) -> bool {
        if self.points_remaining <= 0 {
            return false;
        }
        if !self.power_known.insert(power_id) {
            return false;
        }
        self.points_remaining -= 1;
        self.total_purchased_ranks += 1;
        true
    }

    /// Reset `total_purchased_ranks`, refund `points_remaining`, and
    /// drop every known power. Mirrors the live `ConfirmRespec` server
    /// round-trip.
    pub fn confirm_respec(&mut self) {
        self.points_remaining += self.total_purchased_ranks;
        self.total_purchased_ranks = 0;
        self.power_known.clear();
    }
}
