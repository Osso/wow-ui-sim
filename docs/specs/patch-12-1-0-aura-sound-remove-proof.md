# Retail 12.1.0 aura-sound-remove-proof — C10

Bounded simulator proof for [prose-2026-07-21-181](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L181: renamed C_UnitAuras.RemoveAuraSound returns zero values and removes exactly selected current live registration; legacy remover shares state and remains functional under tainted caller after executing entire unmodified cached Deprecated_12_1_0.lua.
- [ ] Actual playback, acquisition outside seeded live-ID model, and historical intermediate alias timing not asserted.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/private_aura_sounds.rs — native registration removal`
- `cached Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua — legacy wrapper`

## Tests asserting this spec

- `tests/private_aura_sound_removal.rs::direct_modern_and_legacy_removers_share_real_state`
- `tests/private_aura_sound_removal.rs::actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap`

## Known gaps (current cycle)

- [ ] Actual playback, acquisition outside seeded live-ID model, and historical intermediate alias timing not asserted.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
