use super::*;

#[derive(Clone, Debug)]
pub struct AccountStoreCurrencyInfo {
    pub id: i64,
    pub amount: i64,
    pub max_quantity: Option<i64>,
    pub name: String,
    pub icon: i64,
}

/// Account-store item record — mirrors the official `AccountStoreItemInfo`
/// structure in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/AccountStoreDocumentation.lua`
/// (lines 252-271). Every card mixin in `Blizzard_AccountStore` reads
/// `status`, `mode`, `flags`, `price`, and `nonrefundable` from this struct
/// and the optional fields gate model-scene previews, descriptive tooltips,
/// transmog set previews, and the refund countdown overlay.
#[derive(Clone, Debug)]
pub struct AccountStoreItemInfo {
    pub id: i64,
    pub status: i64,
    pub mode: i64,
    pub currency_id: i64,
    pub flags: i64,
    pub custom_ui_model_scene_id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub nonrefundable: bool,
    pub creature_display_id: Option<i64>,
    pub transmog_set_id: Option<i64>,
    pub display_icon: Option<i64>,
    pub refund_seconds_remaining: Option<i64>,
}

/// Account-store category record — mirrors the official
/// `AccountStoreCategoryInfo` structure in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/AccountStoreDocumentation.lua`
/// (lines 229-238). `category_type` corresponds to
/// `Enum.AccountStoreCategoryType` and is read by
/// `AccountStoreItemDisplayMixin:OnCategorySelected` to choose between the
/// item-rack flavours (mounts vs. transmog vs. boost vs. service).
#[derive(Clone, Debug)]
pub struct AccountStoreCategoryInfo {
    pub id: i64,
    pub name: String,
    pub category_type: i64,
    pub icon: i64,
}

/// One shapeshift / stance form record consumed by `GetShapeshiftFormInfo`
/// and the `Blizzard_ActionBar/Shared/StanceBar.lua` mixin. Field shape
/// matches the live API: `(texture, isActive, isCastable, spellID)`. The
/// `name` field is read by `C_TooltipInfo.GetShapeshift` to render the
/// stance tooltip header.
#[derive(Clone, Debug)]
pub struct ShapeshiftForm {
    pub name: String,
    pub texture: String,
    pub spell_id: u32,
    pub is_active: bool,
    pub is_castable: bool,
}

/// First logical pet slot exposed by the simulator's pet APIs.
/// Forever shares this range; native Gamepad storage numbering is unverified.
pub const FIRST_PET_ACTION_SLOT: i32 = 1;

/// One pet action-bar slot (10 slots total; `NUM_PET_ACTION_SLOTS = 10`).
/// Drives `GetPetActionInfo`, `GetPetActionCooldown`, `CastPetAction`,
/// `TogglePetAutocast`, and `PetHasActionBar` consumed by
/// `Blizzard_ActionBar/Shared/PetActionBar.lua`.
///
/// The 9-tuple returned by `GetPetActionInfo` maps as:
/// `(name, texture, is_token, is_active, auto_cast_allowed, auto_cast_enabled,
/// spell_id, _unused, passive)`. `has_action` is `false` for empty slots —
/// the live API returns `(nil, nil, false, false, false, false, nil, false, false)`
/// in that case.
#[derive(Clone, Debug, Default)]
pub struct PetActionSlot {
    pub has_action: bool,
    pub name: Option<String>,
    pub texture: Option<String>,
    pub is_token: bool,
    pub is_active: bool,
    pub auto_cast_allowed: bool,
    pub auto_cast_enabled: bool,
    pub spell_id: Option<u32>,
    pub passive: bool,
    pub cooldown: Option<SpellCooldownState>,
}

