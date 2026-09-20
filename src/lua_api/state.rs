//! Shared state types for the WoW Lua API.
use crate::cvars::CVarStorage;
use crate::event::{EventQueue, ScriptRegistry};
use crate::lua_api::animation::AnimGroupState;
use crate::lua_api::message_frame::MessageFrameData;
use crate::lua_api::simple_html::SimpleHtmlData;
use crate::lua_api::tooltip::TooltipData;
use crate::screen::ScreenKind;
use crate::sound::SoundManager;
use crate::widget::WidgetRegistry;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

macro_rules! build_empty_sim_state {
    ($collections:ident, $runtime:ident) => {
        Self {
            widgets: WidgetRegistry::with_physical_height($runtime.screen_height),
            events: EventQueue::default(),
            #[cfg(feature = "client-wowforever")]
            input_interface_style:
                crate::c_api::c_input_interface_style::InputInterfaceStyle::default(),
            #[cfg(feature = "retail-12-1-5")]
            weather: crate::c_api::c_weather::WeatherState::default(),
            #[cfg(feature = "retail-12-1-5")]
            encounter_timeline: crate::c_api::c_encounter_timeline::Timeline::default(),
            scripts: ScriptRegistry::default(),
            cvars: CVarStorage::new(),
            console_output: $collections.console_output,
            timers: $collections.timers,
            rilua_timers: ::std::collections::VecDeque::new(),
            #[cfg(feature = "timed-signal-maps")]
            timed_signal_maps: ::std::collections::HashMap::new(),
            focused_frame_id: $runtime.focused_frame_id,
            addons: $collections.addons,
            addon_saved_enable_state: None,
            addon_saved_version_check_enabled: None,
            system_chat_log: Vec::new(),
            adventure_map: AdventureMapState::default(),
            encounter_journal: EncounterJournalState::default(),
            anima_diversion: AnimaDiversionState::default(),
            garrison_talents: GarrisonTalentState::default(),
            clipboard: ClipboardState::default(),
            chat_edit_open_state: None,
            quest_portrait_state: None,
            tooltips: $collections.tooltips,
            blocked_auras_by_unit: $collections.blocked_auras_by_unit,
            quest_blobs: $collections.quest_blobs,
            fog_of_war_frames: $collections.fog_of_war_frames,
            unit_position_frames: $collections.unit_position_frames,
            pending_player_reports: $collections.pending_player_reports,
            simple_htmls: $collections.simple_htmls,
            message_frames: $collections.message_frames,
            on_update_frames: $collections.on_update_frames,
            visible_on_update_cache: $runtime.visible_on_update_cache,
            strata_buckets: $runtime.strata_buckets,
            active_toplevel_show_orders: HashMap::new(),
            next_toplevel_show_order: 0,
            pending_hit_grid_changes: $collections.pending_hit_grid_changes,
            pending_texture_preloads: $collections.pending_texture_preloads,
            animation_groups: $collections.animation_groups,
            active_animation_groups: $collections.active_animation_groups,
            next_anim_group_id: $runtime.next_anim_group_id,
            anim_frame_to_group: $collections.anim_frame_to_group,
            anim_frame_to_anim: $collections.anim_frame_to_anim,
            screen_width: $runtime.screen_width,
            screen_height: $runtime.screen_height,
            physical_screen_width: $runtime.screen_width,
            physical_screen_height: $runtime.screen_height,
            screen_kind: $runtime.screen_kind,
            is_logged_in: $runtime.is_logged_in,
            post_event_workarounds_applied: false,
            screen_first_displayed: $runtime.screen_first_displayed,
            saved_account_name: $runtime.saved_account_name,
            saved_account_list: $runtime.saved_account_list,
            uses_token: $runtime.uses_token,
            account_save_enabled: $runtime.account_save_enabled,
            account_save_in_progress: $runtime.account_save_in_progress,
            account_locked_post_save: $runtime.account_locked_post_save,
            last_account_store_purchase_request: $runtime.last_account_store_purchase_request,
            account_store_begin_purchase_succeeds: $runtime.account_store_begin_purchase_succeeds,
            last_account_store_refund_request: $runtime.last_account_store_refund_request,
            account_store_refund_succeeds: $runtime.account_store_refund_succeeds,
            account_store_category_items: $collections.account_store_category_items,
            account_store_currency_for_store: $collections.account_store_currency_for_store,
            account_store_currency_info: $collections.account_store_currency_info,
            account_store_storefront_state: $collections.account_store_storefront_state,
            last_account_store_storefront_info_request: $runtime
                .last_account_store_storefront_info_request,
            account_store_categories: $collections.account_store_categories,
            account_store_items: $collections.account_store_items,
            action_bar_state: ActionBarStateInfo::default(),
            action_bar_page: 1,
            has_vehicle_action_bar: false,
            vehicle_bar_index: 12,
            has_override_action_bar: false,
            override_bar_skin: None,
            override_bar_index: 14,
            has_temp_shapeshift_action_bar: false,
            temp_shapeshift_bar_index: 1,
            has_bonus_action_bar: false,
            bonus_bar_index: 0,
            action_bars: $collections.action_bars,
            action_outfits: $collections.action_outfits,
            action_macros: $collections.action_macros,
            equipped_gear_outfit_action_slots: $collections.equipped_gear_outfit_action_slots,
            assisted_combat: AssistedCombatState::default(),
            action_highlights: ActionHighlightState::default(),
            extra_action_button: ExtraActionButtonState::default(),
            equipped_artifact: None,
            artifact_point_costs: HashMap::new(),
            allied_races: HashMap::new(),
            model_scenes: HashMap::new(),
            active_player_interactions: HashSet::new(),
            azerite_item: None,
            azerite_essence: AzeriteEssenceState::default(),
            azerite_empowered: AzeriteEmpoweredItemState::default(),
            barber_shop: BarberShopState::default(),
            major_factions: HashMap::new(),
            major_faction_renown_levels: HashMap::new(),
            account_wide_reputation_factions: HashSet::new(),
            faction_paragon: HashMap::new(),
            transmog_outfit_locks: HashSet::new(),
            // Simulator initial policy; native default is unverified.
            outfit_situations_enabled: false,
            transmog_set_filters: HashMap::new(),
            #[cfg(feature = "retail-12-0-0")]
            transmog_custom_sets: crate::c_api::c_transmog_collection::CustomSets::default(),
            equipped_outfit_locked: false,
            locked_action_slots: HashSet::new(),
            is_active_battlefield: false,
            spell_trade_skill_links: HashMap::new(),
            spell_id_aliases: HashMap::new(),
            spell_loss_of_control: HashMap::new(),
            spell_flyouts: HashMap::new(),
            action_profession_quality: HashMap::new(),
            addon_base_paths: $collections.addon_base_paths,
            create_frame_initial_hidden: $runtime.create_frame_initial_hidden,
            suppress_runtime_on_load_depth: $runtime.suppress_runtime_on_load_depth,
            mouse_position: $runtime.mouse_position,
            hovered_frame: $runtime.hovered_frame,
            active_drag_frame: $runtime.active_drag_frame,
            active_slider_thumb_drag_frame: $runtime.active_slider_thumb_drag_frame,
            mouse_buttons: $runtime.mouse_buttons,
            next_report_token: $runtime.next_report_token,
            party_members: $collections.party_members,
            party_group_active: $runtime.party_group_active,
            current_target: $runtime.current_target,
            previous_target: None,
            current_focus: $runtime.current_focus,
            enemy_pool: Vec::new(),
            unit_raid_target_icons: HashMap::new(),
            sound_manager: $runtime.sound_manager,
            last_sound_kit_requested: $runtime.last_sound_kit_requested,
            last_sound_file_requested: $runtime.last_sound_file_requested,
            last_stopped_sound_handle: $runtime.last_stopped_sound_handle,
            combat_audio_speaker_speed: 0.0,
            combat_text_active_unit: None,
            combat_audio_speaker_volume: 100.0,
            combat_audio_format_settings: HashMap::new(),
            neighborhood_tracked_tasks: BTreeSet::new(),
            weapon_enchants: Default::default(),
            last_launched_url: $runtime.last_launched_url,
            highlighted_map_scene_character_guid: $runtime.highlighted_map_scene_character_guid,
            secure_attribute_drivers: $collections.secure_attribute_drivers,
            rot_damage_level: $runtime.rot_damage_level,
            fps: $runtime.fps,
            start_time: $runtime.start_time,
            casting: $runtime.casting,
            channeling: $runtime.channeling,
            next_cast_id: $runtime.next_cast_id,
            gcd: $runtime.gcd,
            spell_cooldowns: $collections.spell_cooldowns,
            inventory_item_cooldowns: ::std::collections::HashMap::new(),
            action_ui_buttons: $collections.action_ui_buttons,
            cursor_item: $runtime.cursor_item,
            loading_addon_index: $runtime.loading_addon_index,
            loading_addon_stack: $runtime.loading_addon_stack,
            executing_addon_index: $runtime.executing_addon_index,
            loading_nil_symbol_environment: None,
            xml_load_addon_depth: $runtime.xml_load_addon_depth,
            loading_forbidden: $runtime.loading_forbidden,
            loading_scoped_script_env: $runtime.loading_scoped_script_env,
            loading_add_to_secure_env: $runtime.loading_add_to_secure_env,
            loading_hide_from_global_env: $runtime.loading_hide_from_global_env,
            loading_use_forbidden_object_table: $runtime.loading_use_forbidden_object_table,
            app_frame_metrics: AppFrameMetrics::default(),
            addon_performance_messages_shown: HashSet::new(),
            talents: super::talent_state::TalentState::new(),
            lua_errors: $collections.lua_errors,
            lua_error_records: $collections.lua_error_records,
            lua_error_counts: $collections.lua_error_counts,
            nil_symbol_accesses: $collections.nil_symbol_accesses,
            global_publications: HashSet::new(),
            secure_global_publications: HashSet::new(),
            pending_nested_addon_diagnostics: HashMap::new(),
            runtime_addon_diagnostics: LoadDiagnostics::default(),
            global_show_hide_depth: 0,
            anim_sync_times: $collections.anim_sync_times,
            player: PlayerState::seeded(),
            player_xp: PlayerXpState::default(),
            bind_location: "Stormwind City".into(),
            pvp_honor: PvpHonorState::default(),
            world: super::state_types::seeded_world_state(),
            bag_items: $collections.bag_items,
            tracked_recipes: $collections.tracked_recipes,
            crafting: CraftingState::default(),
            net_stats: NetStats::default(),
            store_frame_shown: false,
            timerunning_season_id: None,
            timerunning_season_seconds_remaining: 0,
            modifier_keys: ModifierKeys::default(),
            game_rules: GameRulesState::default(),
            discord: DiscordState::default(),
            player_choice: PlayerChoiceState::default(),
            housing_service_enabled: true,
            housing: HousingState::default(),
            pet_battles: PetBattleState::default(),
            pet: PetState::default(),
            lfg_list_counts: LfgListCounts::default(),
            can_use_premade_group: true,
            lfg_category_info: default_lfg_category_info(),
            lfg_active_categories: ::std::collections::HashSet::new(),
            lfg_queued_dungeons: ::std::collections::HashMap::new(),
            lfg_queue_pop_delay_seconds: DEFAULT_LFG_QUEUE_POP_DELAY_SECONDS,
            lfg_queue_pop_due_at: None,
            lfg_active_proposal: None,
            lfg_random_cooldown_units: ::std::collections::HashSet::new(),
            lfg_activity_groups: default_lfg_activity_groups(),
            lfg_activities: default_lfg_activities(),
            lfg_applications: Vec::new(),
            lfg_next_application_id: 1,
            lfg_advanced_filter: LfgAdvancedFilter::default(),
            lfg_language_filter: {
                let mut m = std::collections::HashMap::new();
                m.insert("enUS".to_string(), true);
                m
            },
            lfg_roles: LfgRoleSelection::default(),
            lfd_dungeons: default_lfd_dungeons(),
            lfd_enabled_dungeons: ::std::collections::HashMap::new(),
            photo_sharing_authorized: false,
            photo_sharing_enabled: false,
            tutorial_flags: $collections.tutorial_flags,
            wowlabs: WowLabsState::default(),
            archaeology: ArchaeologyState::default(),
            arrow_callouts: ArrowCalloutState::default(),
            gardenweald: GardenwealdState::default(),
            viewed_artifact: ViewedArtifactState::default(),
            relic_forge_at_forge: false,
            artifact_relic_items: HashSet::new(),
            quest_log: Vec::new(),
            quest_log_entries: QuestLogState::seeded(),
            pending_quest_offer: None,
            quest_choice_id: None,
            quest_choice_response_id: None,
            quest_poi_map_id: None,
            selected_quest_log_id: None,
            abandon_quest_id: None,
            tracked_achievements: ::std::collections::HashSet::new(),
            bank_frame_open: false,
            guild_bank_frame_open: false,
            bank_purchased_slots: 0,
            guild_bank_money: 123_456,
            current_guild_bank_tab: 1,
            guild_bank_items: ::std::collections::HashMap::new(),
            merchant_frame_open: false,
            tabard_frame_open: false,
            trainer_frame_open: false,
            socket_frame_open: false,
            loot_frame_open: false,
            guild_registrar_open: false,
            pet_stables_open: false,
            merchant_items: Vec::new(),
            loot_slots: Vec::new(),
            last_loot_roll_choice: None,
            auction_browse_items: Vec::new(),
            loot_method: LootMethodState::default(),
            loot_history: LootHistoryState::default(),
            gossip: GossipState::default(),
            torghast: TorghastState::default(),
            titles: Vec::new(),
            current_title: -1,
            shapeshift_forms: default_paladin_aura_forms(),
            shapeshift_cooldowns: ::std::collections::HashMap::new(),
            pet_actions: vec![PetActionSlot::default(); 10],
            glyph: GlyphState::default(),
            currency_info: super::globals::currency_data::seeded_currency_info_map(),
            equipment_manager: EquipmentManagerState::new(),
            maps: default_maps(),
            achievements: default_achievements(),
            achievement_guild_rep: ::std::collections::HashMap::new(),
            achievement_statistics: ::std::collections::HashMap::new(),
            achievement_comparison_unit: None,
            achievement_comparison_data: AchievementComparisonData::default(),
            guild_achievement_members: ::std::collections::HashMap::new(),
            achievement_search: AchievementSearchState::default(),
            focused_achievement: None,
            area_pois: default_area_pois(),
            bnet_friends: default_bnet_friends(),
            bnet_friend_invites: Vec::new(),
            bnet_appear_offline: false,
            social_friends: default_social_friends(),
            auction_browse_results: default_auction_browse_results(),
            auction_replicate_items: default_auction_replicate_items(),
            auction_owned: Vec::new(),
            auction_bids: Vec::new(),
            auction_item_searches: ::std::collections::HashMap::new(),
            auction_commodity_searches: ::std::collections::HashMap::new(),
            auction_sell_search_results: ::std::collections::HashMap::new(),
            auction_last_browse_query: None,
            auction_queued_browse_query: None,
            auction_favorites: Vec::new(),
            auction_sell_quote: None,
            auction_sell_item: None,
            selected_auction_list: 0,
            selected_auction_bidder: 0,
            selected_auction_owner: 0,
            auction_index: ::std::collections::HashMap::new(),
            commodity_purchase_quote: None,
            auction_throttle_ready: true,
            auction_should_auto_populate_price: true,
            wow_token: WowTokenState::default(),
            mythic_plus: MythicPlusState::default(),
            character_services: CharacterServicesState::default(),
            scenario: ScenarioState::default(),
            death_recaps: Vec::new(),
            chat_bubbles: Vec::new(),
            summon_request: SummonRequestState::default(),
            player_map_position: (0.5, 0.5),
            factions: Vec::new(),
            selected_faction_index: 0,
            watched_faction_index: 0,
            battlefield_queue: BattlefieldQueue::default(),
            battlefield_minimap_visible: false,
            chat_channels: Vec::new(),
            macros: Vec::new(),
            running_macro: None,
            chat_windows: ::std::collections::HashMap::new(),
            chat_type_colors: ::std::collections::HashMap::new(),
            pending_duel: None,
            pending_resurrect: None,
            corpse_available: false,
            active_trade: None,
            open_panels: ::std::collections::HashSet::new(),
            is_party_lfg: false,
            everyone_assistant: false,
            party_leader_index: None,
            ready_check: ReadyCheckState::default(),
            voice_chat: VoiceChatState::default(),
            known_spells: ::std::collections::HashSet::new(),
            harmful_spells: ::std::collections::HashSet::new(),
            helpful_spells: ::std::collections::HashSet::new(),
            pet_spells: ::std::collections::HashSet::new(),
            pvp_last_honor_gain: 0,
            equippable_items: ::std::collections::HashSet::new(),
            consumable_items: ::std::collections::HashSet::new(),
            can_replace_guild_master: false,
            auto_decline_guild_invites: false,
            guild_roster_show_offline: true,
            menu_open: false,
            xp_disabled: false,
            can_teleport: true,
            has_hearthstone: true,
            message_log: Vec::new(),
            keybindings: Keybindings::default(),
            debug_borders: false,
            debug_anchors: false,
            simulator_exit_requested: false,
        }
    };
}

