# Independent saved full-suite comparison — FAIL

**Parent full-suite remains FAIL. No completion or branch-ready claim.**

Read `/home/osso/AgentConfig/skills/verify/SKILL.md`. Independent verifier; read-only inspection of entire saved logs/JSON. No tests/checks/fullsuite rerun, source edits, delegation, cwd switching, push or deployment. Only `/tmp` reports/scripts written. Git invocations use explicit canonical cwd `/home/osso/Projects/wow/wow-ui-sim`.

## Exact run context

| Step | dd710c6 | 614402d56 | Failure identity comparison |
|---|---|---|---|
| Integration | 10,685 run; 10,662 passed; 23 failed; 19 skipped; exit 100 | 10,673 run; 10,650 passed; 23 failed; 19 skipped; exit 100 | Identical 23, no added/removed |
| Prefork | 2,321 run; 2,320 passed; 1 failed; exit 1 | 2,319 run; 2,318 passed; 1 failed; exit 1 | Same Garrison identity |
| Library | 1,979 run; 1,973 passed; 6 failed; exit 100 | Same counts and exit | Identical six, no added/removed |

Saved current revision `dd710c6fc469a53e3b77b97aa6340993ae83e46e`: 2026-10-09 09:30:26–09:44:04 -0500. Baseline `614402d56f726ec34cedea2f28a9609cdd653462`: 2026-10-09 07:59:55–08:15:11 -0500. Runs compile under `/home/osso/Projects/wow/full-suite-checkout`; canonical HEAD at inspection is `bf0e8372057bfdcd3751c9cb62a371fb00ac4b7c`. The saved run is not execution at the later canonical HEAD.

All 30 failure sites match file:line:column. **29/30 assertion payloads byte-equal**. The remaining b28 failure differs only random missing-directory suffix: baseline `b28-nTcI6M`, dd `b28-k4nk10`; same `NotFound`, OS code 2, and boundary `tests/patch_12_0_7_b23_b28.rs:348:10`. Only that basename is normalized in comparison JSON; all 30 normalized boundaries match. Raw payloads retained. Prefork Garrison failure is the `thread main` panic, mapped to its sole failed case and `tests/blizzard_garrison_ui_loads.rs:286:5`; non-3D filter rejects `ipairs(nil)` at `Blizzard_AdventuresCombatLog.lua:90`. No diagnosis/root-cause repair inferred from equal assertions.

## Runner new_failures is not the requested baseline comparison

`dd` and `614` both label Garrison under `new_failures.prefork`; both already fail it. Direct comparison to `614` has **zero newly failing identities in every step**. Observed `master-latest.json` identifies older `511397d02d762f8e4f7185cbcacabc548bf027b1` (October 8), has no prefork failures, and lists the same Garrison identity under integration. Per-step comparison to that older bucket could produce the runner flag. Exact runner baseline selection is **unproven**: neither run JSON declares a comparison-baseline SHA, and installed runner/source was unavailable. Do not report Garrison as a dd regression against 614.

## Executed coverage, not inferred targets

**New since 614: 12 integration cases, all PASS** (p242 2, p302 7, p303 3):

- `patch_2_4_2_factory::patch_2_4_2_current_currency_separator_model` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8725`
- `patch_2_4_2_factory::patch_2_4_2_factory_publication` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8727`
- `patch_3_0_2_factory::bare_framexml_and_tooltip_handler_have_precise_limits` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8726`
- `patch_3_0_2_factory::click_modifier_default_does_not_discriminate_unknown_actions` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8728`
- `patch_3_0_2_factory::console_catalog_distinguishes_cvar_records_from_commands` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8729`
- `patch_3_0_2_factory::factory_rejects_classic_before_reading_cache` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8730`
- `patch_3_0_2_factory::literal_register_keeps_all_373_original_occurrences` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8719`
- `patch_3_0_2_factory::patch_3_0_2_exact_factory_publication_gaps` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8739`
- `patch_3_0_2_factory::retail_event_registration_rejects_unknown_but_not_source_events` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8731`
- `patch_3_0_3_factory::exact_three_factory_values_and_defaults` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8732`
- `patch_3_0_3_factory::fabricated_unknown_cvar_is_not_published` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8736`
- `patch_3_0_3_factory::patch_3_0_3_exact_factory_publication_gaps` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:8734`

