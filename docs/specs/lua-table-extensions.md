# Lua table extensions

Retail 12.1.5 exposes native `table` helpers that `Blizzard_SharedXMLBase/TableUtil.lua` aliases during startup. The simulator publishes the startup-required subset only for the cumulative `retail-12-1-5` epoch.

## What it must do

- [x] `table.indexof` searches consecutive array indexes from 1 through the first nil and returns the first equal value or nil.
- [x] `table.contains`, `count`, and `isempty` inspect all non-nil entries, including hash keys and `false` values.
- [x] `table.keys` and `values` return arrays containing every key or value; result order is unspecified.
- [x] `table.removeunordered` replaces an indexed array value with the last value and shortens the array; omitted index selects the last value.
- [x] `table.removevalue` removes equal values from the array portion and returns the removal count.
- [ ] `table.create`, `freeze`, `getcountinfo`, and `isfrozen` are documented native APIs but are not startup-required by the current PTR 12.1.5 `TableUtil.lua` aliases.

## How it works

- [Lua API](../lua-api.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/globals/real/table_extensions.rs`: native table registrations and mixed-table/array operations.
- `src/lua_api/globals/register.rs`: epoch-gated registration.
- `patch-tests/patch_12_1/table_extensions.rs`: observable PTR behavior.

## Tests asserting this spec

- `patch-tests/patch_12_1/table_extensions.rs`

## Known gaps (current cycle)

- [ ] Full native table extension surface remains unmodeled outside startup-required functions.

## Out of scope

- Secret-value taint behavior: source documents it, but it is not yet modeled for generic table operations.
- Frozen-table semantics: requires a state/dispatch contract beyond this startup fix.