// Re-export game data types so existing `crate::lua_api::state::X` imports keep working.
pub use super::game_data::AuraInfo;
pub use super::game_data::SpellCooldownState;
pub use super::game_data::{
    CLASS_LABELS, CastingState, EmpowerTiming, PartyMember, RACE_DATA, ROT_DAMAGE_LEVELS,
    TargetInfo, XP_LEVELS, tick_party_health,
};
use super::game_data::{
    default_action_bars, default_allied_races, default_major_faction_renown_levels,
    default_major_factions, default_model_scenes, default_party, default_player_buffs,
};
pub use super::state_types::{
    AchievementComparisonData, AchievementGuildRep, AchievementInfo, AchievementSearchState,
    AchievementStatistic, AddonEnableSnapshot, AddonInfo, AddonPerformanceMessageKey,
    AddonRuntimeMetrics, AppFrameMetrics, AreaPoiInfo, AuctionBrowseResult, AuctionItemClassFilter,
    AuctionReplicateItem, AuctionRowInfo, AuctionSellQuote, AuctionSellQuoteKind, AuctionSortSpec,
    AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo, AzeriteEssenceState, BagItem,
    BarberShopAlternateFormRace, BarberShopCategory, BarberShopCharacterData, BarberShopOption,
    BarberShopState, BidAuction, BnetFriend, BnetFriendInvite, BnetGameAccount, BrowseQuery,
    ChatBubble, CommodityPurchaseQuote, CommoditySearchResultInfo, CommoditySearchResults,
    CraftingState, CurrencyInfo, CursorInfo, CursorItemOrigin, DeathRecapEntry,
    EquipmentManagerState, EquipmentSet, EquippedItem, GreatVaultActivity, GuildMember, GuildRank,
    ItemSearchKey, ItemSearchResultInfo, ItemSearchResults, KillingBlowInfo, LfdDungeonInfo,
    LfgActivityGroupInfo, LfgActivityInfo, LfgAdvancedFilter, LfgApplication, LfgCategoryInfo,
    LfgRoleSelection, LoadDiagnosticAttribution, LoadDiagnostics, LootHistoryState, LootRollInfo,
    LuaErrorRecord, MacroInfo, MapChildRect, MapData, MapRect, MirrorTimer, MissingRequirement,
    MissingRequirementKind, MovementState, MythicPlusAffix, MythicPlusRatingMapSummary,
    MythicPlusRatingSummary, MythicPlusRun, MythicPlusState, MythicPlusWeeklyBest, NilSymbolAccess,
    NilSymbolEnvironment, NilSymbolObservation, NilSymbolObservationKind, OwnedAuction,
    PendingTimer, PlayerChoiceInfo, PlayerChoiceOptionButtonInfo, PlayerChoiceOptionInfo,
    PlayerChoiceOptionRewardInfo, PlayerChoiceRewardCurrencyInfo, PlayerChoiceRewardItemInfo,
    PlayerChoiceRewardReputationInfo, PlayerChoiceState, PlayerState, PlayerXpState, PvpHonorState,
    SEEDED_LOCAL_CHARACTER_GUID, SEEDED_LOCAL_CHARACTER_NAME, ScenarioState, ScenarioStep,
    SecondaryPowerState, SocialFriend, SummonRequestState, TokenAuctionInfo, WorldState,
    WowTokenState,
};
pub use super::tracked_recipes::TrackedRecipes;