/// Glyph cursor state read by `Blizzard_ActionBar/Shared/SpellFlyout.lua`'s
/// `SpellFlyoutPopupButtonMixin:UpdateGlyphState` and the spellbook
/// glyph-attach flow. While a glyph is on the cursor,
/// `pending_glyph_name` carries its display name and
/// `pending_glyph_removal` is true if the cursor is the "Remove Glyph"
/// pseudo-glyph rather than a normal glyph item. `attached_glyphs` maps
/// spell id → glyph display name for spells that already have a glyph
/// inscribed; the flyout reads it to badge those spells with the icon.
#[derive(Clone, Debug, Default)]
pub struct GlyphState {
    pub pending_glyph_name: Option<String>,
    pub pending_glyph_removal: bool,
    pub attached_glyphs: HashMap<i32, String>,
}

/// Kind of an on-bar highlight mark — mirrors the action source that caused
/// the bar buttons to glow (spell hover, flyout drag, pet action drag).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionHighlightKind {
    Spell,
    Flyout,
    PetAction,
}

impl ActionHighlightKind {
    /// String tag returned alongside `GetOnBarHighlightMark(action)`.
    pub fn type_tag(self) -> &'static str {
        match self {
            ActionHighlightKind::Spell => "spell",
            ActionHighlightKind::Flyout => "flyout",
            ActionHighlightKind::PetAction => "petaction",
        }
    }
}

/// Action-highlight bookkeeping read by `Blizzard_ActionBar/Shared/ActionButton.lua`.
/// `new` mirrors `ACTION_HIGHLIGHT_MARKS` (set on `MarkNewActionHighlight`,
/// cleared on `ClearNewActionHighlight`); `on_bar` mirrors
/// `ON_BAR_HIGHLIGHT_MARKS` (rebuilt by the spell/flyout/pet update verbs).
#[derive(Default, Clone, Debug)]
pub struct ActionHighlightState {
    pub new: HashSet<i32>,
    pub on_bar: HashMap<i32, ActionHighlightKind>,
}

/// Action-bar transition state read by `MultiActionBar_Update` and
/// `ActionBarController_GetCurrentActionBarState`. `busy` is true while a
/// status-tracking-bar fade or page change is mid-animation, gating
/// `Blizzard_ActionBar/Shared/StanceBar.lua` updates. `current_state` matches
/// `LE_ACTIONBAR_STATE_MAIN` (1) or `LE_ACTIONBAR_STATE_OVERRIDE` (2) — the
/// override bar is only active while a vehicle/possess override is mounted.
#[derive(Clone, Debug)]
pub struct ActionBarStateInfo {
    pub busy: bool,
    pub current_state: i32,
    /// Drives `C_ActionBar.IsPossessBarVisible()`. True while a
    /// pet/vehicle possession is granting the player the 2-slot
    /// `PossessActionBar`. `PossessActionBarMixin:Update`
    /// (`Blizzard_ActionBar/Shared/PossessActionBar.lua:10-22`) calls
    /// `IsPossessBarVisible()` to decide whether to `:Show()` the bar
    /// (when true and not currently shown) or `:Hide()` it (when false
    /// and currently shown). Default false — no possession active.
    pub possess_bar_visible: bool,
}

impl Default for ActionBarStateInfo {
    fn default() -> Self {
        Self {
            busy: false,
            current_state: 1,
            possess_bar_visible: false,
        }
    }
}

/// Minimal assisted-combat recommendation state exposed through
/// `C_AssistedCombat`. `next_cast_spell_id` is the spell Blizzard highlights
/// when assisted combat asks the player to cast a specific action-bar spell.
#[derive(Clone, Debug, Default)]
pub struct AssistedCombatState {
    pub next_cast_spell_id: Option<u32>,
}

