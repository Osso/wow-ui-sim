# Map world-position conversion

`C_Map.GetWorldPosFromMapPos` converts a UI-map position to a continent ID and a world-position vector. Current Retail 12.1.5 assignment data is generated from [`UiMapAssignment.csv`](../../data/db2/retail-12.1.5.70783/UiMapAssignment.csv); parity with the selected Retail 12.1.5.70575 pin is INFERRED, not verified. See the [implementation investigation](../wiki/investigations/patch-8-0-1-api-audit.md#retail-map-world-position).

## What it must do

### API

- [ ] For a Retail UI map position inside a supported assignment's `UiMin`/`UiMax` bounds, return the selected `MapID` as `continentID` and a world-position `vector2`.
- [ ] Return no values when no Retail assignment covers the position or the selected assignment has a negative `MapID`.
- [ ] An explicit, valid `MapWorldRect` for a known map takes precedence over the Retail assignment table and maps normalized x/y from left/top to right/bottom.
- [ ] **INFERRED:** map each axis linearly from `UiMin`/`UiMax` to the `Region` x/y endpoints, include assignment bounds, and select the matching row with the lowest `OrderIndex`. Native transform and overlap selection remain unverified.

### Data scope

- [ ] Compile the `UiMapAssignment` catalog only for the current Retail 12.1.5 profile; do not use it for PTR, Forever, Mists, or other profiles without matching exports.
- [ ] Store only `MapID`, `OrderIndex`, `UiMin`, `UiMax`, and the `Region` x/y endpoints required by this vector2 projection.

## How it works

- [Retail map assignment investigation](../wiki/investigations/patch-8-0-1-api-audit.md#retail-map-world-position)
- [Patch 8.0.1 explicit rectangle model](patch-8-0-1-publication-sweep.md#explicit-world-position-model)

## Implementation inventory

- `src/c_api/c_map.rs`: Retail assignment selection and `GetWorldPosFromMapPos` projection.
- `src/c_api/map_world_coordinates.rs`: explicit rectangle projection and precedence.
- `src/bin/wow_cli/gen_ui_maps.rs`: repository-CSV assignment generator.
- `crates/sim-data-profile/src/lib.rs`: current-Retail profile gate.
- `crates/sim-data-profile/data/ui_map_assignments_retail.rs`: generated vector2 assignments.

## Tests asserting this spec

- `tests/c_map_api.rs`: Mulgore projection, overlapping-row choice, and descendant-Zone probe.
- `tests/patch_8_0_1_map_coordinates.rs`: explicit rectangle precedence and bounds.

## Known gaps (current cycle)

- [ ] **INFERRED:** `70783` data matches the selected `70575` Retail pin; exact row parity is unknown.
- [ ] Native confirmation is pending for bounds, axis orientation, overlap tie-break, returned `MapID`, and no-assignment behavior.

## Out of scope

- Assignment-table support for profiles without matching profile data.
- Replacing the existing non-Retail map-position behavior or changing `GetMapPosFromWorldPos` to consume inferred geography.
