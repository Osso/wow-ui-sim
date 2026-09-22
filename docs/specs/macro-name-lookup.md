# Macro name lookup

`GetMacroIndexByName(name)` exposes the existing macro storage through the legacy Lua API in `src/lua_api/globals/spell_macro_verbs.rs`. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Return the existing 1-based slot for an occupied macro with the requested name, or `0` when absent.
- [ ] Observe creation, rename, deletion, and slot reuse without renumbering surviving macros.
- [ ] Preserve existing account/character slot ranges and exact, case-sensitive name matching used by `EditMacro`.
- [ ] Ignore empty storage entries, including holes before character macros and slots cleared by deletion.

## How it works

- [Lua API architecture](../lua-api.md)
- [Macro action contract](macro-action-showtooltip.md)

## Implementation inventory

- `src/lua_api/globals/spell_macro_verbs.rs` — lookup and registration alongside the existing macro mutation functions.

## Tests asserting this spec

`tests/spell_macro_verbs.rs`, in the existing grouped integration target:

- `macro_name_lookup_tracks_create_rename_delete_and_slot_reuse`
- `macro_name_lookup_preserves_account_and_character_slots`

## Known gaps (current cycle)

- [ ] Compile and run the focused tests; Cargo was not authorized for this implementation slice.
- [ ] Replay unchanged DrinkBot after rebuilding. Frozen `b8f0982be` reproduces the missing-function failure using the new tests' Lua assertions: `/tmp/forever-addon-audit/macro-name-red-oli620t2/ledger.json`.

The local Forever generated macro documentation does not describe this legacy getter. Local wowless `data/products/wow/apis.yaml` records its string input and numeric slot output; the zero-on-miss contract is the requested compatibility behavior. Exact matching and first occupied match preserve the existing simulator model, not independently verified native duplicate-name semantics.

## Out of scope

Changing macro capacity/index allocation, `GetMacroInfo`/`GetMacroBody`, persistence, duplicate-name policy, or other macro APIs. Native Forever conformance remains unverified.
