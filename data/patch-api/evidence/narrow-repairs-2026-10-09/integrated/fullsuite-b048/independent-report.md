<!-- Sanitized retention: absolute host paths replaced; audit findings unchanged. -->
# Independent saved-suite audit — FAIL

Verified: 2026-10-10. Artifact-only audit; no tests, builds, submission, rerun, wait, polling, delegation, or repository edits.

## Binding and provenance

Saved revision: `b048251f38b2d53e33b3365590cd227d40fc07d0`; JSON `ref` and `sha` both match. Started `2026-10-10T13:40:14-0500`; finished `2026-10-10T14:01:43-0500`.

Evidence:

- `external-full-suite-results/b048251f38b2d53e33b3365590cd227d40fc07d0.json` — SHA256 `9538e34a7543d1fa8bff7cecfa5d87007fbe5613df506fef5ecf79f74be468e3`.
- `external-full-suite-results/b048251f38b2d53e33b3365590cd227d40fc07d0.log` — SHA256 `e326b07a64ef4d723d7a75f0cd759f3fa8f84b7685188994ee3f5b4e24670398`; 2,099,274 bytes, 16,513 lines. Entire file loaded and parsed; failure details and phase boundaries inspected.
- `installed-full-suite` — installed executable present; SHA256 `6a4ea7958598a3efb09f8763b8d46523d7880c66dd46902d802160b5d268a2a1` at audit. Current helper source is not an archived execution-time checksum.

One service observation only: `systemctl --user show full-suite-1791657613.service --property=Id,LoadState,ActiveState,SubState,Result,ExecMainCode,ExecMainStatus,ExecStart,ActiveEnterTimestamp,InactiveEnterTimestamp` returned `LoadState=not-found`, `ActiveState=inactive`, `SubState=dead`, `Result=success`, `ExecMainCode=0`, `ExecMainStatus=0`, empty timestamps and no ExecStart. The collected unit cannot independently prove invocation, revision, or service exit. Its default-looking success fields are NOT test acceptance. JSON/log are completion evidence; requested service-to-artifact association remains caller-supplied, not independently recoverable from this observation. No second status observation was made. Installed `full-suite status REF` would fetch origin through `resolve`; it was deliberately not invoked during this read-only audit.

## Exact results

All three recorded phases failed. Commands below are historical log text, NOT commands executed by this audit.

| Phase | Historical command | Counts | Exit | Recorded duration | Log evidence |
|---|---|---|---:|---:|---|
| integration | `cargo nextest run --test integration --no-fail-fast --test-threads 16 --offline --locked` | 10,684 run; 10,682 passed, 2 failed, 19 skipped; 12 slow | 100 | 963.3 s | lines 1, 13, 11391–11394 |
| prefork | `cargo test --test prefork_full_ui --offline --locked` | Main batch: 2,321 total, 2,320 passed, 1 failed. Later batches: 2/2, 1/1, 1/1 passed. Aggregate: 2,325 executions, 2,324 passed, 1 failed | 1 | 183.0 s | lines 11396–11397, 13968–13981, 14472 |
| lib | `cargo nextest run --lib --no-fail-fast --test-threads 16 --offline --locked` | 2,001 run; 2,000 passed, 1 failed, 0 skipped | 100 | 142.6 s | lines 14477, 14489, 16510–16512 |

Combined recorded executions: 15,010 run, 15,006 passed, 4 failed; 19 integration skips outside the run count. These are execution counts, not deduplicated tests across harnesses. Nextest runtime summaries (841.812 s integration; 61.887 s lib) differ from wrapper phase durations, which include command overhead/build time. No missing phase; JSON failure sets match log selectors after deduplicating repeated failure listings.

## Exact failures and evidence

1. `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` — integration. Log line 6827 failure; panic at `saved-checkout/tests/method_diff_coverage.rs:264:5` (log 6853). Message: `diff_methods_extra.txt is out of sync with the current metatable surface.` Stale entries include `Button:SetBlipTexture` and `Button:SetCorpsePOIArrowTexture`; full mismatch details remain in the saved log.
2. `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` — integration. Log 7448; same source assertion at `saved-checkout/tests/method_diff_coverage.rs:264:5` (log 7474). Message: `diff_methods_missing.txt is out of sync with the current metatable surface.` Stale entries include `FontString:GetAlphaGradient`, `FontString:GetScaleAnimationMode`, `FontString:SetScaleAnimationMode`, `FontString:SetVertexColorFromBoolean`, `Texture:SetVertexColorFromBoolean`; additions include `Button:SetPreventSecretValues`.
3. `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` — prefork. Log 12041–12070; panic location `saved-checkout/tests/blizzard_garrison_ui_loads.rs:286:5` at log 12258. Non-3D error: `bad argument #1 to 'ipairs' (table expected, got nil)` at `...ard_GarrisonUI/Mainline/Blizzard_AdventuresCombatLog.lua:90`, during frame creation of `__Blizzard_GarrisonUI_45563`. Additional stderr contains model-method errors; the asserted failure explicitly identifies the non-3D error above.
4. `iced_app::update::update_tests::main_thread_cpu_time_advances_with_busy_work` — lib. Log 14852–14870; source `saved-checkout/src/iced_app/update_tests.rs:872:5` (log prints relative path). Exact message: `30ms of spinning should add CPU time: 100.287µs -> 4.773533ms`. Saved source lines 863–875 loop for 30 ms of wall time and require at least 10 ms of thread CPU time; measured delta was 4.673246 ms. Failure proves that assertion failed; it does not independently prove a broken CPU clock. Scheduling/resource pressure is a possible explanation, not a diagnosed cause.

