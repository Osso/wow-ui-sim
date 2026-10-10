# Startup file instruction diagnostics

Simulator diagnostics expose successful tainted Lua-file owner-budget snapshots without changing execution or quota policy. Source: `src/loader/lua_file.rs` and `src/lua_api/execution_budget.rs`; [loading architecture](../addon-loading-pipeline.md#top-level-file-budget-diagnostics) describes the path. This is diagnostic attribution, not a startup-error repair or native quota contract.

## What it must do

- [ ] Under `retail-12-0-5` and existing `WOW_SIM_LOG_HANDLER_TIMINGS` opt-in, emit one `[file-budget-success]` record per successfully executed tainted top-level Lua chunk; preserve existing `[file-budget-error]` records for failed execution. Duration threshold does not filter these records.
- [ ] Report debug-escaped owner/chunk identities, limit and cumulative `used_before`/`used_after`; label unavailable snapshots `unavailable` and unlimited limit `none`. No arguments, payloads or error text.
- [x] Preserve successful return values, private-table effects and cumulative owner-meter continuity between files: concrete sequential loader fixture passed before instrumentation.
- [ ] Preserve quota initialization, limit/reset/exemption rules, error propagation and opt-in-off behavior. Untainted files, compilation/setup failures and Rust-side work remain outside this logging boundary.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md#top-level-file-budget-diagnostics)
- [Startup evidence boundaries](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md)

## Implementation inventory

- `src/loader/lua_file.rs`: existing budgeted execution path selects success/error diagnostics without changing the result.
- `src/lua_api/execution_budget.rs`: success formatter reuses the existing counter encoding.

## Tests asserting this spec

- `loader::lua_file::tests::startup_file_budget_success_preserves_meter_continuity_returns_and_effects`: returns19/42, final private total42, nested call and meter continuity.
- Adjacent original error/exhaustion loader test and `execution_budget::tests`: preserved failure semantics, escaped metadata, unavailable/unlimited counters.
- Private saved subprocess receipt assertion: exactly two success records for actual sequential loader calls; authentic RED observed0 at12124ded6, compile0/fixture1PASS. GREEN and opt-in-off proof pending.

## Known gaps (current cycle)

- [ ] Execute current success/error/formatter controls and independent fmt/default-check proof.
- [ ] Capture one sealed full-addon startup to attribute earlier cumulative consumption; prior105errors remain unresolved.

## Out of scope

Quota changes, vendor optimization, per-line profiling, exclusive/self instruction costs, elapsed-time/Rust-work attribution and native quota/reset parity. Logged identities can contain paths; raw full-addon output remains private.