// Per-frame side-table state (quest blobs, UnitPositionFrame, etc.)
// lives in `frame_substates.rs`; re-exported for existing
// `crate::lua_api::state::X` call sites.
pub use super::frame_substates::{
    FogOfWarFrameState, PendingPlayerReport, QuestBlobState, UnitPositionFrameState,
    UnitPositionPlayerPingTexture, UnitPositionUnit,
};
mod cursor_tooltips;
mod defaults;
mod sim_state;
mod support_types;

use defaults::*;
pub use sim_state::SimState;
pub use support_types::*;

// Small admin-facing state structs (NetStats, ModifierKeys,
// GameRulesState, GameRuleValue, PetBattleState, LfgListCounts,
// Keybindings) live in `sim_substates.rs`; re-exported here so existing
// `crate::lua_api::state::X` call sites keep working.
pub use super::sim_substates::{
    ArchaeologyArtifact, ArchaeologyRace, ArchaeologyState, ArrowCalloutInfo, ArrowCalloutState,
    ArtifactAppearanceInfo, ArtifactAppearanceSetInfo, ArtifactArtInfo, ArtifactPowerInfo,
    BattlefieldQueue, BattlefieldStatus, CharacterServicesState, ChatChannel, ChatWindow, ColorRgb,
    FactionEntry, GameRuleValue, GameRulesState, GardenwealdState, GossipOption, GossipQuestRow,
    GossipState, Keybindings, LfgListCounts, LootMethodState, MessageLogEntry, MetaPowerEntry,
    ModifierKeys, MouseButtons, NetStats, PetBattlePet, PetBattleState, PetState, QuestLogEntry,
    QuestLogState, QuestRewardCurrency, QuestRewardItem, RelicSlotInfo, SelectedArtifact,
    TorghastState, TradeState, ViewedArtifactState, VoiceChannel, VoiceChatState, VoiceMember,
    WowLabsAreaInfo, WowLabsCircleInfo, WowLabsDataManagerState, WowLabsMatchmakingState,
    WowLabsPartyInvite, WowLabsPartyMember, WowLabsPoint, WowLabsState,
};

