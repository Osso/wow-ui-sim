use std::collections::HashMap;

/// Zone-choice descriptor for the adventure map. A zone choice is one of
/// the competing quests on the Broken Isles starting-zone selection
/// screen, exposed via `C_AdventureMap.GetZoneChoiceInfo`.
/// `texture_kit` is `"alliance"`, `"horde"`, or a faction-neutral kit
/// name; the addon uses it to pick the portrait atlas. `normalized_x`
/// and `normalized_y` are 0..1 canvas coordinates the pin uses to place
/// itself; either being missing makes
/// `AdventureMap_IsQuestValid`-style guards reject the choice.
#[derive(Clone, Debug, Default)]
pub struct AdventureMapZoneChoice {
    pub quest_id: i64,
    pub texture_kit: String,
    pub name: String,
    pub zone_description: String,
    pub normalized_x: f64,
    pub normalized_y: f64,
}

/// Portrait sub-shape consumed by `QuestFrame_ShowQuestPortrait` from
/// the adventure-map quest-choice dialog. Mirrors the fields documented
/// on `AdventureMapQuestPortraitInfo`. The dialog skips rendering when
/// `portrait_display_id == 0`, so a default-constructed entry is the
/// "no portrait" sentinel. `model_scene_id` is `None` when the offer
/// uses the legacy display-id portrait path instead of a model scene.
#[derive(Clone, Debug, Default)]
pub struct AdventureMapQuestPortrait {
    pub portrait_display_id: i64,
    pub mount_portrait_display_id: i64,
    pub model_scene_id: Option<i64>,
    pub text: String,
    pub name: String,
}

/// Dialog-shaped quest text used by the adventure-map quest-choice
/// dialog (`AdventureMapQuestChoiceDialogMixin:RefreshDetails`). Indexed
/// by quest id and surfaced via `C_AdventureMap.GetQuestInfo`. Missing
/// entries cause the API to return zero values; the dialog uses the
/// `if descriptionText then ...` guard to skip rendering when the
/// quest text is unknown.
#[derive(Clone, Debug, Default)]
pub struct AdventureMapQuestInfo {
    pub title: String,
    pub description: String,
    pub objective_text: String,
}

/// Quest-offer descriptor for the adventure map. A quest offer is a
/// standard non-legendary pin advertised on the canvas, surfaced via
/// `C_AdventureMap.GetQuestOfferInfo`. `is_trivial`, `frequency`, and
/// `is_legendary` drive the pin variant the
/// `AdventureMap_QuestOfferDataProviderMixin:RefreshAllData` loop
/// chooses; `inset_index` is `Some` when the offer renders inside an
/// inset and `None` for offers anchored to the main canvas.
#[derive(Clone, Debug, Default)]
pub struct AdventureMapQuestOffer {
    pub quest_id: i64,
    pub is_trivial: bool,
    pub frequency: i64,
    pub is_legendary: bool,
    pub title: String,
    pub description: String,
    pub normalized_x: f64,
    pub normalized_y: f64,
    pub inset_index: Option<i64>,
}

/// Inset frame descriptor for the adventure map. An inset is a sub-region
/// close-up panel published by `C_AdventureMap.GetMapInsetInfo`.
/// `normalized_x` and `normalized_y` are 0..1 canvas coordinates that
/// `AdventureMapInsetMixin:Initialize` converts to a `SetPoint` offset.
/// `num_detail_tiles` is the count `BuildDetailTiles` iterates over;
/// `detail_tiles` holds the BLP file-data ids `Texture:SetTexture`
/// receives one per slot. The two are decoupled so the simulator can
/// publish a different `num_detail_tiles` from `detail_tiles.len()` if
/// the test wants to exercise the iteration shape independently.
#[derive(Clone, Debug, Default)]
pub struct AdventureMapInset {
    pub map_id: i64,
    pub title: String,
    pub description: String,
    pub collapsed_icon: String,
    pub area_table_id: i64,
    pub num_detail_tiles: i64,
    pub normalized_x: f64,
    pub normalized_y: f64,
    pub detail_tiles: Vec<i64>,
}

