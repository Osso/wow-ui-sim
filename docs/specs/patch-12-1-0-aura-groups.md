# Retail 12.1.0 aura-groups — C08

Bounded simulator proof for [prose-2026-07-07-119; prose-2026-07-07-120; prose-2026-07-07-122; prose-2026-07-07-123; prose-2026-07-07-125; prose-2026-07-07-126](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L119/L120: real managed container consumes three native auras into independent HELPFUL and HELPFUL|PLAYER groups; actual shown frame counts reflect filters.
- [x] L122/L123: tainted AddAuraGroup accepts arbitrary nonempty keys, returns zero values, supports lookup, rejects duplicate/empty keys and preserves caller taint.
- [x] L125/L126: empty options and nondefault maxFrameCount operate; changing cap 1 to 3 increases actual shown AuraButtons and changing another group to HARMFUL hides prior assignments. Frames have actual AuraButton type and parent ownership.
- [ ] Coverage uses ManagedAuraContainer plus CustomAuraContainerTemplate; plain AuraContainer route, combat-secret lifecycle, removal events and rendered regions are not established.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `cached Blizzard_AuraContainer/Blizzard_CustomAuraContainer.lua — unmodified public group delegates`
- `src/lua_api/globals/auras.rs — native unit-aura model`

## Tests asserting this spec

- `tests/patch_12_1_0_aura_groups.rs::patch_12_1_0_aura_groups_filter_limit_and_refresh_actual_frames`

## Known gaps (current cycle)

- [ ] Coverage uses ManagedAuraContainer plus CustomAuraContainerTemplate; plain AuraContainer route, combat-secret lifecycle, removal events and rendered regions are not established.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
