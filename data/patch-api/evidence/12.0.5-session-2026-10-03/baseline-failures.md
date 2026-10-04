# Baseline integration failure investigation

Baseline: `9a7a0291b`. Four requested tests; isolated worktree and target directory. Main checkout untouched. Current external dependencies and Blizzard cache used: this proves baseline code behavior now, not historical cache state.

## Setup

```text
git -C /home/osso-test/Projects/wow/wow-ui-sim worktree add --detach /home/osso-test/.worktrees/wow-ui-sim-baseline 9a7a0291b
HEAD is now at 9a7a0291b Accept bounded GetSpellBookItemCastCount for row 318
Preparing worktree (detached HEAD 9a7a0291b)
```

## Baseline execution

Working directory: `/home/osso-test/.worktrees/wow-ui-sim-baseline`. No build timeout.

```text
CARGO_BUILD_JOBS=6 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-baseline --test integration on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases c_system_api::test_c_console_get_all_commands_empty tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges -- --test-threads=1 > /tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/baseline-initial.log 2>&1
```

Cargo rejected several filters before `--`: `error: unexpected argument 'c_system_api::test_c_console_get_all_commands_empty' found`. Retrying authorized placement after `--`.

```text
CARGO_BUILD_JOBS=6 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-baseline --test integration -- on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases c_system_api::test_c_console_get_all_commands_empty tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges --test-threads=1 > /tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/baseline-four.log 2>&1
```

Observed baseline exit: **101**. All four failed; later-commit search is not applicable.

```text
    Finished `test` profile [optimized + debuginfo] target(s) in 8m 59s
     Running tests/integration.rs (/home/osso-test/.cache/wow-ui-sim-target-baseline/debug/deps/integration-8ea324359263a4d2)

running 4 tests
test c_system_api::test_c_console_get_all_commands_empty ... FAILED
test on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases ... FAILED
test tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup ... FAILED
test tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges ... FAILED

failures:

---- c_system_api::test_c_console_get_all_commands_empty stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.008s] [Startup] SimState::default complete in 7.76ms
[  0.008s] [Startup] rilua VM created in 166.76µs
[  0.008s] [Startup] builtin frames initialized in 119.76µs
[  0.008s] [Startup] template registry cleared in 11.75µs
[  0.008s] [Startup] intrinsic templates registered in 6.81µs
[  0.008s] [Startup] initial app_data lua handle installed in 664.00ns
[  0.552s] [Startup] init_lua_state complete in 543.37ms
[  0.552s] [Startup] final app_data lua handle installed in 146.00ns
[  0.552s] [Startup] initial screen globals installed in 139.94µs
[  0.552s] [Startup] WowLuaEnv::new complete

thread 'c_system_api::test_c_console_get_all_commands_empty' (1026488) panicked at /home/osso-test/.worktrees/wow-ui-sim-baseline/tests/c_system_api.rs:66:5:
assertion `left == right` failed
  left: 1647
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases stdout ----
[  0.614s] [Startup] WowLuaEnv::new begin
[  0.616s] [Startup] SimState::default complete in 1.52ms
[  0.616s] [Startup] rilua VM created in 96.45µs
[  0.616s] [Startup] builtin frames initialized in 52.31µs
[  0.624s] [Startup] template registry cleared in 8.04ms
[  0.624s] [Startup] intrinsic templates registered in 14.53µs
[  0.624s] [Startup] initial app_data lua handle installed in 1.43µs
[  0.930s] [Startup] init_lua_state complete in 305.62ms
[  0.930s] [Startup] final app_data lua handle installed in 362.00ns
[  0.930s] [Startup] initial screen globals installed in 113.75µs
[  0.930s] [Startup] WowLuaEnv::new complete
Lua error: Lua Error: Dirty flags were not fully cleared during update pass (remaining flags: 18)
[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:59: in function 'GetErrorData'
[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:72: in function <[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:71>
[Interface/AddOns/Blizzard_SharedXMLBase/ErrorUtil.lua]:19: in function 'assertsafe'
[Interface/AddOns/Blizzard_SharedXML/MixinUtil.lua]:433: in function 'ProcessDirtyFlags'
[Interface/AddOns/Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua]:91: in function <[Interface/AddOns/Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua]:90>
=(tail call): ?
=(tail call): ?


thread 'on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases' (1026508) panicked at /home/osso-test/.worktrees/wow-ui-sim-baseline/tests/on_update_modes.rs:186:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup stdout ----
[  2.607s] [Startup] WowLuaEnv::new begin
[  2.609s] [Startup] SimState::default complete in 1.35ms
[  2.609s] [Startup] rilua VM created in 101.90µs
[  2.609s] [Startup] builtin frames initialized in 48.83µs
[  2.609s] [Startup] template registry cleared in 5.06µs
[  2.609s] [Startup] intrinsic templates registered in 6.90µs
[  2.609s] [Startup] initial app_data lua handle installed in 473.00ns
[  2.756s] [Startup] init_lua_state complete in 147.64ms
[  2.756s] [Startup] final app_data lua handle installed in 179.00ns
[  2.757s] [Startup] initial screen globals installed in 141.13µs
[  2.757s] [Startup] WowLuaEnv::new complete

thread 'tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup' (1026587) panicked at /home/osso-test/.worktrees/wow-ui-sim-baseline/tests/tooltip_item_context.rs:249:22:
actual GetItemByID contract: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges stdout ----
[  2.840s] [Startup] WowLuaEnv::new begin
[  2.842s] [Startup] SimState::default complete in 1.58ms
[  2.842s] [Startup] rilua VM created in 96.24µs
[  2.842s] [Startup] builtin frames initialized in 60.37µs
[  2.842s] [Startup] template registry cleared in 5.48µs
[  2.842s] [Startup] intrinsic templates registered in 7.78µs
[  2.842s] [Startup] initial app_data lua handle installed in 501.00ns
[  3.337s] [Startup] init_lua_state complete in 495.36ms
[  3.337s] [Startup] final app_data lua handle installed in 300.00ns
[  3.337s] [Startup] initial screen globals installed in 172.68µs
[  3.337s] [Startup] WowLuaEnv::new complete
[  3.338s] [Startup] WowFontSystem::new begin casc=true
[  3.349s] [Startup] font FRIZQT__.TTF loaded in 11.74ms
[  3.350s] [Startup] font ARIALN.TTF loaded in 290.51µs
[  3.350s] [Startup] font FRIZQT___CYR.TTF loaded in 262.42µs
[  3.350s] [Startup] cosmic font system built in 12.92µs
[  3.350s] [Startup] swash cache created in 3.07µs
[  3.350s] [Startup] WowFontSystem::new complete in 12.34ms

thread 'tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges' (1026601) panicked at /home/osso-test/.worktrees/wow-ui-sim-baseline/tests/tooltip_text_layout.rs:180:5:
assertion failed: rect.x + rect.width <= state.screen_width + 0.1


failures:
    c_system_api::test_c_console_get_all_commands_empty
    on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases
    tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup
    tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 10089 filtered out; finished in 3.38s

error: test failed, to rerun pass `--test integration`
```

