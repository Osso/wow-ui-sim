# Retail 12.1.0 onupdate — C04

Bounded simulator proof for [prose-2026-06-18-058; prose-2026-06-18-059](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L58/L59: all five numeric modes round-trip, invalid modes reject, default is RunWhenVisible; actual dispatch follows ancestor visibility, hidden RunAlways, one-shot reset/rearm and XML mode selection.
- [x] Actual managed-aura dirty processing remains armed while hidden, then clears dirty state and stays Disabled after visible dispatch. Corrected fixture PASS is recorded in saved c75.
- [ ] Registration retention is not directly asserted by that fixture; populated aura/registration and native parity evidence remain separate.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/mod.rs — update mode API`
- `src/lua_api/on_update.rs — dispatch`
- `cached Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua — dirty phases (read-only)`

## Tests asserting this spec

- `tests/on_update_modes.rs` — prior proof had four passing tests and one ManagedAura failure. The 2026-10-09 fixture preserves Blizzard phases and uses `UpdateAllAuras()`. [Saved c75 case/body reconciliation](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/c75-pending-spec-reconciliation-2026-10-11/README.md) records its named PASS at raw line7176; current test body/Cargo pin match. Assertions cover dirty flags and mode2 while hidden, then clean/mode0 after visible dispatch and another clean/disabled update, plus empty captured Lua errors. No rerun, registration-retention or native-client credit; enclosing suite remains FAIL.

## Known gaps (current cycle)

- [ ] Earlier fixture replaced six native phases with only flag 1, leaving flags 18 after showing requested the full rebuild. Exact 2026-10-09 RED reproduces that boundary. Corrected fixture observes hidden/visible dirty-state transitions without replacing phases; its saved c75 PASS is now credited only for those assertions. Earlier synthetic-phase RED remains historical evidence; no vendor/runtime patch, registration-retention or native-client acceptance follows.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
