# Final independent bounded PTR MapDisplay audit

**OVERALL: PASS — saved PTR runtime evidence: 84/84 tests, zero failures, both test exits 0. All five GetMapDisplayInfo tests PASS. Separate stock stored-error CLI: stdout `[]`, exit 0.**

Exact audited epoch: `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z`.

Audit date: 2026-10-10. Read `/home/osso/AgentConfig/skills/verify/SKILL.md` first; performed artifact inspection as the assigned verifier. No commands, tests, rebuilds, checks, operations, delegation, or repository edits. Only this private report was updated. This supersedes this report's earlier PENDING inspection, not historical receipts.

## Execution and count audit

| Scope | Saved runtime result | Evidence |
|---|---|---|
| Original selector `c_map_probes::` | 33 reached, 33 passed; 0 failed/ignored/measured; 9854 filtered out; 2.24s; exit 0 | `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/map-probes.stdout` and `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/execution-results.json` |
| Continuation selector `c_map_api::` | 51 reached, 51 passed; 0 failed/ignored/measured; 9836 filtered out; 4.50s; exit 0 | `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/continuation/map-api-controls.stdout` and `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/continuation/execution-results.json` |
| Separate stock `lua-errors` CLI | stdout `[]`; exit 0; final stderr summary CLEAN, 0 unique/0 occurrences | `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/continuation/lua-errors.stdout`, `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/continuation/lua-errors.stderr`, and continuation execution receipt above |

Read complete test stdout for both selectors: each named test reports `ok`, and final summaries agree with reached counts and exit receipts. Both selectors use `/usr/bin/timeout 90`, `--nocapture`, and `--test-threads=1` against the same sealed integration executable. Both report a total test inventory of 9887 when selected and filtered counts are added. The 51 controls include two `c_map_api::texture::` tests; they are actual selector executions, not 51 exclusively C_Map methods.

**Correct total: 33 + 51 = 84, not 86. CLI is separate, not a test added to 84.** Continuation control execution occurred 20:09:01.323201–20:09:05.840957 UTC; CLI occurred 20:09:06.649679–20:09:12.167155 UTC.

## Controller error history retained

`<private-state>/verification/map-display-ptr-green-current/controller.stdout` retains `Error: expectedexactdefinedCMapcontrolcount`. `<private-state>/verification/map-display-ptr-green-current/worker.py` hard-coded 35 for the first selector and asserted reached count after writing the execution receipt. The original receipt still says `expected_tests: 35`, `reached_tests: 33`, `exit: 0`, `artifact_unchanged: true`; it was not rewritten to conceal the abort. This is controller accounting failure, not a failed runtime test. The abort prevented the original controller from reaching controls/CLI.

`<private-state>/verification/map-display-ptr-green-current/controller-count-correction.json` records original runtime exit 0, actual 33, `behavioral_failure: false`, and original-record SHA-256 `37e2cdd4fdc4a6c8a38ca34ea84d629b6590f5bd2e1d44bd04faedea898a3580`.

The correction's two exclusions match inspected `<repository>/tests/c_map_probes.rs`: each test has `#[cfg(feature = "client-retail")]`, not the cumulative retail API epoch gate:

- `get_player_map_position_uses_independent_active_party_member_input`.
- `get_player_map_position_party_input_controls_and_roster_reset`.

The sealed PTR feature lists do not contain `client-retail`. These two tests were compiled out, not ignored, failed, or lost by selection. `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/continuation/outcome.json` records `preserved33actualpasses: true`, `build_performed: false`, `new_executions: 2`, and total 84. No second 33-probe execution is credited.

## Source gates, wiring, and behavioral coverage

Inspected `<repository>/Cargo.toml`: `client-ptr` enables `retail-12-1-5`, which transitively enables `retail-12-1-0`. This is PTR profile with cumulative 12.1.5 API epoch, not the `client-retail` profile. `<repository>/src/client_profile.rs` selects `Retail12_1_5` and interface 120105 for that API epoch.

Inspected `<repository>/src/c_api/c_map.rs`: `C_MAP_METHODS` registers `GetMapDisplayInfo` under `retail-12-1-0`; `register_c_map_surface` calls the method registration loop. The implementation unwraps authorized secret input, validates finite integral i32 selector, reads supplied `map_display_hide_icons`, returns zero values for absence, or pushes one boolean and returns one. No DTO or guessed false default is used.

