use super::*;

pub struct SimState {
    pub wrath_relic_slot: crate::lua_api::globals::real::wrath_relic_slot::RelicSlotState,
    pub plain_global_inputs: crate::lua_api::globals::real::publication_12_0_0::PlainGlobalInputs,
    pub(crate) private_aura_anchors: crate::c_api::private_aura_anchors::PrivateAuraAnchors,
    /// Explicit pending records only; empty default and lifecycle are simulator policy.
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    pub(crate) pending_spell_confirmation_prompts: HashMap<
        u64,
        crate::lua_api::globals::real::spell_confirmation_prompts::SpellConfirmationPrompt,
    >,
    #[cfg(feature = "retail-12-0-5")]
    pub(crate) nameplate_hit_test_insets: crate::c_api::c_nameplate_manager::NamePlateHitTestInsets,
    pub(crate) input_interface_style: crate::c_api::c_input_interface_style::InputInterfaceStyle,
    /// Explicit mapped-stick input; None means no configured snapshot, not device absence.
    #[cfg(feature = "client-wowforever")]
    pub gamepad_mapped_sticks: Option<crate::c_api::c_game_pad::MappedStickSnapshot>,
    /// Independent free-look-hover policy; false default is a simulator guess.
    #[cfg(feature = "client-wowforever")]
    pub(crate) gamepad_allow_hover_events_with_free_look: bool,
    /// Explicit Wrath RAF link snapshot, keyed by represented unit GUID.
    #[cfg(feature = "client-wrath")]
    pub refer_a_friend_linked_guids: HashSet<String>,
    pub summon_friend_cooldown: crate::c_api::c_recruit_a_friend::SummonFriendCooldown,
    pub raf_versions: Vec<crate::c_api::c_recruit_a_friend::RafVersionInfo>,
    pub widgets: WidgetRegistry,
    #[cfg(feature = "retail-12-1-5")]
    pub(crate) weather: crate::c_api::c_weather::WeatherState,
    #[cfg(feature = "retail-12-1-0")]
    pub(crate) encounter_timeline: crate::c_api::c_encounter_timeline::Timeline,
    pub events: EventQueue,
    pub scripts: ScriptRegistry,
    pub console_output: Vec<String>,
    /// Requested legacy tutorial IDs in call order; no flag, event or display policy.
    pub tutorial_requests: Vec<i32>,
    pub engraving: crate::c_api::engraving_inputs::EngravingInputs,
    pub timers: VecDeque<PendingTimer>,
    pub rilua_timers: VecDeque<crate::lua_api::timer_layout::RiluaPendingTimer>,
    /// Diagnostic strings only; remains empty without handler timing opt-in.
    pub(crate) timer_callback_sources:
        HashMap<u64, crate::lua_api::handler_timing::TimerCallbackSource>,
    #[cfg(feature = "timed-signal-maps")]
    pub(crate) timed_signal_maps: HashMap<u64, crate::c_api::timed_signal_map::TimedSignalMapState>,
    pub focused_frame_id: Option<u64>,
    pub addons: Vec<AddonInfo>,
    pub addon_saved_enable_state: Option<AddonEnableSnapshot>,
    pub addon_saved_version_check_enabled: Option<bool>,
    pub system_chat_log: Vec<String>,
    pub adventure_map: AdventureMapState,
    pub encounter_journal: EncounterJournalState,
    pub pet_journal_hover: crate::c_api::c_pet_journal_hover::PetJournalHoverState,
    pub anima_diversion: AnimaDiversionState,
    pub garrison_talents: GarrisonTalentState,
    #[cfg(feature = "client-retail")]
    pub garrison_trees: crate::c_api::c_garrison_trees::GarrisonTrees,
    pub clipboard: ClipboardState,
    pub chat_edit_open_state: Option<ChatEditOpenState>,
    /// INFERRED explicit host ingress; empty per environment.
    #[cfg(feature = "retail-12-0-7")]
    pub host_chat_inputs: crate::lua_api::host_chat_inputs::HostChatInputs,
    #[cfg(feature = "retail-12-0-7")]
    pub performance_inputs: crate::lua_api::performance_inputs::PerformanceInputs,
    #[cfg(feature = "retail-12-0-7")]
    pub url_texture_inputs: crate::c_api::url_texture_inputs::UrlTextureInputs,
    /// INFERRED unsupported-token set; empty default preserves modeled units.
    #[cfg(feature = "retail-12-0-7")]
    pub unsupported_unit_tokens: HashSet<String>,
    /// Explicit chat restriction input; false default is simulator policy, not a native producer.
    #[cfg(feature = "retail-12-0-5")]
    pub chat_messaging_lockdown: bool,
    #[cfg(feature = "retail-12-0-5")]
    pub chat_expression_inputs: crate::c_api::chat_expressions::ChatExpressionInputs,
    /// C_PartyInfo restriction enum input; None (0) is an inferred simulator default.
    #[cfg(feature = "retail-12-0-5")]
    pub party_ping_restriction: u8,
    /// Local countdown request; visual expiry does not mutate request state.
    #[cfg(feature = "retail-12-0-5")]
    pub(crate) party_countdown_request:
        Option<crate::c_api::c_party_info::countdown::CountdownRequest>,
    pub quest_portrait_state: Option<QuestPortraitState>,
    pub cvars: CVarStorage,
    /// User-approved signed legacy gamma; simulator state only, never host gamma.
    #[cfg(feature = "client-wrath")]
    pub legacy_gamma: f64,
    /// Zero means none; other values are one-based guild roster indexes.
    pub guild_roster_selection: usize,
    /// Explicit Wrath scenario inputs; empty means unsupplied.
    pub wrath_glyph_sockets: HashMap<(u32, u32), WrathGlyphSocket>,
    pub wrath_unspent_talent_points: HashMap<u32, u32>,
    pub wrath_talent_tabs: HashMap<(u32, u32), WrathTalentTab>,
    pub wrath_talent_rows: HashMap<(u32, u32), Vec<WrathTalentRow>>,
    #[cfg(any(
        feature = "client-wrath",
        feature = "client-era",
        feature = "client-anniversary"
    ))]
    pub wrath_legacy: crate::lua_api::globals::real::wrath_legacy_state::WrathLegacyState,
    #[cfg(feature = "client-wrath")]
    pub who_routing: crate::c_api::who_routing::WhoRouting,
    pub tooltips: HashMap<u64, TooltipData>,
    /// Explicit exact-link fixtures only; initially empty, no catalog producer.
    #[cfg(feature = "retail-12-0-0")]
    pub item_stats: crate::c_api::ItemStatsState,
    /// Explicit (specialization, item) lists only; initially empty.
    #[cfg(feature = "retail-12-0-0")]
    pub item_set_bonuses: crate::c_api::ItemSetBonuses,
    /// Explicit absolute milliseconds; (0, 0) default is simulator policy.
    #[cfg(feature = "retail-12-0-0")]
    pub ping_cooldown_interval: crate::c_api::PingCooldownInterval,
    /// Whole host-supplied mapping, empty by default; not a native kit catalog.
    #[cfg(feature = "retail-12-0-0")]
    pub ping_texture_kits: crate::c_api::PingTextureKits,
    /// Explicit AI identities only; initially empty, no membership producer.
    #[cfg(feature = "retail-12-0-0")]
    pub party_ai_guids: HashSet<String>,
    /// Explicit item-level overrides only; empty default, no native variant catalog.
    #[cfg(feature = "retail-12-0-5")]
    pub item_tooltip_levels: HashMap<crate::c_api::ItemTooltipContext, u16>,
    pub blocked_auras_by_unit: HashMap<String, HashSet<i32>>,
    /// Explicit test scenario only; no combat or spell-secrecy inference.
    #[cfg(feature = "client-wowforever")]
    pub auras_secret_in_context: bool,
    /// Explicit stat-output policy input; no combat or aura activation inference.
    pub unit_stats_restricted: bool,
    /// Explicit weapon contributions keyed by existing unit GUID; rows do not create units.
    #[cfg(feature = "client-retail")]
    pub weapon_attack_power: HashMap<String, crate::lua_api::state_types::WeaponAttackPower>,
    pub quest_blobs: HashMap<u64, QuestBlobState>,
    /// Supplied per-quest minimap containment observations.
    pub minimap_quest_blob_inside: HashMap<i32, bool>,
    pub fog_of_war_frames: HashMap<u64, FogOfWarFrameState>,
    pub unit_position_frames: HashMap<u64, UnitPositionFrameState>,
    pub pending_player_reports: HashMap<i64, PendingPlayerReport>,
    pub message_frames: HashMap<u64, MessageFrameData>,
    pub on_update_frames: HashSet<u64>,
    /// Explicit ScrollFrame range requests, drained after OnUpdate and animations.
    pub(crate) pending_scroll_range_updates: HashSet<u64>,
    pub visible_on_update_cache: Option<Vec<u64>>,
    pub strata_buckets: Option<Vec<Vec<u64>>>,
    pub(crate) active_toplevel_show_orders: HashMap<u64, u64>,
    pub(crate) next_toplevel_show_order: u64,
    pub pending_hit_grid_changes: Vec<(u64, bool)>,
    pub pending_texture_preloads: BTreeSet<String>,
    pub animation_groups: HashMap<u64, AnimGroupState>,
    pub active_animation_groups: HashSet<u64>,
    pub next_anim_group_id: u64,
    pub anim_frame_to_group: HashMap<u64, u64>,
    pub anim_frame_to_anim: HashMap<u64, (u64, usize)>,
    /// Base layout canvas dimensions, before UIParent scale.
    pub screen_width: f32,
    pub screen_height: f32,
    pub physical_screen_width: f32,
    pub physical_screen_height: f32,
    pub screen_kind: ScreenKind,
    pub is_logged_in: bool,
    pub post_event_workarounds_applied: bool,
    pub screen_first_displayed: bool,
    pub saved_account_name: String,
    pub saved_account_list: String,
    pub uses_token: bool,
    pub account_save_enabled: bool,
    pub account_save_in_progress: bool,
    pub account_locked_post_save: bool,
    pub last_account_store_purchase_request: Option<i64>,
    pub account_store_begin_purchase_succeeds: bool,
    pub last_account_store_refund_request: Option<i64>,
    pub account_store_refund_succeeds: bool,
    pub account_store_category_items: HashMap<i64, Vec<i64>>,
    pub account_store_currency_for_store: HashMap<i64, i64>,
    pub account_store_currency_info: HashMap<i64, AccountStoreCurrencyInfo>,
    pub account_store_storefront_state: HashMap<i64, i64>,
    pub last_account_store_storefront_info_request: Option<i64>,
    pub account_store_categories: HashMap<i64, AccountStoreCategoryInfo>,
    pub account_store_items: HashMap<i64, AccountStoreItemInfo>,
    #[cfg(feature = "retail-12-0-0")]
    pub spell_classifications: crate::c_api::c_spell_classification::SpellClassifications,
    #[cfg(feature = "retail-12-0-0")]
    pub nameplate_configuration: crate::c_api::c_nameplate_manager::NamePlateConfiguration,
    #[cfg(feature = "retail-12-0-0")]
    pub recipe_quality_inputs: crate::c_api::c_trade_skill_quality::RecipeQualityInputs,
    /// Explicit per-item supplied quality; independent of recipe inputs, initially empty.
    #[cfg(feature = "retail-12-0-0")]
    pub crafted_item_quality:
        HashMap<u32, crate::c_api::c_trade_skill_quality::CraftingQualityInfo>,
    #[cfg(feature = "retail-12-0-0")]
    pub pvp_catalog: crate::c_api::c_pvp::catalog::PvpCatalog,
    #[cfg(feature = "retail-12-0-0")]
    pub prey_inputs: crate::c_api::c_prey::PreyInputs,
    /// Explicit supplied ButtonHeader records; no generated catalog/default DTOs.
    pub button_header_inputs: crate::c_api::ui_widget_manager::ButtonHeaderInputs,
    pub ui_widget_inputs: crate::c_api::ui_widget_manager::UiWidgetInputs,
    /// INFERRED host policy snapshots, initially inactive; not combat-derived.
    #[cfg(feature = "retail-12-0-0")]
    pub combat_log_restricted: bool,
    #[cfg(feature = "retail-12-0-0")]
    pub pet_bonus_slot_available: bool,
    #[cfg(feature = "retail-12-0-0")]
    pub limited_input_allowed: [bool; 4],
    /// INFERRED empty host range inputs and tier-completion snapshots.
    #[cfg(any(feature = "retail-12-0-0", feature = "client-wrath"))]
    pub action_spell_ranges: HashMap<u32, crate::c_api::action_spell_range::ActionSpellRange>,
    #[cfg(feature = "retail-12-0-0")]
    pub weekly_reward_progress:
        HashMap<i32, Vec<crate::c_api::c_weekly_rewards::ActivityTierProgress>>,
    pub action_bars: HashMap<u32, u32>,
    /// Explicit slot-owned input; no cast/attack/spell-lifetime inference.
    pub(crate) action_repeat_active_slots: HashSet<u32>,
    /// INFERRED host-declared resolved spell IDs on special bars; empty by default.
    /// Independent of direct assignments and bar visibility.
    #[cfg(feature = "retail-12-0-5")]
    pub special_bar_spells: HashSet<u32>,
    /// Explicit slot-keyed use counts; stored spell identity must match the current binding.
    #[cfg(feature = "retail-12-0-5")]
    pub action_use_counts: HashMap<u32, crate::c_api::ActionUseCountInfo>,
    /// Explicit spell-ID usability inputs; missing entries classify as usable.
    pub spell_action_usability: HashMap<u32, crate::c_api::action_usability::ActionUsability>,
    pub action_outfits: HashMap<u32, i64>,
    pub action_macros: HashMap<u32, u32>,
    pub equipped_gear_outfit_action_slots: HashSet<u32>,
    pub assisted_combat: AssistedCombatState,
    pub action_bar_page: u32,
    pub has_override_action_bar: bool,
    pub has_vehicle_action_bar: bool,
    pub vehicle_bar_index: i32,
    pub override_bar_skin: Option<i32>,
    pub override_bar_index: i32,
    pub has_temp_shapeshift_action_bar: bool,
    pub temp_shapeshift_bar_index: i32,
    pub has_bonus_action_bar: bool,
    pub bonus_bar_index: i32,
    pub action_bar_state: ActionBarStateInfo,
    pub action_highlights: ActionHighlightState,
    pub extra_action_button: ExtraActionButtonState,
    pub equipped_artifact: Option<ArtifactInfo>,
    pub artifact_point_costs: HashMap<(i32, i32), i64>,
    pub allied_races: HashMap<i64, AlliedRaceInfo>,
    pub model_scenes: HashMap<i64, Vec<String>>,
    pub active_player_interactions: HashSet<i32>,
    pub azerite_item: Option<AzeriteItemState>,
    pub azerite_essence: AzeriteEssenceState,
    pub azerite_empowered: AzeriteEmpoweredItemState,
    pub barber_shop: BarberShopState,
    /// Explicit DamageMeter snapshots; combat publication is blocked pending secrecy.
    pub damage_meter: crate::c_api::c_damage_meter::DamageMeterInput,
    pub item_interaction: Option<crate::c_api::c_item_interaction::ItemInteractionInfo>,
    pub major_factions: HashMap<i64, MajorFactionData>,
    pub major_faction_renown_levels: HashMap<i64, Vec<RenownLevelInfo>>,
    /// Explicit pair-keyed reward inputs only; no fabricated default rows.
    pub major_faction_renown_rewards:
        HashMap<(i64, i32), Vec<crate::c_api::c_major_factions::RenownRewardInfo>>,
    /// Explicit Recent Allies input; disabled and empty by inferred default policy.
    pub recent_allies: crate::c_api::c_recent_allies::RecentAlliesInput,
    pub account_wide_reputation_factions: HashSet<i64>,
    pub faction_paragon: HashMap<i64, FactionParagonInfo>,
    /// Explicit active-brawl input only; no native default record is assumed.
    #[cfg(feature = "retail-12-0-5")]
    pub active_brawl: Option<PvpBrawlInfo>,
    /// Explicit ordered illusion inputs only; no fabricated native catalog.
    #[cfg(feature = "retail-12-0-5")]
    pub transmog_illusions: Vec<crate::c_api::c_transmog_collection::IllusionInfo>,
    /// Explicit itemModifiedAppearanceID-keyed inputs only.
    #[cfg(feature = "retail-12-0-5")]
    pub transmog_appearance_sources: HashMap<i64, AppearanceSourceInfo>,
    /// Explicit (slot, type, option) inputs only; no viewed-outfit synthesis.
    #[cfg(feature = "retail-12-0-5")]
    pub viewed_outfit_slots: HashMap<(i32, i32, i32), ViewedOutfitSlotInfo>,
    /// Explicit host snapshot only; absence does not fabricate a price.
    #[cfg(feature = "retail-12-0-5")]
    pub pending_transmog_cost: Option<crate::c_api::c_transmog_outfit_info::PendingTransmogCost>,
    pub transmog_outfit_locks: HashSet<i64>,
    #[cfg(feature = "retail-12-0-5")]
    pub transmog_outfit_catalog: crate::c_api::c_transmog_outfit_info::OutfitCatalog,
    #[cfg(feature = "retail-12-0-5")]
    pub transmog_outfits: crate::c_api::c_transmog_outfit_info::OutfitState,
    /// Explicit applied selection; independent of viewed/pending outfit metadata.
    #[cfg(feature = "retail-12-0-5")]
    pub active_transmog_outfit_id: Option<i64>,
    /// Explicit catalog-backed viewed selection; independent of active/pending state.
    #[cfg(feature = "retail-12-0-5")]
    pub viewed_transmog_outfit_id: Option<i64>,
    /// Global setting only; no per-outfit or pending-situation behavior.
    pub outfit_situations_enabled: bool,
    /// Explicit filter values only; native defaults and set filtering are unmodeled.
    pub transmog_set_filters: HashMap<i32, bool>,
    pub transmog_sets: crate::c_api::c_transmog_sets::TransmogSets,
    #[cfg(feature = "retail-12-0-0")]
    pub transmog_custom_sets: crate::c_api::c_transmog_collection::CustomSets,
    pub equipped_outfit_locked: bool,
    pub locked_action_slots: HashSet<i32>,
    pub is_active_battlefield: bool,
    pub spell_trade_skill_links: HashMap<u32, String>,
    pub spell_id_aliases: HashMap<String, u32>,
    /// Host-declared Maw power atlas/link strings by resolved spell ID; empty by default.
    #[cfg(feature = "retail-12-0-5")]
    pub maw_powers: crate::c_api::c_spell_maw_powers::MawPowers,
    /// Explicit host active-delve state, independent of undeclared query arguments.
    pub has_active_delve: bool,
    /// Host lair state: the party has an active lair (a delve variant).
    pub has_active_lair: bool,
    /// Whether the active lair group was formed through LFG matchmaking.
    pub active_lair_is_lfg: bool,
    /// Outgoing in-game ("title") friend requests by character name.
    pub title_friend_requests: Vec<String>,
    /// Guild club member names sent a Battle.net friend request via `C_Club`.
    #[cfg(feature = "retail-12-0-7")]
    pub club_battle_tag_friend_requests: Vec<String>,
    pub clubs: crate::c_api::club_model::ClubState,
    /// Host spell data: cooldown category of each spell that has one.
    pub spell_cooldown_categories: HashMap<u32, i32>,
    /// Supplied low-rank spell IDs for the current scenario.
    pub spellbook_low_rank: HashSet<u32>,
    /// Supplied spell IDs represented in a flyout.
    pub spellbook_loose_flyout_members: HashSet<u32>,
    /// Item whose use started a spell's current cooldown, keyed by spell ID.
    pub spell_cooldown_item_sources: HashMap<u32, i32>,
    /// `C_Roleset.ApplyRolesetFilters` blocklist, in applied order.
    pub active_blocked_rolesets: Vec<String>,
    /// `C_Roleset.ApplyRolesetFilters` allowlist, in applied order.
    pub active_allowed_rolesets: Vec<String>,
    /// Quests currently tied to each quest hub, keyed by hub area POI ID.
    pub quest_hub_related_quests: HashMap<i32, HashSet<i32>>,
    /// Host reward multipliers for neighborhood initiative tasks; absent is unscaled.
    pub neighborhood_task_reward_scales: HashMap<i32, f64>,
    /// Host-owned entrance title; None is no title, not a synthetic location.
    #[cfg(feature = "retail-12-0-7")]
    pub delve_entrance_title: Option<String>,
    /// INFERRED most recent numeric game-account request, not service success.
    #[cfg(feature = "retail-12-0-7")]
    pub last_bnet_invite_game_account_id: Option<i32>,
    /// Explicit host entrance PDEID; INFERRED zero default, not a native sentinel.
    pub tiered_entrance_pde_id: u32,
    /// Host `Enum.TieredEntranceType` of the entrance in use; INFERRED Delve
    /// default, matching the seeded delve entrance surface.
    #[cfg(feature = "retail-12-0-5")]
    pub tiered_entrance_type: i32,
    /// INFERRED source-token ownership and live aura-query timing; empty by default.
    #[cfg(feature = "retail-12-0-7")]
    pub player_controlled_vehicle_sources: HashSet<String>,
    /// INFERRED explicit client catalog; GetFileID resolution does not imply membership.
    #[cfg(feature = "retail-12-0-7")]
    pub known_shipped_asset_ids: HashSet<u32>,
    /// INFERRED normalized lowercase slash paths; host owns selected-root reconciliation.
    #[cfg(feature = "retail-12-0-7")]
    pub known_loose_asset_paths: HashSet<String>,
    /// INFERRED observable request input only; no eligibility response or event model.
    pub last_delve_eligibility_map_id: Option<i32>,
    /// Explicit host links keyed by resolved spell ID and rarity; no fabricated links.
    pub curio_links: HashMap<(u32, u32), String>,
    /// INFERRED host-declared click-bindable resolved spell IDs; empty by default.
    /// Selected interaction/spell bindings do not establish eligibility.
    #[cfg(feature = "retail-12-0-5")]
    pub click_bindable_spells: HashSet<u32>,
    #[cfg(feature = "retail-12-0-5")]
    pub cooldown_aura_associations:
        crate::c_api::c_unit_aura_cooldown_spells::CooldownAuraAssociations,
    /// INFERRED explicit spell flags only; no classification catalog or acquisition.
    #[cfg(feature = "aura-spell-classifications")]
    pub aura_spell_classifications:
        crate::c_api::c_unit_aura_classification::AuraSpellClassifications,
    /// INFERRED host-declared sound IDs only; no Add acquisition or playback model.
    #[cfg(feature = "retail-12-0-5")]
    pub private_aura_sound_registrations:
        crate::c_api::private_aura_sounds::PrivateAuraSoundRegistrations,
    /// Explicit recast metadata only; empty means unknown, never current aura duration.
    #[cfg(feature = "retail-12-0-5")]
    pub spell_aura_durations: HashMap<u32, crate::c_api::aura_duration::SpellAuraDuration>,
    #[cfg(feature = "base-spell-relationships")]
    pub base_spell_relationships: crate::c_api::spell_base::BaseSpellRelationships,
    pub spell_activation_overlays: HashSet<u32>,
    pub challenge_completion: crate::c_api::c_challenge_mode::ChallengeCompletionInfo,
    pub(crate) challenge_run: crate::c_api::challenge_run::ChallengeState,
    pub(crate) manual_clock: Option<crate::lua_api::manual_clock::ManualClock>,
    pub spell_loss_of_control: HashMap<u32, LossOfControlInfo>,
    pub active_loss_of_control:
        HashMap<String, Vec<crate::c_api::c_loss_of_control::ActiveLossOfControlData>>,
    pub spell_flyouts: HashMap<u32, SpellFlyoutInfo>,
    pub action_profession_quality: HashMap<i32, ProfessionQualityInfo>,
    pub addon_base_paths: Vec<PathBuf>,
    pub texture_manager: std::rc::Rc<std::cell::RefCell<crate::texture::TextureManager>>,
    pub create_frame_initial_hidden: Option<bool>,
    pub suppress_runtime_on_load_depth: u32,
    pub xml_load_addon_depth: u32,
    pub mouse_position: Option<(f32, f32)>,
    pub hovered_frame: Option<u64>,
    pub active_drag_frame: Option<u64>,
    pub active_slider_thumb_drag_frame: Option<u64>,
    pub next_report_token: i64,
    pub party_members: Vec<PartyMember>,
    pub party_group_active: bool,
    /// INFERRED: host-owned identity classification; no automatic context inference.
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    pub identity_secret_guids: HashSet<String>,
    /// Explicit host instance/group/control context; no token-name classification.
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    pub instance_identity:
        crate::lua_api::globals::real::instanced_identity::InstanceIdentityContext,
    #[cfg(feature = "client-wrath")]
    pub wrath_unit_tap_states: HashMap<String, crate::lua_api::state::WrathUnitTapState>,
    pub current_target: Option<TargetInfo>,
    pub previous_target: Option<TargetInfo>,
    pub current_focus: Option<TargetInfo>,
    /// Explicit host NPC-follower identities for player-style display.
    /// INFERRED: no followers until host input; does not change human identity.
    #[cfg(feature = "retail-12-0-5")]
    pub npc_follower_guids: std::collections::HashSet<String>,
    /// Soft interaction selection aliases an existing target/focus/enemy token.
    pub soft_interact_target: Option<String>,
    /// Preferred gamepad interaction identity; independent of target aliases.
    pub preferred_gamepad_interact_guid: Option<String>,
    pub enemy_pool: Vec<TargetInfo>,
    /// Unit raid target icons keyed by GUID; independent of world markers.
    pub(crate) unit_raid_target_icons: HashMap<String, u8>,
    /// Modeled setting only; not linked to CVar state or audio playback.
    pub combat_audio_speaker_speed: f64,
    #[cfg(feature = "retail-12-0-5")]
    pub combat_audio_categories: crate::c_api::c_combat_audio_alert::CategorySettings,
    /// Explicit selection only; no combat-text event routing or native identity.
    pub combat_text_active_unit: Option<String>,
    /// Independent setting only; not linked to CVar state or audio playback.
    pub combat_audio_speaker_volume: f64,
    /// Numeric unit/type setting state only; no formatting or CVar coupling.
    pub combat_audio_format_settings: HashMap<(i32, i32), f64>,
    /// INFERRED per-active-spec-index settings; unconfigured zero, no CVar/playback coupling.
    #[cfg(feature = "retail-12-0-0")]
    pub combat_audio_spec_settings: HashMap<(i32, i32), f64>,
    /// INFERRED shared throttle seconds by throttle type; unconfigured zero.
    #[cfg(not(feature = "client-wrath"))]
    pub combat_audio_throttles: HashMap<i32, f64>,
    /// Simulator tracking membership; empty initially, independent of task records.
    pub neighborhood_tracked_tasks: BTreeSet<i32>,
    pub totem_slots: Vec<Option<crate::lua_api::globals::real::totems::Totem>>,
    pub weapon_enchants: [Vec<crate::c_api::weapon_enchants::WeaponEnchant>; 3],
    pub sound_manager: Option<SoundManager>,
    pub last_sound_kit_requested: Option<u32>,
    /// Last formatted Wrath UI error requested by the modeled quest feedback path.
    #[cfg(feature = "client-wrath")]
    pub last_ui_error_message: Option<String>,
    /// Last legacy string bytes, including unknown/ambiguous names; no playback implied.
    #[cfg(feature = "client-wrath")]
    pub last_sound_name_requested: Option<Vec<u8>>,
    /// `C_CooldownViewer` entries keyed by cooldownID; empty until populated.
    #[cfg(feature = "retail-12-0-0")]
    pub encounter_policy: crate::c_api::c_instance_encounter::EncounterPolicy,
    #[cfg(feature = "retail-12-0-0")]
    pub encounter_warning_settings: crate::c_api::c_encounter_warnings::WarningSettings,
    pub cooldown_viewer_cooldowns:
        std::collections::BTreeMap<i32, crate::c_api::c_cooldown_viewer::CooldownViewerCooldown>,
    /// Explicit supplied availability; independent of cooldown catalog and UI settings.
    pub cooldown_viewer_availability: crate::c_api::c_cooldown_viewer::CooldownViewerAvailability,
    #[cfg(feature = "retail-12-1-0")]
    pub last_sound_request: Option<crate::c_api::c_sound::PlaySoundRequest>,
    pub last_sound_file_requested: Option<String>,
    pub last_stopped_sound_handle: Option<u32>,
    pub last_launched_url: Option<String>,
    pub highlighted_map_scene_character_guid: Option<String>,
    pub secure_attribute_drivers: HashMap<u64, HashMap<String, String>>,
    pub rot_damage_level: usize,
    pub fps: f32,
    pub start_time: Instant,
    pub casting: Option<CastingState>,
    pub channeling: Option<CastingState>,
    pub next_cast_id: u32,
    pub gcd: Option<(f64, f64)>,
    pub spell_cooldowns: HashMap<u32, SpellCooldownState>,
    pub spell_charges: HashMap<u32, crate::c_api::charge_state::SpellChargeState>,
    /// Explicit spell-keyed C_Spell count input; no inventory or casting derivation.
    #[cfg(feature = "retail-12-0-0")]
    pub spell_cast_counts: HashMap<u32, u32>,
    /// Explicit cooldown-output policy input, independent of combat and unit stats.
    pub cooldowns_restricted: bool,
    /// Explicit spell-keyed maximum aura stack input; empty means undeclared, never applied stacks.
    #[cfg(feature = "retail-12-0-5")]
    pub spell_max_cumulative_aura_applications: HashMap<u32, u32>,
    /// Explicit unit-aura output policy input, independent of cooldowns and unit stats.
    pub unit_auras_restricted: bool,
    /// INFERRED host input: active PvP match, not queue/instance presence.
    pub pvp_match_active: bool,
    /// Explicit aura-filter classification and raid dispel capabilities.
    pub aura_filter_facts: crate::lua_api::game_data::AuraFilterFacts,
    /// Live target auras; seeded with the legacy target fixture.
    pub target_auras: Vec<crate::lua_api::game_data::AuraInfo>,
    pub inventory_item_cooldowns: HashMap<i32, SpellCooldownState>,
    /// Passive item-ID input owned by the C_Item/C_Container cooldown contract.
    /// Independent of legacy equipment-slot cooldowns; no query producer yet.
    pub item_cooldowns: HashMap<u32, SpellCooldownState>,
    pub action_ui_buttons: Vec<(u64, u32)>,
    pub cursor_item: Option<CursorInfo>,
    pub container_actions: crate::c_api::container_actions::ContainerActions,
    pub item_upgrade_eligibility: crate::c_api::item_upgrade_eligibility::ItemUpgradeEligibility,
    pub loading_addon_index: Option<u16>,
    pub loading_addon_stack: Vec<u16>,
    pub executing_addon_index: Option<u16>,
    pub loading_nil_symbol_environment: Option<NilSymbolEnvironment>,
    pub loading_forbidden: bool,
    pub loading_scoped_script_env: Option<rilua::Val>,
    pub loading_add_to_secure_env: bool,
    pub loading_hide_from_global_env: bool,
    pub loading_use_forbidden_object_table: bool,
    /// ScopedModifier `allowUntaintedCreation`: intrinsics declared in a forbidden-object-table
    /// scope without it reject CreateFrame from tainted callers.
    pub loading_allow_untainted_creation: bool,
    pub app_frame_metrics: AppFrameMetrics,
    pub addon_performance_messages_shown: HashSet<AddonPerformanceMessageKey>,
    pub talents: crate::lua_api::talent_state::TalentState,
    pub lua_errors: Vec<String>,
    pub lua_error_records: Vec<LuaErrorRecord>,
    pub lua_error_counts: HashMap<String, usize>,
    pub nil_symbol_accesses: Vec<NilSymbolAccess>,
    pub global_publications: HashSet<(u16, String)>,
    pub secure_global_publications: HashSet<(u16, String)>,
    pub pending_nested_addon_diagnostics: HashMap<u16, LoadDiagnostics>,
    pub runtime_addon_diagnostics: LoadDiagnostics,
    pub global_show_hide_depth: u32,
    pub anim_sync_times: HashMap<String, std::time::Duration>,

    /// Explicit host-selected nearest party token; no roster/distance inference.
    #[cfg(feature = "retail-12-0-5")]
    pub nearest_party_member_token: Option<String>,
    /// Host-provided identifiers for the next encounter/M+/PvP entry.
    #[cfg(feature = "retail-12-0-5")]
    pub aura_entry_ids: crate::c_api::aura_entry_ids::AuraEntryIds,
    /// Explicit exact-health knowledge by unit token; absent entries are unknown.
    pub unit_health_knowledge: HashMap<String, bool>,
    pub player: PlayerState,
    pub war_mode: crate::c_api::c_pvp::war_mode::WarModeState,
    pub pvp_talents: crate::c_api::c_spec::PvpTalentState,
    pub player_xp: PlayerXpState,
    pub bind_location: String,
    pub pvp_honor: PvpHonorState,
    pub world: WorldState,
    pub bag_items: HashMap<(i32, i32), BagItem>,
    /// Host-supplied metadata keyed by item ID, not bag coordinates.
    pub container_item_quests:
        HashMap<u32, crate::c_api::container_quest::ContainerItemQuestMetadata>,
    pub bag_info: HashMap<i32, crate::c_api::bag_info::BagInfo>,
    pub tracked_recipes: TrackedRecipes,
    pub crafting: CraftingState,
    pub net_stats: NetStats,
    pub store_frame_shown: bool,
    /// Host observations; absence is not evidence of an ineligible group.
    pub store_public_eligibility:
        crate::c_api::store_public_eligibility::GroupEligibilityObservations,
    pub timerunning_season_id: Option<u32>,
    pub timerunning_season_seconds_remaining: u64,
    pub modifier_keys: ModifierKeys,
    pub pressed_keys: HashSet<String>,
    pub mouse_buttons: MouseButtons,
    /// Accepted `SimulateMouse*` input awaiting the GUI mouse dispatcher.
    #[cfg(feature = "retail-12-0-7")]
    pub simulated_mouse_inputs:
        VecDeque<crate::lua_api::globals::real::simulate_mouse::SimulatedMouseInput>,
    pub game_rules: GameRulesState,
    pub discord: DiscordState,
    pub player_choice: PlayerChoiceState,
    pub housing_service_enabled: bool,
    pub housing: HousingState,
    /// Explicit catalog shop product/display records; empty until host-seeded.
    pub catalog_shop_products: crate::c_api::c_catalog_shop_products::CatalogShopProducts,
    /// One bundle record store; preserves the existing simulator storefront seed.
    pub housing_bundles: crate::c_api::c_housing_bundles::HousingBundles,
    pub pet_battles: PetBattleState,
    pub pet: PetState,
    pub lfg_list_counts: LfgListCounts,
    pub lfg_active_entry: Option<crate::lua_api::state_types::LfgActiveEntry>,
    pub can_use_premade_group: bool,
    /// Supplied Mists LFR service availability; None leaves native policy unknown.
    pub lfr_availability: Option<bool>,
    /// Supplied transmog availability; None leaves native policy unknown.
    pub transmog_availability: crate::c_api::transmog_availability::TransmogAvailability,
    /// Supplied spectating scenario; false is a simulator default, not a native observation.
    pub spectating: bool,
    /// Forever: None is unset, Some(None) is an explicit no-result observation.
    pub forever_outfits: Option<Option<Vec<crate::c_api::outfit_entry::OutfitEntry>>>,
    /// Supplied Mists scenario eligibility and failure reason; None leaves policy unknown.
    pub scenario_finder_availability: Option<(bool, String)>,
    /// Supplied Mists active-party minimum; None leaves native policy unknown.
    pub mists_party_min_level: Option<f64>,
    pub lfg_category_info: std::collections::HashMap<i32, LfgCategoryInfo>,
    pub lfg_active_categories: std::collections::HashSet<i32>,
    pub lfg_queued_dungeons: std::collections::HashMap<i32, std::collections::BTreeSet<i32>>,
    pub lfg_queue_pop_delay_seconds: f64,
    pub lfg_queue_pop_due_at: Option<Instant>,
    pub lfg_active_proposal: Option<LfgProposalState>,
    pub lfg_random_cooldown_units: std::collections::HashSet<String>,
    pub lfg_activity_groups: Vec<LfgActivityGroupInfo>,
    pub lfg_activities: Vec<LfgActivityInfo>,
    pub lfg_applications: Vec<LfgApplication>,
    pub lfg_next_application_id: u64,
    pub lfg_advanced_filter: LfgAdvancedFilter,
    pub lfg_language_filter: std::collections::HashMap<String, bool>,
    pub lfg_roles: LfgRoleSelection,
    pub lfd_dungeons: Vec<LfdDungeonInfo>,
    pub lfd_enabled_dungeons: std::collections::HashMap<i32, bool>,
    pub photo_sharing_authorized: bool,
    pub photo_sharing_enabled: bool,
    pub tutorial_flags: HashSet<u32>,
    pub wowlabs: WowLabsState,
    pub archaeology: ArchaeologyState,
    pub arrow_callouts: ArrowCalloutState,
    pub gardenweald: GardenwealdState,
    pub viewed_artifact: ViewedArtifactState,
    pub relic_forge_at_forge: bool,
    pub artifact_relic_items: HashSet<i32>,
    pub quest_log: Vec<u32>,
    pub quest_log_entries: QuestLogState,
    pub super_track: crate::c_api::c_super_track::SuperTrackState,
    /// Explicit ordinary-watch metadata; seeded once, not derived on every query.
    #[cfg(feature = "client-wowforever")]
    pub quest_watch_types: HashMap<i32, crate::c_api::QuestWatchType>,
    /// Explicit eligible IDs, independent of world-quest classification.
    pub quest_war_mode_eligibility: std::collections::HashSet<i32>,
    /// Selected rich offer snapshot; survives gossip closure until accepted/cancelled.
    pub pending_quest_offer: Option<QuestLogEntry>,
    pub quest_choice_id: Option<u32>,
    pub quest_choice_response_id: Option<u32>,
    pub quest_poi_map_id: Option<i32>,
    pub selected_quest_log_id: Option<u32>,
    /// Explicit host quest favor records and nil-query context; empty by default.
    #[cfg(feature = "retail-12-0-5")]
    pub quest_favor: crate::c_api::c_quest_info_system::QuestFavorState,
    /// Host-flagged quests whose expiration warning uses the critical threshold.
    #[cfg(feature = "retail-12-0-7")]
    pub quest_short_expiration_warnings: HashSet<u32>,
    pub abandon_quest_id: Option<u32>,
    /// Unique tracked IDs in insertion order, shared by legacy tracking globals.
    pub tracked_achievements: Vec<i32>,
    /// Supplied legacy trainer visibility: available, unavailable, used.
    #[cfg(feature = "client-wrath")]
    pub wrath_trainer_filters: [bool; 3],
    /// Host inputs for Forever 70532; other profiles do not publish these probes.
    pub forever_startup: crate::c_api::forever_startup::StartupInputs,
    pub bank_frame_open: bool,
    pub guild_bank_frame_open: bool,
    pub bank_purchased_slots: i32,
    pub guild_bank_money: i64,
    pub current_guild_bank_tab: i32,
    pub guild_bank_items: HashMap<(i32, i32), BagItem>,
    pub guild_bank_tabs: Vec<GuildBankTab>,
    pub merchant_frame_open: bool,
    #[cfg(feature = "client-wowforever")]
    pub merchant_repair_capable: bool,
    pub tabard_frame_open: bool,
    pub trainer_frame_open: bool,
    pub socket_frame_open: bool,
    pub loot_frame_open: bool,
    pub guild_registrar_open: bool,
    pub pet_stables_open: bool,
    /// Supplied modern observations; None does not invent native initial membership.
    pub stable_active_pets: Option<Vec<crate::c_api::c_stable_info::active_list::PetInfo>>,
    /// Missing key is unknown; None is an explicit empty slot observation.
    pub stable_pet_slots: HashMap<u32, Option<crate::c_api::c_stable_info::active_list::PetInfo>>,
    /// Supplied specialization catalog; None is not an observed empty list.
    pub stable_pet_spec_infos:
        Option<Vec<crate::c_api::c_stable_info::pet_spec_infos::PetSpecInfo>>,
    #[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
    pub allow_recent_allies_see_location: bool,
    #[cfg(feature = "client-wowforever")]
    pub stable_reads: crate::c_api::c_stable_info::forever::StableReadState,
    pub merchant_items: Vec<u32>,
    #[cfg(feature = "retail-12-0-7")]
    pub merchant_currencies: Vec<i32>,
    #[cfg(feature = "client-wowforever")]
    pub merchant_buyback_items: Vec<crate::lua_api::globals::real::merchant_buyback::BuybackItem>,
    pub loot_slots: Vec<BagItem>,
    pub last_loot_roll_choice: Option<i32>,
    pub auction_browse_items: Vec<u32>,
    pub loot_method: LootMethodState,
    pub loot_history: LootHistoryState,
    pub gossip: GossipState,
    pub torghast: TorghastState,
    pub titles: Vec<String>,
    pub current_title: i32,
    pub shapeshift_forms: Vec<ShapeshiftForm>,
    pub shapeshift_cooldowns: HashMap<i32, SpellCooldownState>,
    pub pet_actions: Vec<PetActionSlot>,
    pub glyph: GlyphState,
    pub currency_info: HashMap<i32, CurrencyInfo>,
    /// Explicit Wrath-only legacy categories; modern currency queries do not read this.
    pub wrath_currency: super::super::globals::real::wrath_currency::CurrencyListState,
    #[cfg(feature = "client-retail")]
    pub(crate) currency_list: crate::c_api::currency_list::CurrencyListState,
    /// Explicit (War Mode applies, limit once per tooltip) flags; no inferred catalog.
    pub currency_war_mode_bonus: HashMap<i32, (Option<bool>, Option<bool>)>,
    /// Explicit immediate reputation grants; no inferred currency/faction catalog.
    pub currency_faction_grants: HashMap<i32, i32>,
    /// Explicit transfer activity; query input only, not a transaction lifecycle.
    #[cfg(feature = "retail-12-0-0")]
    pub currency_transfer_in_progress: bool,
    /// Explicit item-to-mount identities; independent of journal records or ownership.
    pub mount_item_relations: HashMap<i32, i32>,
    pub equipment_manager: EquipmentManagerState,
    pub maps: HashMap<i32, MapData>,
    /// Explicit per-map hideIcons input; absence is not a native unknown-ID claim.
    pub map_display_hide_icons: HashMap<i32, bool>,
    /// Explicit continent/world rectangles; never guessed from map art pixels.
    pub map_world_rects: HashMap<i32, crate::c_api::map_world_coordinates::MapWorldRect>,
    /// Explicit bounty-set membership in supplied order; no inferred/default catalog.
    pub bounty_set_maps: HashMap<i32, Vec<i32>>,
    pub achievements: HashMap<i32, AchievementInfo>,
    /// Explicit supplied relations; no native catalog or inferred chain.
    pub achievement_superceding: HashMap<i32, Vec<i32>>,
    /// Explicit event identity positions; default records are simulator demo data.
    pub calendar_event_indices: HashMap<i32, crate::c_api::c_calendar::CalendarEventIndexInfo>,
    /// Host storage keyed by (offset_months, month_day); Vec order gives dense positions.
    /// Key widths and ordering policy do not establish native argument bounds.
    pub calendar_day_events: HashMap<
        (i32, u32),
        Vec<crate::c_api::c_calendar::day_event_record::CalendarDayEventRecord>,
    >,
    pub achievement_guild_rep: HashMap<i32, AchievementGuildRep>,
    pub achievement_statistics: HashMap<i32, AchievementStatistic>,
    pub achievement_comparison_unit: Option<String>,
    pub achievement_comparison_data: AchievementComparisonData,
    pub guild_achievement_members: HashMap<i32, Vec<String>>,
    pub achievement_search: AchievementSearchState,
    pub focused_achievement: Option<i32>,
    pub area_pois: HashMap<i32, AreaPoiInfo>,
    /// INFERRED row048: ordered race POI identifiers per hub POI; empty by default.
    #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
    pub quest_hub_dragonriding_races: HashMap<i32, Vec<i32>>,
    pub bnet_friends: Vec<BnetFriend>,
    pub bnet_friend_invites: Vec<BnetFriendInvite>,
    pub bnet_appear_offline: bool,
    /// INFERRED local broadcast input; no network service or persistence.
    #[cfg(feature = "retail-12-0-0")]
    pub bnet_custom_message: String,
    pub social_friends: Vec<SocialFriend>,
    pub social_restrictions: SocialRestrictionsState,
    /// Wrath selection retains identity when the display projection changes.
    #[cfg(feature = "client-wrath")]
    pub selected_friend_guid: Option<String>,
    pub auction_browse_results: Vec<AuctionBrowseResult>,
    pub auction_replicate_items: Vec<AuctionReplicateItem>,
    pub auction_owned: Vec<OwnedAuction>,
    pub auction_bids: Vec<BidAuction>,
    pub auction_item_searches: ::std::collections::HashMap<ItemSearchKey, ItemSearchResults>,
    pub auction_commodity_searches: ::std::collections::HashMap<i32, CommoditySearchResults>,
    pub auction_sell_search_results: ::std::collections::HashMap<ItemSearchKey, ItemSearchResults>,
    pub auction_last_browse_query: Option<BrowseQuery>,
    pub auction_queued_browse_query: Option<BrowseQuery>,
    pub auction_favorites: Vec<ItemSearchKey>,
    pub auction_sell_quote: Option<AuctionSellQuote>,
    pub auction_sell_item: Option<BagItem>,
    pub selected_auction_list: i32,
    pub selected_auction_bidder: i32,
    pub selected_auction_owner: i32,
    pub auction_index: HashMap<i64, AuctionRowInfo>,
    pub commodity_purchase_quote: Option<CommodityPurchaseQuote>,
    pub auction_throttle_ready: bool,
    pub auction_should_auto_populate_price: bool,
    pub wow_token: WowTokenState,
    pub mythic_plus: MythicPlusState,
    pub character_services: CharacterServicesState,
    pub scenario: ScenarioState,
    pub death_recaps: Vec<DeathRecapEntry>,
    pub chat_bubbles: Vec<ChatBubble>,
    pub summon_request: SummonRequestState,
    pub player_map_position: (f64, f64),
    pub factions: Vec<FactionEntry>,
    pub selected_faction_index: i32,
    pub watched_faction_index: i32,
    /// Explicit map observations, visible only while is_active_battlefield is true.
    pub battlefield_vehicles: ::std::collections::HashMap<
        i32,
        Vec<crate::c_api::c_pvp::battlefield_vehicles::BattlefieldVehicle>,
    >,
    pub battlefield_queue: BattlefieldQueue,
    pub battlefield_minimap_visible: bool,
    pub chat_channels: Vec<ChatChannel>,
    pub macros: Vec<MacroInfo>,
    pub running_macro: Option<u32>,
    pub chat_windows: ::std::collections::HashMap<i32, ChatWindow>,
    pub chat_type_colors: ::std::collections::HashMap<String, (f32, f32, f32)>,
    pub pending_duel: Option<String>,
    pub pending_resurrect: Option<String>,
    pub corpse_available: bool,
    /// Supplied corpse coordinates by UI map; never inferred from player position.
    pub corpse_map_positions: HashMap<i32, (f64, f64)>,
    pub active_trade: Option<TradeState>,
    pub open_panels: ::std::collections::HashSet<String>,
    pub is_party_lfg: bool,
    pub everyone_assistant: bool,
    /// INFERRED explicit per-member roles; empty by default, names match roster identity.
    #[cfg(feature = "retail-12-0-7")]
    pub party_assistants: HashSet<String>,
    #[cfg(feature = "retail-12-0-7")]
    pub party_assistant_exclusions: HashSet<String>,
    /// INFERRED explicit restriction input; false, not inferred from combat or rank.
    #[cfg(feature = "retail-12-0-7")]
    pub party_operations_restricted: bool,
    /// Explicit non-home category GUID membership; empty default, no invented GUIDs.
    #[cfg(feature = "retail-12-0-7")]
    pub party_category_guids: HashMap<i32, HashSet<String>>,
    /// Explicit solo-entry inputs; incomplete payload never publishes GROUP_FORMED.
    #[cfg(feature = "retail-12-0-7")]
    pub solo_follower_dungeon: bool,
    #[cfg(feature = "retail-12-0-7")]
    pub solo_group_category: Option<i32>,
    #[cfg(feature = "retail-12-0-7")]
    pub solo_group_guid: Option<String>,
    /// INFERRED identity latch reset on observed exit, not a server formation epoch.
    #[cfg(feature = "retail-12-0-7")]
    pub(crate) solo_group_last: Option<(i32, String)>,
    /// Explicit optional display role by existing player/party GUID; unset by default.
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    pub(crate) unit_roles:
        HashMap<String, Option<crate::lua_api::globals::real::unit_roles_assignment::UnitRole>>,
    pub party_leader_index: Option<usize>,
    pub ready_check: ReadyCheckState,
    pub voice_chat: VoiceChatState,
    /// Explicit (unit token, channel mode) legacy presentation inputs.
    #[cfg(feature = "client-wrath")]
    pub legacy_unit_voice:
        HashMap<(String, String), crate::lua_api::globals::real::wrath_voice::LegacyUnitVoice>,
    #[cfg(feature = "retail-12-0-5")]
    pub voice_channels: Vec<crate::c_api::c_voice_chat_channels::VoiceChannel>,
    #[cfg(feature = "retail-12-0-5")]
    pub lfg_endgame_policy: crate::c_api::c_lfg_list_policy::EndgameEditPolicy,
    #[cfg(feature = "retail-12-0-5")]
    pub arena_crowd_control: HashMap<String, crate::c_api::c_pvp_crowd_control::CrowdControlWindow>,
    #[cfg(feature = "retail-12-0-5")]
    pub hidden_spell_item_enchantments: std::collections::HashSet<u32>,
    #[cfg(feature = "retail-12-0-5")]
    pub bnet_presence: crate::c_api::c_battle_net::Presence,
    /// Accepted public SpeakText requests in call order; no audio playback.
    #[cfg(feature = "retail-12-0-5")]
    pub voice_chat_speak_requests: Vec<crate::c_api::c_voice_chat_speak::SpeakTextRequest>,
    pub wrath_spell_slots: crate::lua_api::globals::real::wrath_spell_slots::SpellSlots,
    pub wrath_spell_selection: (Option<u32>, Option<u32>),
    pub known_spells: ::std::collections::HashSet<u32>,
    pub harmful_spells: ::std::collections::HashSet<u32>,
    pub helpful_spells: ::std::collections::HashSet<u32>,
    pub pet_spells: ::std::collections::HashSet<u32>,
    pub pvp_last_honor_gain: i32,
    /// Explicit host booleans; only Retail/PTR publish the query, never inferred.
    pub item_dressability: HashMap<u32, bool>,
    pub equippable_items: ::std::collections::HashSet<u32>,
    pub consumable_items: ::std::collections::HashSet<u32>,
    pub can_replace_guild_master: bool,
    pub auto_decline_guild_invites: bool,
    pub auto_decline_neighborhood_invites: bool,
    #[cfg(feature = "retail-12-0-0")]
    pub dye_colors: crate::c_api::c_dye_color::DyeColorState,
    pub guild_roster_show_offline: bool,
    pub menu_open: bool,
    pub xp_disabled: bool,
    pub can_teleport: bool,
    pub has_hearthstone: bool,
    pub message_log: Vec<MessageLogEntry>,
    pub keybindings: Keybindings,
    pub debug_borders: bool,
    pub debug_anchors: bool,
    pub simulator_exit_requested: bool,
    pub ui_reload_requested: bool,
    /// Explicit host-supplied cinematic playback state; no playback engine implied.
    pub cinematic_active: bool,
    /// World entry is distinct from IsLoggedIn; supplied/transitioned by the owner.
    pub in_world: bool,
    /// Wrath reload's temporary protected-action phase, not the player's combat bit.
    pub reload_allows_protected_actions: bool,
    /// Supplied readiness of the two account CVar chunks; not a network client.
    pub account_cvar_chunks_ready: [bool; 2],
    pub(crate) account_cvar_callback_pending: bool,
    /// Simulator lifecycle marker, not a native event-order assertion.
    pub ui_reloaded: bool,
}

impl SimState {
    /// Share source lookup with every environment-associated renderer, synchronizing
    /// the publicly mutable root vector before any manager borrow.
    pub fn texture_manager(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::texture::TextureManager>> {
        self.texture_manager
            .borrow_mut()
            .set_source_roots(&self.addon_base_paths);
        self.texture_manager.clone()
    }
}
