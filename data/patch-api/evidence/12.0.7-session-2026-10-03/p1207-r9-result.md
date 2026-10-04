# Round 9 progress
Goal: only A/B/C; RED/GREEN public Lua tests; three commits; six requested filters; fmt/check. No vendor/wiki/coverage edits or operational actions.
Step 1: reviews read; worktree inspected. Initial helper mistake ran read-only status in canonical repo; subsequent commands use cli with explicit cwd.
Step 2: inspected binding sampling, core producer, and cached consumer call sites. File search tools lack rg; using read-only Python scans.
Step 3: A regression added. Cached consumer scan: only CustomAuraButton SetFontString call; validates/initializes inbound FontString. No duration method overrides found. RED A on base fbe3e2040 plus regression, no producer changes.
Step 4: A RED exit 101; log r9-RED-p1207_binding_ignores_replaced_duration_methods.log.
Step 5: A implemented, formatted, committed f405e01fb1caa3cdc8a713165f6e8f859ab9081d. Shared read_query_value used by native duration methods and host binding callbacks; no Lua method lookup for numeric/zero/expired production. RED: 0 pass / 1 fail, expected constant override defect.
Step 6: A GREEN at f405e01fb exit 0; log r9-GREEN-p1207_binding_ignores_replaced_duration_methods.log.
Step 7: B regression added: forged table rejected atomically, wrong real widget rejected despite spoofed method, genuine FontString accepted despite replaced method, secret authentication first. RED scope f405e01fb plus regression.
Step 8: B RED exit 101; log r9-RED-p1207_binding_rejects_forged_fontstring_without_storing_it.log.
Step 9: B implemented using existing native_frame_id_from_val plus registry WidgetType, after setter authentication. Formatted and committed cb8d3fb4ac625502497fa077ff90c643ebc6f3c1. RED 0 pass / 1 fail: binding accepted forged FontString.
Step 10: B GREEN at cb8d3fb4ac625502497fa077ff90c643ebc6f3c1 exit 101; log r9-GREEN-p1207_binding_rejects_forged_fontstring_without_storing_it.log.
Step 11: first B GREEN failed after rejecting forgery; fixture expected "secret" instead of established "untainted" denial. Corrected diagnostic expectation; B commit amended to 21250e00bc1dcf78abd35fe0feffe126fe4d0293 to retain one commit per finding. Previous GREEN log preserved.
Step 12: B GREEN at 21250e00bc1dcf78abd35fe0feffe126fe4d0293 exit 0; log r9-GREEN-p1207_binding_rejects_forged_fontstring_without_storing_it-attempt2.log.
Step 13: C comments-only markers added, formatted and committed 416bacf603418df46384b77df4b0c22b7b4b7ca7. Zero-span expiration, nil modifier, optional clock initial time, finite/noncoercing signed fractional numbers and overflow, namespace ignored extras/function-only policy labeled INFERRED. No behavior change.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter patch_12_0_7_duration_text_binding::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter patch_12_0_7_duration_text_binding:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10437 filtered out; finished in 2.43s']; log r9-GREEN-patch_12_0_7_duration_text_binding.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_0_7_duration_text_binding::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter duration_text_binding_copy::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter duration_text_binding_copy:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 10441 filtered out; finished in 2.03s']; log r9-GREEN-duration_text_binding_copy.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'duration_text_binding_copy::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter duration_core::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter duration_core:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10421 filtered out; finished in 4.41s']; log r9-GREEN-duration_core.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'duration_core::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter patch_12_0_7_duration_clocks::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter patch_12_0_7_duration_clocks:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10439 filtered out; finished in 1.40s']; log r9-GREEN-patch_12_0_7_duration_clocks.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_0_7_duration_clocks::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter p1207_pending_callbacks::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter p1207_pending_callbacks:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 10436 filtered out; finished in 1.91s']; log r9-GREEN-p1207_pending_callbacks.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'p1207_pending_callbacks::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Proof scope: 416bacf603418df46384b77df4b0c22b7b4b7ca7; first final intersecting filter startup_globals::; prior RED/GREEN ledger consulted, no duplicate final coverage.
Final filter startup_globals:: at 416bacf603418df46384b77df4b0c22b7b4b7ca7: exit 0; ['test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 1949 filtered out; finished in 3.63s']; log r9-GREEN-startup_globals.log. Command cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9, CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts argv=['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--lib', 'startup_globals::', '--', '--test-threads=1']. Proof valid unless intersecting source/config changes.
Final gate ['cargo', 'fmt'] cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9 CARGO_BUILD_JOBS=4 at 416bacf60: exit 0; log r9-GREEN-cargo-fmt.log.
Final gate ['cargo', 'fmt', '--check'] cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9 CARGO_BUILD_JOBS=4 at 416bacf60: exit 0; log r9-GREEN-cargo-fmt-check.log.
Final gate ['cargo', 'check', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--jobs', '4'] cwd=/home/osso-test/.worktrees/wow-ui-sim-p1207-r9 CARGO_BUILD_JOBS=4 at 416bacf60: exit 0; log r9-GREEN-cargo-check.log.
Step 14: final scoped diff/readability audit, warning scan and clean-state proof complete.

## Final result
A, B, C fixed. Worktree clean on p1207-r9. No push, merge, deploy, PR, agents/model CLIs, vendor/cache/wiki/coverage edits.

### Commits (one per finding)
f405e01fb1caa3cdc8a713165f6e8f859ab9081d Sample binding duration through trusted Rust producer
21250e00bc1dcf78abd35fe0feffe126fe4d0293 Validate binding FontString by native frame identity
416bacf603418df46384b77df4b0c22b7b4b7ca7 Label inferred duration and ping input policies

### Changed files
- `docs/specs/duration-text-binding-12-0-7-audit.md`
- `src/c_api/c_ping_secure.rs`
- `src/c_api/duration_clock.rs`
- `src/c_api/duration_text_binding.rs`
- `src/c_api/duration_text_binding/state.rs`
- `src/lua_api/globals/lua_duration_object.rs`
- `src/lua_api/globals/lua_duration_object/core.rs`
- `tests/patch_12_0_7_duration_text_binding.rs`

### RED/GREEN proof
Two independent RED regressions each failed on the intended defect (0 passed / 1 failed). Both subsequently passed (1 passed / 0 failed). First B GREEN attempt reached the authentication assertion but failed on the fixture's wrong error-word expectation; corrected to existing "untainted" wording, then passed. Logs retained, no producer change for that correction.

| Log / filter | Counts |
|---|---|
| `r9-GREEN-duration_core.log` | test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10421 filtered out; finished in 4.41s |
| `r9-GREEN-duration_text_binding_copy.log` | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 10441 filtered out; finished in 2.03s |
| `r9-GREEN-p1207_binding_ignores_replaced_duration_methods.log` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10446 filtered out; finished in 0.27s |
| `r9-GREEN-p1207_binding_rejects_forged_fontstring_without_storing_it-attempt2.log` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10447 filtered out; finished in 0.24s |
| `r9-GREEN-p1207_binding_rejects_forged_fontstring_without_storing_it.log` | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10447 filtered out; finished in 0.26s |
| `r9-GREEN-p1207_pending_callbacks.log` | test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 10436 filtered out; finished in 1.91s |
| `r9-GREEN-patch_12_0_7_duration_clocks.log` | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10439 filtered out; finished in 1.40s |
| `r9-GREEN-patch_12_0_7_duration_text_binding.log` | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10437 filtered out; finished in 2.43s |
| `r9-GREEN-startup_globals.log` | test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 1949 filtered out; finished in 3.63s |
| `r9-RED-p1207_binding_ignores_replaced_duration_methods.log` | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10446 filtered out; finished in 0.26s |
| `r9-RED-p1207_binding_rejects_forged_fontstring_without_storing_it.log` | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10447 filtered out; finished in 0.21s |

Final six intersecting filters at `416bacf603418df46384b77df4b0c22b7b4b7ca7`: **92 passed, 0 failed**, including actual cached CustomAuraButton consumer execution. No zero-selected filter. No final proof invalidated by later source changes; final cargo fmt left worktree clean.
`cargo fmt`, `cargo fmt --check`, `CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --jobs 4`: all exit 0; no compiler warnings found. `git diff --check fbe3e2040..HEAD`: exit 0. Logs r9-GREEN-cargo-*.log.

### Scope/readability check
Changed Rust reviewed manually under rust-readability skill (rust-code-analysis-cli unavailable). No new suppressions, deep nesting, duplicated substantial helpers, or speculative abstractions. Existing Query/match producer extracted for shared trusted reads; frame validation reuses native_frame_id_from_val, authoritative VM backing and widget registry.
Cached Retail consumers scanned read-only: one SetFontString call in CustomAuraButton validates/initializes an inbound FontString; no duration query method overrides found. Retained cached-consumer test passed with identity validation.

### Unfixed / merge risk
Nothing outstanding in authorized A/B/C scope. NumericFormatter identity (review finding 2's separate formatter concern), full formatting-component/native historical conformance and earlier-profile execution remain outside this task; spec retains those gaps.
Merge risk: bounded/default-Retail proof is green, but core query extraction is shared across profiles, which were not executed by authorization. Forged FontString inputs and binding dependence on duration method overrides are intentionally no longer accepted; cached tested consumer remains compatible. No native-client parity or exhaustive consumer claim.
