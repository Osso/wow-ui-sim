# Saved full-suite comparison: 8e versus 96

**Overall: FAIL.** Complete saved streams audited privately; no tests/builds/reruns/operations.

## Counts and exits

| Revision | Step | Executed | Passed | Failed | Skipped | Exit | Seconds |
|---|---|---:|---:|---:|---:|---:|---:|
| 96e88494 | integration | 10679 | 10664 | 15 | 19 | 100 | 1477.6 |
| 96e88494 | prefork | 2325 | 2324 | 1 | 0 | 1 | 821.7 |
| 96e88494 | lib | 1978 | 1977 | 1 | 0 | 100 | 844.7 |
| 8e6bc113 | integration | 10679 | 10677 | 2 | 19 | 100 | 856.2 |
| 8e6bc113 | prefork | 2325 | 2324 | 1 | 0 | 1 | 90.9 |
| 8e6bc113 | lib | 1993 | 1993 | 0 | 0 | 0 | 102.3 |

Prefork totals include the 2321-case main group (2320 pass, 1 fail) plus auxiliary groups of 2, 1, and 1 cases, all passing, in both runs. Nextest skip counts are separate from executed counts. Every counted outcome is a unique executed selector; failure-summary repetitions excluded. Full counts reconcile with summaries and JSON exits/failure lists.
Total: 14982 executed / 17 failed → 14997 executed / 3 failed; 19 skipped each. Integration selection remains 10679; lib selection grows 1978 → 1993.

## Time/source/profile

| Source revision | Commit time | Run start | Run finish | Wall seconds |
|---|---|---|---|---:|
| `96e88494a6844833b79a574d37a2115dc03b44eb` | 2026-10-09T15:56:12-05:00 | 2026-10-09T16:02:03-0500 | 2026-10-09T16:54:27-0500 | 3144.0 |
| `8e6bc113f4c2c47ad693fa7fcb19ddf6bef48476` | 2026-10-09T21:48:55-05:00 | 2026-10-09T21:54:05-0500 | 2026-10-09T22:11:34-0500 | 1049.0 |

Offsets are -05:00; these are receipt timestamps, not audit timestamps. Sum of step durations: 96 = 3144.0s; 8e = 1049.4s. No individual test wall timestamps are present.

Observed build profile: `test [optimized + debuginfo]`. Commands omit feature overrides; both source manifests default to `sound,gui,casc,client-retail`, whose epoch is `retail-12-1-0` including `retail-12-0-7`. This is source/command inference, not environment/binary attestation. Evidence belongs to the source SHA above, never current HEAD, Mists, PTR, or full native parity.

## Original 17 failures

11 exact original selectors execute and pass with revised fixtures; 3 original selectors are absent from selection and have passing replacements; 3 exact original selectors still fail. No zero-selection credit. The 17→3 reduction is real for observed failure counts, not 14 unchanged assertions repaired.

