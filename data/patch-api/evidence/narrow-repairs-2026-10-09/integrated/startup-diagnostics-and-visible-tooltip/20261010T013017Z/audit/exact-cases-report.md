# Existing-execution exact-case audit

**PASS: 14/14 cases**, each exactly one test passed, zero failed/ignored/measured, exit 0, recorded artifact_hash_unchanged=true. Complete stdout/stderr read locally, including expected errors; summary and per-case metadata agree exactly. No zero-test selector, panic, hidden/nested failing harness or unexplained failure found.

Read/followed verify skill in independent verifier role. Existing proof ONLY: no Cargo/test/runtime reruns, delegation/backend, source edits or operational changes. Only requested audit files written.

## Source epoch and exclusions

Epoch `20261010T013017Z`; compiled source `fc824cb921342c06f506eb92e8c81d52bdf28fcd`. Captured build starts October 10, 2026 01:30:17 UTC; cases run 01:35:44–01:35:52 UTC that day. Literal capture timestamps, not a local-calendar “today” claim. Future style fixes/current-head revisions are separate, unaccepted proof epochs.

Native-client parity, all-profile coverage, startup acceptance and current-head acceptance excluded. Neither startup-debug nor concurrently added startup-control is used. External dependencies/data/runtime caches/inherited environment and untracked state outside captured scope are not established.

## Complete case matrix

Every recorded argv: `/usr/bin/timeout 90 SELECTED_EXECUTABLE --exact SELECTOR --nocapture --test-threads=1`; override WOW_SIM_NO_SOUND=1. Counts are independently parsed stdout P/F/I; bytes stdout/stderr.

