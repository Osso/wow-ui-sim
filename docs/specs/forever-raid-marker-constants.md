# Forever raid-marker constants

Forever publishes the `Constants.RaidMarkerConsts` table specified by build 1.60.1.69913 `RaidMarkerConstantsDocumentation.lua`. Its type is `Constants`, not `Enumeration`; no `Enum.RaidMarkerConsts` alias is required.

## What it must do

- [x] Publish `MAX_RAID_TARGETS_USER=8`, `MAX_RAID_TARGETS_RESTRICTED=8`, `MAX_VALID_RAID_TARGETS=0`, and `MAX_RAID_MARKERS=8` only for Forever.
- [x] Let the unchanged gamepad `TargetActionBars/Shared.lua` select clear-marker state at the forward eighth-marker and reverse first-marker boundaries.

## How it works

- [Lua API initialization](../lua-api.md)

## Implementation inventory

- `src/lua_api/env_init/enums.rs`: profile-bounded constant publication.

## Tests asserting this spec

- `tests/wowforever_raid_marker_constants.rs`: exact values and real Shared.lua button state using real frame/texture objects with isolated event, target, and icon fixtures.

## Known gaps (current cycle)

- [ ] Other-profile isolation and full startup verification remain outside this bounded development run.

## Out of scope

Marker search implementation, native roster/marker conformance, retail epoch exposure, and the separate `RaidMarkerSpellids` enumeration.