**New since 614: two prefork cases, both PASS:**
- `patch_3_0_8_publication_sweep::patch_3_0_8_hook_without_script` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:14420`
- `patch_3_0_8_publication_sweep::patch_3_0_8_publication_sweep` — `dd710c6fc469a53e3b77b97aa6340993ae83e46e.log:14421`

Existing pre-login named target `patch_12_1_0_aura_access_restrictions::pre_login_buttons_stay_accessible_through_player_login` is PASS in both logs; dd line 8665. This is the AuraButton loaded-UI test, **not** Forever configured-name SOURCE model or proof of actual native pre-login lifecycle.

Selected-name inventories retain 85 integration, 94 prefork, 10 library records with exact names, PASS/FAIL and log lines in JSON. These substring-selected counts are not a claim that all are publication sweeps. Prefork has 78 records from `*_publication_sweep` modules: 75 sweep targets plus three helper cases (10.0.0 animation factories, 3.0.8 hook, 3.1.0 player-facing). All PASS. Their exact executed sweep target identities follow:

| Executed prefork sweep | Log line |
|---|---|
| `patch_10_0_2_publication_sweep::patch_10_0_2_publication_sweep` | 14374 |
| `patch_10_0_0_publication_sweep::patch_10_0_0_publication_sweep` | 14376 |
| `patch_10_0_5_publication_sweep::patch_10_0_5_publication_sweep` | 14378 |
| `patch_10_0_7_publication_sweep::patch_10_0_7_publication_sweep` | 14380 |
| `patch_10_1_0_publication_sweep::patch_10_1_0_publication_sweep` | 14382 |
| `patch_10_1_5_publication_sweep::patch_10_1_5_publication_sweep` | 14384 |
| `patch_10_1_7_publication_sweep::patch_10_1_7_publication_sweep` | 14386 |
| `patch_10_2_0_publication_sweep::patch_10_2_0_publication_sweep` | 14388 |
| `patch_10_2_5_publication_sweep::patch_10_2_5_publication_sweep` | 14390 |
| `patch_10_2_6_publication_sweep::patch_10_2_6_publication_sweep` | 14392 |
| `patch_10_2_7_publication_sweep::patch_10_2_7_publication_sweep` | 14393 |
| `patch_11_0_2_publication_sweep::patch_11_0_2_publication_sweep` | 14396 |
| `patch_11_0_5_publication_sweep::patch_11_0_5_publication_sweep` | 14398 |
| `patch_11_0_0_publication_sweep::patch_11_0_0_publication_sweep` | 14399 |
| `patch_11_0_7_publication_sweep::patch_11_0_7_publication_sweep` | 14401 |
| `patch_11_1_5_publication_sweep::patch_11_1_5_publication_sweep` | 14402 |
| `patch_11_1_0_publication_sweep::patch_11_1_0_publication_sweep` | 14403 |
| `patch_11_1_7_publication_sweep::patch_11_1_7_publication_sweep` | 14404 |
| `patch_11_2_0_publication_sweep::patch_11_2_0_publication_sweep` | 14405 |
| `patch_11_2_5_publication_sweep::patch_11_2_5_publication_sweep` | 14406 |
| `patch_11_2_7_publication_sweep::patch_11_2_7_publication_sweep` | 14407 |
| `patch_12_0_1_publication_sweep::patch_12_0_1_publication_sweep` | 14411 |
| `patch_12_0_0_publication_sweep::patch_12_0_0_publication_sweep` | 14413 |
| `patch_12_0_5_publication_sweep::patch_12_0_5_publication_sweep` | 14414 |
| `patch_12_0_7_publication_sweep::patch_12_0_7_publication_sweep` | 14419 |
| `patch_3_0_8_publication_sweep::patch_3_0_8_publication_sweep` | 14421 |
| `patch_3_1_0_publication_sweep::patch_3_1_0_publication_sweep` | 14423 |
| `patch_12_1_0_publication_sweep::patch_12_1_0_publication_sweep` | 14424 |
| `patch_3_2_0_publication_sweep::patch_3_2_0_publication_sweep` | 14425 |
| `patch_3_3_0_publication_sweep::patch_3_3_0_publication_sweep` | 14427 |
| `patch_3_3_3_publication_sweep::patch_3_3_3_publication_sweep` | 14428 |
| `patch_3_3_5_publication_sweep::patch_3_3_5_publication_sweep` | 14429 |
| `patch_4_1_0_publication_sweep::patch_4_1_0_publication_sweep` | 14431 |
| `patch_4_0_1_publication_sweep::patch_4_0_1_publication_sweep` | 14432 |
| `patch_4_2_0_publication_sweep::patch_4_2_0_publication_sweep` | 14433 |
| `patch_4_3_0_publication_sweep::patch_4_3_0_publication_sweep` | 14434 |
| `patch_4_3_4_publication_sweep::patch_4_3_4_publication_sweep` | 14435 |
| `patch_5_0_1_publication_sweep::patch_5_0_1_publication_sweep` | 14437 |
| `patch_5_1_0_publication_sweep::patch_5_1_0_publication_sweep` | 14440 |
| `patch_5_0_4_publication_sweep::patch_5_0_4_publication_sweep` | 14442 |
| `patch_5_2_0_publication_sweep::patch_5_2_0_publication_sweep` | 14444 |
| `patch_5_3_0_publication_sweep::patch_5_3_0_publication_sweep` | 14447 |
| `patch_5_4_0_publication_sweep::patch_5_4_0_publication_sweep` | 14449 |
| `patch_5_4_1_publication_sweep::patch_5_4_1_publication_sweep` | 14452 |
| `patch_5_4_7_publication_sweep::patch_5_4_7_publication_sweep` | 14454 |
| `patch_5_4_2_publication_sweep::patch_5_4_2_publication_sweep` | 14455 |
| `patch_5_4_8_publication_sweep::patch_5_4_8_publication_sweep` | 14456 |
| `patch_6_0_1_publication_sweep::patch_6_0_1_publication_sweep` | 14457 |
| `patch_6_1_0_publication_sweep::patch_6_1_0_publication_sweep` | 14461 |
| `patch_6_2_0_publication_sweep::patch_6_2_0_publication_sweep` | 14464 |
| `patch_6_2_2_publication_sweep::patch_6_2_2_publication_sweep` | 14465 |
| `patch_6_0_2_publication_sweep::patch_6_0_2_publication_sweep` | 14466 |
| `patch_7_0_1_publication_sweep::patch_7_0_1_publication_sweep` | 14468 |
| `patch_6_2_4_publication_sweep::patch_6_2_4_publication_sweep` | 14470 |
| `patch_7_1_0_publication_sweep::patch_7_1_0_publication_sweep` | 14474 |
| `patch_7_0_3_publication_sweep::patch_7_0_3_publication_sweep` | 14477 |
| `patch_7_2_0_publication_sweep::patch_7_2_0_publication_sweep` | 14483 |
| `patch_7_2_5_publication_sweep::patch_7_2_5_publication_sweep` | 14486 |
| `patch_7_3_0_publication_sweep::patch_7_3_0_publication_sweep` | 14487 |
| `patch_7_3_2_publication_sweep::patch_7_3_2_publication_sweep` | 14488 |
| `patch_8_1_0_publication_sweep::patch_8_1_0_publication_sweep` | 14495 |
| `patch_8_0_1_publication_sweep::patch_8_0_1_publication_sweep` | 14496 |
| `patch_8_1_5_publication_sweep::patch_8_1_5_publication_sweep` | 14499 |
| `patch_8_2_0_publication_sweep::patch_8_2_0_publication_sweep` | 14501 |
| `patch_8_2_5_publication_sweep::patch_8_2_5_publication_sweep` | 14503 |
| `patch_8_3_7_publication_sweep::patch_8_3_7_publication_sweep` | 14504 |
| `patch_8_3_0_publication_sweep::patch_8_3_0_publication_sweep` | 14505 |
| `patch_9_0_2_publication_sweep::patch_9_0_2_publication_sweep` | 14507 |
| `patch_9_0_5_publication_sweep::patch_9_0_5_publication_sweep` | 14510 |
| `patch_9_0_1_publication_sweep::patch_9_0_1_publication_sweep` | 14512 |
| `patch_9_1_0_publication_sweep::patch_9_1_0_publication_sweep` | 14514 |
| `patch_9_2_0_publication_sweep::patch_9_2_0_publication_sweep` | 14515 |
| `patch_9_1_5_publication_sweep::patch_9_1_5_publication_sweep` | 14516 |
| `patch_9_2_7_publication_sweep::patch_9_2_7_publication_sweep` | 14519 |
| `patch_9_2_5_publication_sweep::patch_9_2_5_publication_sweep` | 14521 |

**Excluded/unproven:** `patch_3_4_1_factory` and `patch_3_4_2_factory` have zero executed log records; Wrath-profile content is not credited to Retail. `patch_1_60_1_source_model` also has zero records and requires `client-wowforever`; its separate receipt is not this fullsuite. Commands select `--test integration`, `--test prefork_full_ui`, `--lib`; standalone target registration does not prove standalone execution. Default aggregate skips 19 tests; no skip execution credit. All executed record identities, including the preexisting suite, are preserved in JSON.

## Scope through 0b64e636c and Retail gate

`dd..0b64e636c`: **1,398 changed paths: 1,364 under `data/patch-api/`, 34 docs**. No `src`, `tests`, `patch-tests`, Cargo manifest/lock, build.rs/build or scripts changes. These are source-audit data/fixtures, source-audit/evidence tooling/receipts, and docs; **no new simulator runtime edits**. Exact name/status inventory and commit log retained in JSON. Tracked working-tree/staged diff empty; unrelated untracked `.code-index.db` present. No fullsuite re-execution at latest HEAD is claimed.

`1044215d0386dd809c51cb8f7b47e08bbf4016a1` adds exactly ten `#[cfg(feature = "retail-12-0-0")]` attributes to existing import/constants/functions in `src/c_api/addon_messages.rs` and `src/c_api/c_combat_log.rs`; function bodies unchanged. Retail feature chain makes every added predicate true. Earlier module predicate broadening retains Retail availability. Thus **enabled Retail bodies unchanged by those gates**; non-Retail body compilation is the changed boundary. `1044215..dd` differs only Cargo.toml in runtime/build/test scope (adds explicit Forever source-model test target); both gated source files remain hash-identical through dd and 0b64. Do not generalize this narrow static equivalence to all-profile behavior or all changes in the branch.

