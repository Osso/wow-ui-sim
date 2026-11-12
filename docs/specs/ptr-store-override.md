# PTR rewards-shop override and session validation

## Evidence

[Tracker lt-4183, coordinator comment 217 (R75)](https://tasks.sakuin.org/tasks/lt-4183)
reports native decompilation findings for PTR 12.1.7.70668. This implementation
models only its VERIFIED contracts. The runtime consumer is the pinned PTR
12.1.7.70493 `Blizzard_StoreUI/Blizzard_Shared_StoreUIToggle.lua:60–72`;
this is not a claim that the 70493 native image was inspected.

## Verified contracts

- `C_StorePublic.IsRewardsShopOverrideEnabled()` returns exactly one Boolean:
  `rewardsShopOverrideEnabled && !bpayStoreEnable`. Both are MirrorVar inputs;
  the override default is false. Both true still returns false. The native
  incompatibility diagnostic is not observable in Lua and is not modeled.
- Updating the override mirror reports `STORE_STATUS_CHANGED` with no payload.
  The notification does not itself write the value. Per clarification #227, the
  host seam writes then invokes the callback without an invented equality check;
  duplicate-write behavior remains UNKNOWN and is not asserted.
- `C_CatalogShop.ValidateOverrideShopInteraction([shoppingSessionUUIDStr])`
  returns one Boolean. Nil/absent returns false. Numbers convert to strings.
  Boolean/table/other unsupported values raise argument 1 with Usage reason
  `Usage: local isValid = C_CatalogShop.ValidateOverrideShopInteraction([shoppingSessionUUIDStr])`.
  The outer bad-argument formatting is UNKNOWN and is not asserted.
- A string validates only when host catalog mode is 3 and its bytes match the
  host session bytes exactly. Empty matches empty only in mode 3. This query
  does not consult the override CVar.
- Both APIs are registered in Game and Glue, on PTR only. Retail is unchanged.

## Host inputs and limits

The existing `WowLuaEnv::set_server_mirrored_cvar` seam supplies the PTR override
mirror. `SimState::catalog_shop_interaction` supplies mode and raw session bytes;
its neutral defaults are mode 0 (not 3) and empty session. These defaults are
simulator policy, not invented native session initialization. No Lua setter is
added. Server packet sources, native mode/session writers, and secret-argument
or taint behavior remain UNKNOWN and unmodeled.

## Consumer coverage

Tests drive Blizzard's real `ToggleStoreUI`: disabled store with override off
leaves the menu and both stores unchanged; supplied override/session state gates
entry through the original Lua. Boundary tests cover the truth table, payload-free
notification, Game/Glue registration, optional arguments, number coercion,
byte-exact matching, and the Usage reason. The narration fixture remains the real
`Blizzard_Narration` TOC from the earlier merged PR #55, not a stub.

## Existing workaround boundary

Per coordinator clarification #227, PTR bootstrap never installs a guessed
`C_StorePublic.IsEnabled` fallback. The labelled, unevidenced non-PTR fallback
and unrelated `EventStoreUISetShown` bookkeeping remain untouched in behavior,
pending evidence for those profiles.