## Historical comparison: new is not necessarily regression

JSON `new_failures`: integration empty; prefork Garrison selector above; lib CPU selector above. Installed helper calculates this against the mutable `external-full-suite-results/master-latest.json`; saved result does not identify that baseline revision. Therefore this field cannot establish historical novelty or causality.

Direct prior evidence: `external-full-suite-results/5aa6cb277eb5919e9276100c023f1c98809c7bc9.json`, finished `2026-10-10T12:53:33-0500`, contains the same two method failures and same Garrison failure, no lib failures. Its matching `external-full-suite-results/5aa6cb277eb5919e9276100c023f1c98809c7bc9.log:12043–12044` records the same Garrison nil-`ipairs` error at AdventuresCombatLog.lua:90; line 14853 records the CPU test PASS. Thus Garrison is historical despite the new_failures label; CPU failure is newly observed relative to this immediate prior saved run, not a demonstrated source regression. Earlier saved JSONs c6bc, bc58, 6751, and 8e likewise contain the same method/Garrison selectors and no lib failures.

## Source and profile scope

Source inspection used read-only Git commands with explicit `.cwd('/home/osso/Projects/wow/wow-ui-sim')`. Saved commit is a docs/receipt commit. It includes the earlier successful file-budget diagnostic implementation `0e4bf4161d58cbbf6572a5be08cac7f5019e316f`, changing `repo/src/loader/lua_file.rs` and `repo/src/lua_api/execution_budget.rs`.

Saved lib log records PASS for both loader cumulative-budget tests (lines 15035–15036), three error formatter tests (15638–15641), and three success formatter tests (15642–15644). This is positive bounded evidence for the diagnostic change, but does not turn the failing full suite green or establish full-addon runtime acceptance.

Historical commands do not select alternate features. Saved `repo/Cargo.toml` defaults are sound/gui/casc/client-retail; client-retail enables retail-12-1-0 and prefork-full-ui. Evidence is for these selected integration/prefork/lib targets, not every Cargo test target or every client profile. Environment, toolchain versions, dependency path content, and Blizzard cache provenance are not archived in this JSON/log; complete hermetic reconstruction is not claimed. The 19 skipped integration tests remain unproved.

Observed current HEAD was `1ee814f895d5b3f262bd4576745d41add0da47cb`, not the saved revision. Later source changes excluded:

| Later commit | Exact scope | Saved-suite proof |
|---|---|---|
| `72971773c5014774471a28ea065aeff2ced099d6` | `repo/src/c_api/c_secrets.rs`: `is_never_secret_aura` cfg changed from `aura-containers` to `retail-12-1-0` | NONE for changed compilation behavior, especially Forever |
| `20e808b945188ce2b952f3dd81f2aa6547bc8587` | `repo/tests/wowforever_finite_constants.rs`: event regression test | NONE |
| `d5dcc8b5953ef1e62a49f58845919077167a4a0a` | `repo/src/event/valid_events.rs`: adds `UNIT_AURA_BLOCK_LIST_CLEARED` to Forever registerable events | NONE; default-retail suite cannot validate Forever repair |
| `1ee814f895d5b3f262bd4576745d41add0da47cb` | Map display inputs/probes in `repo/src/lua_api/state.rs`, `repo/src/lua_api/state/sim_state.rs`, `repo/tests/c_map_probes.rs` | NONE |

The helper commit occurred during the wall-clock run, but the saved SHA predates it; temporal overlap is not coverage. Current dirty/untracked checkout content is also outside this saved-result scope. No later-source validation, fixes, or reruns authorized by this audit.

## Verdict

**FAIL at saved b048 revision.** Three phases completed with four exact failing selectors. Successful diagnostic tests are covered; later helper/event/map changes and non-retail profile acceptance are excluded. Service invocation provenance remains unavailable after unit collection. No current-HEAD acceptance claim.