## Exact failure identities and assertion sites

Full raw assertion payloads (including left/right snapshots and Lua traceback text), byte hashes and both source/log sites live in comparison JSON. Compact index:

| Step / identity | Assertion boundary |
|---|---|
| integration: `blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain` | `tests/blizzard_core_frame_lane.rs:159:5` — assertion `left == right` failed: FrameXML must preserve all 17 current dependencies in retail TOC order, including Blizzard_UIParentPanelManager but no direct  |
| integration: `blizzard_frame_xml_loads::blizzard_frame_xml_toc_is_load_first_with_current_dependencies` | `tests/blizzard_frame_xml_loads.rs:64:5` — assertion `left == right` failed: Blizzard_FrameXML declares its current 17 dependencies in TOC order |
| integration: `blizzard_reforging_ui_loads::find_toc_file_resolves_classic_suffix_via_fallthrough` | `tests/blizzard_reforging_ui_loads.rs:56:44` — Blizzard_ReforgingUI TOC should resolve |
| integration: `blizzard_ui_blizzard_accountstore::behavior_currency_format::format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero` | `tests/blizzard_ui/blizzard_accountstore/behavior_currency_format.rs:206:9` — assertion `left == right` failed: Expected the literal output `1234 \|T9999777:12:12:0:0\|t` for amount=1234, icon=9999777. The body at line 62 returns `BreakUpLa |
| integration: `blizzard_ui_blizzard_achievementui::behavior_search_filter::search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length` | `tests/blizzard_ui/blizzard_achievementui/behavior_search_filter.rs:174:9` — assertion `left == right` failed: Expected below-threshold drive (text=`ab`, 2 chars < MIN_CHARACTER_SEARCH=3) to produce signature `set_called=0 hide_called=1  |
| integration: `c_api_surface::chat_info_no_state_defaults_are_not_c_api_temporary_shims` | `tests/c_api_surface.rs:602:5` — C_ChatInfo no-state defaults should not be wired through c_api registration |
| integration: `c_api_surface::scenario_defaults_are_not_c_api_temporary_shims` | `tests/c_api_surface.rs:855:5` — C_Scenario not-in-scenario defaults should not be wired through c_api registration |
| integration: `c_spell_static_fallbacks::test_spell_static_fallback_shims_return_inert_values` | `tests/c_spell_static_fallbacks.rs:25:10` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "(string):4: attempt to call field 'GetMawPowerBorderAtlasBySpellID' (a nil val |
| integration: `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size` | `tests/edit_mode_api/enums.rs:44:5` — assertion `left == right` failed |
| integration: `generated_data_refresh_coverage::generated_lua_refresh_manifest_matches_current_file_contents` | `tests/generated_data_refresh_coverage.rs:68:9` — assertion `left == right` failed: byte count changed for src/lua_api/globals/enum_data/missing_constants.lua; update the refresh manifest in the same change |
| integration: `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | `tests/method_diff_coverage.rs:264:5` — diff_methods_extra.txt is out of sync with the current metatable surface. |
| integration: `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | `tests/method_diff_coverage.rs:264:5` — diff_methods_missing.txt is out of sync with the current metatable surface. |
| integration: `numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text` | `tests/numeric_rule_formatter.rs:181:6` — native formatter feeds the duration binding consumer: Lua(Runtime(RuntimeError { message: "expected LuaDurationObject at argument 1", level: 0, traceback: [] }) |
| integration: `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases` | `tests/on_update_modes.rs:186:6` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] })) |
| integration: `patch_12_0_7_b23_b28::b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries` | `tests/patch_12_0_7_b23_b28.rs:348:10` — called `Result::unwrap()` on an `Err` value: Custom { kind: NotFound, error: PathError { path: "/home/osso-test/.cache/wow-ui-sim-audit/b28-k4nk10", err: Os { c |
| integration: `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed` | `tests/spell_api.rs:474:10` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "(string):2: attempt to call field 'GetMawPowerBorderAtlasBySpellID' (a nil val |
| integration: `spell_api::test_spell_get_spell_charges` | `tests/spell_api.rs:375:5` — assertion failed: is_table |
| integration: `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | `tests/tooltip_text_layout.rs:180:5` — assertion failed: rect.x + rect.width <= state.screen_width + 0.1 |
| integration: `toplevel_render_groups::native_controls_keep_unraised_and_raised_groups_in_owner_strata` | `tests/toplevel_render_groups.rs:63:64` — called `Option::unwrap()` on a `None` value |
| integration: `toplevel_render_groups::screen_roots_do_not_capture_independent_render_groups` | `tests/toplevel_render_groups.rs:63:64` — called `Option::unwrap()` on a `None` value |
| integration: `utility_api::test_table_create_returns_empty_mutable_tables_for_capacity_variants` | `tests/utility_api.rs:286:10` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "bad argument #1 to 'create' (number expected, got nil)", level: 0, traceback:  |
| integration: `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles` | `tests/wowforever_cooldown_categories.rs:58:6` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] })) |
| integration: `wowforever_table::wowforever_table_does_not_leak_into_earlier_profiles` | `tests/wowforever_table.rs:47:6` — called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "count", level: 0, traceback: [] })) |
| prefork: `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | `tests/blizzard_garrison_ui_loads.rs:286:5` — Blizzard_GarrisonUI emitted non-3D-model Lua errors during explicit load: |
| lib: `loader::tests::wow_api_globals::transmog_situation::test_patch_12_0_0_transmog_situation_enum_values` | `src/loader/tests/wow_api_globals/transmog_situation.rs:76:5` — assertion `left == right` failed: TransmogSituation did not match the 12.0.0 source register |
| lib: `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults` | `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:240:14` — debug environment defaults probe should run: Lua(Runtime(RuntimeError { message: "(string):7: attempt to call a userdata value", level: 0, traceback: [] })) |
| lib: `lua_api::workarounds::temporary::housing_catalog_state::tests::installs_seeded_housing_catalog_surface` | `src/lua_api/workarounds/temporary/housing_catalog_state.rs:60:9` — assertion `left == right` failed |
| lib: `lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect` | `src/lua_api/workarounds_editmode_tests/apply_system_anchors/cast_and_player.rs:380:10` — apply system anchors should avoid player-frame cast-bar side effects: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors |
| lib: `lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons` | `src/lua_api/workarounds_editmode_tests/apply_system_anchors/singletons.rs:193:10` — apply nil-index fallback singleton anchors: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors' (a nil value)", level: 0 |
| lib: `lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes` | `src/lua_api/workarounds_editmode_tests/apply_system_anchors/unit_frames.rs:437:10` — apply compact unit frame batched settings: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors' (a nil value)", level: 0, |

## Artifact integrity

| Input | SHA-256 |
|---|---|
| `/home/osso/Projects/wow/full-suite-results/dd710c6fc469a53e3b77b97aa6340993ae83e46e.json` | `504ecf16ebea68ba16416b329196118203bc5623f2451297564b9dbd48ea5b5a` |
| `/home/osso/Projects/wow/full-suite-results/dd710c6fc469a53e3b77b97aa6340993ae83e46e.log` | `0ac4dfd984729ace754fbce0fdf4749c7193ff197cda008bbd4655dfe8ea3028` |
| `/home/osso/Projects/wow/full-suite-results/614402d56f726ec34cedea2f28a9609cdd653462.json` | `ce013458b822e883a0442e495186cfa37d7ef0f78ba6d8cce95774af334248b8` |
| `/home/osso/Projects/wow/full-suite-results/614402d56f726ec34cedea2f28a9609cdd653462.log` | `a3efd6c27231a0b93c2b24064db588278f02a0e7aa5bf31a289969a0c2871bb3` |
| `/home/osso/Projects/wow/full-suite-results/master-latest.json` | `23f12db18abd0b58fdf15aaaf3c2104a2fa2d18c48d084e6e72ead14d4165e78` |

Exact comparison: `/tmp/fullsuite-dd-independent-comparison.json` (9999646 bytes), SHA-256 `15af78818ba8398aa107e1644128d715d0501ddc58dd3b3980606cc08cb130a6`. It includes file hashes, revision/blob context, all recorded execution identities, changed-path inventory, full failure assertion payloads, normalization rule and actual log-line citations. Input hashes rechecked unchanged; actual failed execution records equal JSON failure arrays across all six runs/steps.

**OVERALL: FAIL — existing 23 integration + 1 Garrison prefork + 6 library failures remain. Baseline parity and bounded passing targets are not completion.**
