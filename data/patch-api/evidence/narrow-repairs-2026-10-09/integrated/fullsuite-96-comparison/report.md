# Saved fullsuite proof audit — 2026-10-09

## Scope and verdict

**FAIL.** Artifact revision `96e88494a6844833b79a574d37a2115dc03b44eb`. Started 2026-10-09T16:02:03-0500; finished 2026-10-09T16:54:27-0500. This is pre-host-restart evidence, not verification of any post-restart runtime or later revision. No tests, builds, reruns, network, delegation, services, environment/secret inspection, session filesystem reads, source/docs edits, commits, or cwd changes were performed. Writes are restricted to this requested persistent report directory. Verify skill and complete wiki index read.

## Concrete execution coverage

| Boundary | Run | Passed | Failed | Skipped | Exit | Evidence: current log lines |
|---|---:|---:|---:|---:|---:|---|
| Integration | 10,679 | 10,664 | 15 | 19 | 100 | 15, 11774 |
| Prefork full game UI | 2,321 | 2,320 | 1 | not reported | 1 aggregate | 11793, 14364 |
| Prefork exact chat fixture | 2 | 2 | 0 | not reported | no per-fixture status recorded | 14365–14369 |
| Prefork exact cast-bar fixture | 1 | 1 | 0 | not reported | no per-fixture status recorded | 14370–14373 |
| Prefork exact spellbook fixture | 1 | 1 | 0 | not reported | no per-fixture status recorded | 14374–14377 |
| Library | 1,978 | 1,977 | 1 | 0 | 100 | 14889, 16898 |

**Visible total: 14,982 case invocations; 14,965 passed; 17 failed, plus 19 skipped integration cases.** Prefork total is 2,325, not 2,321. All four original fixture environments ran despite the full-game-UI Garrison failure. Totals exclude hidden nested conformance/controlled subprocess invocations; these are not unique-test or native-client coverage counts. Each nextest FAIL is deduplicated by target + exact name, since summaries repeat FAIL rows. Log failure identities were independently parsed and checked against each saved JSON; all three agree.

## Fixed comparison anchors

| Saved revision | Integration pass/fail/run | Visible prefork pass/fail/run | Library pass/fail/run | End |
|---|---|---|---|---|
| `96e88494a6844833b79a574d37a2115dc03b44eb` | 10664/15/10679 | 2324/1/2325 | 1977/1/1978 | 2026-10-09T16:54:27-0500 |
| `3de87465828db7cc7f6f900d4b63e24f1b399825` | 10658/23/10681 | 2324/1/2325 | 1973/6/1979 | 2026-10-09T11:49:01-0500 |
| `dd710c6fc469a53e3b77b97aa6340993ae83e46e` | 10662/23/10685 | 2320/1/2321 | 1973/6/1979 | 2026-10-09T09:44:04-0500 |

Against **each** named baseline: 0 new exact failure identities; shared 15 integration + 1 prefork + 1 lib. Eight integration and five library failure identities disappeared. Disappearance is not passing: only five integration and three library identities have direct PASS rows; three integration and two lib identities have no outcome in this log. Garrison was already failing in BOTH baseline logs. Current JSON `new_failures.prefork` incorrectly labels it new relative to these anchors; that field and master-latest were not used as the comparison authority.

### Disappeared baseline failure identities (same for both baselines)