#[derive(Default)]
struct EmptyStateCollections {
    console_output: Vec<String>,
    timers: VecDeque<PendingTimer>,
    addons: Vec<AddonInfo>,
    lua_errors: Vec<String>,
    lua_error_records: Vec<LuaErrorRecord>,
    lua_error_counts: HashMap<String, usize>,
    nil_symbol_accesses: Vec<NilSymbolAccess>,
    tooltips: HashMap<u64, TooltipData>,
    blocked_auras_by_unit: HashMap<String, HashSet<i32>>,
    quest_blobs: HashMap<u64, QuestBlobState>,
    fog_of_war_frames: HashMap<u64, FogOfWarFrameState>,
    unit_position_frames: HashMap<u64, UnitPositionFrameState>,
    pending_player_reports: HashMap<i64, PendingPlayerReport>,
    account_store_category_items: HashMap<i64, Vec<i64>>,
    account_store_currency_for_store: HashMap<i64, i64>,
    account_store_currency_info: HashMap<i64, AccountStoreCurrencyInfo>,
    account_store_storefront_state: HashMap<i64, i64>,
    account_store_categories: HashMap<i64, AccountStoreCategoryInfo>,
    account_store_items: HashMap<i64, AccountStoreItemInfo>,
    simple_htmls: HashMap<u64, SimpleHtmlData>,
    message_frames: HashMap<u64, MessageFrameData>,
    animation_groups: HashMap<u64, AnimGroupState>,
    active_animation_groups: HashSet<u64>,
    anim_sync_times: HashMap<String, std::time::Duration>,
    anim_frame_to_group: HashMap<u64, u64>,
    anim_frame_to_anim: HashMap<u64, (u64, usize)>,
    on_update_frames: HashSet<u64>,
    pending_hit_grid_changes: Vec<(u64, bool)>,
    pending_texture_preloads: BTreeSet<String>,
    action_bars: HashMap<u32, u32>,
    action_outfits: HashMap<u32, i64>,
    action_macros: HashMap<u32, u32>,
    equipped_gear_outfit_action_slots: HashSet<u32>,
    addon_base_paths: Vec<PathBuf>,
    spell_cooldowns: HashMap<u32, SpellCooldownState>,
    action_ui_buttons: Vec<(u64, u32)>,
    secure_attribute_drivers: HashMap<u64, HashMap<String, String>>,
    party_members: Vec<PartyMember>,
    bag_items: HashMap<(i32, i32), BagItem>,
    tracked_recipes: TrackedRecipes,
    tutorial_flags: HashSet<u32>,
}

