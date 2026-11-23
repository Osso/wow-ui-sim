# Startup file watchdog diagnostics

The loader reports successful and failed tainted Lua-file invocations through the elapsed-time watchdog in `src/loader/lua_file.rs` and `src/lua_api/execution_budget.rs`.

## What it must do

- [ ] Apply the watchdog to each tainted top-level Lua-file invocation, with a new deadline for every file. Do not apply it to untainted secure Blizzard execution.
- [ ] VERIFIED (R88bn, both builds): check elapsed time every 10,001 VM dispatch steps and raise `script ran too long` when the threshold is exceeded; the dispatch counter is only the polling trigger, not a quota.
- [ ] INFERRED (R88bn): interpret thresholds as milliseconds. Use 19,000 ms for file loading and startup callbacks through the initial `PLAYER_ENTERING_WORLD`; use 200 ms afterwards. `WowLuaEnv` provides a test-only timeout override.
- [ ] Under the existing `WOW_SIM_LOG_HANDLER_TIMINGS` opt-in, emit one `[file-budget-success]` record for a successful invocation and `[file-budget-error]` for a failed invocation, regardless of the duration-record filter.
- [ ] Include debug-escaped owner/file identities and `timeout_ms`/`elapsed_ms`; do not add Lua arguments, payloads or error text. A file error record means execution failed, not that the watchdog caused it.
- [ ] Preserve file return values, private-table effects and error propagation.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md#top-level-file-watchdog-diagnostics)
- [Rilua watchdog contract](rilua-capability-rows.md)

## Implementation inventory

- `src/loader/lua_file.rs`: wraps tainted file execution and emits success/error records.
- `src/lua_api/execution_budget.rs`: selects the phase timeout and formats file/handler records.
- `src/lua_api/state/sim_state.rs`: tracks the initial startup phase and deterministic test override.
- `src/lua_api/env_events.rs`: ends the initial startup phase after the first `PLAYER_ENTERING_WORLD` dispatch.

## Tests asserting this spec

- `tests/rilua_rows.rs::rilua_rows_owner_watchdog_loads_file_then_rejects_dispatch_overrun`: actual addon loading below a test-set timeout, followed by a tainted frame callback over a test-set timeout.
- `src/loader/lua_file.rs::tests::startup_file_watchdog_allows_a_short_owner_invocation`: successful file effects and return value.
- `src/lua_api/execution_budget.rs::tests`: watchdog diagnostic field and escaping contracts.

## Known gaps (current cycle)

- [ ] Native timer units and the mapping of 19,000/200 values to simulator phases are inferred, not verified.
- [ ] Full-addon Retail `lua-errors` still needs to confirm whether the AllTheThings `PLAYER_LOGIN` error is gone.

## Out of scope

The separate CVar-driven script bucket throttle, its accounting period and its limits are not implemented by this watchdog. Full timer, slash-command and key-binding ownership coverage is not established here.