| Target | Exact identity | Current outcome |
|---|---|---|
| integration | `blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain` | PASS, log:1243 |
| integration | `blizzard_frame_xml_loads::blizzard_frame_xml_toc_is_load_first_with_current_dependencies` | PASS, log:1442 |
| integration | `blizzard_reforging_ui_loads::find_toc_file_resolves_classic_suffix_via_fallthrough` | ABSENT_FROM_LOG; no passing credit |
| integration | `c_api_surface::chat_info_no_state_defaults_are_not_c_api_temporary_shims` | ABSENT_FROM_LOG; no passing credit |
| integration | `c_api_surface::scenario_defaults_are_not_c_api_temporary_shims` | ABSENT_FROM_LOG; no passing credit |
| integration | `generated_data_refresh_coverage::generated_lua_refresh_manifest_matches_current_file_contents` | PASS, log:6016 |
| integration | `numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text` | PASS, log:7932 |
| integration | `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases` | PASS, log:7954 |
| lib | `loader::tests::wow_api_globals::transmog_situation::test_patch_12_0_0_transmog_situation_enum_values` | ABSENT_FROM_LOG; no passing credit |
| lib | `lua_api::workarounds::temporary::housing_catalog_state::tests::installs_seeded_housing_catalog_surface` | ABSENT_FROM_LOG; no passing credit |
| lib | `lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect` | PASS, log:16619 |
| lib | `lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons` | PASS, log:16626 |
| lib | `lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes` | PASS, log:16633 |

## Requested capability matrix

| Capability | Concrete proof | Limits / missing fields |
|---|---|---|
| Three controlled conformance cases | Exact revision registers `conformance::exact_group_selection`, `conformance::exact_group_failure`, `conformance::exact_group_timeout` in the 24-case conformance registry (tests/prefork_full_ui.rs:122–124). `run_full_ui_and_exact_fixtures` requires successful conformance before full-UI setup (:309–312); each exact fixture also requires it (:437–441). All four fixtures visibly produced outcomes. | **0 individual conformance PASS rows** retained. Successful conformance subprocess stdout/stderr is discarded (:472–484). Source-gated successful execution is supported for four conformance invocations, not independently retained per-case transcripts, PIDs, timings, or controlled counter dumps. |
| Controlled selection | Revision assertions cover each of 3 groups: one exact pass; skip and unmatched selection each give 4 zero-test batches and no setup/cache markers. | Contracts from revision source :888–936; numeric intermediate values not printed in successful saved log. |
| Controlled failure | Revision assertions inject failure in each of 3 groups, require child stdout/stderr markers, other groups passing, and 3 distinct setup PIDs. | Source :940–974; actual PID values and per-child stream transcripts discarded. |
| Controlled timeout | Each of 3 groups asserts 120ms timeout, both parent/descendant death, cache snapshot equality, fresh pass with different setup PID, cache equality again. | Source :978–1024; raw PID/cache snapshots not retained. No post-reboot process claims. |
| Typed formatter | Numeric formatter family **7/7 PASS**, log:7922–7932. Exact repaired duration binding PASS :7932. Revision test :139–179 checks 1.2→`2`, label `2`, 8.2→`9`, copied Down formatter→`8`, rejects a table custom formatter and preserves `8`. | Assertion values are tied to source plus PASS, not printed numeric output. No formatter invocation counter is present in this test or the log. No native parity credit. |
| ManagedAura dirty phases | OnUpdate family **5/5 PASS**, log:7945–7954. ManagedAura exact test PASS :7954. Revision :152–199 creates actual `CustomAuraContainerTemplate`; private ProcessDirtyFlags is a function; dirty/mode2 while hidden, still dirty/mode2 after hidden tick, clean/mode0 after visible tick, clean/mode0 after another tick; empty Lua error list asserted. | Three ticks (hidden/visible/extra); no ProcessDirtyFlags invocation counter or per-phase callback counter retained. Clean state does not itself measure invocation count. |
| Other OnUpdate counters | Passed tests assert ModeCalls for modes0..4 `[0,0,0,1,2]` after two hidden ticks then `[0,2,1,1,4]` after two visible ticks; rearm handler/hook counts both 2 for modes2 and3; XML RunOnce calls1 across two ticks. | Source assertions + respective PASS rows, not runtime diagnostic dumps; NOT ManagedAura-specific counters. |
| Exact three repaired EditMode tests | All **3/3 explicit PASS**: player size/cast side effect :16619; nil singleton minus-one :16626; compact refresh batching :16633. Exact identities above. | 0 missing of these three. Separate integration enum test still FAIL (22 vs21); passing these does not repair that boundary. |
| Generated manifest | Exact test PASS :6016; revision manifest has exactly 2 entries. Independent read-only Git-object byte/line/hash comparison matches both entries. | This proves revision-file manifest consistency; no upstream regeneration or native API correctness. |
| Tooltip diagnostic | FAIL :10712–10713: `LayoutRect { x: 0.0, y: 0.0, width: 416.7, height: 58.0 }`, viewport400×300, right416.7; right overflow **16.7**. | Exact measured assertion boundary; no inferred layout root cause or native-client reproduction. |

