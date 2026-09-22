# Cached addon runtime audit staging

`tools/cached_addon_runtime.py` stages existing archives and parses bounded startup observations for the [cached-addon audit](../forever-addon-comparison.md). It never executes the simulator. Startup evidence is not full addon compatibility.

## What it must do

- [x] Preserve every indexed project; select only successfully cached Forever records. Projects with only comparison archives remain explicit blocked rows, with no substitution or acquisition.
- [x] Verify archive SHA-256 before staging. Reject absolute, drive-qualified, traversal and symlink members, conflicting normalized members, file/directory collisions, destination symlinks, and conflicting existing bytes before writing. Identical restaging preserves content and the original ZIP.
- [x] Treat ZIP names ending in a slash after backslash normalization as directories, including zero-attribute directory entries; preserve path-security checks and original archive bytes.
- [x] Stage an explicit list of `(archive path, expected SHA-256)` pairs together. Validate every archive and destination before writes; reject duplicate roots and contradictory file content. Generate one union-root observer and enable file without dependency discovery, downloads, or version substitution.
- [x] Stage package-root directories under a caller-supplied isolated root, with fake install/WTF directories, a load observer, and `AddOns.txt`. Disable repository addons except Admin, SimCommands, TestFramework and Blizzard_FrameXML; enable packaged roots.
- [x] Return argv for the already-built simulator with addon/WTF/install isolation, CASC disabled, inherited `WOW_SIM_NO_ADDONS` removed, SavedVariables disabled, a timeout of 1–90 seconds, and the observer followed by `lua-errors`. The caller supplies its execution context and cache environment.
- [ ] Create `<isolated-root>/xdg-data` and set `XDG_DATA_HOME` to that path in the returned argv. Separate staging roots must use separate CVar and fallback SavedVariables storage, preserve their local files when restaged, and leave inherited host data untouched. Do not override the caller's shared `XDG_CACHE_HOME`.
- [x] Record each root's loading-or-loaded and fully-loaded flags separately, boolean LoadOnDemand state, and the reason returned by `GetAddOnInfo`; do not infer unavailable dependency or game-filter explanations.
- [x] Parse complete root observations and trailing Lua-error JSON. Clean startup requires exit zero, no recorded Lua errors, a completed observer, and at least one fully loaded root. Report failed, unloaded or incomplete evidence explicitly; retain every root observation so unloaded roots are not hidden. An unloaded LoadOnDemand root alone is not an error.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Cached-addon evidence and limits](../forever-addon-comparison.md)

## Implementation inventory

- `tools/cached_addon_runtime.py` — inventory selection, authenticated `stage_packages(packages, isolated_root, repo_addons)` staging, observer/argv generation and result classification. Single-package callers supply a one-element list; the old `stage_package` API is removed.

## Tests asserting this spec

- `tools/tests/test_cached_addon_runtime.py` — concrete temporary archives and hashes, normalized backslash directories, explicit consumer/provider composition, invalid second-archive hash and collision rejection before writes, exact argv contract, distinct data-home paths, host-data preservation, independent CVar-file restaging, loaded/LoadOnDemand observations, malformed/incomplete output and Lua-error handling.

Targeted development proof: 22/22 tests pass. The two previously rejected cached ZIPs (`8906484`, `8934390`) stage unchanged after directory normalization; `/tmp/forever-addon-audit/backslash-directory-ledger.json` records the reproduction and real-archive results. This is staging proof, not simulator execution.

`--no-saved-vars` does not disable CVar persistence: `src/cvars.rs` uses the local data directory independently of the isolated WTF path. Per-root `XDG_DATA_HOME` prevents audit processes from sharing that state. Python tests cover staging, argv, and file preservation; the caller owns the cross-process simulator sentinel and invalidated inventory reruns.

## Known gaps (current cycle)

- [ ] Targeted Python GREEN for the new data-home isolation assertions is pending.
- [ ] Runtime orchestration and integration verification belong to the caller; this helper does not classify the loader's exact non-load reason.

## Out of scope

Downloads, subprocess execution, dependency substitution, arbitrary slash commands, native-contract guesses, major-interaction acceptance, and inventory-wide completion. This helper reports observations; it cannot establish why an addon intentionally declines to load.
