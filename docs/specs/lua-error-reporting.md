# Lua error reporting

The `lua-errors` command in `src/lua_errors.rs` reports uncaught failures collected while loading addons and dispatching startup scripts. Its process result must reflect the JSON error report, not merely successful execution of the loader.

## What it must do

- [x] Return exit status 1 and nonempty JSON for addon chunk execution, Lua syntax, script/event, nested `C_AddOns.LoadAddOn`, and uncaught `--exec-lua` failures.
- [x] Include messages and positive occurrence counts in JSON, retaining source context where supplied by the runtime.
- [x] Return exit status 0 and `[]` for clean startup; errors caught and handled with `pcall` must not be reported as uncaught failures.
- [x] Collect an uncaught `--exec-lua` error through the normal error sink before printing JSON, including when the configured Lua error handler is a no-op.

### Addon load summary

- [x] `Failed during loading` counts the unique union of failed addon load transactions and addons attributed uncaught Lua errors during the loading batch, including nested loads and `ADDON_LOADED` callbacks.
- [x] `Load failures` counts failed load transactions separately; `Loaded with Lua errors during loading` counts attributed addons whose runtime loaded state is true. An addon present in both failure sources is counted once in `Failed during loading`.
- [x] Preserve runtime loaded flags and the existing `Loaded` transaction count. Nested loaded addons can contribute Lua failures without being separate top-level load transactions.

The loading summary is explicitly labeled `before startup events`; its counts cover only records added during the loading batch, not later `PLAYER_LOGIN`, update ticks, or `--exec-lua` failures.

### Final observed Lua error summary

- [x] Always print a final summary to stderr after startup events, update ticks, and optional `--exec-lua` plus its follow-up ticks.
- [x] Report `CLEAN` or `FAILED`, unique JSON error count, and total JSON occurrence count without changing JSON stdout or exit status.
- [x] Count existing attributed owner names and their occurrences separately from unattributed occurrences. Synthetic owners such as `__BuiltIn` remain owners, not third-party addons; do not infer ownership from the last loaded addon.
- [x] Keep loading-transaction outcomes separate from final observed Lua errors; this does not redefine load accounting.

## How it works

- [Event dispatch](../event-system.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_errors.rs` — startup execution, post-startup probe error reporting, final stderr summary, JSON generation, and clean/error result.
- `src/bin/wow_sim/main.rs` — maps the clean/error result to process status.
- `src/bin/wow_sim/addon_loading.rs` — unique-addon failure accounting and distinct summary labels.
- `src/lua_api/script_helpers.rs` — canonical error sink and Lua error handler invocation.

## Tests asserting this spec

- `tests/lua_error_cli.rs` — four bounded process invocations using the Cargo-built simulator: combined addon failures, clean addon with handled exceptions, uncaught exec failure, and loading-success followed by event/startup-update/exec-update errors. Final-summary assertions cover clean/error status, counts, explicit owners, and unattributed errors. The deferred fixture installs its own no-op error handler to isolate canonical failures from secondary Blizzard error-UI messages. Temporary first-priority fixtures disable merged external addon names without changing installed addons; existing load markers remain intact.

- `src/bin/wow_sim/addon_loading/tests.rs` — actual scan/load fixture with file errors, repeated event errors, a nested LoD error, and a missing dependency; asserts loaded state and exact summary output.

## Known gaps (current cycle)

- Targeted coverage is four CLI process cases plus one load-summary case: three unchanged CLI cases and the load-summary case pass in `/tmp/pi-lua-summary-chronology-green.*`; the corrected post-load case passes in `/tmp/pi-lua-summary-chronology-postload-green.*` at `9500ac1bf`. The combined run itself was not all-green: its post-load test initially mistook the display-only `__BuiltIn` label for recorded ownership. Reporting continues to use canonical metadata, leaving that exec-created frame unattributed. Valid RED `/tmp/pi-lua-summary-chronology-postload-red.*` collects three errors and exits 1 but lacks the final summary.
- Actual user-addon/SavedVariables startup at `9500ac1bf` returns `[]`, exit 0 and final `CLEAN` with zero unique and zero occurrences (`/tmp/pi-accepted-final-startup.*`). It records three loader warnings, so this is not a zero-warning claim. Independent final Cargo checks remain in progress.

## Out of scope

Changing vendor/addon error handlers, suppressing reported exceptions, or modifying API compatibility to make reporting tests pass. Existing secondary messages emitted by Blizzard's error UI remain visible; this slice does not redefine their deduplication.