impl EmptyStateCollections {
    fn new() -> Self {
        let mut c = Self::empty();
        c.bag_items = default_backpack_items();
        c
    }
    fn empty() -> Self {
        Self::default()
    }
}

struct EmptyRuntimeState {
    focused_frame_id: Option<u64>,
    visible_on_update_cache: Option<Vec<u64>>,
    strata_buckets: Option<Vec<Vec<u64>>>,
    create_frame_initial_hidden: Option<bool>,
    suppress_runtime_on_load_depth: u32,
    mouse_position: Option<(f32, f32)>,
    hovered_frame: Option<u64>,
    active_drag_frame: Option<u64>,
    active_slider_thumb_drag_frame: Option<u64>,
    mouse_buttons: MouseButtons,
    next_report_token: i64,
    party_group_active: bool,
    current_target: Option<TargetInfo>,
    current_focus: Option<TargetInfo>,
    sound_manager: Option<SoundManager>,
    last_sound_kit_requested: Option<u32>,
    last_sound_file_requested: Option<String>,
    last_stopped_sound_handle: Option<u32>,
    last_launched_url: Option<String>,
    highlighted_map_scene_character_guid: Option<String>,
    casting: Option<CastingState>,
    channeling: Option<CastingState>,
    gcd: Option<(f64, f64)>,
    cursor_item: Option<CursorInfo>,
    loading_addon_index: Option<u16>,
    loading_addon_stack: Vec<u16>,
    executing_addon_index: Option<u16>,
    xml_load_addon_depth: u32,
    loading_forbidden: bool,
    loading_scoped_script_env: Option<rilua::Val>,
    loading_add_to_secure_env: bool,
    loading_hide_from_global_env: bool,
    loading_use_forbidden_object_table: bool,
    next_anim_group_id: u64,
    next_cast_id: u32,
    screen_width: f32,
    screen_height: f32,
    screen_kind: ScreenKind,
    is_logged_in: bool,
    screen_first_displayed: bool,
    saved_account_name: String,
    saved_account_list: String,
    uses_token: bool,
    account_save_enabled: bool,
    account_save_in_progress: bool,
    account_locked_post_save: bool,
    last_account_store_purchase_request: Option<i64>,
    account_store_begin_purchase_succeeds: bool,
    last_account_store_refund_request: Option<i64>,
    account_store_refund_succeeds: bool,
    last_account_store_storefront_info_request: Option<i64>,
    fps: f32,
    rot_damage_level: usize,
    start_time: Instant,
}

