# Loot specialization

`GetLootSpecialization` and `SetLootSpecialization` expose the stored player choice. The original selected `UnitPopupSharedButtonMixins.lua` consumer sends choices through `SetLootSpecialization`. Retail behavior is based on the native evidence for build 70688.

## What it must do

- [x] `GetLootSpecialization()` returns the stored simulator value; the initial value is 0.
- [x] On Retail, `SetLootSpecialization` accepts a numeric argument, returns no values, raises `Usage: SetLootSpecialization(specializationID)` for a non-number, and does not reject an ID because it is absent from the local class catalog.
- [x] On Retail, a setter call leaves the stored getter value unchanged and emits no `PLAYER_LOOT_SPEC_UPDATED` before or after the next OnUpdate tick.

## How it works

- See the [C API and Lua API boundaries](../wiki/index.md).

## Implementation inventory

- `src/lua_api/globals/real/loot_specialization.rs` — Retail getter and setter argument boundary; server transport is not modeled.
- `src/lua_api/state/sim_state.rs` — stores the current loot-specialization value.

## Tests asserting this spec

- `tests/loot_specialization.rs::loot_specialization_original_popup_does_not_fabricate_a_server_response` — invokes the selected original `UnitPopupSharedButtonMixins.lua` click handler.
- `tests/loot_specialization.rs::loot_specialization_numeric_requests_do_not_apply_locally` — checks numeric request calls do not produce a local value or event.
- `tests/loot_specialization.rs::loot_specialization_usage_error` — checks the exact non-number diagnostic.

## Known gaps (current cycle)

- [ ] Retail request submission, send availability, rate limiting, server acceptance, and replicated update producer are not modeled.
- [ ] Retail `HasLootSpecializations` remains unavailable because the ordinary `true` result is INFERRED; the original menu's `CanShow` path cannot be verified.
- [ ] Exact rate-limit reset semantics, request-send availability, and repeated-value response behavior remain unknown.
- [ ] Non-Retail behavior remains the pre-existing simulator model; R44-Q3 does not establish its native parity.

## Out of scope

- Do not infer server validation, loot generation, persistence, or chat output from the setter's request path.
