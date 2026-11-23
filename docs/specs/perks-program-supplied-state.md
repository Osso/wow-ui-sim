# Perks Program supplied state

The simulator exposes supplied vendor, currency, and pending chest reward state through `C_PerksProgram` on Retail and WoW Forever. This contract excludes the category catalogue in [Perks vendor categories](perks-vendor-categories.md); implementation flow belongs in [the Perks Program supplied-state wiki page](../wiki/systems/perks-program-supplied-state.md).

## What it must do

### Supplied data

- [x] `GetAvailableVendorItemIDs()` and `GetPendingChestRewards()` read supplied state without arguments. The simulator uses the documented table results and preserves supplied record fields. This shape is INFERRED (R88ep); native result details remain unknown.
- [x] `GetVendorItemInfo(id)` returns the supplied matching vendor record. This record shape is INFERRED from the API documentation.
- [x] `GetCurrencyAmount()` takes no arguments and publishes one integer, or zero when the owner/player is absent. This behavior is VERIFIED (R88ep).
- [x] The original Retail Perks Program panel displays supplied vendor IDs and prices, Trader's Tender, and pending rewards when opened through `ShowUIPanel`.
- [x] On Retail and Forever, `RequestPendingChestRewards()` takes no arguments and returns zero Lua values. This is VERIFIED (R88ep); the supplied-state tests exercise the eligible request path.

### Refresh behavior

- [x] Supplied catalog, currency, and reward changes notify the Blizzard consumer paths. The local response events are INFERRED (R88ep), because native response event names and order are unknown.

## How it works

- [Perks Program supplied-state flow](../wiki/systems/perks-program-supplied-state.md)

## Implementation inventory

- `src/c_api/c_perks_program.rs` — supplied data, C API members, and simulator-only `A_Admin` inputs.
- `src/c_api/mod.rs` and `src/c_api/registration.rs` — C API module and profile registration.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — Perks Program state storage.
- `src/lua_api/globals/admin.rs` — `A_Admin` registration.
- `src/lua_api/workarounds/temporary/perks_program_defaults.rs` and its parent modules — compatibility surface outside Retail and Forever.

## Tests asserting this spec

- `tests/perks_program_supplied_consumers.rs` — supplied state, eligible request, and Retail Blizzard panel consumer.
- `tests/c_namespace_noop_replacements.rs::perks_program_pending_chest_rewards_return_empty_table` — retained shim coverage outside Retail and Forever.

## Known gaps (current cycle)

- [ ] Verify native Lua result shapes and pending reward payload fields; R88ep did not establish them.
- [ ] Verify the native response event name and update order; R88ep did not establish them.

## Out of scope

- `GetAvailableCategoryIDs()` and `GetCategoryInfo()` remain in the existing category model and are unchanged.
- Purchases, refunds, live vendor availability, and other Perks Program members are not part of this supplied-state model.