/// Server-pushed payload for the single-button extra action bar
/// (`ExtraActionBarFrame` / `ExtraActionButton1`). `spell_id` is `Some`
/// while a quest/encounter/scenario has granted the player a temporary
/// extra-action ability and `None` once it is revoked.
///
/// The flag drives `C_ActionBar.HasExtraActionBar()`
/// (`src/lua_api/globals/action_bar_api.rs:378-380`), which
/// `ExtraActionBar_Update`
/// (`Blizzard_ActionBar/Shared/ExtraActionBar.lua:10-30`) reads to decide
/// whether to call `bar:Show()` (when true and currently hidden) or play
/// the `outro` animation (when false and currently shown). Default `None`
/// — no extra-action ability granted.
#[derive(Clone, Debug, Default)]
pub struct ExtraActionButtonState {
    pub spell_id: Option<u32>,
}

/// `C_Housing` favor-bar payload read by `HouseFavorBarMixin:Update`
/// (`vendor/wow-ui-source/Interface/AddOns/Blizzard_ActionBar/Mainline/HouseFavorBar.lua`).
/// `tracked_house_guid` is `Some` when a house is currently tracked for
/// favor display — a `None` value disables the bar entirely. `level_thresholds`
/// is indexed by level (1-based) and consumed by `GetHouseLevelFavorForLevel`;
/// out-of-range lookups (including the `level + 1` next-threshold probe past
/// the cap) return `0`, which is the sentinel the mixin checks to skip
/// `SetBarValues`.
#[derive(Clone, Debug, Default)]
pub struct ReadyCheckState {
    pub active: bool,
    pub response: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct DiscordState {
    pub oauth_authorized: bool,
    pub auth_refresh_requested: bool,
    pub guild_linked: bool,
    pub guild_link_status: Option<i32>,
    pub guild_settings: HashMap<String, bool>,
    pub server_update_requested: bool,
    pub guild_lobby_update_requested: bool,
    pub user_id: Option<String>,
    pub display_name_type: i32,
    pub servers: Vec<DiscordServer>,
    pub channels: Vec<DiscordChannel>,
}

impl DiscordState {
    pub fn channel_name(&self, channel_index: i32) -> Option<&str> {
        self.channels
            .get(one_based_index(channel_index)?)
            .map(|channel| channel.name.as_str())
    }

    pub fn server_name(&self, server_index: i32) -> Option<&str> {
        self.servers
            .get(one_based_index(server_index)?)
            .map(|server| server.name.as_str())
    }