macro_rules! build_empty_runtime_state {
    (
        start_time: $start_time:expr,
        next_report_token: $next_report_token:expr,
        next_anim_group_id: $next_anim_group_id:expr,
        next_cast_id: $next_cast_id:expr,
        screen_width: $screen_width:expr,
        screen_height: $screen_height:expr
    ) => {
        EmptyRuntimeState {
            focused_frame_id: None,
            visible_on_update_cache: None,
            strata_buckets: None,
            create_frame_initial_hidden: None,
            suppress_runtime_on_load_depth: 0,
            mouse_position: None,
            hovered_frame: None,
            active_drag_frame: None,
            active_slider_thumb_drag_frame: None,
            mouse_buttons: MouseButtons::default(),
            next_report_token: $next_report_token,
            party_group_active: false,
            current_target: None,
            current_focus: None,
            sound_manager: None,
            last_sound_kit_requested: None,
            last_sound_file_requested: None,
            last_stopped_sound_handle: None,
            last_launched_url: None,
            highlighted_map_scene_character_guid: None,
            casting: None,
            channeling: None,
            gcd: None,
            cursor_item: None,
            loading_addon_index: None,
            loading_addon_stack: Vec::new(),
            executing_addon_index: None,
            xml_load_addon_depth: 0,
            loading_forbidden: false,
            loading_scoped_script_env: None,
            loading_add_to_secure_env: false,
            loading_hide_from_global_env: false,
            loading_use_forbidden_object_table: false,
            next_anim_group_id: $next_anim_group_id,
            next_cast_id: $next_cast_id,
            screen_width: $screen_width,
            screen_height: $screen_height,
            screen_kind: ScreenKind::Game,
            is_logged_in: false,
            screen_first_displayed: false,
            saved_account_name: String::new(),
            saved_account_list: String::new(),
            uses_token: false,
            account_save_enabled: false,
            account_save_in_progress: false,
            account_locked_post_save: false,
            last_account_store_purchase_request: None,
            account_store_begin_purchase_succeeds: true,
            last_account_store_refund_request: None,
            account_store_refund_succeeds: true,
            last_account_store_storefront_info_request: None,
            fps: 0.0,
            rot_damage_level: 0,
            start_time: $start_time,
        }
    };
}