/// Adventure-map (Broken Isles / Garrison-style world map) state. Drives
/// the `C_AdventureMap` namespace consumed by the Blizzard_AdventureMap
/// addon. `map_id` is the currently-selected adventure-map UI map id
/// (0 when no adventure map is active). `last_closed` is the elapsed
/// game time (seconds since `start_time`) when the player last closed
/// the adventure map, or `None` if the map has not been closed this
/// session. `insets` is `None` until inset metadata has been published
/// for the active map, mirroring `C_AdventureMap.GetNumMapInsets`'s
/// nil-or-number contract. `zone_choices` defaults to empty (not
/// `None`) because `C_AdventureMap.GetNumZoneChoices` always returns
/// a number — zero before any choices are published. `quest_offers`
/// likewise defaults to empty: `C_AdventureMap.GetNumQuestOffers`
/// always returns a number, and the
/// `AdventureMap_QuestOfferDataProviderMixin:RefreshAllData` loop
/// drives off it without a nil guard. `quest_info` carries the
/// dialog-shaped (title, description, objective) text consumed by
/// `AdventureMapQuestChoiceDialogMixin:RefreshDetails`; missing keys
/// make `C_AdventureMap.GetQuestInfo` return zero values so the
/// dialog's `if descriptionText then ...` guard can short-circuit.
/// `quest_portraits` carries the portrait sub-shape consumed by the
/// same dialog's `QuestFrame_ShowQuestPortrait` branch; missing keys
/// make `C_AdventureMap.GetQuestPortraitInfo` return zero values so
/// the dialog's `if portraitInfo and ...` guard short-circuits.
/// `texture_kit` is the per-map texture-kit identifier returned by
/// `C_AdventureMap.GetAdventureMapTextureKit`; the
/// `AdventureMap_QuestOfferDataProviderMixin` portrait-atlas branch
/// reads this string and switches on `"midnight"`. Defaults to the
/// empty string so the kit-specific branch is taken only when a
/// scenario explicitly seeds a kit.
/// State backing the Adventure Guide / `C_EncounterJournal` surface.
/// Mirrors the small slice of UI state the addon would otherwise keep
/// in client-side globals: which tier the player has selected, which
/// instance/encounter is being displayed, the current difficulty,
/// loot filters (class+spec, slot), and an in-flight search.
#[derive(Clone, Debug)]
pub struct EncounterJournalState {
    /// Tier order index (1..N), matching `JournalTier.order`. Defaults
    /// to the latest visible expansion.
    pub current_tier: u32,
    /// Selected `JournalInstance.id` (raid or dungeon), or 0 when none.
    pub current_instance: u32,
    /// UNKNOWN (R88gb.1, Retail 12.1.5.70688 / Forever 1.60.1.70707):
    /// host-supplied per-instance mask; native DB2 source was not identified.
    #[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
    pub instance_difficulty_masks: HashMap<u32, u64>,
    /// Selected `JournalEncounter.id`, or 0 when no boss tab is open.
    pub current_encounter: u32,
    /// Active raid/dungeon `DifficultyID`. Defaults to Normal Raid (14).
    pub difficulty: u32,
    /// Loot filter — `classID` (1..13) or 0 for "all".
    pub class_filter: u32,
    /// Loot filter — `specID` or 0 for "all".
    pub spec_filter: u32,
    /// Slot filter — `Enum.ItemSlotFilterType` member, or -1 for "all".
    pub slot_filter: i32,
    /// Whether the panel is currently showing raids (true) or dungeons.
    pub is_raid: bool,
    /// In-flight search text (set by `EJ_SetSearch`).
    pub search_text: String,
    /// Cached search results (item/encounter IDs hit by the last search).
    pub search_results: Vec<EncounterJournalSearchResult>,
    /// Whether the last search has finished indexing.
    pub search_finished: bool,
    /// Currently active EJ tab (1=Suggested, 2=Dungeons, 3=Raids,
    /// 4=Loot, 5=Search).
    pub current_tab: u32,
    /// Zero-based carousel offset for `C_AdventureJournal` suggestions.
    pub adventure_primary_offset: u32,
    /// INFERRED: R88dn (native evidence pending). The pinned
    /// `AdventureGuideUtil.lua:10-11` consumer gates the Encounter Journal
    /// on this supplied availability observation; normal characters default true.
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub adventure_can_be_shown: bool,
    /// INFERRED R88dj (native evidence pending): supplied encounter rows by UI map ID.
    #[cfg(feature = "client-retail")]
    pub(crate) map_encounters:
        HashMap<u32, Vec<crate::c_api::c_encounter_journal_map::EncounterJournalMapEncounter>>,
    /// INFERRED R88cm host-supplied item-set rows for Retail and Forever.
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub item_set_catalog: crate::c_api::c_loot_journal::ItemSetCatalog,
}

