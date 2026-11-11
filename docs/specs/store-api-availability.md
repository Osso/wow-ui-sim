# PTR/Retail Store API availability

`C_StorePublic.IsEnabled()` and `C_CatalogShop.IsShop2Enabled()` report the two store availability inputs used by the shipped GameMenu. This contract covers PTR 12.1.7 and the limited Retail 12.1.5 comparison in R38. See the [state model](../wiki/systems/store-api-availability.md).

## What it must do

### API values

- [ ] VERIFIED: `C_StorePublic.IsEnabled()` takes no arguments and returns exactly one boolean from the hidden server-mirrored `bpayStoreEnable` value. Its built-in default is `false`.
- [ ] VERIFIED: `C_CatalogShop.IsShop2Enabled()` returns exactly one boolean from the independent hidden server-mirrored `shop2Enabled` value. Its built-in default is `true`; no dependency on `bpayStoreEnable` is allowed.
- [ ] VERIFIED: `C_CVar.GetCVar` refuses both hidden values. The exact Lua result shape is INFERRED and must not be asserted; global `GetCVar` parity is UNKNOWN.

### Updates and GameMenu

- [ ] VERIFIED: a changed `bpayStoreEnable` value fires one `STORE_STATUS_CHANGED` event with no payload and no `CVAR_UPDATE`.
- [ ] VERIFIED: a `shop2Enabled` change on an initialized path fires no Lua event and no `CVAR_UPDATE`. First-use event behavior is UNKNOWN.
- [ ] VERIFIED source behavior: the shipped GameMenu creates its Store button only when both API values are true. INFERRED: the built-in `(false, true)` values hide that button by default.
- [ ] Do not assert real-account server values, a `shop2Enabled` first-use event result, or 70493 behavior. The 70493 image was not available for R38.

## How it works

- [Host state, registration and event routing](../wiki/systems/store-api-availability.md).

## Implementation inventory

- `src/cvars.rs`: hidden per-environment server-mirror values and PTR/Retail defaults.
- `src/c_api/store_public_eligibility.rs`: `C_StorePublic.IsEnabled()` registration and query.
- `src/c_api/c_catalog_shop.rs`: `C_CatalogShop.IsShop2Enabled()` registration and query.
- `src/lua_api/env_events.rs`: host input updates and `STORE_STATUS_CHANGED` delivery.
- `src/c_api/ptr_i4_game_rules.rs`: PTR RewardsShop gate reads the same `bpayStoreEnable` input.

## Tests asserting this spec

- `tests/game_menu.rs`: API defaults/shape, inferred default button visibility, bpay event behavior, hidden CVar non-exposure, and Shop 2 versus legacy Store consumers.
- `tests/ptr_rewards_shop.rs`: shared PTR `bpayStoreEnable` input behavior.
- `src/lua_api/workarounds/temporary/housing_catalog_state.rs`: Shop 2 query default on covered profiles.

## Known gaps (current cycle)

- [ ] Verify 70493 behavior; R38 evidence covers PTR 70668 and a limited Retail 70688 comparison.
- [ ] Verify the exact Lua result shape of `C_CVar.GetCVar` for these hidden variables.
- [ ] Verify whether a `shop2Enabled` change fires an event on a first-use, uninitialized path.
- [ ] Verify real-account server values; R38 establishes built-in defaults only.

## Out of scope

Purchase transactions, account entitlements, server message transport, and legacy global `GetCVar` parity are not modeled by this contract.
