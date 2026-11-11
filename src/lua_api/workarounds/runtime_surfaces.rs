use super::*;

pub(super) fn patch_ui_parent_panel_toggles(env: &crate::lua_api::WowLuaEnv) {
    temporary::ui_parent_panel_toggles::patch(env);
    temporary::main_menu_microbutton_click::patch(env);
}

pub(super) fn patch_damage_meter_initial_scrollbox_extent(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::damage_meter_scrollbox::patch(env);
}

pub(super) fn patch_housing_dashboard_preload(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::housing_dashboard_preload::patch(env);
}

pub(super) fn patch_uiparent_onupdate_worklists(env: &crate::lua_api::WowLuaEnv) {
    temporary::uiparent_onupdate_worklists::patch(env);
}

pub(super) fn patch_vignette_pin_template(env: &crate::lua_api::WowLuaEnv) {
    temporary::vignette_pin_template::patch(env);
}

pub(super) fn patch_character_select_selected_name(env: &crate::lua_api::WowLuaEnv) {
    temporary::character_select_selected_name::patch(env);
}

pub(super) fn patch_chat_voice_button_surface(env: &crate::lua_api::WowLuaEnv) {
    temporary::chat_voice_button_surface::patch(env);
}

pub(super) fn patch_item_socketing_tooltips(env: &crate::lua_api::WowLuaEnv) {
    temporary::item_socketing_tooltips::patch(env);
}

pub(super) fn patch_action_bar_button_event_fanout(env: &crate::lua_api::WowLuaEnv) {
    temporary::action_bar_button_event_fanout::patch(env);
}

pub(super) fn patch_game_time_defaults(env: &crate::lua_api::WowLuaEnv) {
    temporary::game_time_calendar_invites::patch(env);
}

pub(super) fn patch_lfg_lock_list(env: &crate::lua_api::WowLuaEnv) {
    temporary::lfg_lock_list::patch(env);
}

pub(super) fn patch_tooltip_nineslice_surface(env: &crate::lua_api::WowLuaEnv) {
    temporary::tooltip_nineslice_surface::patch(env);
}

pub(super) fn patch_container_frame_token_tracker(env: &crate::lua_api::WowLuaEnv) {
    temporary::container_frame_token_tracker::patch(env);
}

pub(super) fn patch_paging_controls_page_text(env: &crate::lua_api::WowLuaEnv) {
    temporary::paging_controls_page_text::patch(env);
}

pub(super) fn patch_achievement_display_set_achievements(env: &crate::lua_api::WowLuaEnv) {
    permanent::reapply_achievement_display_after_blizzard_load(env);
}

pub(super) fn patch_talent_edge_frame_level_sync(env: &crate::lua_api::WowLuaEnv) {
    temporary::talent_edge_frame_level_sync::patch(env);
}

pub(super) fn patch_catalog_shop_product_card_defaults(env: &crate::lua_api::WowLuaEnv) {
    temporary::store_public_defaults::patch(env);
    temporary::catalog_shop_product_card_defaults::patch(env);
}

pub(super) fn patch_objective_tracker_quest_header(env: &crate::lua_api::WowLuaEnv) {
    temporary::objective_tracker_quest_header::patch(env);
}

pub(super) fn patch_fog_of_war_pin_mixin(env: &crate::lua_api::WowLuaEnv) {
    temporary::fog_of_war_pin::patch(env);
}

pub(super) fn patch_map_exploration_pin_mixin(env: &crate::lua_api::WowLuaEnv) {
    temporary::map_exploration_pin::patch(env);
}

pub(super) fn patch_map_canvas_data_provider_attachment(env: &crate::lua_api::WowLuaEnv) {
    temporary::map_canvas_data_provider_pin::patch(env);
}

pub(super) fn patch_character_create_defaults(env: &crate::lua_api::WowLuaEnv) {
    temporary::character_create_defaults::patch(env);
}

pub(super) fn patch_character_frame_title_refresh(env: &crate::lua_api::WowLuaEnv) {
    refresh_character_frame_surface(env);
}

pub(super) fn refresh_character_frame_surface(env: &crate::lua_api::WowLuaEnv) {
    temporary::character_frame_surface_refresh::patch(env);
}

pub(super) fn patch_fog_of_war_pin_mixin_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::fog_of_war_pin::patch_for_runtime_addon_load(env);
}

pub(super) fn patch_map_exploration_pin_mixin_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::map_exploration_pin::patch_for_runtime_addon_load(env);
}

pub(super) fn patch_toggle_collections_journal_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::ui_parent_panel_toggles::patch_collections_journal_loader(env);
    temporary::mount_journal_dynamic_flight_popup::patch(env);
}

pub(super) fn patch_toggle_encounter_journal_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::ui_parent_panel_toggles::patch_encounter_journal_loader(env);
}

pub(super) fn patch_map_canvas_data_provider_attachment_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::map_canvas_data_provider_pin::patch_for_runtime_addon_load(env);
}

pub(super) fn ensure_adventure_map_frame_surface(env: &crate::lua_api::WowLuaEnv) {
    temporary::adventure_map_frame_surface::patch(env);
}

pub(super) fn ensure_adventure_map_frame_surface_for_runtime_addon_load(
    env: &crate::lua_api::LoaderEnv<'_>,
) {
    temporary::adventure_map_frame_surface::patch_loader(env);
}

pub(super) fn patch_item_quality_color_data_methods(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::item_quality_color_data_methods::patch(env);
}

pub(super) fn patch_artifact_ui_show_panel_guard(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::artifact_ui_show_panel_guard::patch(env);
}

pub(super) fn patch_auction_house_categories_refresh_count(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::auction_house_categories_refresh_count::patch(env);
}

pub(super) fn patch_auction_house_browse_results_event(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::auction_house_browse_results_event::patch(env);
}

pub(super) fn patch_auction_house_browse_results_event_from_env(env: &crate::lua_api::WowLuaEnv) {
    temporary::auction_house_browse_results_event::patch_env(env);
}

pub(super) fn patch_auction_house_search_context_aliases(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::auction_house_search_context_aliases::patch(env);
}

pub(super) fn patch_auction_house_search_context_aliases_from_env(env: &crate::lua_api::WowLuaEnv) {
    temporary::auction_house_search_context_aliases::patch_env(env);
}

pub(super) fn patch_auth_challenge_frame_parent(env: &crate::lua_api::LoaderEnv<'_>) {
    temporary::auth_challenge_frame_parent::patch(env);
}

pub(super) fn patch_auth_challenge_frame_parent_from_env(env: &crate::lua_api::WowLuaEnv) {
    temporary::auth_challenge_frame_parent::patch_env(env);
}

pub(crate) fn patch_account_store_set_storefront(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::account_store_set_storefront::patch(env)
}

pub(crate) fn patch_playerspells_onload_backfill(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::player_spells_onload_backfill::patch(env)
}

pub(crate) fn patch_shared_xml_anim_mixins(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::shared_xml_anim_mixins::patch(env)
}

pub(crate) fn patch_glueparent_uiparent_attributes(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::glueparent_uiparent_attributes::patch(env)
}

pub fn patch_uiparent_managed_frame_mixin(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::uiparent_managed_frame_mixin::patch(env)
}

pub(crate) fn patch_unit_position_frame_mixin(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::unit_position_frame_mixin::patch(env)
}

pub(crate) fn patch_quest_log_mixin(
    env: &crate::lua_api::LoaderEnv<'_>,
) -> Result<(), crate::Error> {
    temporary::quest_log_mixin::patch(env)
}
