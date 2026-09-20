//! Post-load workarounds that are still required on the live rilua path.

mod logging;
mod permanent;
mod runtime_surfaces;
mod temporary;

pub(crate) use temporary::environment_cleanup_restore::restore_post_cleanup_globals;
pub(crate) use temporary::source_patches::patch_lua_source;

use logging::log_step;
pub use runtime_surfaces::patch_uiparent_managed_frame_mixin;
use runtime_surfaces::*;
pub(crate) use runtime_surfaces::{
    patch_account_store_set_storefront, patch_glueparent_uiparent_attributes,
    patch_playerspells_onload_backfill, patch_quest_log_mixin, patch_shared_xml_anim_mixins,
    patch_unit_position_frame_mixin,
};

pub(crate) fn patch_action_bar_button_event_fanout_for_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::action_bar_button_event_fanout::patch_loader(env);
}

struct WorkaroundStep {
    label: &'static str,
    apply: fn(&crate::lua_api::WowLuaEnv),
}

const POST_LOAD_WORKAROUNDS: &[WorkaroundStep] = &[
    WorkaroundStep {
        label: "patch_edit_mode_manager",
        apply: patch_edit_mode_manager,
    },
    WorkaroundStep {
        label: "init_edit_mode_layout",
        apply: init_edit_mode_layout,
    },
    WorkaroundStep {
        label: "patch_ui_parent_panel_toggles",
        apply: patch_ui_parent_panel_toggles,
    },
    WorkaroundStep {
        label: "patch_uiparent_onupdate_worklists",
        apply: patch_uiparent_onupdate_worklists,
    },
    WorkaroundStep {
        label: "init_chat_type_colors",
        apply: init_chat_type_colors,
    },
    WorkaroundStep {
        label: "patch_chat_voice_button_surface",
        apply: patch_chat_voice_button_surface,
    },
    WorkaroundStep {
        label: "patch_item_socketing_tooltips",
        apply: patch_item_socketing_tooltips,
    },
    WorkaroundStep {
        label: "patch_character_select_selected_name",
        apply: patch_character_select_selected_name,
    },
    WorkaroundStep {
        label: "patch_character_create_defaults",
        apply: patch_character_create_defaults,
    },
    WorkaroundStep {
        label: "patch_character_frame_title_refresh",
        apply: patch_character_frame_title_refresh,
    },
    WorkaroundStep {
        label: "patch_vignette_pin_template",
        apply: patch_vignette_pin_template,
    },
    WorkaroundStep {
        label: "patch_fog_of_war_pin_mixin",
        apply: patch_fog_of_war_pin_mixin,
    },
    WorkaroundStep {
        label: "patch_map_exploration_pin_mixin",
        apply: patch_map_exploration_pin_mixin,
    },
    WorkaroundStep {
        label: "patch_map_canvas_data_provider_attachment",
        apply: patch_map_canvas_data_provider_attachment,
    },
    WorkaroundStep {
        label: "ensure_adventure_map_frame_surface",
        apply: ensure_adventure_map_frame_surface,
    },
    WorkaroundStep {
        label: "patch_action_bar_button_event_fanout",
        apply: patch_action_bar_button_event_fanout,
    },
    WorkaroundStep {
        label: "patch_paging_controls_page_text",
        apply: patch_paging_controls_page_text,
    },
    WorkaroundStep {
        label: "patch_talent_edge_frame_level_sync",
        apply: patch_talent_edge_frame_level_sync,
    },
    WorkaroundStep {
        label: "patch_catalog_shop_product_card_defaults",
        apply: patch_catalog_shop_product_card_defaults,
    },
    WorkaroundStep {
        label: "patch_game_time_defaults",
        apply: patch_game_time_defaults,
    },
    WorkaroundStep {
        label: "patch_tooltip_nineslice_surface",
        apply: patch_tooltip_nineslice_surface,
    },
    WorkaroundStep {
        label: "patch_container_frame_token_tracker",
        apply: patch_container_frame_token_tracker,
    },
    WorkaroundStep {
        label: "patch_achievement_display_set_achievements",
        apply: patch_achievement_display_set_achievements,
    },
    WorkaroundStep {
        label: "patch_housing_dashboard_preload",
        apply: patch_housing_dashboard_preload_from_env,
    },
    WorkaroundStep {
        label: "patch_lfg_lock_list",
        apply: patch_lfg_lock_list,
    },
    WorkaroundStep {
        label: "patch_auction_house_browse_results_event",
        apply: patch_auction_house_browse_results_event_from_env,
    },
    WorkaroundStep {
        label: "patch_auction_house_search_context_aliases",
        apply: patch_auction_house_search_context_aliases_from_env,
    },
    WorkaroundStep {
        label: "patch_auth_challenge_frame_parent",
        apply: patch_auth_challenge_frame_parent_from_env,
    },
    WorkaroundStep {
        label: "patch_settings_surface_defaults",
        apply: patch_settings_surface_defaults,
    },
    WorkaroundStep {
        label: "patch_settings_canvas_layout_visibility",
        apply: patch_settings_canvas_layout_visibility,
    },
    #[cfg(feature = "client-wrath")]
    WorkaroundStep {
        label: "wrath::post_load",
        apply: apply_wrath_post_load,
    },
    #[cfg(feature = "client-mists")]
    WorkaroundStep {
        label: "mists::post_load",
        apply: apply_mists_post_load,
    },
];