## Aura assertion diagnostics

Only temporary assertion messages added in worktree `tests/on_update_modes.rs:180-182`; no behavior changes. Baseline proof remains valid for unmodified source; rerun identifies assertion.

```text
CARGO_BUILD_JOBS=6 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-baseline --test integration on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases -- --test-threads=1 > /tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/baseline-aura-instrumented.log 2>&1
```

Instrumented aura exit: **101**. Exact failing assertion is `assert(not GetForbiddenObjectTable(ModeAuraContainer):IsDirty())` at `tests/on_update_modes.rs:181`; preceding `AuraPhaseCalls == 1` passed. Diagnostic panic: `container still dirty; flags=18`.

```text
native source=/home/osso-test/.worktrees/wow-ui-sim-baseline target=/home/osso-test/.worktrees/wow-ui-sim-baseline/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']
   Compiling wow-ui-sim v0.1.0 (/home/osso-test/.worktrees/wow-ui-sim-baseline)
    Finished `test` profile [optimized + debuginfo] target(s) in 40.63s
     Running tests/integration.rs (/home/osso-test/.cache/wow-ui-sim-target-baseline/debug/deps/integration-8ea324359263a4d2)

running 1 test
test on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases ... FAILED

failures:

---- on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 1.86ms
[  0.002s] [Startup] rilua VM created in 130.19µs
[  0.002s] [Startup] builtin frames initialized in 96.31µs
[  0.002s] [Startup] template registry cleared in 7.76µs
[  0.002s] [Startup] intrinsic templates registered in 5.46µs
[  0.002s] [Startup] initial app_data lua handle installed in 398.00ns
[  0.167s] [Startup] init_lua_state complete in 165.05ms
[  0.167s] [Startup] final app_data lua handle installed in 162.00ns
[  0.167s] [Startup] initial screen globals installed in 105.25µs
[  0.167s] [Startup] WowLuaEnv::new complete
Lua error: Lua Error: Dirty flags were not fully cleared during update pass (remaining flags: 18)
[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:59: in function 'GetErrorData'
[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:72: in function <[Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua]:71>
[Interface/AddOns/Blizzard_SharedXMLBase/ErrorUtil.lua]:19: in function 'assertsafe'
[Interface/AddOns/Blizzard_SharedXML/MixinUtil.lua]:433: in function 'ProcessDirtyFlags'
[Interface/AddOns/Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua]:91: in function <[Interface/AddOns/Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua]:90>
=(tail call): ?
=(tail call): ?


thread 'on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases' (1028699) panicked at /home/osso-test/.worktrees/wow-ui-sim-baseline/tests/on_update_modes.rs:186:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "container still dirty; flags=18", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10092 filtered out; finished in 0.48s

error: test failed, to rerun pass `--test integration`
```

