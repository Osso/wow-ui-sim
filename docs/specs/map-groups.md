# Map groups

`C_Map.GetMapGroupID` and `C_Map.GetMapGroupMembersInfo` expose supplied UI map-group membership to the WorldMap floor-navigation consumer on Retail and WoW Forever. See [map-group model](../../src/c_api/c_map.rs) for implementation details.

## What it must do

- [x] Return a supplied group ID for a grouped UI map.
- [x] Return the supplied group members with `mapID`, `relativeHeightIndex`, and `name` for the WorldMap floor menu.
- [x] Order supplied members by ascending `relativeHeightIndex` (INFERRED; R88cy native evidence pending with aso).

## How it works

- [map-group model](../../src/c_api/c_map.rs)

## Implementation inventory

- `src/c_api/c_map.rs` — registers and implements the map-group APIs and `A_Admin.SetMapGroupMember`.
- `src/lua_api/state_types/collections.rs` — stores optional group identity and relative height on `MapData`.
- `src/lua_api/state/defaults/maps.rs` — initializes ungrouped map records.
- `src/lua_api/workarounds/temporary/map_group_defaults.rs` — preserves the unchanged compatibility bootstrap outside Retail and Forever.

## Tests asserting this spec

- `tests/blizzard_map_lane.rs` — asserts that the loaded WorldMap floor-navigation consumer builds distinct menus for two supplied groups.

## Known gaps (current cycle)

- [ ] Confirm unknown-group return behavior and relative-height ordering with native evidence requested as R88cy.

## Out of scope

- PTR, Mists, Wrath, Era, Anniversary, and non-WorldMap consumers.