pub fn apply(env: &crate::lua_api::WowLuaEnv) {
    for step in POST_LOAD_WORKAROUNDS {
        log_step(env, step.label, || (step.apply)(env));
    }
}

pub(crate) fn apply_permanent_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    permanent::apply_bootstrap(lua)
}

pub(crate) fn apply_temporary_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_temporary_state_bootstrap(lua)?;
    apply_temporary_namespace_bootstrap(lua)
}

fn apply_temporary_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_runtime_state_bootstrap(lua)?;
    apply_account_and_social_state_bootstrap(lua)?;
    apply_player_state_bootstrap(lua)?;
    apply_secure_and_store_state_bootstrap(lua)?;
    apply_unit_state_bootstrap(lua)?;
    temporary::uiparent_onupdate_worklists::apply_bootstrap(lua)?;
    temporary::video_options_state::apply_bootstrap(lua)
}

fn apply_runtime_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::global_placeholder_tables::apply_bootstrap(lua)?;
    temporary::frame_helper_defaults::apply_bootstrap(lua)?;
    temporary::event_scheduler_state::apply_bootstrap(lua)?;
    temporary::combat_log_state::apply_bootstrap(lua)?;
    temporary::damage_meter_state::apply_bootstrap(lua)?;
    temporary::encounter_state::apply_bootstrap(lua)?;
    temporary::housing_catalog_state::apply_bootstrap(lua)?;
    temporary::map_runtime_state::apply_bootstrap(lua)?;
    temporary::perks_activities_state::apply_bootstrap(lua)?;
    temporary::private_aura_state::apply_bootstrap(lua)?;
    temporary::reputation_state::apply_bootstrap(lua)?;
    temporary::roleset_defaults::apply_bootstrap(lua)
}

fn apply_account_and_social_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::battle_net_account_defaults::apply_bootstrap(lua)?;
    temporary::club_notification_defaults::apply_bootstrap(lua)?;
    temporary::social_queue_defaults::apply_bootstrap(lua)?;
    temporary::merchant_filter_state::apply_bootstrap(lua)
}

fn apply_player_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::player_spells_onload_backfill::apply_bootstrap(lua)?;
    temporary::possess_info_defaults::apply_bootstrap(lua)?;
    temporary::totem_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_secure_and_store_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::restricted_actions_defaults::apply_bootstrap(lua)?;
    temporary::secure_reference_defaults::apply_bootstrap(lua)?;
    temporary::secure_types_defaults::apply_bootstrap(lua)?;
    temporary::secure_transfer_state::apply_bootstrap(lua)?;
    temporary::store_glue_state::apply_bootstrap(lua)?;
    temporary::store_public_defaults::apply_bootstrap(lua)
}

