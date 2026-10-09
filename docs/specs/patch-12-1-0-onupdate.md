# Retail 12.1.0 onupdate — C04

Bounded simulator proof for [prose-2026-06-18-058; prose-2026-06-18-059](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L58/L59: all five numeric modes round-trip, invalid modes reject, default is RunWhenVisible; actual dispatch follows ancestor visibility, hidden RunAlways, one-shot reset/rearm and XML mode selection.
- [ ] Actual managed-aura dirty processing must retain native registration, remain armed while hidden, then clear dirty state and stay Disabled after visible dispatch. Earlier synthetic-phase failure remains historical RED evidence; corrected execution pending.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/mod.rs — update mode API`
- `src/lua_api/on_update.rs — dispatch`
- `cached Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua — dirty phases (read-only)`

## Tests asserting this spec

- `tests/on_update_modes.rs` — prior proof had four passing tests and one ManagedAura failure. The 2026-10-09 fixture preserves native phases and uses `UpdateAllAuras()`; fresh runtime proof pending.

## Known gaps (current cycle)

- [ ] Earlier fixture replaced six native phases with only flag 1, leaving flags 18 after showing requested the full rebuild. Exact 2026-10-09 RED reproduces that boundary. Fixture now observes actual hidden/visible dirty-state transitions without replacing phases; no vendor/runtime patch or ManagedAura PASS yet credited.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