## Remaining exact failures and observed boundaries

| Target / exact identity | Observed failure (not inferred cause) | Log line |
|---|---|---|
| integration `blizzard_ui_blizzard_accountstore::behavior_currency_format::format_currency_display_returns_amount_space_texture_markup_with_size_twelve_twelve_zero_zero` | 1,234 texture markup vs expected 1234 texture markup | 2869 |
| integration `blizzard_ui_blizzard_achievementui::behavior_search_filter::search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length` | set_called=0 hide_called=2 update_called=0 show_called=0 vs expected hide_called=1 | 3005 |
| integration `c_spell_static_fallbacks::test_spell_static_fallback_shims_return_inert_values` | GetMawPowerBorderAtlasBySpellID is nil (Lua string:4) | 4916 |
| integration `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size` | enum metadata: left22 vs right21 | 5709 |
| integration `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | extra snapshot out of sync: {'remove_stale': 96, 'add_new': 491} | 6973 |
| integration `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | missing snapshot out of sync: {'remove_stale': 5, 'add_new': 16} | 7594 |
| integration `patch_12_0_7_b23_b28::b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries` | NotFound path under /home/osso-test/.cache/wow-ui-sim-audit; OS code2 | 8369 |
| integration `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_is_stubbed` | GetMawPowerBorderAtlasBySpellID is nil (Lua string:2) | 9375 |
| integration `spell_api::test_spell_get_spell_charges` | assertion failed: is_table | 9405 |
| integration `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | rect(0,0,416.7,58), viewport400x300, right416.7 | 10712 |
| integration `toplevel_render_groups::native_controls_keep_unraised_and_raised_groups_in_owner_strata` | Option::unwrap() on None at tests/toplevel_render_groups.rs:63:64 | 10790 |
| integration `toplevel_render_groups::screen_roots_do_not_capture_independent_render_groups` | Option::unwrap() on None at tests/toplevel_render_groups.rs:63:64 | 10821 |
| integration `utility_api::test_table_create_returns_empty_mutable_tables_for_capacity_variants` | bad argument #1 to create (number expected, got nil) | 11283 |
| integration `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles` | Lua assertion failed (string) | 11480 |
| integration `wowforever_table::wowforever_table_does_not_leak_into_earlier_profiles` | Lua RuntimeError message: count | 11522 |
| prefork `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | explicit Garrison load emitted non-3D-model Lua errors: ipairs(table expected, got nil), Blizzard_AdventuresCombatLog.lua:90; test assertion tests/blizzard_garrison_ui_loads.rs:286:5 | 12654–12656 |
| lib `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults` | debug environment defaults probe: `(string):7: attempt to call a userdata value`; source :240:14 | 16328–16329 |

Full observed panic diagnostics and method-diff entry lists are retained in `failure-diagnostics.json`; concrete identities, comparisons and log-line references in `data.json`.

## Manifest byte evidence

| Revision path | Bytes | Lines | SHA256 |
|---|---:|---:|---|
| `src/lua_api/globals/enum_data/missing_enums.lua` | 320252 | 16342 | `60a333e293bcc26af280e487e3b546231330e3cd9feb51c77aecb258dab3f3a6` |
| `src/lua_api/globals/enum_data/missing_constants.lua` | 135511 | 1420 | `d5fcc4b6a4d197bb9090cb6c996b022cb9807f8dc19b693e26d1a25111c402ff` |