fn apply_unit_state_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::unit_auras_state::apply_bootstrap(lua)?;
    temporary::unit_stagger_defaults::apply_bootstrap(lua)?;
    temporary::unit_threat_defaults::apply_bootstrap(lua)
}

fn apply_temporary_namespace_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_core_temporary_namespace_bootstrap(lua)?;
    apply_feature_temporary_namespace_bootstrap(lua)
}

fn apply_core_temporary_namespace_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_core_foundation_defaults(lua)?;
    apply_core_legacy_defaults(lua)?;
    Ok(())
}

fn apply_core_foundation_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_core_foundation_frame_defaults(lua)?;
    apply_core_foundation_state_defaults(lua)?;
    apply_core_dispatcher_and_format_defaults(lua)
}

fn apply_core_foundation_frame_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_core_foundation_addon_defaults(lua)?;
    apply_core_foundation_chat_defaults(lua)?;
    apply_core_foundation_journal_defaults(lua)?;
    apply_core_foundation_catalog_defaults(lua)?;
    apply_core_foundation_environment_defaults(lua)
}

fn apply_core_foundation_catalog_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::camera_tutorial_defaults::apply_bootstrap(lua)?;
    temporary::catalog_shop_inbound_globals::apply_bootstrap(lua)?;
    temporary::catalog_shop_product_card_defaults::apply_bootstrap(lua)?;
    temporary::character_create_defaults::apply_bootstrap(lua)?;
    temporary::class_trial_defaults::apply_bootstrap(lua)?;
    temporary::click_bindings_defaults::apply_bootstrap(lua)?;
    temporary::client_info_defaults::apply_bootstrap(lua)?;
    temporary::collections_journal_namespace::apply_bootstrap(lua)?;
    temporary::color_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_core_foundation_environment_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_core_foundation_tracking_defaults(lua)?;
    temporary::debug_environment_defaults::apply_bootstrap(lua)?;
    temporary::difficulty_pvp_util_defaults::apply_bootstrap(lua)?;
    temporary::edit_mode_cache_defaults::apply_bootstrap(lua)?;
    temporary::equipment_set_lock_defaults::apply_bootstrap(lua)?;
    temporary::global_frame_defaults::apply_bootstrap(lua)?;
    temporary::gossip_poi_defaults::apply_bootstrap(lua)?;
    temporary::major_faction_display_defaults::apply_bootstrap(lua)?;
    temporary::map_group_defaults::apply_bootstrap(lua)?;
    temporary::merchant_raid_defaults::apply_bootstrap(lua)?;
    temporary::mythic_plus_cache_defaults::apply_bootstrap(lua)?;
    temporary::shared_character_services_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_core_foundation_chat_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::c_chat_info_defaults::apply_bootstrap(lua)?;
    temporary::character_services_defaults::apply_bootstrap(lua)?;
    temporary::chat_voice_button_surface::apply_bootstrap(lua)?;
    temporary::chat_window_defaults::apply_bootstrap(lua)
}

fn apply_core_foundation_tracking_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::contribution_collector_defaults::apply_bootstrap(lua)?;
    temporary::content_tracking_defaults::apply_bootstrap(lua)?;
    temporary::cooldown_viewer_defaults::apply_bootstrap(lua)?;
    temporary::configuration_warnings_defaults::apply_bootstrap(lua)?;
    temporary::container_default_shapes::apply_bootstrap(lua)?;
    temporary::container_portrait_texture::apply_bootstrap(lua)?;
    temporary::minimap_tracking_defaults::apply_bootstrap(lua)?;
    temporary::super_track_defaults::apply_bootstrap(lua)
}

