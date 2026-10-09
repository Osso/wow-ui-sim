# Full-suite artifact comparison: 3de874658 vs dd710c6fc

- 3de: `3de87465828db7cc7f6f900d4b63e24f1b399825`; started 2026-10-09T11:35:05-0500; finished 2026-10-09T11:49:01-0500.
- dd: `dd710c6fc469a53e3b77b97aa6340993ae83e46e`; started 2026-10-09T09:30:26-0500; finished 2026-10-09T09:44:04-0500.
- Sources: exact SHA `.json` and `.log` files under `/home/osso/Projects/wow/full-suite-results`. JSON names/counts were cross-checked against completed log output. No tests rerun.
- Set terms: **intersection** = failed in both; **additions** = failed only in 3de; **removals** = failed only in dd.

## integration

- Log exit: 3de `100`, dd `100`.
- 3de log: Summary [ 660.897s] 10681 tests run: 10658 passed (9 slow), 23 failed, 19 skipped
- dd log: Summary [ 625.810s] 10685 tests run: 10662 passed (8 slow), 23 failed, 19 skipped
- Nextest invocation IDs: 3de `eaa2b88f-2c5d-4e93-bbfe-a667ed5c796d`, dd `b9684415-e47a-4350-8f57-5e323fd6b73d`.
- JSON failure counts: 3de 23; dd 23; intersection 23; additions 0; removals 0.

### Intersection

- `blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain`
- `blizzard_frame_xml_loads::blizzard_frame_xml_toc_is_load_first_with_current_dependencies`
- `blizzard_reforging_ui_loads::find_toc_file_resolves_classic_suffix_via_fallthrough`
- `blizzard_ui_blizzard_accountstore::behavior_currency_format::format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero`
- `blizzard_ui_blizzard_achievementui::behavior_search_filter::search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length`
- `c_api_surface::chat_info_no_state_defaults_are_not_c_api_temporary_shims`
- `c_api_surface::scenario_defaults_are_not_c_api_temporary_shims`
- `c_spell_static_fallbacks::test_spell_static_fallback_shims_return_inert_values`
- `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size`
- `generated_data_refresh_coverage::generated_lua_refresh_manifest_matches_current_file_contents`
- `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface`
- `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface`
- `numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text`
- `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases`
- `patch_12_0_7_b23_b28::b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries`
- `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed`
- `spell_api::test_spell_get_spell_charges`
- `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges`
- `toplevel_render_groups::native_controls_keep_unraised_and_raised_groups_in_owner_strata`
- `toplevel_render_groups::screen_roots_do_not_capture_independent_render_groups`
- `utility_api::test_table_create_returns_empty_mutable_tables_for_capacity_variants`
- `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles`
- `wowforever_table::wowforever_table_does_not_leak_into_earlier_profiles`

### Additions in 3de only

- None.

### Removals from 3de to dd (dd only)

- None.

## lib

- Log exit: 3de `100`, dd `100`.
- 3de log: Summary [  36.311s] 1979 tests run: 1973 passed, 6 failed, 0 skipped
- dd log: Summary [  38.868s] 1979 tests run: 1973 passed, 6 failed, 0 skipped
- Nextest invocation IDs: 3de `a39a3730-0ae4-45f8-ae79-5152aad27a16`, dd `55b2a527-bf9f-434a-92c3-6c1f88951f5e`.
- JSON failure counts: 3de 6; dd 6; intersection 6; additions 0; removals 0.

### Intersection

- `loader::tests::wow_api_globals::transmog_situation::test_patch_12_0_0_transmog_situation_enum_values`
- `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults`
- `lua_api::workarounds::temporary::housing_catalog_state::tests::installs_seeded_housing_catalog_surface`
- `lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect`
- `lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons`
- `lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes`

### Additions in 3de only

- None.

### Removals from 3de to dd (dd only)

- None.

## prefork

- Log exit: 3de `1`, dd `1`.
- 3de log: test result: FAILED. 2320 passed; 1 failed; 2321 total
- dd log: test result: FAILED. 2320 passed; 1 failed; 2321 total
- JSON failure counts: 3de 1; dd 1; intersection 1; additions 0; removals 0.

### Intersection

- `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors`

### Additions in 3de only

- None.

### Removals from 3de to dd (dd only)

- None.

## Artifact-level observations

- All three steps have identical failure-name sets between these two artifacts: integration 23/23, prefork 1/1, lib 6/6; additions/removals are zero in each step.
- Logs report integration totals 3de 10,681 run / 23 failed / 19 skipped; dd 10,685 run / 23 failed / 19 skipped. Prefork each reports 2,320 passed / 1 failed / 2,321 total. Lib each reports 1,979 run / 1,973 passed / 6 failed / 0 skipped.
- Both prefork logs identify the sole failure as `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors`, with the same logged Lua error: `Blizzard_AdventuresCombatLog.lua:90`, `ipairs` received nil instead of a table.
- These are artifact comparisons only; no acceptance or causal claim is made.