impl Default for EncounterJournalState {
    fn default() -> Self {
        Self {
            current_tier: 12,
            current_instance: 0,
            #[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
            instance_difficulty_masks: HashMap::new(),
            current_encounter: 0,
            difficulty: 14,
            class_filter: 0,
            spec_filter: 0,
            slot_filter: -1,
            is_raid: true,
            search_text: String::new(),
            search_results: Vec::new(),
            search_finished: true,
            current_tab: 3,
            adventure_primary_offset: 0,
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            adventure_can_be_shown: true,
            #[cfg(feature = "client-retail")]
            map_encounters: HashMap::new(),
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            item_set_catalog: Default::default(),
        }
    }
}

/// One row returned by `EJ_GetSearchResult`. `kind` mirrors the EJ
/// search-result type id (1=instance, 2=encounter, 3=section, 4=item).
#[derive(Clone, Debug, Default)]
pub struct EncounterJournalSearchResult {
    pub id: u32,
    pub kind: u8,
    pub difficulty_id: u32,
    pub instance_id: u32,
    pub encounter_id: u32,
    pub icon: u32,
    pub item_link: String,
}

#[derive(Clone, Debug, Default)]
pub struct AdventureMapState {
    pub map_id: i64,
    pub last_closed: Option<f64>,
    pub insets: Option<Vec<AdventureMapInset>>,
    pub zone_choices: Vec<AdventureMapZoneChoice>,
    pub quest_offers: Vec<AdventureMapQuestOffer>,
    pub quest_info: HashMap<i64, AdventureMapQuestInfo>,
    pub quest_portraits: HashMap<i64, AdventureMapQuestPortrait>,
    pub texture_kit: String,
}

/// Recorded args from the most recent `QuestFrame_ShowQuestPortrait`
/// invocation. The dialog re-uses these globals to attach an NPC
/// portrait to whichever frame opens a quest, so tests assert against
/// this recorded state to verify the right parent / portrait IDs were
/// requested. `QuestFrame_HideQuestPortrait` clears the field by
/// setting `SimState.quest_portrait_state` to `None`.
#[derive(Clone, Debug, PartialEq)]
pub struct QuestPortraitState {
    pub parent_frame_id: Option<u64>,
    pub portrait_display_id: i32,
    pub mount_portrait_display_id: i32,
    pub model_scene_id: i32,
    pub text: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    /// Mirrors the `useCompactDescription` flag the dialog passes through
    /// (named `hideModel` by the PLAN since `AM_QuestDialog.lua` sets
    /// `true` here purely to suppress the 3D model frame).
    pub hide_model: bool,
}

/// One currency cost row for an anima-diversion node — mirrors the
/// official `AnimaDiversionCostInfo` documented in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/AnimaDiversionUIDocumentation.lua`
/// (lines 124-127). `currency_id` is the currency rewarded/spent and
/// `quantity` is the magnitude. Each `AnimaDiversionNodeInfo` carries
/// an array of these rows; the `ReinforceInfoFrameMixin` iterates them
/// to render the bullet list of refund/spend currencies.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimaDiversionCostInfo {
    pub currency_id: i64,
    pub quantity: i64,
}

/// One anima-diversion node descriptor — mirrors the official
/// `AnimaDiversionNodeInfo` documented in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/AnimaDiversionUIDocumentation.lua`
/// (lines 130-141). `state` is `Enum.AnimaDiversionNodeState` (0 =
/// Unavailable, 1 = Available, 2 = SelectedTemporary, 3 =
/// SelectedPermanent, 4 = Cooldown). `normalized_position` is the
/// 0..1 canvas coordinate that `AnimaDiversionPinMixin:Init` consumes
/// via `Vector2DMixin`. The `talent_id` is the unique per-node anchor
/// the addon uses for `C_AnimaDiversion.SelectAnimaNode` round-trips.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimaDiversionNodeInfo {
    pub talent_id: i64,
    pub name: String,
    pub description: String,
    pub costs: Vec<AnimaDiversionCostInfo>,
    pub currency_id: i64,
    pub icon: i64,
    pub normalized_position_x: f64,
    pub normalized_position_y: f64,
    pub state: i64,
}