fn apply_core_foundation_addon_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::addon_compartment_defaults::apply_bootstrap(lua)?;
    temporary::addons_beta_policy_defaults::apply_bootstrap(lua)?;
    temporary::auth_challenge_frame_parent::apply_bootstrap(lua)?;
    temporary::auto_complete_defaults::apply_bootstrap(lua)?;
    temporary::behavioral_messaging_defaults::apply_bootstrap(lua)?;
    temporary::black_market_defaults::apply_bootstrap(lua)?;
    temporary::calendar_defaults::apply_bootstrap(lua)
}

fn apply_core_foundation_journal_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::achievement_ui_access_defaults::apply_bootstrap(lua)?;
    temporary::achievement_search_preview::apply_bootstrap(lua)?;
    temporary::alert_frame_defaults::apply_bootstrap(lua)?;
    temporary::adventure_journal_fallbacks::apply_bootstrap(lua)?;
    temporary::perks_program_defaults::apply_bootstrap(lua)?;
    temporary::loot_journal_defaults::apply_bootstrap(lua)
}

fn apply_core_foundation_state_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::pet_battle_runtime_state::apply_bootstrap(lua)?;
    temporary::settings_surface_defaults::apply_bootstrap(lua)?;
    temporary::tooltip_data_processor_defaults::apply_bootstrap(lua)?;
    temporary::tts_settings_defaults::apply_bootstrap(lua)?;
    temporary::ui_widget_manager_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_core_dispatcher_and_format_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_dispatcher_and_date_defaults(lua)?;
    apply_format_and_game_defaults(lua)?;
    apply_inventory_and_spell_defaults(lua)
}

fn apply_dispatcher_and_date_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::dispatcher_callback_defaults::apply_bootstrap(lua)?;
    temporary::dispatcher_surface::apply_bootstrap(lua)?;
    temporary::date_and_time_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_format_and_game_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::display_scale_defaults::apply_bootstrap(lua)?;
    temporary::dropdown_list_defaults::apply_bootstrap(lua)?;
    temporary::formatting_utility_defaults::apply_bootstrap(lua)?;
    temporary::game_time_calendar_invites::apply_bootstrap(lua)?;
    temporary::gamepad_cursor_control_defaults::apply_bootstrap(lua)?;
    temporary::game_rules_namespace_fallback::apply_bootstrap(lua)?;
    temporary::glue_character_select_defaults::apply_bootstrap(lua)?;
    temporary::guild_info_namespace_fallback::apply_bootstrap(lua)?;
    temporary::inert_global_defaults::apply_bootstrap(lua)?;
    temporary::patch_12_1_inert_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_inventory_and_spell_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::dye_color_defaults::apply_bootstrap(lua)?;
    temporary::inventory_query_defaults::apply_bootstrap(lua)?;
    temporary::item_button_helper_defaults::apply_bootstrap(lua)?;
    temporary::item_targeting_defaults::apply_bootstrap(lua)?;
    temporary::item_upgrade_availability_defaults::apply_bootstrap(lua)?;
    temporary::spell_metadata_defaults::apply_bootstrap(lua)?;
    temporary::spell_static_defaults::apply_bootstrap(lua)?;
    temporary::spell_target_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_core_legacy_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::kiosk_namespace_defaults::apply_bootstrap(lua)?;
    temporary::level_link_spell_lock_state::apply_bootstrap(lua)?;
    temporary::lfg_legacy_defaults::apply_bootstrap(lua)?;
    temporary::legacy_action_bar_globals::apply_bootstrap(lua)?;
    temporary::legacy_container_globals::apply_bootstrap(lua)?;
    temporary::legacy_spell_globals::apply_bootstrap(lua)?;
    temporary::legacy_talent_skill_defaults::apply_bootstrap(lua)?;
    temporary::modified_click_defaults::apply_bootstrap(lua)?;
    temporary::performance_metric_defaults::apply_bootstrap(lua)?;
    temporary::script_bucket_throttle_limits::apply_bootstrap(lua)?;
    temporary::pool_constructor_defaults::apply_bootstrap(lua)?;
    temporary::misc_global_frame_defaults::apply_bootstrap(lua)?;
    Ok(())
}

