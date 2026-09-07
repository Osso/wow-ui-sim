# Lua error reporting

The `lua-errors` command in `src/lua_errors.rs` reports uncaught failures collected while loading addons and dispatching startup scripts. Its process result must reflect the JSON error report, not merely successful execution of the loader.

## What it must do

- [ ] Return exit status 1 and nonempty JSON for addon chunk execution, Lua syntax, script/event, nested `C_AddOns.LoadAddOn`, and uncaught `--exec-lua` failures.
- [ ] Include messages and positive occurrence counts in JSON, retaining source context where supplied by the runtime.
- [ ] Return exit status 0 and `[]` for clean startup; errors caught and handled with `pcall` must not be reported as uncaught failures.
- [ ] Collect an uncaught `--exec-lua` error through the normal error sink before printing JSON, including when the configured Lua error handler is a no-op.

### Addon load summary

- [ ] `Failed` counts the unique union of failed addon load transactions and addons attributed uncaught Lua errors during the loading batch, including nested loads and `ADDON_LOADED` callbacks.
- [ ] `Load failures` counts failed load transactions separately; `Loaded with Lua errors` counts attributed addons whose runtime loaded state is true. An addon present in both failure sources is counted once in `Failed`.
- [ ] Preserve runtime loaded flags and the existing `Loaded` transaction count. Nested loaded addons can contribute Lua failures without being separate top-level load transactions. Earlier errors outside the batch do not inflate its summary; later startup errors remain covered by the final CLI error report.

## How it works

- [Event dispatch](../event-system.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_errors.rs` — startup execution, post-startup probe error reporting, JSON generation, and clean/error result.
- `src/bin/wow_sim/main.rs` — maps the clean/error result to process status.
- `src/bin/wow_sim/addon_loading.rs` — unique-addon failure accounting and distinct summary labels.
- `src/lua_api/script_helpers.rs` — canonical error sink and Lua error handler invocation.

## Tests asserting this spec

- `tests/lua_error_cli.rs` — three bounded process invocations using the Cargo-built simulator: combined addon failures, clean addon with handled exceptions, and uncaught post-startup execution failure. Temporary first-priority addon fixtures disable merged external addon names without changing installed addons. A stdout marker proves the intended addon reached the end of loading.

- `src/bin/wow_sim/addon_loading/tests.rs` — actual scan/load fixture with file errors, repeated event errors, a nested LoD error, and a missing dependency; asserts loaded state and exact summary output.

## Known gaps (current cycle)

- [ ] Grouped Rust tests await compilation after the observed disk-space blocker. Direct current-binary probes establish the uncaught `--exec-lua` RED: exit 0 with `[]`, despite an exception.
- [ ] Full user-addon startup acceptance remains open; summary accounting has a focused regression pending GREEN. A clean isolated fixture is not evidence that personal addons work.

## Out of scope

Changing vendor/addon error handlers, suppressing reported exceptions, or modifying API compatibility to make reporting tests pass. Existing secondary messages emitted by Blizzard's error UI remain visible; this slice does not redefine their deduplication.