/// Anima-diversion frame state — backs the `C_AnimaDiversion`
/// namespace consumed by `Blizzard_AnimaDiversionUI`. `texture_kit` is
/// the asset prefix `AnimaDiversionFrameMixin:SetupTextureKits` reads
/// (default empty string so the kit-specific code path is taken only
/// when a scenario seeds one). `map_id` is the UI map id the bolster
/// progress bar / reinforce frame anchor against. `title` is the
/// dialog title `TryShow` displays. `origin_position` is the player's
/// position on the canvas, returned as a `Vector2DMixin`-shaped
/// `{x, y}` table — `None` means the API returns nil, matching the
/// official Nilable annotation. `reinforce_progress` is the
/// 0..1 fill fraction of the bolster bar; `nodes` is the talent grid
/// returned by `GetAnimaDiversionNodes`. `last_selected_talent_id` and
/// `last_selected_temporary` record the most recent
/// `SelectAnimaNode(talentID, temporary)` request so tests can assert
/// the round-trip.
#[derive(Clone, Debug, Default)]
pub struct AnimaDiversionState {
    pub texture_kit: String,
    pub title: String,
    pub map_id: i64,
    pub origin_position: Option<(f64, f64)>,
    pub reinforce_progress: f64,
    pub nodes: Vec<AnimaDiversionNodeInfo>,
    pub last_selected_talent_id: Option<i64>,
    pub last_selected_temporary: Option<bool>,
}

/// One currency-cost row on a Garrison talent — mirrors the official
/// `GarrisonTalentCurrencyCostInfo` structure documented in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/GarrisonSharedDocumentation.lua`
/// (lines 34-39). `Blizzard_AnimaDiversionUI/AnimaDiversionDataProvider.lua:289`
/// iterates this list to compare against `GetCurrencyInfo(currencyType).quantity`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarrisonTalentCurrencyCostInfo {
    pub currency_type: i64,
    pub currency_quantity: i64,
}

/// One Garrison talent descriptor — mirrors the official
/// `GarrisonTalentInfo` documented in
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_APIDocumentationGenerated/GarrisonSharedDocumentation.lua`
/// (lines 43-73). The simulator surfaces only the fields that
/// `Blizzard_AnimaDiversionUI` actually reads through
/// `C_Garrison.GetTalentInfo` plus the formatter helper
/// `GetGarrisonTalentCostString`. Field names use the canonical Blizzard
/// names (`talent_max_rank` not `max_talent_rank`, `start_time` not
/// `research_start_time`) so the Lua surface marshals into the official
/// keys without translation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GarrisonTalentInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub icon: i64,
    pub tier: i64,
    pub ui_order: i64,
    pub talent_rank: i64,
    pub talent_max_rank: i64,
    pub is_being_researched: bool,
    pub researched: bool,
    pub selected: bool,
    pub perk_spell_id: i64,
    pub talent_availability: i64,
    pub research_duration: i64,
    pub start_time: i64,
    pub time_remaining: i64,
    pub research_gold_cost: i64,
    pub research_currency_costs: Vec<GarrisonTalentCurrencyCostInfo>,
}

/// Garrison-talent state backing `C_Garrison.GetTalentInfo` and
/// `C_Garrison.GetTalentUnlockWorldQuest`. The lookups are keyed by
/// talent id so seeding a single talent (`talents.insert(102, info)`)
/// answers `GetTalentInfo(102)` directly. `unlock_world_quests` powers
/// the unlock-quest probe used by `Blizzard_AnimaDiversionUI` to gate
/// the "click to channel" branch and feed `HaveQuestRewardData`.
#[derive(Clone, Debug, Default)]
pub struct GarrisonTalentState {
    pub talents: HashMap<i64, GarrisonTalentInfo>,
    pub unlock_world_quests: HashMap<i64, i64>,
}

/// OS clipboard state backing the `CopyToClipboard(text, removeMarkup?)`
/// global. The simulator never touches the real OS clipboard — tests
/// inspect `last_text` to assert what the addon would have copied. The
/// `removeMarkup` flag mirrors the second arg so callers can verify the
/// requested mode without re-running the strip helper.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipboardState {
    pub last_text: Option<String>,
    pub last_remove_markup: bool,
}

/// Args captured by the `ChatFrameUtil.OpenChat(text, chatType?,
/// cursorPosition?)` helper. `chat_type` is stored as a string when
/// real WoW callers pass either a chat-type token or a chat-frame
/// name; `None` matches the common `nil` second-arg case (e.g.
/// `Blizzard_APIDocumentation.lua:81`). `cursor_position` is the
/// byte-offset the addon wants the cursor parked at — the
/// APIDocumentation `/api dump` flow uses this to land just past the
/// `"/dump "` prefix.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatEditOpenState {
    pub text: String,
    pub chat_type: Option<String>,
    pub cursor_position: Option<i64>,
}

pub const DEFAULT_LFG_QUEUE_POP_DELAY_SECONDS: f64 = 5.0;

#[derive(Clone, Debug, PartialEq)]
pub struct LfgProposalState {
    pub category: i32,
    pub dungeon_id: i32,
}
