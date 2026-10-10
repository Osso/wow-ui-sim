# Currency list field projection

Expose existing currency-list backing fields through `C_CurrencyInfo.GetCurrencyListInfo` and the Mists legacy tuple. Source lives in `src/c_api/item_spell/c_currency.rs` and `src/mists/compat_bootstrap.lua`.

## What it must do

- [ ] Return non-nil boolean `isShowInBackpack` from `CurrencyEntry.is_show_in_backpack`, preserving true and false.
- [ ] Return non-nil number `maxQuantity` from `CurrencyEntry.max_quantity`, preserving zero and positive caps.
- [ ] Forward those fields directly through Mists `GetCurrencyListInfo` tuple positions 5 and 8.
- [ ] Preserve all other fields, unsupported tuple slots, catalog data, defaults, and registrations.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/currency_data.rs` — existing `CurrencyEntry` fields and static list; unchanged.
- `src/c_api/item_spell/c_currency.rs` — namespaced list-info projection with capacity for nine fields.
- `src/mists/compat_bootstrap.lua` — existing Mists legacy tuple adapter.
- `tests/mists_currency_list.rs` — existing currency-list integration test file; owned by the separate fixture actor.

## Tests asserting this spec

Two new test names in `tests/mists_currency_list.rs` (GREEN unverified here):

- `currency_list_info_preserves_watched_flags_in_namespace_and_legacy_tuple`
- `currency_list_info_preserves_max_quantity_in_namespace_and_legacy_tuple`

## Known gaps (current cycle)

- [ ] Main must compile and run GREEN; this production-fix actor runs no tests or acceptance gates.

## Out of scope

Native Mists catalog and comprehensive currency parity are not claimed. No catalog/default/registration changes, alternate paths, new fallbacks, vendor edits, or adjacent serializer refactors.