| Step | Original selector | 8e evidence | Classification / fixture change |
|---|---|---|---|
| integration | `blizzard_ui_blizzard_accountstore::behavior_currency_format::format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero` | PASS at log:2841 | Expected enUS grouping changed from 1234 to 1,234; locale assertion added. Revised local fixture, not new native capture. |
| integration | `blizzard_ui_blizzard_achievementui::behavior_search_filter::search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length` | PASS at log:2947 | OnTextChanged binding temporarily cleared/restored to avoid double-driving handler after SetText. |
| integration | `c_spell_static_fallbacks::test_spell_static_fallback_shims_return_inert_values` | NOT SELECTED; replacement `c_spell_static_fallbacks::test_spell_override_maw_epoch_and_explicit_charge_state` PASS at log:4828 | Renamed/replaced inert shim expectation with seeded charge-state values and retirement-aware Maw surface query. |
| integration | `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size` | PASS at log:5591 | Enum fixture now checks BuffIconSize=22, MaxValue=22, NumValues=23. |
| integration | `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | FAIL at log:6878 | Still FAIL; no repair credit. |
| integration | `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | FAIL at log:6822 | Still FAIL; no repair credit. |
| integration | `patch_12_0_7_b23_b28::b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries` | PASS at log:8218 | Physical-file fixture uses owned tempdir instead of externally provisioned osso-test audit directory. |
| integration | `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed` | NOT SELECTED; replacement `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_follows_retirement_epoch` PASS at log:9194 | Replaced callable-stub expectation with epoch-aware member absence at retail-12-0-7 and later. |
| integration | `spell_api::test_spell_get_spell_charges` | PASS at log:9198 | Charge fixture explicitly seeds spell 19750 before asserting a table return. |
| integration | `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | PASS at log:10466 | Tooltip explicitly shown before glyph-backed sizing/clamp assertion. |
| integration | `toplevel_render_groups::native_controls_keep_unraised_and_raised_groups_in_owner_strata` | PASS at log:10517 | Tooltip explicitly shown; visibility/order diagnostics strengthened. Model layering fixture, not new native capture. |
| integration | `toplevel_render_groups::screen_roots_do_not_capture_independent_render_groups` | PASS at log:10520 | Tooltip explicitly shown; visibility/order diagnostics strengthened. Model layering fixture, not new native capture. |
| integration | `utility_api::test_table_create_returns_empty_mutable_tables_for_capacity_variants` | PASS at log:10954 | Missing table.create hint rejection checked; valid zero-hint case uses table.create(0). |
| integration | `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles` | NOT SELECTED; replacement `wowforever_cooldown_categories::forever_cooldown_categories_preserve_retail_12_1_0_epoch` PASS at log:11122 | Original remains under a non-retail-12-1-0 cfg; new retail epoch fixture checks categories 0..8 and metadata. |
| integration | `wowforever_table::wowforever_table_does_not_leak_into_earlier_profiles` | PASS at log:11133 | table.count checked against retail three-value contract instead of universal absence. |
| prefork | `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | FAIL at log:12032 | Still FAIL; no repair credit. |
| lib | `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults` | PASS at log:15903 | Debug fixture unwraps secret value before invoking, checks identity/secrecy; tracing added. |

Source comparison uses Git snapshots 96 and 8e, not working-tree contents. All 11 same-selector passes have fixture/input/assertion changes; none is presented as an unchanged-contract runtime repair. The three replacements execute, but do not prove their retired/original expectations. The cooldown original remains compiled only outside the retail-12-1-0 epoch.

## Remaining failures

| Selector | 8e failure source | Exit |
|---|---|---:|
| `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | `tests/method_diff_coverage.rs:264:5` | 100 |
| `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | `tests/method_diff_coverage.rs:264:5` | 100 |
| `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | `tests/blizzard_garrison_ui_loads.rs:286:5` | 1 |

Method snapshot mismatches and explicit Garrison loading failure persist versus 96. Lib exits 0, but integration exits 100 and prefork exits 1: full-suite FAIL. No raw mismatch lists, Lua errors, or failure payloads reproduced.

## Baseline semantics and integrity

`new_failures` subtracts `master-latest.json` at runner execution; baseline identity/content is not archived in these receipts. It does **not** mean newly failing relative to 96. Garrison is listed in `new_failures` in both runs and fails in both. Explicit selector subtraction versus 96 finds zero new failures.

96 logged commands include `/home/osso/.worktrees/build-lock.sh` and `--offline --locked`; 8e logged commands omit them. Test-thread limit is 16 in both nextest steps. Tracked runner snapshots are identical and include the wrapper; current installed runner differs and matches 8e command shape. This is a runner-provenance discrepancy, not permission to fix/redeploy. No historical installed-script or environment attestation exists.

Both full logs and JSON files were read completely. Counts, unique selectors, summaries, failure lists, exits, and timestamp arithmetic reconcile. Hash manifest records exact receipt bytes, source snapshot hashes, and output hashes. Hashing now does not establish historical immutability, exact build host/compiler/cache, or executable linkage. Runner concatenates stdout then stderr; stream ordering is not event chronology.

`aggregate.json` contains exact commands, group counts, selected-set deltas, original/replacement selectors, evidence line numbers and qualifications. `hashmanifest.json` intentionally omits its own hash to avoid circularity.