fn apply_feature_temporary_namespace_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    apply_feature_tool_defaults(lua)?;
    apply_feature_model_defaults(lua)?;
    apply_feature_ui_defaults(lua)
}

fn apply_feature_tool_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::assisted_combat_manager_defaults::apply_bootstrap(lua)?;
    temporary::base_nine_slice_dialog_defaults::apply_bootstrap(lua)?;
    temporary::callback_registry_defaults::apply_bootstrap(lua)?;
    temporary::macro_defaults::apply_bootstrap(lua)?;
    temporary::navigation_defaults::apply_bootstrap(lua)?;
    temporary::object_api_request_load_callbacks::apply_bootstrap(lua)?;
    temporary::party_info_instance_abandon_defaults::apply_bootstrap(lua)?;
    temporary::party_info_static_defaults::apply_bootstrap(lua)?;
    temporary::ping_defaults::apply_bootstrap(lua)?;
    temporary::player_location_defaults::apply_bootstrap(lua)?;
    apply_feature_progression_defaults(lua)
}

fn apply_feature_progression_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::profession_specs_defaults::apply_bootstrap(lua)?;
    temporary::prototype_dialog_state::apply_bootstrap(lua)?;
    temporary::proxy_object_factories::apply_bootstrap(lua)?;
    temporary::pvp_talent_defaults::apply_bootstrap(lua)?;
    temporary::quest_objective_defaults::apply_bootstrap(lua)?;
    temporary::reincarnation_defaults::apply_bootstrap(lua)?;
    temporary::scenario_defaults::apply_bootstrap(lua)?;
    temporary::seconds_formatter_defaults::apply_bootstrap(lua)?;
    temporary::scripted_animation_effect_defaults::apply_bootstrap(lua)?;
    temporary::shared_xml_utility_defaults::apply_bootstrap(lua)?;
    temporary::spell_book_static_defaults::apply_bootstrap(lua)
}

fn apply_feature_model_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::sound_driver_defaults::apply_bootstrap(lua)?;
    temporary::static_model_info_defaults::apply_bootstrap(lua)?;
    temporary::static_popup_defaults::apply_bootstrap(lua)?;
    temporary::taxi_map_defaults::apply_bootstrap(lua)?;
    temporary::texture_file_data_defaults::apply_bootstrap(lua)?;
    temporary::top_level_parent_defaults::apply_bootstrap(lua)?;
    temporary::tracking_namespace_defaults::apply_bootstrap(lua)?;
    temporary::transmog_sets_defaults::apply_bootstrap(lua)?;
    temporary::transmog_outfit_slot_defaults::apply_bootstrap(lua)?;
    temporary::trade_info_defaults::apply_bootstrap(lua)
}

fn apply_feature_ui_defaults(lua: &mut rilua::Lua) -> crate::Result<()> {
    temporary::trade_skill_ui_fallbacks::apply_bootstrap(lua)?;
    temporary::transmog_util_defaults::apply_bootstrap(lua)?;
    temporary::campaign_covenant_defaults::apply_bootstrap(lua)?;
    temporary::ui_parent_panel_toggles::apply_bootstrap(lua)?;
    temporary::ui_widget_power_bar_defaults::apply_bootstrap(lua)?;
    temporary::ui_frame_manager_defaults::apply_bootstrap(lua)
}

pub fn close_startup_special_windows_before_first_frame(env: &crate::lua_api::WowLuaEnv) {
    temporary::startup_windows::close_before_first_frame(env);
}

pub(crate) fn sanitize_imported_wtf_addon_saved_variables(
    state: &mut rilua::vm::state::LuaState,
    addon_name: &str,
) {
    temporary::details_saved_variables::sanitize_imported_wtf_addon(state, addon_name);
}

pub(crate) fn apply_cpp_mixin_stubs_after_lua_file(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = temporary::cpp_mixin_stubs::patch_after_lua_file(env);
}

pub(crate) fn patch_callback_registry_defaults(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = temporary::callback_registry_defaults::patch_for_addon_load(env);
}

