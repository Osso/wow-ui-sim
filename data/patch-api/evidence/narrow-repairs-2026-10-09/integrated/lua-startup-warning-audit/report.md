# Lua/startup warning audit

Date: 2026-10-09. Read-only; no repository files, fixtures, builds, tests, services, profiles, or environment inspected/changed. Evidence is limited to existing retained output plus source-path inspection. See [selected message headlines](selected-messages.md) and [hash/size ledger](evidence-hashes.md).

## Findings

- **No current clean-startup result is available.** The newest direct captured Mists execution is a targeted integration test, not a fresh standalone startup or native acceptance run. Its stdout shows `cast_bar_respects_edit_mode_lock_setting_after_startup_fix ... FAILED` and test result 0/1. Its stderr contains seven distinct Lua error headlines, each appearing twice in the stream (14 headline lines); paired `Lua error:`/`Lua Error:` presentations are duplicate reporting of the same message, not evidence of 14 separate error occurrences. Do not reinterpret this as startup's unique-error count: this is a Rust integration test process with its own capture boundaries.
- Those seven captured messages are: failed `__Blizzard_UIPanels_Game_20250` frame creation from self-anchor; `AchievementFrameAchievements` OnLoad nil call; `AchievementFrameStats_OnLoad` nil global; `AchievementFrameComparison` OnLoad nil call; `next` called on non-table; nil table index in `Blizzard_CombatLogBase/Wrath/CombatLogColors.lua`; and `AchievementFrameAchievements` OnEvent nil call. Exact headlines are in `selected-messages.md`. The saved test failure is specifically the cast-bar lock assertion; the retained output does not prove these Lua diagnostics caused that assertion.
- The full-suite log is a completed 10,679-test nextest transcript, not a standalone startup capture. It contains 13 `Lua error:` presentation lines representing five distinct normalized messages (including a five-line duplicate of one template error), plus six Cargo manifest lint deprecations during compilation. It cannot prove that latest startup is clean, that all startup diagnostics are represented, or exact runtime occurrence totals: test parallelism/multiple test environments aggregate output and stdout/stderr ordering is not wall-clock attribution. It does establish that errors were emitted in a test run with passing tests; suite success is not clean-startup proof.
- The Mists `compile.stderr` contains exactly six `iced-wgpu-patched/Cargo.toml` deprecated lint-key warnings (and the summary `iced_wgpu (manifest) generated 6 warnings`), then successful compilation. These are Cargo manifest warnings, not Lua or simulator startup warnings. Full-suite compile output has the same six warnings; those two logs are separate captures, not one combined count.
- Neither selected current log includes a complete normal startup timeline or an `Addon loading summary` / `Total: ... warnings` block. The standalone startup's warning summaries are path-dependent: third-party addon `LoadResult.warnings` are counted in `src/bin/wow_sim/addon_loading.rs` and verbose addon warning counts require `WOW_SIM_VERBOSE`; warning details are only printed under `WOW_SIM_DEBUG_NIL_GLOBALS` (with a further addon allowlist for third-party warnings in `src/bin/wow_sim/addon_loading/warning_report.rs`). Thus absent warning lines in these captured test streams are not evidence of zero loader warnings.
- The `lua-errors` command is not a clean substitute for ordinary startup output. Source in `src/lua_errors.rs` intentionally suppresses stderr while `collect_lua_error_startup` runs, then prints a deduplicated JSON array and final observed Lua error summary. That path inventories collected Lua errors for its startup-event/tick boundary, not all loader warnings, fatal startup failures, or warnings captured outside the collector. It was not run for this audit.

## Coverage / gaps

| Signal | Observed coverage | Missing / boundary |
|---|---|---|
| Lua runtime errors | Seven distinct messages in retained Mists targeted-test stderr; five normalized message kinds in the full-suite transcript. | Neither is current standalone full startup; test-worker aggregation and duplicate reporter paths prevent claiming exact startup occurrence counts. |
| Addon-loader warnings | Source confirms warning counts and opt-in detail output. | No current complete standalone startup log with loader summary; no exact current warning count. |
| Rust/Cargo warnings | Six manifest deprecations in each retained compile capture. | Not runtime Lua/startup warnings; no compiler-warning claim beyond these captures. |
| Startup failures/load omissions | Source routes loader errors and fatal errors through separate output paths; existing Mists test process failed its assertion. | No current standalone startup run/result; saved suite cannot certify all enabled addons loaded or ordinary startup phases completed. |
| Stored/suppressed warnings | `LoadResult.warnings`, `nil_symbol_observations`, and `missing_requirements` are retained on load results; output is conditional. Lua-error command deliberately redirects/suppresses startup stderr and relies on SimState collection. | No exhaustive audit of every tracing/logging/store-only warning producer or source-to-output route. No basis for asserting none are silently unreported. |

## To establish current startup status

A fresh standalone startup capture is required; no existing retained log satisfies it. No command was launched per scope. When authorized, capture a bounded `wow-sim lua-errors` run (profile-appropriate built binary and flags) with both stdout/stderr and exit status, plus a normal startup command with stderr/stdout and exit status so loader/fatal warning paths remain visible. Record profile, exact command, selected binary/revision, and completion status; do not combine streams and infer global chronological order. The machine's project rules require build separately from run and bound runtime execution; follow current profile/cache prerequisites. A standalone `lua-errors` zero-error result alone still would not establish zero addon-loader warnings.

## Provenance

Inputs were exactly:

- `/home/osso/Projects/wow/full-suite-results/96e88494a6844833b79a574d37a2115dc03b44eb.log`
- `/home/osso/.local/state/wow-ui-sim/verification/mists-cast-diagnostic/20261009T221327Z/exact-run.stderr`
- `/home/osso/.local/state/wow-ui-sim/verification/mists-cast-diagnostic/20261009T221327Z/exact-run.stdout`
- `/home/osso/.local/state/wow-ui-sim/verification/mists-cast-diagnostic/20261009T221327Z/compile.stderr`

Their sizes and SHA-256 digests are preserved in `evidence-hashes.md`. The repository worktree already had unrelated fixture modifications and an untracked index artifact; none were read or touched beyond observing status. No fixes proposed or applied.