| Case | Exact selector | Artifact | P/F/I | Exit | Bytes |
|---|---|---|---|---|---|
| case01 | `lua_api::execution_budget::tests::budget_error_log_distinguishes_exhausted_entry_from_callback_consumption` | `wow_ui_sim-test` | 1/0/0 | 0 | 234/0 |
| case02 | `lua_api::execution_budget::tests::budget_error_log_labels_unavailable_snapshots_without_inventing_counts` | `wow_ui_sim-test` | 1/0/0 | 0 | 232/0 |
| case03 | `lua_api::execution_budget::tests::budget_error_log_reports_unlimited_meter_and_escapes_event_metadata` | `wow_ui_sim-test` | 1/0/0 | 0 | 229/0 |
| case04 | `lua_api::execution_budget::tests::file_budget_error_log_distinguishes_exhausted_entry_from_file_consumption` | `wow_ui_sim-test` | 1/0/0 | 0 | 235/0 |
| case05 | `lua_api::execution_budget::tests::file_budget_error_log_labels_missing_snapshots` | `wow_ui_sim-test` | 1/0/0 | 0 | 208/0 |
| case06 | `lua_api::execution_budget::tests::file_budget_error_log_escapes_metadata_and_reports_unlimited_meter` | `wow_ui_sim-test` | 1/0/0 | 0 | 228/0 |
| case07 | `loader::lua_file::tests::startup_file_budget_error_preserves_cumulative_usage_and_loader_outcome` | `wow_ui_sim-test` | 1/0/0 | 0 | 224/1032 |
| case08 | `addon_loading::warning_report::tests::debug_enabled_emits_all_eight_warnings_and_keeps_diagnostic_allowlist` | `wow-sim-test` | 1/0/0 | 0 | 233/0 |
| case09 | `addon_loading::warning_report::tests::debug_disabled_emits_no_warning_or_diagnostic_lines` | `wow-sim-test` | 1/0/0 | 0 | 215/0 |
| case10 | `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | `integration-test` | 1/0/0 | 0 | 198/1044 |
| case11 | `tooltip_text_layout::test_tooltip_layout_is_clamped_to_top_left_viewport_edge` | `integration-test` | 1/0/0 | 0 | 206/1042 |
| case12 | `utility_api::test_table_create_returns_empty_mutable_tables_for_capacity_variants` | `integration-test` | 1/0/0 | 0 | 210/640 |
| case13 | `spell_api::test_spell_get_maw_power_border_atlas_by_spell_id_follows_retirement_epoch` | `integration-test` | 1/0/0 | 0 | 214/639 |
| case14 | `wowforever_table::wowforever_table_does_not_leak_into_earlier_profiles` | `integration-test` | 1/0/0 | 0 | 199/641 |

## Behavioral coverage and stderr interpretation

| Cases | Established behavior | Proof level |
|---|---|---|
| 01–03 | Handler exhausted-entry/consumption format, unavailable snapshots, unlimited meter/escaped event | Three formatting tests |
| 04–06 | File exhausted-entry/consumption format, missing snapshots, unlimited meter/escaped metadata | Three formatting tests |
| 07 | Cumulative owner instruction usage and preserved loader error outcome | One real rilua/loader fixture |
| 08–09 | Debug-on eight warnings plus two allowlisted diagnostics; debug-off zero lines | Two binary-target synthetic fixture tests |
| 10–11 | Shown-tooltip viewport clamps | Two integration controls, not visual/native acceptance |
| 12–14 | table.create capacity/mutation, Maw retirement, table.count and later-extension non-leak | Three selected retail-profile fixture tests |

Seven relevant revision blobs SHA-256 match captured source hashes (read-only Git cat-file); current copies also match those seven epoch hashes. This does not certify whole current source. Locations:

- src/lua_api/execution_budget.rs:156–261: literal handler/file diagnostic contracts, concrete budgets, missing/unlimited snapshots and escaping.
- src/loader/lua_file.rs:381–444: case07 executes return 42 under limit 100; asserts 0 < pre-use < 100; infinite-loop file returns Lua LoadError with correct owner/chunk; cumulative usage reaches exactly 100; next file errors without resetting usage; unrelated eval succeeds. Private state entered=true, completed=nil, next_file=nil.
- src/bin/wow_sim/addon_loading/warning_report.rs:153–184: debug-on asserts eight warnings plus two allowlisted diagnostics including synthetic nested-load failure; debug-off asserts no lines. Synthetic failure strings are fixture data, not hidden failures.
- tests/tooltip_text_layout.rs:145–245: 400×300 viewport, explicit GameTooltip:Show after SetOwner, sizing/layout update, edge-bound assertions. tests/utility_api.rs:262–290 rejects missing hint and checks four initially empty mutable capacity variants.
- tests/spell_api.rs:488–497 selected retail-12-0-7 branch checks Maw member absent. tests/wowforever_table.rs:36–62 selected retail-12-1-0 branch asserts three table.count returns (total=3, array=2, maximum=2) and seven later extension names absent; does not execute Forever consumer branch.

case01–06 and 08–09 stderr empty. case10–14 contain initialization/font timings only. case07 contains initialization plus expected FileBudgetProbe instruction-budget exhaustion, traceback, and **“Lua error suppressed 1 additional times”** for same owner/chunk. Source proves initial exhaustion followed by exhausted-entry error; not suppressed failing harness and not clean-startup proof. No file-budget-error counter record printed there: cumulative usage/error outcome proved by assertions, not emitted stderr counters.

## Compile/artifact/hash correlation

Full compile.stdout parsed: 665 compiler-artifact records, 74 build-script-executed, one build-finished success=true, zero compiler-message records. Each of four selection records matches exactly one compile record. cargo-result/outcome compile exit=0. compile.stderr has six iced-wgpu-patched/Cargo.toml deprecated hyphenated Clippy lint-name warnings, plus aggregate generated-6-warnings line. **Not warning-clean proof.** Captured compile was offline/locked/no-run lib + integration + wow-sim; never rerun here.

Artifacts use client-retail/profile-retail, retail-12-1-0 and earlier epochs, default/casc/gui/sound. Optimized test profile opt_level=1, line-table debuginfo, debug/overflow checks; not release or alternate client. Normal wow-sim selection is correlated but not used by these cases.

- `wow_ui_sim-test`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`; SHA-256 `cbde2b9ad3ae7a0d832de6f4fea679f05d1b2f31384c4257f76bac8f024c5fdf`; profile.test=true.
- `wow-sim-normal`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`; SHA-256 `4c4730adbd8bc1c4b92988b138d6f941e325ffe7207c64ba73db264548fc8ecc`; profile.test=false.
- `wow-sim-test`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_sim-e99132585a38db9e`; SHA-256 `aa180a5b174638c27f23abff15db2eadde0f7d7ef44ff096ad4a9aa8fab37759`; profile.test=true.
- `integration-test`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`; SHA-256 `d592e1f1fd077b623e01b6491d260a249a1c32d6c6b4a9bf89ed5cd1a863258b`; profile.test=true.

Source snapshots byte-identical: 3,839 path/hash entries; outcome source_equal=true. Seven inspected revision blobs match snapshots; remaining/external scopes not asserted. Capture bindings:

- `compile.stdout`: `ebccde7fdff0a5e4f64708e2a0565623613c99e7a3ce8268491a5bc81c4c666b`.
- `compile.stderr`: `2619d8a22ddcc8a33c7b650389a989158671fdc32276cbca000e8d95682f4170`.
- `artifact-selection.json`: `82fb9ff6a210b648a8059c6613f51a5065f24c8accf6c81f9417b584809ed730`.
- `source-before.json`: `89b95674b7d3c5683a47097a8909aaab0a320c9145912e1e268bdc2d46a6f1bc`.
- `source-after.json`: `89b95674b7d3c5683a47097a8909aaab0a320c9145912e1e268bdc2d46a6f1bc`.

Current-disk SHA-256 corroboration already read without execution: all three test binaries equal their recorded selection hashes. Historical unchanged flags remain independently recorded in case results; hashes are integrity/correlation, not signed execution attestation.

## Privacy and manifest

Before retaining audit output: bounded local full-byte screening of original epoch files for private keys, credential assignments, provider tokens, credential URLs found no candidates. Retained scope additionally screened for Bearer/JWT patterns; no candidates. Pattern screening cannot guarantee detection of arbitrary unlabelled secrets. No raw logs/source/payloads/private credentials copied into audit documents or externally published.

hashmanifest.json binds exactly the 66 original non-startup inputs and this report by relative path, byte count and SHA-256; excludes itself. Concurrent startup-control and other auditors' files excluded explicitly. Original evidence untouched. Manifest records artifact hashes separately; future changes are not retrospectively accepted.
