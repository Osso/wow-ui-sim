# Cached addon runtime audit staging

`tools/cached_addon_runtime.py` stages existing archives and parses bounded startup observations for the [cached-addon audit](../forever-addon-comparison.md). It never executes the simulator. Startup evidence is not full addon compatibility.

## What it must do

- [x] Preserve every indexed project; select only successfully cached Forever records. Projects with only comparison archives remain explicit blocked rows, with no substitution or acquisition.
- [x] Verify archive SHA-256 before staging. Reject absolute, drive-qualified, traversal and symlink members, conflicting normalized members, file/directory collisions, destination symlinks, and conflicting existing bytes before writing. Identical restaging preserves content and the original ZIP.
- [x] Stage package-root directories under a caller-supplied isolated root, with fake install/WTF directories, a load observer, and `AddOns.txt`. Disable repository addons except Admin, SimCommands, TestFramework and Blizzard_FrameXML; enable packaged roots.
- [x] Return argv for the already-built simulator with addon/WTF/install isolation, CASC disabled, inherited `WOW_SIM_NO_ADDONS` removed, SavedVariables disabled, a timeout of 1–90 seconds, and the observer followed by `lua-errors`. The caller supplies its execution context and cache environment.
- [x] Record each root's loading-or-loaded and fully-loaded flags separately, boolean LoadOnDemand state, and the reason returned by `GetAddOnInfo`; do not infer unavailable dependency or game-filter explanations.
- [x] Parse complete root observations and trailing Lua-error JSON. Clean startup requires exit zero, no recorded Lua errors, a completed observer, and at least one fully loaded root. Report failed, unloaded or incomplete evidence explicitly; retain every root observation so unloaded roots are not hidden. An unloaded LoadOnDemand root alone is not an error.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Cached-addon evidence and limits](../forever-addon-comparison.md)

## Implementation inventory

- `tools/cached_addon_runtime.py` — inventory selection, authenticated safe staging, observer/argv generation and result classification.

## Tests asserting this spec

- `tools/tests/test_cached_addon_runtime.py` — concrete temporary archives and hashes, safe extraction and conflict rejection, exact argv contract, loaded/LoadOnDemand observations, malformed/incomplete output and Lua-error handling.

## Known gaps (current cycle)

- [ ] Runtime orchestration and integration verification belong to the caller; this helper does not classify the loader's exact non-load reason.

## Out of scope

Downloads, subprocess execution, dependency substitution, arbitrary slash commands, native-contract guesses, major-interaction acceptance, and inventory-wide completion. This helper reports observations; it cannot establish why an addon intentionally declines to load.
