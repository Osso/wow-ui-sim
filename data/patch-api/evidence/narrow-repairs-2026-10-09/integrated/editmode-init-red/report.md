# EditMode initializer — independent bounded RED reproduction

**RED reproduced: all three exact tests fail at the missing fixture `InitSystemAnchors` precondition. No named postcondition reached; no readiness, fix, native/model parity or parent-goal closure claim.**

## Authorization and revisions

Read/followed verify skill as assigned verifier; no delegation. Read both supplied decision/falsification notes. Scope: local provenance and source equivalence, exact listings, then exactly one execution per target, each bounded to 90 seconds. No Cargo/build/check, other test execution, network, service/operational changes, repository edits, assertion removal or initializer changes. A static no-op initializer would omit required semantics; none installed.

Literal supplied `/home/osso/Projects/wow/wow-ui-sim4142f3d41` does not exist. Existing canonical `/home/osso/Projects/wow/wow-ui-sim` independently resolves HEAD to requested `4142f3d416ff0fa9d5505926e85e5d48ac662f7c`. All execution before/after HEAD snapshots equal that SHA. It is the **requested/observed source revision**, not a claim this binary was compiled at that SHA.

Reused **default GUI library** artifact `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`. SHA256 before/after each run and final: `97f8c673c65a614ad4d8188935a631f770bc58f987e6f02f4ab8bbc1bee8b994`. Matches retained `data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/gui-window/report.md` and selected compiler-artifact receipt. Artifact features include default/gui/client-retail; test profile opt-level=1, debuginfo=line-tables-only.

Retained GUI compile observation epoch: `b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f`, warm artifact (`fresh=true`), successful build-finished receipt. Historical compile exit integer was not retained; this limitation remains, not manufactured as exit 0. Warm reuse does not establish the original byte-generation epoch. Separate retained headless epoch `4163299fac218bcd84057897cc4ffdc81a80e42a` used a different historical artifact (`b51dc5c...`); separate integration epoch `41f1abbeb83d508b323f51d1f371a3bc506b63b5` used another integration binary. Neither is relabeled as current, executed here, or substituted for the required b02 GUI artifact. Actual 41/b02 receipts and current requested SHA remain distinct.

## Source/hash equivalence and exclusions

Compared b02 tracked Git blobs with current filesystem bytes for 1701 files under src, crates, selected in-repo iced trees, .cargo and root Cargo.toml/Cargo.lock/build.rs. **All equal**; rehashed after runs with zero changes. Per-file b02/current Git blob identities and current SHA256 in `provenance.json`. Aggregate JSON scope SHA256 `8d7bc0ada0361f7192b3b3e5f9cc4e1fa08b9b258d0411a9f5ae46a736cc89ea`. This scope is not a universal filesystem hash.

- `src/lua_api/workarounds/editmode/apply_system_anchors.lua`: `89367b425518abe393a1d13568206fa63d9aefe9e8d56a80403dd7d96d700e52` (b02 blob equals current filesystem blob).
- `src/lua_api/workarounds_editmode_tests/apply_system_anchors/cast_and_player.rs`: `949fdcb17fbcbf9239e6ad61c5e0d740ac4adcd06a722d643faa7a7c7bf8c1c2` (b02 blob equals current filesystem blob).
- `src/lua_api/workarounds_editmode_tests/apply_system_anchors/singletons.rs`: `23e39a4c867d907b96deff2c60dfa2d890201fdb4b639419606b8773bcd4c82a` (b02 blob equals current filesystem blob).
- `src/lua_api/workarounds_editmode_tests/apply_system_anchors/unit_frames.rs`: `31a8b94f201fa261484b3f745b2d5a0f58311a7bb4af2c1cde6a8958c79baef1` (b02 blob equals current filesystem blob).

Complete b02→current name-status retained in `git-diff.json`: 737 changed paths, only `tests/numeric_rule_formatter.rs` outside docs/data. Numeric fixture belongs to an integration target, not this library test artifact; it does not require rebuilding the library. Docs/data changes are retained source-audit/evidence material, not a blanket claim of arbitrary data equivalence. No relevant src/manifest/lock/build/config change observed. Initial/final status: existing untracked `.code-index.db` only.

External exclusions: external path dependencies (including rilua/game-engine), registry caches, toolchain/environment values, Blizzard/CASC cache/install assets, untracked files and files outside recorded scope. Not rehashed or asserted equivalent. Exact executions supply observed failures with those current external inputs; no universal dependency/cache-integrity or immutable-host claim.

## Exact selection and execution

Source module declaration is `workarounds_editmode::tests`, not source filename `workarounds_editmode_tests`. Initial filename-derived candidates each listed zero tests; these discovery mistakes retained in `*.list.*`, **never executed**. Corrected full identities each `--exact --list` selected **1 test, 0 benchmarks**, exit 0; full receipts `*.exact-list.*`. No broad listing or unrelated execution. Each correct target then executed once with `--exact --nocapture`, timeout 90s.

| Fixture | Selected | Outcome | Exit | Wall time | Panic source |
|---|---:|---|---:|---:|---|
| `cast_and_player` | 1 | 0 passed / 1 failed / 1977 filtered | 101 | 0.114145s | `cast_and_player.rs:380:10` |
| `singletons` | 1 | 0 passed / 1 failed / 1977 filtered | 101 | 0.114147s | `singletons.rs:193:10` |
| `unit_frames` | 1 | 0 passed / 1 failed / 1977 filtered | 101 | 0.114150s | `unit_frames.rs:437:10` |

