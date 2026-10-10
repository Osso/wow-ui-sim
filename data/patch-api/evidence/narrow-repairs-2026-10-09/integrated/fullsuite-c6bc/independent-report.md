> Historical independent audit, sanitized for retention. Its retirement lookup is incorrect as an answer to the requested case; see `main-erratum.md`. Preserved for audit history, not endorsed.

# Independent saved-receipt gate: c6bc versus bc58

Verified 2026-10-10. **OVERALL FAIL: three unchanged failing selectors.** Comparison PASS for no added failing identity; XML cases 3/3 PASS within the saved library scope. These are distinct conclusions, not a clean-suite acceptance.

Read `[private verifier instructions]` as assigned independent verifier. No reviewer, delegation, tests, builds, polling, reruns, operational changes, source edits or commits. Only this private report written. Read-only Git queries qualify revision membership; Python parses saved artifacts, not executable tests.

## Frozen evidence and actual steps

Receipt/log roots: `[private saved-receipt root]/`.

- Current: `c6bc87c120e43199dadeb299691511b03a81408a.{json,log}`, October 10, 2026 11:36:26–11:59:56 -0500, 23m30s wall; step sum 1410.4s.
- Prior: `bc58b43b577f1ac6647d4765d50e43afa36302ca.{json,log}`, October 10, 2026 03:38:51–04:05:27 -0500, 26m36s wall; step sum 1595.8s.
- Prior independent report: `[private verification root]/fullsuite-bc58-independent/report.md`.

Both complete logs were consumed and all named execution records compared, not sampled. c6bc: 16,478 lines / 2,096,205 bytes; bc58: 16,490 lines / 2,097,902 bytes. Repeated nextest failure summaries are not extra tests. Prefork multiline failure records were included in comparison.

| Receipt | Scope | Executed | Passed | Failed | Skipped | Exit | Step seconds |
|---|---|---:|---:|---:|---:|---:|---:|
| bc58 | integration | 10681 | 10679 | 2 | 19 | 100 | 1275.5 |
| c6bc | integration | 10681 | 10679 | 2 | 19 | 100 | 1103.6 |
| bc58 | prefork | 2325 | 2324 | 1 | 0 reported | 1 | 102.5 |
| c6bc | prefork | 2325 | 2324 | 1 | 0 reported | 1 | 166.5 |
| bc58 | lib | 1993 | 1993 | 0 | 0 | 0 | 217.8 |
| c6bc | lib | 1996 | 1996 | 0 | 0 | 0 | 140.3 |

Totals: c6bc **15002 executed /14999 passed /3 failed /19 skipped**; bc58 **14999 executed /14996 passed /3 failed /19 skipped**. Prefork main group is 2321=2320 PASS+1 FAIL; auxiliary groups 2+1+1 all PASS. Do not report main group alone as total prefork scope.

c6bc log evidence: integration summary line 11381; prefork summaries 13958,13963,13967,13971; lib summary 16477. Prior equivalents: 11396; 13973,13978,13982,13986; 16489.

Actual commands, identical between receipts:

1. `cargo nextest run --test integration --no-fail-fast --test-threads 16 --offline --locked`
2. `cargo test --test prefork_full_ui --offline --locked`
3. `cargo nextest run --lib --no-fail-fast --test-threads 16 --offline --locked`

Current step headers lines 1,11386,14467 record exits 100,1,0. Actual failure records and terminal summaries corroborate them; prefork process exit 1 is explicit at line 14465. Successful test-profile compilation records are lines 10,13980,14476, respectively 1m51s,38.41s,1m08s. Compilation success does not override execution failures. This is actual saved execution evidence, not queue/controller/submission success. No fmt/check command is part of this full-suite log.

## Failure identity comparison

**Zero newly failing identities, zero resolved failures, zero removals, zero shared-selector status changes across all three scopes.** JSON failure sets agree exactly with parsed actual log failures.