    pub fn linkable_channel_names(&self) -> Vec<String> {
        self.channels
            .iter()
            .filter(|channel| channel.linkable)
            .map(|channel| channel.name.clone())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct DiscordServer {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct DiscordChannel {
    pub name: String,
    pub linkable: bool,
}

fn one_based_index(index: i32) -> Option<usize> {
    usize::try_from(index.checked_sub(1)?).ok()
}

#[derive(Clone, Debug, Default)]
pub struct HousingState {
    // Preserve the simulator's enabled initial policy without claiming a native default.
    pub free_place_disabled: bool,
    pub tracked_house_guid: Option<String>,
    pub inside_owned_house: bool,
    pub inside_owned_plot: bool,
    pub last_exported_blueprint_id: Option<String>,
    pub last_exported_room_blueprint_id: Option<String>,
    pub last_imported_blueprint_code: Option<String>,
    pub last_imported_room_blueprint_code: Option<String>,
    pub last_deleted_blueprint_id: Option<String>,
    pub last_renamed_blueprint: Option<(String, String)>,
    pub requested_blueprint_collection: bool,
    pub last_requested_blueprint_contents_id: Option<String>,
    pub requested_blueprint_context_contents: bool,
    pub blueprint_export_availability: i32,
    pub blueprint_feature_availability: i32,
    pub blueprint_import_availability: i32,
    pub active_house_editor_mode: i32,
    pub house_editor_player_type: Option<i32>,
    pub selected_decor_pet_guid: Option<String>,
    pub selected_decor_pet_name: Option<String>,
    pub rooms_with_decor: Vec<i32>,
    pub decor_pet_names: HashMap<String, String>,
    pub pet_attachable_decor_guids: Vec<String>,
    pub max_indoor_placement_budget: Option<i32>,
    pub max_outdoor_placement_budget: Option<i32>,
    pub spent_indoor_placement_budget: Option<i32>,
    pub spent_outdoor_placement_budget: Option<i32>,
    pub max_pet_placement_budget: Option<i32>,
    pub spent_pet_placement_budget: Option<i32>,
    pub base_room_floors: HashMap<i32, i32>,
    pub room_player_is_in: Option<i32>,
    pub selected_blueprint_floorplan: Option<i32>,
    pub current_level: i32,
    pub current_favor: i32,
    pub next_threshold: i32,
    pub max_level: i32,
    pub level_thresholds: Vec<i64>,
}

/// Profession-quality overlay payload returned by
/// `C_ActionBar.GetProfessionQualityInfo(slot)`. `inventoryQuality` matches
/// the retail 1..N tier band (T1 = lowest); `iconInventory` /
/// `iconQualityContainer` are the atlas keys
/// `ActionBarActionButtonMixin:UpdateProfessionQuality` passes to
/// `Texture:SetAtlas` for the overlay frame.
#[derive(Clone, Debug, Default)]
pub struct ProfessionQualityInfo {
    pub inventory_quality: i32,
    pub icon_inventory: String,
    pub icon_quality_container: String,
}

/// Loss-of-control cooldown payload returned by
/// `C_Spell.GetSpellLossOfControlCooldownInfo(spellID)`. Mirrors the retail
/// table shape consumed by `ActionButton_UpdateCooldown`'s lossOfControl
/// overlay branch — `isActive` flips the overlay on, the rest position the
/// swipe geometry. `should_replace_normal_cooldown` hides the underlying
/// spell cooldown swipe so the LoC overlay is the only one visible.
#[derive(Clone, Debug, Default)]
pub struct LossOfControlInfo {
    pub start_time: f64,
    pub duration: f64,
    pub mod_rate: f32,
    pub is_active: bool,
    pub should_replace_normal_cooldown: bool,
}

/// A single spell row in a legacy spell flyout. `SpellFlyout.lua` reads
/// these through `GetFlyoutSlotInfo` before creating popup buttons.
#[derive(Clone, Debug, Default)]
pub struct SpellFlyoutSlot {
    pub spell_id: u32,
    pub override_spell_id: u32,
    pub is_known: bool,
    pub spell_name: String,
    pub spec_id: i32,
}

/// Legacy spell flyout metadata returned by `GetFlyoutInfo`. Empty by
/// default so unknown flyouts are treated as unavailable.
#[derive(Clone, Debug, Default)]
pub struct SpellFlyoutInfo {
    pub name: String,
    pub description: String,
    pub is_known: bool,
    pub slots: Vec<SpellFlyoutSlot>,
}

pub use crate::c_api::c_reputation::FactionParagonInfo;

/// `MajorFactionData` row returned by `C_MajorFactions.GetMajorFactionData`
/// for a single faction. Drives `ReputationStatusBarMixin:Update` when the
/// watched faction is a major faction (Dragonflight Renown style bar).
#[derive(Clone, Debug)]
pub struct MajorFactionData {
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

/// One entry in the renown level table returned by
/// `C_MajorFactions.GetRenownLevels`. `ReputationStatusBarMixin:GetMaxLevel`
/// reads the last entry's `level` to clamp the bar.
#[derive(Clone, Debug)]
pub struct RenownLevelInfo {
    pub faction_id: i64,
    pub level: i32,
    pub locked: bool,
    pub is_milestone: bool,
    pub is_capstone: bool,
}

/// `ItemLocation` payload returned by C surfaces that hand opaque locations
/// back to Lua (e.g. `C_AzeriteItem.FindActiveAzeriteItem`). Mirrors the
/// fields populated by `ItemLocationMixin` so callers that walk
/// `:GetBagAndSlot()`/`:GetEquipmentSlot()` see the expected shape.
#[derive(Clone, Debug, Default)]
pub struct ItemLocationData {
    pub bag_id: Option<i32>,
    pub slot_index: Option<i32>,
    pub equipment_slot_index: Option<i32>,
}

/// Heart-of-Azeroth state read by `C_AzeriteItem.*`. `None` on
/// `state.azerite_item` means no Azerite item is equipped —
/// `FindActiveAzeriteItem` returns nil so `AzeriteBarMixin:Update`
/// short-circuits.
#[derive(Clone, Debug)]
pub struct AzeriteItemState {
    pub item_location: ItemLocationData,
    pub current_xp: i64,
    pub max_xp: i64,
    pub power_level: i32,
    pub unlimited_power_level: i32,
    pub unlimited_unlocked: bool,
    pub at_max_level: bool,
    pub enabled: bool,
}

/// `C_AzeriteEmpoweredItem` backing state read by
/// `Blizzard_AzeriteRespecUI` (respec flow) and `Blizzard_AzeriteUI`
/// (panel flow). The respec frame asks once for the cost, runs
/// `IsAzeriteEmpoweredItem` whenever the cursor changes, and invokes
/// `ConfirmAzeriteEmpoweredItemRespec` from the static popup. The panel
/// reads `power_text` / `spec_available` / `heart_equipped` per row and
/// writes selections via `SelectPower`. `last_close_request` /
/// `last_confirmed_respec` are write-only audit fields the simulator's
/// tests assert against — Blizzard's panel never reads them back.
#[derive(Clone, Debug, Default)]
pub struct AzeriteEmpoweredItemState {
    /// Gold (in copper) returned by `GetAzeriteEmpoweredItemRespecCost`.
    /// Drives `AzeriteRespecMixin:RefreshCostFrame`'s
    /// `SmallMoneyFrameTemplate`.
    pub respec_cost: i64,
    /// Item ids that `IsAzeriteEmpoweredItem` reports true for. Tests
    /// seed this with the ids that should drive the empowered branches.
    pub empowered_items: HashSet<i32>,
    /// Last `itemLocation` argument seen by
    /// `CloseAzeriteEmpoweredItemRespec`. Recorded for tests that want
    /// to assert close was called with a specific shape; the addon
    /// itself passes none (it's the `UIPanelWindows.showFailedFunc`
    /// signature).
    pub last_close_request: Option<ItemLocationData>,
    /// Last `itemLocation` argument handed to
    /// `ConfirmAzeriteEmpoweredItemRespec`. Tests assert on this to
    /// verify the static popup wired the right item through.
    pub last_confirmed_respec: Option<ItemLocationData>,
    /// Power text shown in the panel's per-power tooltip. Keyed by
    /// `(itemID, powerID, powerLevel)` (powerLevel = 0 Base / 1
    /// Upgraded / 2 Downgraded — `Enum.AzeritePowerLevel`).
    /// `GetPowerText` returns nil when not seeded.
    pub power_text: HashMap<(i32, i32, i32), AzeriteEmpoweredPowerText>,
    /// Whether the player has the Heart of Azeroth equipped. Drives
    /// the panel's tier grey-out via
    /// `AzeriteEmpoweredItemPowerMixin:Update`. Default `false`.
    pub heart_equipped: bool,
    /// Per-spec availability for each power. Default true unless an
    /// entry is seeded false — the panel dims unavailable powers.
    pub spec_available: HashMap<(i32, i32), bool>,
    /// Powers selected per item, accumulated by `SelectPower` and
    /// cleared by `ConfirmAzeriteEmpoweredItemRespec`. The key
    /// captures the originating `itemLocation` so two stacks of the
    /// same item id keep independent selections.
    pub selections: HashMap<AzeriteEmpoweredSelectionKey, Vec<i32>>,
}

/// Power text struct returned by `C_AzeriteEmpoweredItem.GetPowerText`
/// (`AzeriteEmpoweredItemPowerText` in
/// `AzeriteEmpoweredItemDocumentation.lua`). Both fields are required
/// — the addon reads `.name` and `.description` directly.
#[derive(Clone, Debug, Default)]
pub struct AzeriteEmpoweredPowerText {
    pub name: String,
    pub description: String,
}

/// Composite key for `AzeriteEmpoweredItemState::selections`. Combines
/// the resolved item id with the originating bag/slot or equipment
/// slot so two empowered items with the same id but different
/// locations don't share a selection list.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct AzeriteEmpoweredSelectionKey {
    pub item_id: i32,
    pub bag_id: Option<i32>,
    pub slot_index: Option<i32>,
    pub equipment_slot_index: Option<i32>,
}

/// Equipped artifact metadata read by `C_ArtifactUI.GetEquippedArtifactInfo`,
/// `GetEquippedArtifactItemID`, `IsEquippedArtifactMaxed`, and
/// `IsEquippedArtifactDisabled`. `None` on `state.equipped_artifact` means no
/// artifact is wielded — `C_ArtifactUI.GetEquippedArtifactItemID` returns nil
/// and the `Blizzard_ActionBar/Mainline/ArtifactBar.lua` mixin stays hidden.
#[derive(Clone, Debug)]
pub struct ArtifactInfo {
    pub item_id: i32,
    pub alt_item_id: i32,
    pub name: String,
    pub icon: String,
    pub total_xp: i64,
    pub points_spent: i32,
    pub quality: i32,
    pub artifact_appearance_id: i32,
    pub appearance_mod_id: i32,
    pub item_appearance_id: i32,
    pub alt_item_appearance_id: i32,
    pub alt_on_top: bool,
    pub tier: i32,
    pub maxed: bool,
    pub disabled: bool,
    /// Artifact category (`Enum.ArtifactCategory`). Used by
    /// `C_ArtifactUI.GetArtifactXPRewardTargetInfo` to gate the
    /// reward-display name/icon to the matching artifact only.
    pub category: i32,
}

/// Allied-race directory entry consumed by `C_AlliedRaces.GetRaceInfoByID`.
/// `race_file_string` is the `strupper`-folded suffix the consumer joins onto
/// `RACE_INFO_` to look up the race description (`raceFileString` =
/// `"lightforgeddraenei"` → `_G.RACE_INFO_LIGHTFORGEDDRAENEI`). `banner_color`
/// is RGB in the 0..1 range; the simulator wraps it in `CreateColor` so the
/// returned table carries the `ColorMixin:GetRGB` method the addon calls.
#[derive(Clone, Debug)]
pub struct AlliedRaceInfo {
    pub race_id: i64,
    pub male_model_id: i64,
    pub female_model_id: i64,
    pub achievement_ids: Vec<i64>,
    pub male_name: String,
    pub female_name: String,
    pub description: String,
    pub race_file_string: String,
    pub crest_atlas: String,
    pub model_background_atlas: String,
    pub banner_color: (f64, f64, f64),
    /// Racial-ability list returned by `C_AlliedRaces.GetAllRacialAbilitiesFromID`.
    /// `AlliedRacesFrameMixin:RacialAbilitiesData` iterates this with `ipairs`
    /// and feeds each entry into `AlliedRaceAbilityTemplate`.
    pub racial_abilities: Vec<AlliedRaceRacialAbility>,
}

/// One racial ability shown in the allied-race intro panel. Mirrors the
/// official `AlliedRaceRacialAbility` structure in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/AlliedRacesFrameDocumentation.lua`.
/// `icon` is the WoW fileID the ability button uses; the simulator stores
/// it as `i64` to match the API contract.
#[derive(Clone, Debug)]
pub struct AlliedRaceRacialAbility {
    pub name: String,
    pub description: String,
    pub icon: i64,
}

mod adventure_map;

pub use adventure_map::*;