const INITIAL_REPORT_TOKEN: i64 = 1;
const INITIAL_ANIM_GROUP_ID: u64 = 1;
const INITIAL_CAST_ID: u32 = 1;
const DEFAULT_SCREEN_WIDTH: f32 = 1024.0;
const DEFAULT_SCREEN_HEIGHT: f32 = 768.0;

fn default_paladin_aura_forms() -> Vec<ShapeshiftForm> {
    const PALADIN_AURA_FORMS: &[(u32, &str)] = &[
        (465, "Devotion Aura"),
        (32223, "Crusader Aura"),
        (183435, "Retribution Aura"),
    ];

    PALADIN_AURA_FORMS
        .iter()
        .map(|(spell_id, fallback_name)| ShapeshiftForm {
            name: spell_name_or_fallback(*spell_id, fallback_name).to_string(),
            texture: spell_texture_path(*spell_id),
            spell_id: *spell_id,
            is_active: false,
            is_castable: true,
        })
        .collect()
}

fn spell_name_or_fallback<'a>(spell_id: u32, fallback: &'a str) -> &'a str {
    crate::spells::get_spell(spell_id)
        .map(|spell| spell.name)
        .unwrap_or(fallback)
}

fn spell_texture_path(spell_id: u32) -> String {
    crate::spells::get_spell(spell_id)
        .and_then(|spell| crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id))
        .unwrap_or("Interface/Icons/INV_Misc_QuestionMark")
        .to_string()
}