pub(crate) fn patch_dispatcher_surface_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = temporary::dispatcher_surface::patch_for_addon_load(env);
}

pub(crate) fn patch_achievement_search_preview_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = temporary::achievement_search_preview::patch_for_addon_load(env);
}

pub(crate) fn patch_quest_objective_defaults_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::quest_objective_defaults::patch_loader(env);
}

/// Re-run the runtime-surface bootstrap repair hooks for addons whose load
/// replaces the patched objects. The bootstrap exposes these as `__wow_patch_*`
/// globals; the old `hooksecurefunc(C_AddOns, "LoadAddOn")` route is refused
/// by the shared bootstrap and never fired.
pub(crate) fn patch_runtime_surface_for_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
    folder_name: &str,
) {
    let patch_fns: &[&str] = match folder_name {
        "Blizzard_CharacterSelectNavBar" => &["__wow_patch_character_select_nav_bar"],
        "Blizzard_UIParent"
        | "Blizzard_UIParent_Mainline"
        | "Blizzard_FrameXML"
        | "Blizzard_ChatFrameBase" => &["__wow_patch_uiparent_onupdate_worklists"],
        "Blizzard_MapCanvas"
        | "Blizzard_SharedMapDataProviders"
        | "Blizzard_WorldMap"
        | "Blizzard_BattlefieldMap" => &[
            "__wow_patch_map_canvas_scroll_container_methods",
            "__wow_patch_fog_of_war_pin_methods",
        ],
        _ => return,
    };
    for patch_fn in patch_fns {
        let _ = env.exec(&format!(
            r#"if type({patch_fn}) == "function" then {patch_fn}() end"#
        ));
    }
}

fn patch_edit_mode_manager(env: &crate::lua_api::WowLuaEnv) {
    crate::lua_api::workarounds_editmode::patch_edit_mode_manager(env);
}

fn init_edit_mode_layout(env: &crate::lua_api::WowLuaEnv) {
    crate::lua_api::workarounds_editmode::init_edit_mode_layout(env);
}

fn init_chat_type_colors(env: &crate::lua_api::WowLuaEnv) {
    crate::lua_api::chat_init::init_chat_type_colors(env);
}

fn patch_settings_surface_defaults(env: &crate::lua_api::WowLuaEnv) {
    temporary::settings_surface_defaults::patch(env);
}

fn patch_settings_canvas_layout_visibility(env: &crate::lua_api::WowLuaEnv) {
    temporary::settings_canvas_visibility::patch(env);
}

fn patch_housing_dashboard_preload_from_env(env: &crate::lua_api::WowLuaEnv) {
    patch_housing_dashboard_preload(&env.loader_env());
}

#[cfg(feature = "client-wrath")]
fn apply_wrath_post_load(env: &crate::lua_api::WowLuaEnv) {
    crate::wrath::post_load::apply(env);
}

#[cfg(feature = "client-mists")]
fn apply_mists_post_load(env: &crate::lua_api::WowLuaEnv) {
    crate::mists::post_load::apply(env);
}

pub fn apply_post_event(env: &crate::lua_api::WowLuaEnv) {
    apply_post_event_bootstrap(env);
    patch_post_event_frame_layout(env);
    refresh_post_event_surfaces(env);
}

fn apply_post_event_bootstrap(env: &crate::lua_api::WowLuaEnv) {
    temporary::post_event_action_button_refresh::patch(env);
    crate::lua_api::workarounds_editmode::init_edit_mode_layout(env);
    crate::lua_api::workarounds_editmode::reapply_player_frame_anchor(env);
    crate::lua_api::chat_init::init_chat_type_colors(env);
    crate::lua_api::chat_init::show_chat_frame(env);
}

fn patch_post_event_frame_layout(env: &crate::lua_api::WowLuaEnv) {
    temporary::post_event_frame_layout::patch(env);
}

fn refresh_post_event_surfaces(env: &crate::lua_api::WowLuaEnv) {
    refresh_character_frame_surface(env);
    patch_chat_voice_button_surface(env);
    patch_objective_tracker_quest_header(env);
}

