# Documented API surface audit

This audit compares the API functions that Blizzard documents for a client profile with the functions that the simulator registers at runtime after full Blizzard UI startup. It replaces startup errors as the way to find missing API gaps. The test is `tests/api_surface_audit.rs`. The committed gap lists are `tests/data/api_surface/<profile>-missing.txt`. The maintenance tool is `tools/api_surface_audit.py`.

## What it must do

### Documented surface

- [x] Load every `Blizzard_APIDocumentationGenerated/*.lua` file from the active profile's Blizzard UI cache into a separate sandboxed Lua VM. `APIDocumentation:AddDocumentationTable` collects each `Type = "Function"` entry of each `Type = "System"` table as `Namespace.Name`, or as `Name` when the table has no `Namespace` (`_G`). `ScriptObject` tables document widget methods, not globals, so the audit excludes them.
- [x] Retail only: add the `_G` and namespace rows of `data/secure-publication/retail-12-1-5-70688.csv` whose context is not `Glue`. Glue-only rows do not exist in the native Game environment.

### Runtime surface

- [x] Run after full game-screen startup: Retail uses the prefork full-UI parent, and Forever uses the production startup inventory.
- [x] A name is present only when `rawget(_G, name)` or `rawget(rawget(_G, namespace), name)` is a function. Namespace `__index` fallbacks that only record nil access do not count ([namespace lookup](../wiki/systems/ui-widget-manager-state.md)).
- [x] A present function does not count as implemented when it is a simulator-embedded Lua closure (`debug.getinfo(fn, "S").source == "=(string)"`) and `src/lua_api/workarounds/temporary` defines it in Lua (`function X.Name(` or `Name = function`). These names are gaps with the label `not-yet: temporary Lua workaround`.

### Allowlist

- [x] Each allowlist line is `<name> # <category>`, sorted by byte order and unique. Lines that start with `#` are comments. The category starts with `intentionally-unsupported` or `not-yet`.
- [x] The test fails when a documented gap is not in the allowlist. The failure output gives the line to add.
- [x] The test fails when an allowlisted name is no longer a gap (now implemented or no longer documented).
- [x] The test fails when the `not-yet: temporary Lua workaround` label does not agree with the runtime flag.
- [x] `tools/api_surface_audit.py bless <profile> <test log>` applies a failure output to the allowlist. It labels new names as `intentionally-unsupported` when they are SecureOnly in the profile documentation, Glue-only in the Retail publication CSV, or 3D camera/model controls (see "Intentional Gaps" in `AGENTS.md`). It labels all other new names `not-yet`.
- [x] `tools/api_surface_audit.py rank <profile> [limit]` ranks the allowlisted names by call sites in the profile's cached Blizzard UI Lua.

## How it works

- [Namespace lookup and nil diagnostics](../wiki/systems/ui-widget-manager-state.md)
- [Missing C namespace members](c-namespace-nil-members.md)

## Implementation inventory

- `tests/api_surface_audit.rs`: the documentation sandbox, the runtime probe, temporary-workaround detection and the allowlist comparison.
- `tests/data/api_surface/retail-missing.txt`, `tests/data/api_surface/wowforever-missing.txt`: the committed gap lists.
- `tools/api_surface_audit.py`: the bless and rank commands.

## Tests asserting this spec

- `tests/api_surface_audit.rs`: `api_surface_audit::documented_api_surface_matches_allowlist` (Retail, `prefork_full_ui` target) and `api_surface_audit::forever_documented_api_surface_matches_allowlist` (Forever, `integration` target).

## Known gaps (current cycle)

- [ ] Temporary workarounds that are registered from Rust (not Lua) cannot be attributed at runtime, so they count as implemented.
- [ ] Temporary-workaround detection matches the member name in the workaround source text. A same-named Lua definition for a different namespace can mark a name as a workaround.

## Out of scope

- PTR, Mists, Wrath, Era and Anniversary are not gated runtime profiles.
- Frame and widget methods (`ScriptObject` documentation) are covered by the [method surface baselines](method-surface-baselines.md).
