
## Recovery integration 2026-10-04
Goal: B18–B22 surviving public API contracts; behavioral RED/GREEN and intersecting tests, formatted Rust, cargo check, coherent commits/specs. Excludes vendor/cache edits, coverage JSON, push/merge/deploy/PR, agents/models and alternate profiles/startup CLI.
git stash -u: exit 0
```
Saved working directory and index state WIP on p1207-r4: 9d40163da Authenticate 12.0.7 assets and numeric invites; model delve titles

```
git rebase master: exit 0
```

[KSuccessfully rebased and updated refs/heads/p1207-r4.

```
git stash pop: exit 0
```
Auto-merging src/c_api/mod.rs
On branch p1207-r4
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   src/c_api/mod.rs
	modified:   src/lua_api/state.rs
	modified:   src/lua_api/state/sim_state.rs
	modified:   src/lua_api/state/support_types.rs
	modified:   src/lua_api/state_types/mythic_plus_scenario.rs
	modified:   tests/c_mythic_plus_probes.rs

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	src/c_api/c_mythic_plus_calendar.rs
	tests/p1207_housing_door.rs
	tests/p1207_housing_floor.rs
	tests/p1207_housing_names.rs
	tests/p1207_merchant_currencies.rs
	tests/p1207_mythic_calendar.rs

no changes added to commit (use "git add" and/or "git commit -a")
Dropped refs/stash@{0} (daec2728287292a649386ad0c8deb086d54e86ec)

```
Step 1: recovery completed without conflicts; inspecting surviving tests/state and cached consumers.
RED p1207_housing_names::: exit 101; unchanged master producers plus survivor state/types; log /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_housing_names.log.
RED p1207_housing_door::: exit 101; unchanged producers, survivor host fields/types retained for compilation. Log /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_housing_door.log.
native source=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4 target=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']

running 3 tests
test p1207_housing_door::door_authenticates_all_three_arguments_before_any_validation ... FAILED
test p1207_housing_door::door_compatibility_distinguishes_room_component_and_type_live ... FAILED
test p1207_housing_door::door_empty_invalid_selectors_and_read_only_isolation ... FAILED

failures:

---- p1207_housing_door::door_authenticates_all_three_arguments_before_any_validation stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 2.42ms
[  0.003s] [Startup] rilua VM created in 130.37µs
[  0.003s] [Startup] builtin frames initialized in 104.03µs
[  0.003s] [Startup] template registry cleared in 8.02µs
[  0.003s] [Startup] intrinsic templates registered in 6.16µs
[  0.003s] [Startup] initial app_data lua handle installed in 387.00ns
[  0.164s] [Startup] init_lua_state complete in 161.60ms
[  0.164s] [Startup] final app_data lua handle installed in 209.00ns
[  0.164s] [Startup] initial screen globals installed in 124.71µs
[  0.164s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_door::door_authenticates_all_three_arguments_before_any_validation' (100484) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_door.rs:127:9:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- p1207_housing_door::door_compatibility_distinguishes_room_component_and_type_live stdout ----
[  0.178s] [Startup] WowLuaEnv::new begin
[  0.180s] [Startup] SimState::default complete in 1.28ms
[  0.180s] [Startup] rilua VM created in 82.59µs
[  0.180s] [Startup] builtin frames initialized in 36.92µs
[  0.180s] [Startup] template registry cleared in 3.70µs
[  0.180s] [Startup] intrinsic templates registered in 5.29µs
[  0.180s] [Startup] initial app_data lua handle installed in 519.00ns
[  0.276s] [Startup] init_lua_state complete in 96.03ms
[  0.276s] [Startup] final app_data lua handle installed in 94.00ns
[  0.276s] [Startup] initial screen globals installed in 158.56µs
[  0.276s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_door::door_compatibility_distinguishes_room_component_and_type_live' (100489) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_door.rs:41:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_housing_door::door_empty_invalid_selectors_and_read_only_isolation stdout ----
[  0.282s] [Startup] WowLuaEnv::new begin
[  0.283s] [Startup] SimState::default complete in 1.40ms
[  0.283s] [Startup] rilua VM created in 97.11µs
[  0.283s] [Startup] builtin frames initialized in 42.89µs
[  0.283s] [Startup] template registry cleared in 5.09µs
[  0.283s] [Startup] intrinsic templates registered in 7.96µs
[  0.283s] [Startup] initial app_data lua handle installed in 621.00ns
[  0.384s] [Startup] init_lua_state complete in 100.57ms
[  0.384s] [Startup] final app_data lua handle installed in 88.00ns
[  0.384s] [Startup] initial screen globals installed in 119.31µs
[  0.384s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_door::door_empty_invalid_selectors_and_read_only_isolation' (100494) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_door.rs:75:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))


failures:
    p1207_housing_door::door_authenticates_all_three_arguments_before_any_validation
    p1207_housing_door::door_compatibility_distinguishes_room_component_and_type_live
    p1207_housing_door::door_empty_invalid_selectors_and_read_only_isolation

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.39s


RED p1207_housing_floor::: exit 101; unchanged producers, survivor host fields/types retained for compilation. Log /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_housing_floor.log.
native source=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4 target=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']

running 3 tests
test p1207_housing_floor::floor_missing_malformed_and_independent_state ... FAILED
test p1207_housing_floor::floor_permissions_are_explicit_live_and_read_only ... FAILED
test p1207_housing_floor::floor_secret_selector_and_extra_authentication_precede_miss ... FAILED

failures:

---- p1207_housing_floor::floor_missing_malformed_and_independent_state stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 2.05ms
[  0.002s] [Startup] rilua VM created in 115.80µs
[  0.002s] [Startup] builtin frames initialized in 105.62µs
[  0.002s] [Startup] template registry cleared in 15.84µs
[  0.002s] [Startup] intrinsic templates registered in 9.11µs
[  0.002s] [Startup] initial app_data lua handle installed in 550.00ns
[  0.155s] [Startup] init_lua_state complete in 152.68ms
[  0.155s] [Startup] final app_data lua handle installed in 201.00ns
[  0.155s] [Startup] initial screen globals installed in 171.68µs
[  0.155s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_floor::floor_missing_malformed_and_independent_state' (100831) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_floor.rs:67:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- p1207_housing_floor::floor_permissions_are_explicit_live_and_read_only stdout ----
[  0.163s] [Startup] WowLuaEnv::new begin
[  0.165s] [Startup] SimState::default complete in 1.76ms
[  0.165s] [Startup] rilua VM created in 116.69µs
[  0.165s] [Startup] builtin frames initialized in 73.64µs
[  0.165s] [Startup] template registry cleared in 5.21µs
[  0.165s] [Startup] intrinsic templates registered in 7.99µs
[  0.165s] [Startup] initial app_data lua handle installed in 605.00ns
[  0.289s] [Startup] init_lua_state complete in 123.38ms
[  0.289s] [Startup] final app_data lua handle installed in 638.00ns
[  0.289s] [Startup] initial screen globals installed in 171.57µs
[  0.289s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_floor::floor_permissions_are_explicit_live_and_read_only' (100834) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_floor.rs:33:10:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_housing_floor::floor_secret_selector_and_extra_authentication_precede_miss stdout ----
[  0.296s] [Startup] WowLuaEnv::new begin
[  0.299s] [Startup] SimState::default complete in 3.09ms
[  0.299s] [Startup] rilua VM created in 120.51µs
[  0.299s] [Startup] builtin frames initialized in 77.60µs
[  0.299s] [Startup] template registry cleared in 7.84µs
[  0.299s] [Startup] intrinsic templates registered in 8.19µs
[  0.299s] [Startup] initial app_data lua handle installed in 762.00ns
[  0.414s] [Startup] init_lua_state complete in 114.60ms
[  0.414s] [Startup] final app_data lua handle installed in 435.00ns
[  0.414s] [Startup] initial screen globals installed in 132.18µs
[  0.414s] [Startup] WowLuaEnv::new complete

thread 'p1207_housing_floor::floor_secret_selector_and_extra_authentication_precede_miss' (100835) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_housing_floor.rs:106:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))


failures:
    p1207_housing_floor::floor_missing_malformed_and_independent_state
    p1207_housing_floor::floor_permissions_are_explicit_live_and_read_only
    p1207_housing_floor::floor_secret_selector_and_extra_authentication_precede_miss

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.42s


RED p1207_merchant_currencies::: exit 101; unchanged producers, survivor host fields/types retained for compilation. Log /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_merchant_currencies.log.
native source=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4 target=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']

running 3 tests
test p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids ... FAILED
test p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments ... FAILED
test p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached ... FAILED

failures:

---- p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.003s] [Startup] SimState::default complete in 2.80ms
[  0.003s] [Startup] rilua VM created in 138.65µs
[  0.003s] [Startup] builtin frames initialized in 139.85µs
[  0.003s] [Startup] template registry cleared in 13.22µs
[  0.003s] [Startup] intrinsic templates registered in 7.32µs
[  0.003s] [Startup] initial app_data lua handle installed in 1.05µs
[  0.280s] [Startup] init_lua_state complete in 276.41ms
[  0.280s] [Startup] final app_data lua handle installed in 296.00ns
[  0.280s] [Startup] initial screen globals installed in 218.86µs
[  0.280s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids' (101353) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:70:10:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "(string):8: attempt to index global 'MenuUtil' (a nil value)", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments stdout ----
[  0.296s] [Startup] WowLuaEnv::new begin
[  0.297s] [Startup] SimState::default complete in 1.67ms
[  0.297s] [Startup] rilua VM created in 84.66µs
[  0.297s] [Startup] builtin frames initialized in 42.09µs
[  0.297s] [Startup] template registry cleared in 5.21µs
[  0.297s] [Startup] intrinsic templates registered in 6.93µs
[  0.297s] [Startup] initial app_data lua handle installed in 636.00ns
[  0.482s] [Startup] init_lua_state complete in 184.41ms
[  0.482s] [Startup] final app_data lua handle installed in 426.00ns
[  0.482s] [Startup] initial screen globals installed in 343.13µs
[  0.482s] [Startup] WowLuaEnv::new complete
[  0.482s] [Startup] WowLuaEnv::new begin
[  0.485s] [Startup] SimState::default complete in 3.24ms
[  0.486s] [Startup] rilua VM created in 180.26µs
[  0.486s] [Startup] builtin frames initialized in 84.45µs
[  0.486s] [Startup] template registry cleared in 11.78µs
[  0.486s] [Startup] intrinsic templates registered in 8.06µs
[  0.486s] [Startup] initial app_data lua handle installed in 1.44µs
[  0.621s] [Startup] init_lua_state complete in 135.09ms
[  0.621s] [Startup] final app_data lua handle installed in 261.00ns
[  0.621s] [Startup] initial screen globals installed in 146.03µs
[  0.621s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments' (101354) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:60:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached stdout ----
[  0.641s] [Startup] WowLuaEnv::new begin
[  0.644s] [Startup] SimState::default complete in 3.29ms
[  0.644s] [Startup] rilua VM created in 91.97µs
[  0.644s] [Startup] builtin frames initialized in 48.20µs
[  0.644s] [Startup] template registry cleared in 5.91µs
[  0.644s] [Startup] intrinsic templates registered in 8.11µs
[  0.644s] [Startup] initial app_data lua handle installed in 696.00ns
[  0.764s] [Startup] init_lua_state complete in 119.84ms
[  0.764s] [Startup] final app_data lua handle installed in 254.00ns
[  0.764s] [Startup] initial screen globals installed in 165.12µs
[  0.764s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached' (101355) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:23:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))


failures:
    p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids
    p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments
    p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.77s


RED p1207_mythic_calendar::: exit 101; unchanged producers, survivor host fields/types retained for compilation. Log /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_mythic_calendar.log.
native source=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4 target=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']

running 5 tests
test p1207_mythic_calendar::calendar_authenticates_every_flag_map_and_extra_before_validation ... FAILED
test p1207_mythic_calendar::history_calendar_fields_are_distinct_live_and_detached ... FAILED
test p1207_mythic_calendar::missing_dates_are_explicit_data_gaps_not_fabricated_calendar_values ... FAILED
test p1207_mythic_calendar::season_in_time_overtime_dates_and_nil_sides_are_live ... FAILED
test p1207_mythic_calendar::weekly_calendar_uses_six_declared_tuple_slots_live ... FAILED

failures:

---- p1207_mythic_calendar::calendar_authenticates_every_flag_map_and_extra_before_validation stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.004s] [Startup] SimState::default complete in 4.36ms
[  0.005s] [Startup] rilua VM created in 174.00µs
[  0.005s] [Startup] builtin frames initialized in 144.60µs
[  0.005s] [Startup] template registry cleared in 12.88µs
[  0.005s] [Startup] intrinsic templates registered in 7.88µs
[  0.005s] [Startup] initial app_data lua handle installed in 771.00ns
[  0.373s] [Startup] init_lua_state complete in 367.85ms
[  0.373s] [Startup] final app_data lua handle installed in 190.00ns
[  0.373s] [Startup] initial screen globals installed in 228.91µs
[  0.373s] [Startup] WowLuaEnv::new complete

thread 'p1207_mythic_calendar::calendar_authenticates_every_flag_map_and_extra_before_validation' (102058) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_mythic_calendar.rs:329:9:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "expected number, got userdata at argument 1", level: 1, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- p1207_mythic_calendar::history_calendar_fields_are_distinct_live_and_detached stdout ----
[  0.401s] [Startup] WowLuaEnv::new begin
[  0.403s] [Startup] SimState::default complete in 1.61ms
[  0.403s] [Startup] rilua VM created in 99.82µs
[  0.403s] [Startup] builtin frames initialized in 55.20µs
[  0.403s] [Startup] template registry cleared in 7.60µs
[  0.403s] [Startup] intrinsic templates registered in 9.52µs
[  0.403s] [Startup] initial app_data lua handle installed in 1.45µs
[  0.633s] [Startup] init_lua_state complete in 230.14ms
[  0.633s] [Startup] final app_data lua handle installed in 168.00ns
[  0.634s] [Startup] initial screen globals installed in 187.72µs
[  0.634s] [Startup] WowLuaEnv::new complete

thread 'p1207_mythic_calendar::history_calendar_fields_are_distinct_live_and_detached' (102063) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_mythic_calendar.rs:135:9:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_mythic_calendar::missing_dates_are_explicit_data_gaps_not_fabricated_calendar_values stdout ----
[  0.643s] [Startup] WowLuaEnv::new begin
[  0.644s] [Startup] SimState::default complete in 1.34ms
[  0.644s] [Startup] rilua VM created in 88.74µs
[  0.644s] [Startup] builtin frames initialized in 56.43µs
[  0.644s] [Startup] template registry cleared in 5.01µs
[  0.644s] [Startup] intrinsic templates registered in 7.02µs
[  0.644s] [Startup] initial app_data lua handle installed in 569.00ns
[  0.784s] [Startup] init_lua_state complete in 139.51ms
[  0.784s] [Startup] final app_data lua handle installed in 136.00ns
[  0.784s] [Startup] initial screen globals installed in 171.92µs
[  0.784s] [Startup] WowLuaEnv::new complete

thread 'p1207_mythic_calendar::missing_dates_are_explicit_data_gaps_not_fabricated_calendar_values' (102066) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_mythic_calendar.rs:266:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_mythic_calendar::season_in_time_overtime_dates_and_nil_sides_are_live stdout ----
[  0.791s] [Startup] WowLuaEnv::new begin
[  0.793s] [Startup] SimState::default complete in 1.72ms
[  0.793s] [Startup] rilua VM created in 86.50µs
[  0.793s] [Startup] builtin frames initialized in 52.94µs
[  0.793s] [Startup] template registry cleared in 5.50µs
[  0.793s] [Startup] intrinsic templates registered in 6.18µs
[  0.793s] [Startup] initial app_data lua handle installed in 417.00ns
[  0.963s] [Startup] init_lua_state complete in 170.38ms
[  0.963s] [Startup] final app_data lua handle installed in 261.00ns
[  0.964s] [Startup] initial screen globals installed in 200.22µs
[  0.964s] [Startup] WowLuaEnv::new complete

thread 'p1207_mythic_calendar::season_in_time_overtime_dates_and_nil_sides_are_live' (102074) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_mythic_calendar.rs:217:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_mythic_calendar::weekly_calendar_uses_six_declared_tuple_slots_live stdout ----
[  0.973s] [Startup] WowLuaEnv::new begin
[  0.975s] [Startup] SimState::default complete in 2.15ms
[  0.976s] [Startup] rilua VM created in 135.36µs
[  0.976s] [Startup] builtin frames initialized in 65.14µs
[  0.976s] [Startup] template registry cleared in 8.29µs
[  0.976s] [Startup] intrinsic templates registered in 10.58µs
[  0.976s] [Startup] initial app_data lua handle installed in 977.00ns
[  1.168s] [Startup] init_lua_state complete in 191.96ms
[  1.168s] [Startup] final app_data lua handle installed in 229.00ns
[  1.168s] [Startup] initial screen globals installed in 149.46µs
[  1.168s] [Startup] WowLuaEnv::new complete

thread 'p1207_mythic_calendar::weekly_calendar_uses_six_declared_tuple_slots_live' (102076) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_mythic_calendar.rs:168:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))


failures:
    p1207_mythic_calendar::calendar_authenticates_every_flag_map_and_extra_before_validation
    p1207_mythic_calendar::history_calendar_fields_are_distinct_live_and_detached
    p1207_mythic_calendar::missing_dates_are_explicit_data_gaps_not_fabricated_calendar_values
    p1207_mythic_calendar::season_in_time_overtime_dates_and_nil_sides_are_live
    p1207_mythic_calendar::weekly_calendar_uses_six_declared_tuple_slots_live

test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 10385 filtered out; finished in 1.18s


Step 2: all five RED modules compiled and ran: names 0/3, door 0/3, floor 0/3, merchant 0/3, calendar 0/5. Wrapper test failed on missing MenuUtil prerequisite, not producer; repair fixture and reprove before implementation. Names consumer assumes consistent category labels for referenced entry subcategory IDs; empty catalog default has no such entries. Door/floor false and merchant empty table match guarded consumers.
Merchant fixture corrected using real cached MenuUtil; ordinary global lookup checks retirement before wrapper. RED repeat exit 101; /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_merchant_currencies-fixture.log
native source=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4 target=/home/osso-test/.worktrees/wow-ui-sim-p1207-r4/target scope=['/home/osso-test/Projects/world-of-osso/game-engine/scripts/agent/agent-run', 'native-build']

running 3 tests
test p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids ... FAILED
test p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments ... FAILED
test p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached ... FAILED

failures:

---- p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids stdout ----
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.004s] [Startup] SimState::default complete in 4.15ms
[  0.004s] [Startup] rilua VM created in 138.17µs
[  0.004s] [Startup] builtin frames initialized in 124.00µs
[  0.005s] [Startup] template registry cleared in 8.87µs
[  0.005s] [Startup] intrinsic templates registered in 6.39µs
[  0.005s] [Startup] initial app_data lua handle installed in 518.00ns
[  0.192s] [Startup] init_lua_state complete in 187.91ms
[  0.192s] [Startup] final app_data lua handle installed in 327.00ns
[  0.193s] [Startup] initial screen globals installed in 230.85µs
[  0.193s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids' (109257) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:71:10:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "(string):260: attempt to index global 'MenuTemplates' (a nil value)", level: 0, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments stdout ----
[  0.205s] [Startup] WowLuaEnv::new begin
[  0.207s] [Startup] SimState::default complete in 1.63ms
[  0.207s] [Startup] rilua VM created in 92.90µs
[  0.207s] [Startup] builtin frames initialized in 47.95µs
[  0.207s] [Startup] template registry cleared in 5.96µs
[  0.207s] [Startup] intrinsic templates registered in 7.71µs
[  0.207s] [Startup] initial app_data lua handle installed in 462.00ns
[  0.365s] [Startup] init_lua_state complete in 157.94ms
[  0.365s] [Startup] final app_data lua handle installed in 115.00ns
[  0.365s] [Startup] initial screen globals installed in 129.11µs
[  0.365s] [Startup] WowLuaEnv::new complete
[  0.365s] [Startup] WowLuaEnv::new begin
[  0.367s] [Startup] SimState::default complete in 1.82ms
[  0.367s] [Startup] rilua VM created in 88.76µs
[  0.367s] [Startup] builtin frames initialized in 42.52µs
[  0.367s] [Startup] template registry cleared in 7.07µs
[  0.367s] [Startup] intrinsic templates registered in 5.26µs
[  0.367s] [Startup] initial app_data lua handle installed in 612.00ns
[  0.486s] [Startup] init_lua_state complete in 118.86ms
[  0.486s] [Startup] final app_data lua handle installed in 147.00ns
[  0.486s] [Startup] initial screen globals installed in 129.89µs
[  0.486s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments' (109264) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:60:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))

---- p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached stdout ----
[  0.499s] [Startup] WowLuaEnv::new begin
[  0.501s] [Startup] SimState::default complete in 1.93ms
[  0.502s] [Startup] rilua VM created in 87.32µs
[  0.502s] [Startup] builtin frames initialized in 43.95µs
[  0.502s] [Startup] template registry cleared in 4.54µs
[  0.502s] [Startup] intrinsic templates registered in 6.86µs
[  0.502s] [Startup] initial app_data lua handle installed in 474.00ns
[  0.608s] [Startup] init_lua_state complete in 106.37ms
[  0.608s] [Startup] final app_data lua handle installed in 286.00ns
[  0.608s] [Startup] initial screen globals installed in 136.86µs
[  0.608s] [Startup] WowLuaEnv::new complete

thread 'p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached' (109272) panicked at /home/osso-test/.worktrees/wow-ui-sim-p1207-r4/tests/p1207_merchant_currencies.rs:23:6:
called `Result::unwrap()` on an `Err` value: Lua(Runtime(RuntimeError { message: "assertion failed!", level: 0, traceback: [] }))


failures:
    p1207_merchant_currencies::merchant_cached_deprecated_wrapper_unpacks_exact_host_ids
    p1207_merchant_currencies::merchant_empty_defaults_and_independent_environments
    p1207_merchant_currencies::merchant_ids_are_live_ordered_and_detached

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.61s


Step 3 housing implemented and formatted; staged only housing plus shared authentication boundary. [p1207-r4 093efa28c] Model live 12.0.7 housing names and permissions
 9 files changed, 551 insertions(+), 3 deletions(-)
 create mode 100644 docs/specs/retail-12-0-7-housing-queries.md
 create mode 100644 src/c_api/c_housing/patch_12_0_7.rs
 create mode 100644 src/c_api/patch_12_0_7_inputs.rs
 create mode 100644 tests/p1207_housing_door.rs
 create mode 100644 tests/p1207_housing_floor.rs
 create mode 100644 tests/p1207_housing_names.rs

GREEN p1207_housing_names:: at housing commit 093efa28c plus unchanged survivor calendar/merchant state and corrected merchant tests: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_housing_names.log
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10388 filtered out; finished in 0.45s
GREEN p1207_housing_door:: at housing commit 093efa28c plus unchanged survivor calendar/merchant state and corrected merchant tests: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_housing_door.log
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10388 filtered out; finished in 0.39s
GREEN p1207_housing_floor:: at housing commit 093efa28c plus unchanged survivor calendar/merchant state and corrected merchant tests: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_housing_floor.log
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10388 filtered out; finished in 0.36s
RED p1207_merchant_currencies:: at housing commit 093efa28c plus unchanged survivor calendar/merchant state and corrected merchant tests: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/r4-RED-p1207_merchant_currencies-corrected.log
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.42s
Step 4: merchant corrected RED 0/4 now behavioral (real cached wrapper declaration alone, no fake prerequisite); producer reads host order and rejects secret extras. [p1207-r4 ea6c68d3d] Read live merchant currency snapshots
 5 files changed, 188 insertions(+), 3 deletions(-)
 create mode 100644 docs/specs/retail-12-0-7-merchant-currencies.md
 create mode 100644 tests/p1207_merchant_currencies.rs

Step 5: CalendarTime producers implemented with exact six-field dates and tuple ordering; rooted nested snapshots; missing-date behavior INFERRED; existing history filtering intentionally unchanged. [p1207-r4 42140d3f6] Encode live Mythic+ CalendarTime results
 7 files changed, 594 insertions(+), 2 deletions(-)
 create mode 100644 docs/specs/retail-12-0-7-mythic-calendar.md
 create mode 100644 src/c_api/c_mythic_plus_calendar.rs
 create mode 100644 tests/p1207_mythic_calendar.rs

GREEN p1207_merchant_currencies:: at 42140d3f6: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_merchant_currencies.log
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10387 filtered out; finished in 0.80s
GREEN p1207_mythic_calendar:: at 42140d3f6: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_mythic_calendar.log
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10386 filtered out; finished in 0.83s
GREEN c_mythic_plus_probes:: at 42140d3f6: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-c_mythic_plus_probes.log
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 10370 filtered out; finished in 2.75s
GREEN housing_category_search:: at 42140d3f6: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-housing_category_search.log
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 10379 filtered out; finished in 1.73s
GREEN patch_12_0_7_removed_native_surface:: at 42140d3f6: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-patch_12_0_7_removed_native_surface.log
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 10361 filtered out; finished in 12.17s
Step 6: merchant/calendar GREEN 9/9; intersecting Mythic+ 21/21, housing category/search 12/12 and removed native surface 30/30. Readability self-audit: live producers wired; no new suppressions, nested-query depth <=2. Calendar season encoder trimmed shared numeric fields to keep function bounded. No independent agents allowed by task; verification direct.
Step 7: manual Rust readability audit completed (metrics tool absent); season numeric encoder consolidated, batch labels corrected and passed spec requirements checked. [p1207-r4 60965d6e8] Record bounded query proof and simplify season encoding
 6 files changed, 24 insertions(+), 25 deletions(-)

Housing/merchant proof preserved: only comment/spec changes since respective GREEN; calendar producer proof invalidated by equivalent numeric-field loop pending targeted repeat.
GREEN p1207_mythic_calendar:: at 60965d6e8 after numeric encoding change: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/r4-GREEN-p1207_mythic_calendar-final.log
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10386 filtered out; finished in 1.91s
fmt-check at 60965d6e8: exit 0; full output /home/osso-test/.cache/wow-ui-sim-audit/r4-fmt-check.log
cargo-check at 60965d6e8: exit 0; full output /home/osso-test/.cache/wow-ui-sim-audit/r4-cargo-check.log

## Final integration evidence

Final revision: `60965d6e81aed58715278fe5c6e6e9eb028d2df7`. Rebased base: `63b32995d0a7d5336fac89866dc73f4902161297`. Worktree clean on p1207-r4. No push/merge/deploy/PR or model/agent invocation. Coverage JSON unchanged: True. No prohibited vendor/cache paths changed.

### Commits and exact changed files
```
commit 60965d6e81aed58715278fe5c6e6e9eb028d2df7
Author: Alessio Deiana <adeiana@gmail.com>
Commit: Alessio Deiana <adeiana@gmail.com>

    Record bounded query proof and simplify season encoding
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

docs/specs/retail-12-0-7-housing-queries.md
docs/specs/retail-12-0-7-merchant-currencies.md
docs/specs/retail-12-0-7-mythic-calendar.md
src/c_api/c_housing/patch_12_0_7.rs
src/c_api/c_mythic_plus_calendar.rs
src/c_api/patch_12_0_7_inputs.rs
```
```
commit 42140d3f6f12b3a352463735f369c55ecdc8bd20
Author: Alessio Deiana <adeiana@gmail.com>
Commit: Alessio Deiana <adeiana@gmail.com>

    Encode live Mythic+ CalendarTime results
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

docs/specs/retail-12-0-7-mythic-calendar.md
src/c_api/c_mythic_plus_calendar.rs
src/c_api/mod.rs
src/lua_api/globals/missing_surface/mythic_plus.rs
src/lua_api/state_types/mythic_plus_scenario.rs
tests/c_mythic_plus_probes.rs
tests/p1207_mythic_calendar.rs
```
```
commit ea6c68d3dbe943a8c5a926e27b72ef31313085e0
Author: Alessio Deiana <adeiana@gmail.com>
Commit: Alessio Deiana <adeiana@gmail.com>

    Read live merchant currency snapshots
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

docs/specs/retail-12-0-7-merchant-currencies.md
src/c_api/c_merchant_frame.rs
src/lua_api/state.rs
src/lua_api/state/sim_state.rs
tests/p1207_merchant_currencies.rs
```
```
commit 093efa28c5acd4ab28717c0b49ec7d02aa57efe9
Author: Alessio Deiana <adeiana@gmail.com>
Commit: Alessio Deiana <adeiana@gmail.com>

    Model live 12.0.7 housing names and permissions
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

docs/specs/retail-12-0-7-housing-queries.md
src/c_api/c_housing.rs
src/c_api/c_housing/patch_12_0_7.rs
src/c_api/mod.rs
src/c_api/patch_12_0_7_inputs.rs
src/lua_api/state/support_types.rs
tests/p1207_housing_door.rs
tests/p1207_housing_floor.rs
tests/p1207_housing_names.rs
```

### Proof ledger

All tests used explicit worktree cwd and exactly one filter per invocation: `CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b102 --test integration FILTER -- --test-threads=1`. Output captured once, persisted then read. Never concurrent builds. Actual integration executable in logs points to exclusive b102 target despite generic runner banner mentioning repo target. No undefined-symbol link failure; no clean required.

| Filter | RED pass/fail | GREEN pass/fail | Revision/scope and evidence |
|---|---:|---:|---|
| p1207_housing_names:: | 0/3 | 3/0 | 093efa28c host+producer slice; later comments/specs only. r4-RED-p1207_housing_names.log; r4-GREEN-p1207_housing_names.log |
| p1207_housing_door:: | 0/3 | 3/0 | 093efa28c; later comments/specs only. r4-RED-p1207_housing_door.log; r4-GREEN-p1207_housing_door.log |
| p1207_housing_floor:: | 0/3 | 3/0 | 093efa28c; later comments/specs only. r4-RED-p1207_housing_floor.log; r4-GREEN-p1207_housing_floor.log |
| p1207_merchant_currencies:: | 0/4 | 4/0 | Corrected RED at housing slice; GREEN 42140d3f6, merchant producer unchanged later. r4-RED-p1207_merchant_currencies-corrected.log; r4-GREEN-p1207_merchant_currencies.log |
| p1207_mythic_calendar:: | 0/5 | 5/0 | Original RED with unchanged producers/types only; GREEN 42140d3f6; numeric-loop change invalidated that season proof, renewed at 60965d6e8. r4-GREEN-p1207_mythic_calendar-final.log |
| c_mythic_plus_probes:: | not run | 21/0 | 42140d3f6. Later season-only encoding change does not intersect this module (no season-best calls); proof retained. r4-GREEN-c_mythic_plus_probes.log |
| housing_category_search:: | not run | 12/0 | 42140d3f6, no later intersecting behavior change. r4-GREEN-housing_category_search.log |
| patch_12_0_7_removed_native_surface:: | not run | 30/0 | 42140d3f6, no later intersecting behavior change. r4-GREEN-patch_12_0_7_removed_native_surface.log |

Original survivors: 17 cases; all captured RED. Added merchant secret-extra case yields 18 new cases GREEN. 63 intersecting existing cases GREEN; 81 unique tests passed. Initial merchant wrapper RED and first fixture repair failed on MenuUtil/MenuTemplates prerequisites, not a producer failure; not counted as behavioral producer proof. Final fixture executes actual cached GetMerchantCurrencies declaration only and fails behaviorally against old empty producer. Full deprecated addon/CVar-gate proof is not claimed by that new test; existing migration module supplies bounded separate proof.

`cargo fmt` exit 0 before commits; final `cargo fmt --check` exit 0 (`r4-fmt-check.log`), `CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b102 --jobs 4` exit 0 (`r4-cargo-check.log`) at 60965d6e8. Full logs inspected: no warnings/errors on GREEN/check. `git diff --check` exit 0. Manual Rust readability review done; metrics CLI absent. No new warning suppression, source-substring behavioral assertions, runtime fallbacks or shims.

### Reconstructed survivor contracts / producer gaps at recovery

| Survivor module | Missing/incomplete original producer | Recovered implementation |
|---|---|---|
| p1207_housing_names | c_housing returned one nil, no relation/label lookup | existing category/subcategory maps; live parent relation and two labels |
| p1207_housing_door | constant false | empty host (roomGUID, componentID) -> supported-type set |
| p1207_housing_floor | constant false | empty host signed-floor -> permission map |
| p1207_merchant_currencies | constant empty table; survivor Vec unused | ordered host IDs, one fresh table; raw global retirement retained |
| p1207_mythic_calendar | history omitted dates; weekly old nil/fraction tuple; season member fabricated by namespace __index | survivor CalendarTime/member types + explicit optional dates, affixes/members and independent season best sides; three live C API producers |

### Per-source-row scope and recommended status

Statuses below are recommendations, NOT ledger edits or native-client/full-page acceptance. No independent agent/model gate ran, as explicitly prohibited. Five requested rows conservatively remain development-qualified until audit owner accepts cached/historical qualifications.

| Source row ID | Proven bounded scope | Unproved scope | Recommended status |
|---|---|---|---|
| global api-C_HousingCatalog-GetCatalogCategoryAndSubcategoryNames-034 (B18) | distinct host subcategories resolve live parent labels; exactly two public strings; zero-result misses, missing subcategory label, removed parent, malformed inputs; independent environments; AllowedWhenUntainted selector and secret extras authenticate before validation | native/localized labels; historical signature/policy provenance; missing category-label case separately; consistency of entry subcategoryIDs with labels and actual tooltip interaction; off-feature lookup absence | partial-development-green |
| global api-C_MerchantFrame-GetMerchantCurrencies-037 (B19) | ordered two-ID/one-ID/empty live arrays, exact one-table arity, detached snapshots, isolation; public extras ignored, secret extras rejected for both callers; real cached wrapper unpacks exact IDs; retired raw global nil by ordinary lookup | native currency/order derivation; absent merchant MayReturnNothing; historical input-secret policy (INFERRED without annotation); full deprecated addon prerequisites/CVar gate through new populated producer; off-feature lookup absence | partial-development-green |
| global api-C_HousingCustomizeMode-RoomConnectionSupportsDoorType-035 (B20) | room/component/type distinctions; populated supported sets, live change/removal, empty false; read-only/isolation; three secret selectors + extras authenticated before invalid/missing selectors; public one-boolean result | geometric/service compatibility; roomGUID validation beyond string; ownership/editor state; native/historical policy; other-profile publication | partial-development-green |
| global api-C_HousingLayout-CanSetViewedFloor-036 (B21) | explicit negative/zero/positive floor permissions, live changes/removal, unknown/malformed false; read-only/isolation; secret selector/extras authenticate before validation; exact public boolean | ownership/mode/availability derivation; actual SetViewedFloor coupling/editor interaction; native/historical policy; other-profile publication | partial-development-green |
| prose-undated-016 (B22) | all THREE producers expose exact six-field CalendarTime with distinct leap-day/other dates; history metadata, corrected six-slot weekly tuple and two season-side records; live changes/removal, nested snapshot detachment, nil/undated/malformed policy, isolation, all flag/map/extra secret authentication and GC/taint preservation | authenticated 12.0.7 pinned CalendarTime fields; native indexing/timezone/date ranges; history filters (unchanged, ignored); native data derivation; no older-profile parity | partial-development-green |
| global api-GetMerchantCurrencies-071 (related removal) | ordinary global lookup absent before cached wrapper; existing native-removal module 30/30, including disabled cached migrations and loaded game-UI path; real wrapper restores exact host IDs in new fixture | universal addon load permutations and native-client removal evidence | bounded-coverage (absence/migration subset only, master behavior retained) |
| deprecated-api-175 (combined mapping row) | merchant mapping executes actual cached wrapper with populated/empty producer; intersecting existing migration tests pass | no NEW end-to-end producer proof for the other click-binding/spell mappings; new fixture does not exercise whole deprecated file or CVar guard | partial-development-green (merchant subset only; no full-row credit) |

### Exclusions and cached-consumer evidence

No requested producer excluded: empty housing/catalog state carries no references requiring labels; false door/floor results are consumed as booleans; empty merchant table and empty Mythic+ history/nil best results are accepted by inspected consumers. Exclude NIL merchant default: cached MerchantFrame.lua:972 uses `#currencies` without a nil guard, and Deprecated_12_0_7.lua:30 uses `unpack(C_MerchantFrame.GetMerchantCurrencies())`. Returning one empty table preserves these consumers; native MayReturnNothing behavior is not claimed.

HousingCatalogUIDocumentation.lua:97 states: "If found, returns the names of the parent category and the specified subcategory". Blizzard_HousingTemplates/Blizzard_HousingCatalogEntry.lua:532–533 passes those names directly to string.gmatch. Missing records/labels with an explicitly supplied catalog entry can therefore break that tooltip; no fallback labels added, no tooltip-safety claim for inconsistent host input. Empty default has no such catalog entries.

Blizzard_HouseEditorLayoutMode.lua:317–318 uses CanSetViewedFloor to enable buttons; Blizzard_HouseEditorCustomizationRoomTemplates.lua:248 uses door support as a predicate. Blizzard_ChallengesUI.lua guards absent best results; weekly rewards iterate history arrays. Completion-less host records are intentionally unavailable by INFERRED policy, not synthetic dates. Cached TimeDocumentation.lua supplies six fields, not authenticated historical/native execution.

No history-filter implementation, housing geometry/3D/service, currency service, date arithmetic/validation/timezone, startup CLI, alternate feature set, unfiltered suite or coverage JSON mutation. Known pre-existing failing lib duration/startup, c_api_surface shim assertions and C_Console empty-command test were not run and not changed. No new stubs or placeholder credit. Consumer inspection is read-only evidence, not GUI/native proof.

### Merge risk today

**Moderate, bounded development green.** Four coherent local commits, clean worktree, 81 unique focused tests GREEN, final fmt/check clean. No observed failures in authorized scope. Risks: INFERRED invalid/missing/secret-default policy; later-cache signatures may not exactly match 12.0.7; configured undated Mythic+ records disappear from results; weekly tuple now follows cached CalendarTime shape rather than old fraction slot; inconsistent housing host references may break tooltip. Filters remain ignored and full startup/native/older-profile compatibility is not certified. Accept only explicit-state/query coverage, not complete client or source-page compatibility. No push, merge or deployment performed.

Step 8 complete: final SHAs/files, RED/GREEN counts, exclusions, source-row matrix and merge risk recorded; no further authorized implementation work outstanding.