pub fn apply_for_runtime_addon_load(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    #[cfg(feature = "client-mists")]
    crate::mists::post_load::apply_for_runtime_addon_load(env, addon_name);
    patch_runtime_core_addon_surfaces(env, addon_name);
    patch_runtime_journal_addon_surfaces(env, addon_name);
    patch_runtime_feature_addon_surfaces(env, addon_name);
}

fn patch_runtime_core_addon_surfaces(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    if matches!(
        addon_name,
        "Blizzard_ChatFrame"
            | "Blizzard_QuickJoin"
            | "Blizzard_Channels"
            | "Blizzard_VoiceToggleButton"
    ) {
        temporary::chat_voice_button_surface::patch_loader(env);
    }
    if addon_name == "Blizzard_PagedContent" {
        temporary::paging_controls_page_text::patch_loader(env);
    }
    if matches!(
        addon_name,
        "Blizzard_SharedTalentUI" | "Blizzard_PlayerSpells"
    ) {
        temporary::talent_edge_frame_level_sync::patch_loader(env);
    }
    if addon_name == "Blizzard_PlayerSpells" {
        temporary::pvp_talent_defaults::patch_loader(env);
    }
    if addon_name == "Blizzard_FrameXMLUtil" {
        temporary::quest_objective_defaults::patch_loader(env);
    }
    patch_runtime_map_addon_surfaces(env, addon_name);
}

fn patch_runtime_journal_addon_surfaces(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    if addon_name == "Blizzard_Collections" {
        patch_toggle_collections_journal_for_runtime_addon_load(env);
        temporary::collections_journal_namespace::patch(env);
    }
    if addon_name == "Blizzard_EncounterJournal" {
        patch_toggle_encounter_journal_for_runtime_addon_load(env);
    }
    if addon_name == "Blizzard_AdventureMap" {
        ensure_adventure_map_frame_surface_for_runtime_addon_load(env);
    }
}

fn patch_runtime_feature_addon_surfaces(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    if matches!(addon_name, "Blizzard_ArtifactUI" | "Blizzard_Colors") {
        patch_item_quality_color_data_methods(env);
    }
    if addon_name == "Blizzard_ArtifactUI" {
        patch_artifact_ui_show_panel_guard(env);
    }
    if addon_name == "Blizzard_AuctionHouseUI" {
        patch_auction_house_runtime_surface(env);
    }
    if addon_name == "Blizzard_AuthChallengeUI" {
        patch_auth_challenge_frame_parent(env);
    }
    if addon_name == "Blizzard_AccountStore" {
        let _ = patch_account_store_set_storefront(env);
    }
    if addon_name == "Blizzard_CatalogShop" {
        temporary::catalog_shop_product_card_defaults::patch_for_runtime_addon_load(env);
    }
    if addon_name == "Blizzard_DamageMeter" {
        patch_damage_meter_initial_scrollbox_extent(env);
    }
}

fn patch_auction_house_runtime_surface(env: &crate::lua_api::LoaderEnv<'_>) {
    patch_auction_house_categories_refresh_count(env);
    patch_auction_house_browse_results_event(env);
    patch_auction_house_search_context_aliases(env);
}

fn patch_runtime_map_addon_surfaces(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    if addon_name == "Blizzard_MapCanvas" {
        let _ = temporary::map_canvas_scroll_container::patch(env);
    }
    if matches!(
        addon_name,
        "Blizzard_MapCanvas"
            | "Blizzard_SharedMapDataProviders"
            | "Blizzard_WorldMap"
            | "Blizzard_BattlefieldMap"
    ) {
        patch_fog_of_war_pin_mixin_for_runtime_addon_load(env);
        patch_map_exploration_pin_mixin_for_runtime_addon_load(env);
        patch_map_canvas_data_provider_attachment_for_runtime_addon_load(env);
    }
}