Inspected `<repository>/tests/integration.rs` and `<repository>/build.rs`: integration includes the generated harness, whose generation discovers top-level tests and emits named modules. `<repository>/tests/c_map_api.rs` explicitly includes its texture submodule. Runtime test names independently demonstrate that both selected modules reached the sealed harness.

All five tests in `<repository>/tests/c_map_probes.rs` are gated by `retail-12-1-0`, enabled on PTR:

| Exact test suffix under `c_map_probes::` | Coverage observed in source | Runtime |
|---|---|---|
| `get_map_display_info_returns_one_supplied_bool` | Supplied true/false; exactly one boolean | PASS |
| `get_map_display_info_tracks_updates_with_map_and_environment_isolation` | Independent maps and environments after updates | PASS |
| `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy` | Empty input and removal; zero values rather than explicit nil/false | PASS |
| `get_map_display_info_untainted_secret_returns_one_supplied_bool` | Authentic host-secret number accepted; secure caller/wrapper retained | PASS |
| `get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret` | Stamped addon closure permits ordinary ID, rejects authentic secret; taint/wrapper and parent restoration asserted | PASS |

Map 85 is explicit test-only catalog input. Absence/removal is simulator-input policy, not native unknown-ID parity. Parser implementation inspection does not establish malformed-number native parity.

## Seals and source evidence

`<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/submission.json` records revision `63f3a8a7373c8e52ffcc005bb443a9cf64b59fbf`, integration no-run compilation with defaults disabled and `sound,gui,casc,client-ptr`. `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/cargo-result.json` records exit 0, no stream errors; `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/compile-result.json` records exit 0 and source equality.

| Sealed executable | Recorded SHA-256 | Seal receipt |
|---|---|---|
| `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/integration-sealed` | `26c3a0dd5716207f7d009b7feb073c08dea726c0e0e23d065280fcc28890f8b5` | `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/artifact.json` |
| `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/wow-sim-sealed` | `70a1ba6261d67311a2420ac062b706c14dc52dd0edbae2e5e2d73bd40ecb2479` | `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/normal-artifact.json` |

Both files were present in the exact epoch inventory. Both artifact receipts list `client-ptr`, `retail-12-1-5`, its cumulative gates, and `sound/gui/casc`; this is artifact feature evidence, not merely requested flags. Original and continuation execution receipts report artifact unchanged. The worker seals copies and checks SHA-256 before recording seals and after execution. Continuation outcome reports source equality and no rebuild.

Source manifest paths: `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/source-before.json` and `<private-state>/verification/map-display-ptr-green-current/20261010T200242Z/source-after.json`. Worker scope covers tracked source/tests, Cargo files, build.rs, Cargo config, profile UI manifests, and listfiles. Equality here is admitted from saved comparison receipts; this read-only audit did not recalculate binary/source hashes or independently compare the complete manifests. A manifest read was truncated; no whole-manifest manual comparison claim is made. Working-source inspection confirms relevant gates/behavior but is not a newly computed current-checkout seal.

## Context limits and disposition

Bounded PTR simulator ordinary/security behavior is now supported by saved actual runtime evidence. No new compiler/check/fmt result is claimed. No default-profile counts, 97-control claim, P801 sweep, broad suite, or complete handoff claim is imported into this 84-test result.

CLI is the separate sealed `wow-sim` stored-error mode with `--no-addons --no-saved-vars`. Its stderr explicitly says `WowFontSystem::new begin casc=false`, sound/addon/SavedVariables loading disabled, and final CLEAN/zero-error summary. Outcome records `cli_separate_casc_disabled: true`. Feature `casc` in a build does not establish CASC-enabled runtime execution. Stderr also records read-only EditMode cache loading and bytecode cache hits: these are observations, not proof of cache correctness or independence.

No authenticated native WoW execution/parity, native assets, full normal startup, full GUI/rendering, asset-cache integrity/origin, or external dependency-to-artifact provenance is established. Submission explicitly excludes external dependency provenance, untracked/index state, inherited environment, and runtime assets outside fixtures. Hash preservation receipts do not authenticate those excluded inputs.

Read/search tooling limitation: two requested local text searches failed because tool ripgrep was unavailable; relevant source was then inspected through direct reads. No shell/search command workaround was run. Historical controller failure remains visible and explained; it is not softened into an original uninterrupted success.