Common error: `(string):8: attempt to call method 'InitSystemAnchors' (a nil value)` during `env.exec(APPLY_SYSTEM_ANCHORS_LUA)`. Fixture manager lacks the method; wrapper invokes it before replay. Player scale/anchor assertions, nil/-1 lookup assertions, compact refresh counters are not reached. This confirms only the exact setup failure, not validity of later assertion contracts or sufficiency of any proposed repair.

## Full execution streams

### cast_and_player

Exact identity: `lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect`

UTC 2026-10-09T19:55:08.308840+00:00 → 2026-10-09T19:55:08.423010+00:00. Command: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect --exact --nocapture`. Timeout 90s.

Complete stdout:
```text

running 1 test
test lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect ... FAILED

failures:

failures:
    lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.10s

```
Complete stderr:
```text
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.003s] [Startup] SimState::default complete in 2.47ms
[  0.003s] [Startup] rilua VM created in 106.99µs
[  0.003s] [Startup] builtin frames initialized in 69.46µs
[  0.003s] [Startup] template registry cleared in 6.93µs
[  0.003s] [Startup] intrinsic templates registered in 4.55µs
[  0.003s] [Startup] initial app_data lua handle installed in 430.00ns
[  0.093s] [Startup] init_lua_state complete in 90.41ms
[  0.093s] [Startup] final app_data lua handle installed in 170.00ns
[  0.093s] [Startup] initial screen globals installed in 56.19µs
[  0.093s] [Startup] WowLuaEnv::new complete

thread 'lua_api::workarounds_editmode::tests::apply_system_anchors::cast_and_player::apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect' (4074940) panicked at src/lua_api/workarounds_editmode_tests/apply_system_anchors/cast_and_player.rs:380:10:
apply system anchors should avoid player-frame cast-bar side effects: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors' (a nil value)", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### singletons

Exact identity: `lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons`

UTC 2026-10-09T19:55:08.780990+00:00 → 2026-10-09T19:55:08.895163+00:00. Command: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons --exact --nocapture`. Timeout 90s.

Complete stdout:
```text

running 1 test
test lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons ... FAILED

failures:

failures:
    lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.10s

```
Complete stderr:
```text
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 2.18ms
[  0.002s] [Startup] rilua VM created in 101.32µs
[  0.002s] [Startup] builtin frames initialized in 70.62µs
[  0.002s] [Startup] template registry cleared in 8.71µs
[  0.002s] [Startup] intrinsic templates registered in 4.73µs
[  0.002s] [Startup] initial app_data lua handle installed in 361.00ns
[  0.093s] [Startup] init_lua_state complete in 90.52ms
[  0.093s] [Startup] final app_data lua handle installed in 170.00ns
[  0.093s] [Startup] initial screen globals installed in 53.03µs
[  0.093s] [Startup] WowLuaEnv::new complete

thread 'lua_api::workarounds_editmode::tests::apply_system_anchors::singletons::apply_system_anchors_falls_back_to_minus_one_for_nil_singletons' (4074950) panicked at src/lua_api/workarounds_editmode_tests/apply_system_anchors/singletons.rs:193:10:
apply nil-index fallback singleton anchors: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors' (a nil value)", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### unit_frames

Exact identity: `lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes`

UTC 2026-10-09T19:55:09.254278+00:00 → 2026-10-09T19:55:09.368451+00:00. Command: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes --exact --nocapture`. Timeout 90s.

Complete stdout:
```text

running 1 test
test lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes ... FAILED

failures:

failures:
    lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.10s

```
Complete stderr:
```text
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 1.98ms
[  0.002s] [Startup] rilua VM created in 103.18µs
[  0.002s] [Startup] builtin frames initialized in 68.60µs
[  0.002s] [Startup] template registry cleared in 7.92µs
[  0.002s] [Startup] intrinsic templates registered in 5.11µs
[  0.002s] [Startup] initial app_data lua handle installed in 260.00ns
[  0.093s] [Startup] init_lua_state complete in 91.16ms
[  0.093s] [Startup] final app_data lua handle installed in 181.00ns
[  0.093s] [Startup] initial screen globals installed in 51.29µs
[  0.093s] [Startup] WowLuaEnv::new complete

thread 'lua_api::workarounds_editmode::tests::apply_system_anchors::unit_frames::apply_system_anchors_batches_compact_unit_frame_startup_refreshes' (4074960) panicked at src/lua_api/workarounds_editmode_tests/apply_system_anchors/unit_frames.rs:437:10:
apply compact unit frame batched settings: Lua(Runtime(RuntimeError { message: "(string):8: attempt to call method 'InitSystemAnchors' (a nil value)", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Receipts

`provenance.json`, retained GUI report/artifact/ledger copies, input note copies, Git diff/status/log receipts, `final-snapshot.json`, each `*.exact-list.json/stdout/stderr` and `*.run.json/stdout/stderr`. Run JSON includes argv/cwd, UTC start/end, elapsed time, bound, exit, before/after observed SHA and binary SHA256, source-scope SHA256, complete separated stdout/stderr. SHA256 seal manifest accompanies report.

**Bounded result: 3/3 requested RED failures reproduced. Zero builds, fixes or extra tests.**
