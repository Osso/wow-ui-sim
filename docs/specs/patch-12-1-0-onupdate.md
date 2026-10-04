# Retail 12.1.0 onupdate — C04

Bounded simulator proof for [prose-2026-06-18-058; prose-2026-06-18-059](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L58/L59: all five numeric modes round-trip, invalid modes reject, default is RunWhenVisible; actual dispatch follows ancestor visibility, hidden RunAlways, one-shot reset/rearm and XML mode selection.
- [ ] Existing managed-aura dirty-phase fixture fails: showing container requests full rebuild mask 19 after fixture replaces phases with only flag 1; unprocessed flags 18 remain. No managed aura proof credited or vendor/fixture patch made.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/mod.rs — update mode API`
- `src/lua_api/on_update.rs — dispatch`
- `cached Blizzard_AuraContainer/Blizzard_ManagedAuraContainer.lua — dirty phases (read-only)`

## Tests asserting this spec

- `tests/on_update_modes.rs — four passing tests; on_update_modes_process_actual_managed_aura_dirty_phases fails`

## Known gaps (current cycle)

- [ ] Existing managed-aura dirty-phase fixture fails: showing container requests full rebuild mask 19 after fixture replaces phases with only flag 1; unprocessed flags 18 remain. No managed aura proof credited or vendor/fixture patch made.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
