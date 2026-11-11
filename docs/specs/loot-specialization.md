# Loot specialization

`GetLootSpecialization` and `SetLootSpecialization` expose the stored player choice. The original selected `UnitPopupSharedButtonMixins.lua` consumer sends choices through `SetLootSpecialization`. Retail behavior is based on the native evidence for build 70688.

## What it must do

- [x] `GetLootSpecialization()` returns the stored simulator value; the initial value is 0.
- [x] `SetLootSpecialization` accepts a numeric argument, returns no values, raises `Usage: SetLootSpecialization(specializationID)` for a non-number, and does not reject an ID because it is absent from the local class catalog.
- [ ] A setter call does not synchronously change the getter value or emit `PLAYER_LOOT_SPEC_UPDATED`.
- [ ] Do not validate requested specialization IDs against the player's class catalog before sending.
- [ ] Do not synthesize a server response or emit `PLAYER_LOOT_SPEC_UPDATED` without one.
- [ ] Do not model `HasLootSpecializations`; the ordinary Retail `true` result is INFERRED, not VERIFIED.

## How it works

- See the [C API and Lua API boundaries](../wiki/index.md).

## Implementation inventory

- `src/lua_api/globals/real/loot_specialization.rs` — getter, setter argument validation, and request-rate bookkeeping.
- `src/lua_api/state/sim_state.rs` — stores the current loot-specialization value and rate-limit window.

## Tests asserting this spec

- `tests/loot_specialization.rs::loot_specialization_original_popup_does_not_fabricate_a_server_response` — invokes the selected original `UnitPopupSharedButtonMixins.lua` click handler.
- `tests/loot_specialization.rs::loot_specialization_numeric_requests_do_not_apply_locally` — checks numeric request calls do not produce a local value or event.
- `tests/loot_specialization.rs::loot_specialization_usage_error` — checks the exact non-number diagnostic.

## Known gaps (current cycle)

- [ ] The outgoing request transport, send availability, server acceptance, and replicated update producer are not modeled.
- [ ] `HasLootSpecializations` remains unavailable, so the original menu's `CanShow` path cannot be verified.
- [ ] Exact rate-limit reset semantics and repeated-value response behavior remain unknown.

## Out of scope

- Do not infer server validation, loot generation, persistence, or chat output from the setter's request path.