impl EmptyRuntimeState {
    fn new() -> Self {
        build_initialized_empty_runtime_state(Instant::now())
    }
}

fn build_initialized_empty_runtime_state(start_time: Instant) -> EmptyRuntimeState {
    build_empty_runtime_state!(
        start_time: start_time,
        next_report_token: INITIAL_REPORT_TOKEN,
        next_anim_group_id: INITIAL_ANIM_GROUP_ID,
        next_cast_id: INITIAL_CAST_ID,
        screen_width: DEFAULT_SCREEN_WIDTH,
        screen_height: DEFAULT_SCREEN_HEIGHT
    )
}

impl Default for SimState {
    fn default() -> Self {
        let mut state = Self::new_empty();
        state.seed_default_game_state();
        state
    }
}

impl SimState {
    fn seed_default_game_state(&mut self) {
        self.action_bars = default_action_bars();
        self.party_members = default_party();
        self.party_group_active = false;
        self.allied_races = default_allied_races();
        self.model_scenes = default_model_scenes();
        self.major_factions = default_major_factions();
        self.major_faction_renown_levels = default_major_faction_renown_levels();
        crate::lua_api::globals::keybindings::init_keybindings(self);
        self.player.name = SEEDED_LOCAL_CHARACTER_NAME.to_string();
        self.player.power = 50_000;
        self.player.power_max = 100_000;
        self.player.buffs = default_player_buffs();
        self.titles = vec![
            "the Patient".to_string(),
            "the Explorer".to_string(),
            "Champion of the Naaru".to_string(),
            "Hand of A'dal".to_string(),
            "the Argent Champion".to_string(),
        ];
    }

    fn new_empty() -> Self {
        Self::build_empty_state(EmptyStateCollections::new(), EmptyRuntimeState::new())
    }

    fn build_empty_state(c: EmptyStateCollections, r: EmptyRuntimeState) -> Self {
        build_empty_sim_state!(c, r)
    }

    /// Look up bag item at (bag, slot). Returns (item_id, stack_count).
    pub fn get_bag_item(&self, bag: i32, slot: i32) -> Option<(u32, i32)> {
        self.bag_items
            .get(&(bag, slot))
            .map(|i| (i.item_id, i.stack_count))
    }

    /// Count occupied slots in a bag.
    pub fn bag_occupied_slots(&self, bag: i32) -> i32 {
        self.bag_items.keys().filter(|(b, _)| *b == bag).count() as i32
    }

    pub fn set_screen_kind(&mut self, screen_kind: ScreenKind) {
        self.screen_kind = screen_kind;
        self.screen_first_displayed = false;
        if screen_kind.is_glue() {
            self.is_logged_in = false;
        }
    }

    pub fn mark_post_event_workarounds_applied(&mut self) -> bool {
        if self.post_event_workarounds_applied {
            return false;
        }
        self.post_event_workarounds_applied = true;
        true
    }

    pub(crate) fn is_addon_loading(&self, addon_name: &str) -> bool {
        self.loading_addon_stack.iter().any(|loading_idx| {
            self.addons
                .get(*loading_idx as usize)
                .is_some_and(|addon| addon.folder_name == addon_name)
        })
    }

    pub fn set_mouse_position(&mut self, pos: Option<(f32, f32)>) {
        self.mouse_position = pos;
        let Some((mx, my)) = pos else {
            return;
        };
        cursor_tooltips::reanchor_visible_cursor_tooltips(self, mx, my);
    }

    pub fn set_active_drag_frame(&mut self, frame_id: Option<u64>) {
        self.active_drag_frame = frame_id;
    }

    pub fn set_active_slider_thumb_drag_frame(&mut self, frame_id: Option<u64>) {
        self.active_slider_thumb_drag_frame = frame_id;
    }

    pub fn set_mouse_button_down(&mut self, button: &str, down: bool) -> bool {
        self.mouse_buttons.set_down(button, down)
    }

    pub fn enqueue_texture_preloads<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.pending_texture_preloads.extend(paths);
    }

    pub fn drain_texture_preloads(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_texture_preloads)
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests;