| Exact failing selector | c6bc actual failure line | bc58 actual failure line | Assertion boundary |
|---|---:|---:|---|
| `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | 6819 | 6885 | `tests/method_diff_coverage.rs:264:5` |
| `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | 7440 | 6828 | `tests/method_diff_coverage.rs:264:5` |
| `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | 12031 | 12046 | `tests/blizzard_garrison_ui_loads.rs:286:5` |

Garrison reports non-3D-model Lua errors during explicit addon load. No root-cause diagnosis or baseline repair inferred. Both JSON receipts label Garrison under `new_failures`; this runner-relative metadata is **not** a new failure versus bc58. Actual comparison establishes it already failed there.

## XML coverage and source boundary

Read-only ancestry checks establish `33d62d705`, `2bf8766ef`, and `da944ee94` are ancestors of c6bc (exit 0). `9d074f6a8` and `84dcf2723` are not (exit 1). c6bc itself is a documentation commit following XML production commit `33d62d705`.

The only three added execution identities versus bc58 are library cases, each actual PASS. Prefix: `iced_app::mouse::mouse_test_modules::registration_tests::`.

| Added test suffix | c6bc log line | Result |
|---|---:|---|
| `lua_create_frame_applies_xml_template_click_edges_before_onload_and_dispatch` | 14683 | PASS |
| `xml_click_templates_replace_inherited_edges_before_onload_and_physical_dispatch` | 14689 | PASS |
| `xml_register_for_clicks_applies_literal_edges_preserves_default_and_allows_lua_mutation` | 14690 | PASS |

Integration and prefork identity sets are unchanged. c6bc includes shared XML production and these new cases. **It does not cover the later real TOC private-table test at `9d074f6a8`, or cast-count production at `84dcf2723`.** Same pre-existing selector name/status cannot transfer proof to a subsequently changed implementation/assertion.

Separate XML selected-run reports were consulted for interpretation, not credited as additional full-suite execution. Their selected nine-case module run and fmt/check receipts are separate proof scopes. The full-suite's +3 library delta is not a nine-test addition.

## Original cached retirement lookup

Actual original cached-startup retirement case present:

`test patch_12_0_7_removed_native_surface::minimap_removals_survive_full_cached_game_ui_startup ... ok`

**c6bc prefork line 13811: PASS; bc58 prefork line 13826: PASS.** Source at the frozen c6bc revision declares this through `prefork_full_ui_case!`; it checks removed Minimap methods after full cached startup. This is the original prefork case, not its similarly named isolated cached-deprecation-load test, not orchestrator success, and not native WoW proof.

Also explicitly present: `patch_12_0_1_secure_delegate::patch_12_0_1_cached_secure_delegate_removed`, PASS at c6bc 13808 / bc58 13823. No execution selector literally containing `cached21`, `b21`, or `patch_2_1` appears in either saved stream. Therefore the compressed label “cached21” is not independently resolved to a different exact case; the exact recorded names above preserve the distinction rather than inventing a mapping.

## Warnings, skips and limits

Each scope in each receipt emits the same six iced-wgpu manifest deprecations: `large-enum-variant`, `map-entry`, `match-wildcard-for-single-variants`, `redundant-closure-for-method-calls`, `trivially-copy-pass-by-ref`, `type-complexity`. Each also emits a `generated 6 warnings` summary, not a seventh issue. **18 detail emissions per receipt / six distinct issues**, unchanged between receipts. Not warning-free acceptance; warnings were neither suppressed nor repaired.

19 integration skips are recorded as aggregates; exact skipped identities are unavailable in these streams. Prefork aggregate summaries expose no skipped count; all 2325 named outcomes reconcile with their totals. Startup messages such as `Skipping EDIT_MODE_LAYOUTS_UPDATED` are runtime event decisions, not additional test skips. Nextest `default` denotes its configuration profile, not independently attested client-profile coverage.

Receipt SHA is declared source identity. Logs do not independently seal historical source-to-binary provenance, external path dependencies, inherited environment, caches/addons/SavedVariables/CASC/listfiles or deployed state. Concatenated stdout/stderr is not strict event chronology. Equal named outcomes do not prove equal fixtures/assertions. No current-HEAD PASS, all-profile acceptance, native WoW parity, clean startup, deployment health, or CI acceptance claimed.

Audit-time log SHA-256:

- c6bc: `d394086d1c3de149fd2fb042505c28631c0708038a39e23f8367d1d748a90d21`
- bc58: `f6e1c9b3799dee73295306aff841e6f63eb7da137b4edd5eca29361f1f04416f`

Hashes bind retained bytes at audit time, not historical immutability or signed execution provenance. This single saved-receipt gate is complete; suite outcome remains FAIL.