## Conclusion

All four requested tests already fail at session-start commit `9a7a0291b05e1d0c0801200b659421db99a57cf0` in the current environment. None passed baseline, so later commits were not tested and no first introducing commit was established.

| Test | Baseline | First failing commit | Cause / observed failure |
|---|---|---|---|
| `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases` | FAIL | Already failing at `9a7a0291b`; origin not searched | `tests/on_update_modes.rs:181`: container remains dirty, flags **18**. `AuraPhaseCalls == 1` passes. |
| `c_system_api::test_c_console_get_all_commands_empty` | FAIL | Already failing at `9a7a0291b`; origin not searched | `tests/c_system_api.rs:66`: count **1647**, expected **0**; baseline `src/c_api/c_console.rs:11-19` enumerates the CVar catalog. |
| `tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup` | FAIL | Already failing at `9a7a0291b`; origin not searched | `tests/tooltip_item_context.rs:249`: `actual GetItemByID contract: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))`. Exact inner assertion/root cause not isolated. |
| `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | FAIL | Already failing at `9a7a0291b`; origin not searched | `tests/tooltip_text_layout.rs:180`: `assertion failed: rect.x + rect.width <= state.screen_width + 0.1`; tooltip right edge exceeds viewport. |

### Aura explanation

Test replaces real dirty phases with a handler for bit 1 only (`tests/on_update_modes.rs:162`). Cached vendor `Blizzard_AuraContainer.lua:71-78` calls `UpdateAllAuras` on show/hide; `Blizzard_ManagedAuraContainer.lua:26,46` marks `FullAuraRebuild` (1 + 2 + 16). The replacement processes bit 1, leaving **18 = ResetAuraFrames (2) + RebuildLayoutGroups (16)**. `Blizzard_SharedXML/MixinUtil.lua:415-433` processes only registered phases and reports remaining dirty flags. This explains a fixture/vendor lifecycle mismatch, not a failed phase-call count.

Original panic at `tests/on_update_modes.rs:186:6`: `called Result::unwrap() on an Err value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))`.
Diagnostic rerun: `message: "container still dirty; flags=18"`. Third assertion (OnUpdateMode) is not reached.

### Observed result lines

```text
Finished `test` profile [optimized + debuginfo] target(s) in 8m 59s
Running tests/integration.rs (/home/osso-test/.cache/wow-ui-sim-target-baseline/debug/deps/integration-8ea324359263a4d2)
test c_system_api::test_c_console_get_all_commands_empty ... FAILED
test on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases ... FAILED
test tooltip_item_context::tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup ... FAILED
test tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges ... FAILED
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 10089 filtered out; finished in 3.38s
```

Both baseline and instrumented runs exited **101**. No timeout used. The target directory shown by the test executable confirms isolated target usage despite the helper's misleading initial `target=.../worktree/target` status line.

Manual readability audit: only Lua diagnostic messages within a Rust raw string changed; conditions unchanged, no Rust control-flow changes or suppressions. Temporary copy removed with worktree.

## Cleanup

```text
git -C /home/osso-test/Projects/wow/wow-ui-sim worktree remove --force /home/osso-test/.worktrees/wow-ui-sim-baseline
exit: 0
```

Main checkout status unchanged: False. Worktree removed. No commits or pushes. Logs and separate build target retained.

Cleanup clarification: main status changed during execution and main HEAD advanced from `ce418cbe4` to `74e8f4a9c`; other worktrees also advanced/appeared. This investigation issued no main-checkout edits or checkout commands. Concurrent changes were preserved; no attempt made to restore initial main state.

Final main status:
```text
```