## Provenance, warnings, and lost fields

Saved JSON supplies SHA/ref, start/end and aggregate step exits/durations. **The raw log has no explicit Git SHA/revision attestation, `pwd`, dirty-tree status or complete environment.** Source excerpts are obtained from the JSON-named Git revision, not from a claim that today’s master equals that run. Log compile paths (:11, :14387, :14885) name `/home/osso/Projects/wow/full-suite-checkout`; prefork failure (:14873) names its absolute target/debug binary. These establish reported source/binary paths, not an independently logged process cwd. No host identity/toolchain version/cache or vendor asset hashes are recorded.

Literal log argv headers:

```text
===== integration: /home/osso/.worktrees/build-lock.sh cargo nextest run --test integration --no-fail-fast --test-threads 16 --offline --locked (exit 100)
===== prefork: /home/osso/.worktrees/build-lock.sh cargo test --test prefork_full_ui --offline --locked (exit 1)
===== lib: /home/osso/.worktrees/build-lock.sh cargo nextest run --lib --no-fail-fast --test-threads 16 --offline --locked (exit 100)
```

No explicit --features/--no-default-features/profile selection is recorded in these headers; Nextest profile is `default` (:14, :14888). Do not infer a runtime native-client profile from that profile name.

Six distinct iced-wgpu manifest deprecations recur in all three Cargo phases: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. **18 individual warning emissions plus 3 “generated 6 warnings” summaries = 21 warning-prefixed lines**, not 21 distinct warnings. No non-vendor warning-prefixed diagnostic was found. This is not a standalone warning-clean compiler gate.

The saved log combines streams: prefork test outcomes/summaries precede build-lock/Cargo/startup diagnostics for that same step. Thus raw line order is not reliable wall-clock stdout/stderr ordering. Nextest failure child stdout/stderr labels remain useful, but global stream provenance/timestamps are lost. Header exits100/1/100 are step exits; hidden subcommand statuses are not independently logged. Step durations1477.6/821.7/844.7s include lock/build overhead, not pure runtime: Nextest integration621.279s and lib35.557s. Prefork runtime duration is not separately summarized.

Lost/unavailable: suppressed successful conformance transcripts and individual statuses/timings; controlled child PIDs, setup markers/cache snapshots; successful formatter values/callback counters; ManagedAura callback invocation counts; per-fixture exact process exits; global stream separation/order; raw cwd/revision cleanliness/host attestation; post-restart runtime and vanished /tmp-only evidence. Missing fields were not reconstructed by rerunning.

## Privacy and retention boundary

All three saved raw logs were read in full locally and scanned line-by-line for credential assignments, private-key markers, common token shapes, email addresses and HTTP(S) URLs: **0 candidate lines in each category for each log**. This bounded scan is not a guarantee against arbitrary secrets or personal/game content. Current raw log contains **24 lines with /home/ paths**, local usernames, temporary paths and process IDs; test diagnostics and Lua stacks reveal internal paths and synthetic fixture content. This report keeps necessary private/local diagnostic evidence and does not propose public raw-log retention. No raw logs were copied here. Main owns further privacy judgment, retention policy and commits.

## Read-only proof ledger / deliverables

Artifact reads + independent parsing: exactly the three requested saved JSON/log pairs; file SHA256s stored in data.json. Complete wiki index and verify skill read. Git `show <96...>:<path>` used only for revision-bound test sources, manifest and two generated Lua file objects, with `cli.git(...).cwd(/home/osso/Projects/wow/wow-ui-sim)`; every Git read exited0. No test executable or build command invoked by this audit. Static consistency results stored in manifest-static-check.json.

Report/data apply to the saved run only. Current master-latest, newer revisions, native parity, a root-cause diagnosis, current CI, clean build, deployment and post-reboot conformance remain unverified.