pub fn apply_for_runtime_addon_preload(env: &crate::lua_api::LoaderEnv<'_>, addon_name: &str) {
    if addon_name == "Blizzard_Collections" {
        temporary::collections_journal_namespace::patch(env);
    }
    if matches!(
        addon_name,
        "Blizzard_HousingDashboard" | "Blizzard_HousingHouseFinder"
    ) {
        patch_housing_dashboard_preload(env);
    }
}
#[cfg(test)]
mod tests {
    use super::temporary::settings_canvas_visibility::SETTINGS_CANVAS_LAYOUT_HIDE_LUA;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn runtime_surface_exposes_post_load_patch_globals() {
        // patch_runtime_surface_for_addon_load and the CreateFrame wrapper in
        // frame_helper_defaults.rs call these by global name from other
        // chunks; if the bootstrap keeps them local, the arms silently no-op.
        let env = WowLuaEnv::new().expect("env should initialize");
        for patch_fn in [
            "__wow_patch_character_select_nav_bar",
            "__wow_patch_uiparent_onupdate_worklists",
            "__wow_patch_map_canvas_scroll_container_methods",
            "__wow_patch_fog_of_war_pin_methods",
        ] {
            let is_function: bool = env
                .eval(&format!(r#"return type({patch_fn}) == "function""#))
                .expect("patch global probe should run");
            assert!(is_function, "{patch_fn} should be a global function");
        }
    }

    #[test]
    fn settings_canvas_registration_hides_frame_until_displayed() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            SettingsLayoutMixin = { LayoutType = { Canvas = "Canvas" } }

            local categories = {}
            local layouts = {}

            SettingsPanel = {
                shown = false,
                currentLayout = nil,
                currentCategory = nil,
                GetAllCategories = function()
                    return categories
                end,
                GetLayout = function(_, category)
                    return layouts[category]
                end,
                IsShown = function(self)
                    return self.shown
                end,
                GetCurrentLayout = function(self)
                    return self.currentLayout
                end,
                GetCurrentCategory = function(self)
                    return self.currentCategory
                end,
            }

            Settings = {
                RegisterCanvasLayoutCategory = function(frame, name)
                    local category = { name = name }
                    local layout = {
                        frame = frame,
                        GetFrame = function(self)
                            return self.frame
                        end,
                        GetLayoutType = function()
                            return SettingsLayoutMixin.LayoutType.Canvas
                        end,
                    }
                    table.insert(categories, category)
                    layouts[category] = layout
                    return category, layout
                end,
                OpenToCategory = function(category)
                    SettingsPanel.shown = true
                    SettingsPanel.currentCategory = category
                    SettingsPanel.currentLayout = layouts[category]
                    return category
                end,
            }
            "#,
        )
        .expect("fake settings surface should install");

        env.exec(SETTINGS_CANVAS_LAYOUT_HIDE_LUA)
            .expect("settings canvas workaround should apply");

        let hidden_after_register: bool = env
            .eval(
                r#"
                local frame = CreateFrame("Frame", "SettingsCanvasLeakProbe")
                frame:Show()
                local category, layout = Settings.RegisterCanvasLayoutCategory(frame, "Probe")
                return not frame:IsShown()
                "#,
            )
            .expect("registration probe should run");

        assert!(
            hidden_after_register,
            "settings canvas frame should be hidden after registration"
        );

        let opened_canvas_visible_others_hidden: bool = env
            .eval(
                r#"
                local first = SettingsCanvasLeakProbe
                local firstCategory = SettingsPanel:GetAllCategories()[1]
                local second = CreateFrame("Frame", "SettingsSecondCanvasLeakProbe")
                second:Show()
                local secondCategory = Settings.RegisterCanvasLayoutCategory(second, "Second")

                Settings.OpenToCategory(firstCategory)
                local firstOpened = first:IsShown() and not second:IsShown()

                Settings.OpenToCategory(secondCategory)
                return firstOpened and (not first:IsShown()) and second:IsShown()
                "#,
            )
            .expect("open category probe should run");

        assert!(
            opened_canvas_visible_others_hidden,
            "opening a settings category should show only that category's canvas"
        );
    }
}
