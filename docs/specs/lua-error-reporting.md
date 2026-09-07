# Lua error reporting

The `lua-errors` command in `src/lua_errors.rs` reports uncaught failures collected while loading addons and dispatching startup scripts. Its process result must reflect the JSON error report, not merely successful execution of the loader.

## What it must do

- [ ] Return exit status 1 and nonempty JSON for addon chunk execution, Lua syntax, script/event, nested `C_AddOns.LoadAddOn`, and uncaught `--exec-lua` failures.
- [ ] Include messages and positive occurrence counts in JSON, retaining source context where supplied by the runtime.
- [ ] Return exit status 0 and `[]` for clean startup; errors caught and handled with `pcall` must not be reported as uncaught failures.
- [ ] Collect an uncaught `--exec-lua` error through the normal error sink before printing JSON, including when the configured Lua error handler is a no-op.

## How it works

- [Event dispatch](../event-system.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_errors.rs` — startup execution, post-startup probe error reporting, JSON generation, and clean/error result.
- `src/bin/wow_sim/main.rs` — maps the clean/error result to process status.
- `src/lua_api/script_helpers.rs` — canonical error sink and Lua error handler invocation.

## Tests asserting this spec

- `tests/lua_error_cli.rs` — three bounded process invocations using the Cargo-built simulator: combined addon failures, clean addon with handled exceptions, and uncaught post-startup execution failure. Temporary first-priority addon fixtures disable merged external addon names without changing installed addons. A stdout marker proves the intended addon reached the end of loading.

## Known gaps (current cycle)

- [ ] Grouped Rust tests await compilation after the observed disk-space blocker. Direct current-binary probes establish the uncaught `--exec-lua` RED: exit 0 with `[]`, despite an exception.
- [ ] Full user-addon startup acceptance and addon load-summary accounting are separate work; a clean isolated fixture is not evidence that personal addons work.

## Out of scope

Changing vendor/addon error handlers, suppressing reported exceptions, or modifying API compatibility to make reporting tests pass. Existing secondary messages emitted by Blizzard's error UI remain visible; this slice does not redefine their deduplication.
