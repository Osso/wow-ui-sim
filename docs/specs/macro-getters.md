# Macro getter model

Macro getters expose the shared `MacroInfo` records and icon catalog to legacy Lua APIs and `C_Macro` on Retail and WowForever. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Read macro names, icons, and bodies from supplied `MacroInfo` rows on Retail and WowForever.
- [x] Return account and character counts from the shared rows using the existing 120/30 slot split.
- [x] Return macro icon lists from supplied `MacroIconInputs` without fabricated fixture entries.
- [x] Back `C_Macro.GetNumMacros`, `GetMacroName`, and `GetSelectedMacroIcon` from the same records.
- [ ] Treat `C_Macro` macro IDs as one-based for the cached MacroFrame consumer. **INFERRED** (R88l.24; native index semantics pending).
- [x] Keep the legacy macro fixtures on profiles other than Retail and WowForever.

## How it works

- [Lua API architecture](../lua-api.md)
- [Macro name lookup](macro-name-lookup.md)

## Implementation inventory

- `src/lua_api/globals/real/macro_getters.rs` — legacy getter registrations and shared row projections.
- `src/c_api/c_macro.rs` — `C_Macro` getters and macro test-state setters.
- `src/lua_api/workarounds/temporary/macro_defaults.rs` — legacy fixtures for other profiles.

## Tests asserting this spec

- `tests/macro_getter_consumers.rs`
- `tests/macro_icon_consumers.rs`
- `tests/test_showuipanel_lod.rs`

## Known gaps (current cycle)

- [ ] Native `C_Macro` index and missing-row results remain unverified; the one-based adapter serves the pinned MacroFrame consumer and is labeled INFERRED (R88l.24).
- [ ] Native macro persistence and icon enumeration remain outside this model.

## Out of scope

Macro persistence, native creation/editing parity, the existing 18-slot character writer limit, and native icon enumeration.
