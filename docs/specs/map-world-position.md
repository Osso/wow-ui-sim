# Map world-position conversion

Retail `C_Map.GetWorldPosFromMapPos` converts a UI-map point by using `UiMapAssignment`. **VERIFIED (R88i.41):** a negative `MapID` uses the assignment's `AreaID` to look up an `AreaTable` row; a failed lookup skips that row. **INFERRED:** the unsigned-16 value read from a successful row is `AreaTable.ContinentID`. `C_Map.GetMapPosFromWorldPos` uses explicit simulator rectangles; this contract covers its verified argument and Usage behavior, not native assignment selection.

## What it must do

### Map to world

- [x] **VERIFIED (Retail 12.1.5.70688):** accept a numeric UI-map ID in signed 32-bit range and truncate fractions toward zero. Accept a plain table with readable numeric `x` and `y` fields.
- [x] **VERIFIED:** include this byte-exact argument Usage string: `Usage: local continentID, worldPosition = C_Map.GetWorldPosFromMapPos(uiMapID, mapPosition)`.
- [x] **VERIFIED:** return exactly two values after a successful conversion and zero values when no usable result exists.
- [x] **VERIFIED:** select the eligible containing assignment nearest the point to its UI-rectangle center. If none contains the point, select the nearest-center outside candidate. Keep the first visited row on a tie.
- [x] **VERIFIED:** include rectangle edges, reject a row when either UI span is below single-precision epsilon, do not require coordinates in `[0,1]`, and do not clamp the transform.
- [ ] **INFERRED (R88i.41):** map Y controls world X, blending `Region_3` toward `Region_0`; map X controls world Y, blending `Region_4` toward `Region_1`.
- [ ] **VERIFIED (R88x):** a nonnegative `MapID` is the returned map number. For a negative `MapID`, look up the assignment's `AreaID` in `AreaTable`; skip the assignment if the lookup fails. After a successful lookup, accept the unsigned-16 value, including zero.
- [ ] **INFERRED (R88x and the 12.1.5.70783 DBD):** the native unsigned-16 result maps to the `ContinentID` column. The generated Rust field is named `continent_id`.
- [ ] **INFERRED (R88x plus the complete 12.1.5.70783 export):** UI maps 1408 and 2311 reference AreaID 0, but AreaTable has no ID 0 row. Their negative assignments therefore have no resolved map number under the verified skip rule. This conflicts with treating those maps as successful conversions.
- [x] **VERIFIED (simulator contract):** an explicit `MapWorldRect` for a known map takes precedence over generated Retail assignment data.

### World to map argument boundary

- [x] **VERIFIED (Retail 12.1.5.70688):** accept a numeric continent ID and optional override UI-map ID in signed 32-bit range, truncating fractions toward zero. An absent or nil override is equivalent to zero. Accept a plain table with readable numeric `x` and `y` fields.
- [x] **VERIFIED:** include this byte-exact argument Usage string: `Usage: local uiMapID, mapPosition = C_Map.GetMapPosFromWorldPos(continentID, worldPosition [, overrideUiMapID])`.
- [ ] Native assignment selection, inverse coordinate projection, and parent-map promotion remain outside this contract.

## Data scope

- **VERIFIED:** the Retail `UiMapAssignment` export is build 12.1.5.70783. Its applicability to the selected 12.1.5.70575 pin is **INFERRED**; exact parity is not verified.
- **VERIFIED (12.1.5.70783):** negative-`MapID` assignment rows reference AreaIDs 0 and 14620. The complete AreaTable export has 10,046 rows and zero unreadable records. AreaID 14620 is present with exported `ContinentID` 2444; ID 0 is absent. The committed reduced CSV keeps only the referenced ID/field pair that exists.
- **VERIFIED (12.1.5.70783 catalog):** UI maps 1408 and 2311 are Zones with negative-`MapID` assignment rows that reference AreaID 0. Zones 997, 1158, 1523, and 2107 have no assignment rows. The first pair is not equivalent to the second: their assignment rows fail the AreaTable lookup in the committed export.
- **INFERRED:** CSV row order is the simulator's tie-visitation choice; native iterator order is unknown.
- Do not claim assignment-data support for PTR, Forever, Mists, or Wrath without matching profile data.

## How it works

- The [Retail API investigation](../wiki/investigations/patch-8-0-1-api-audit.md#retail-map-world-position) records native evidence and its limits.
- The [layout system](../layout-system.md) describes simulator coordinates and frame geometry.

## Implementation inventory

- `src/c_api/c_map.rs`: Retail assignment selection, forward conversion, and vector creation.
- `src/c_api/map_world_coordinates.rs`: explicit rectangle projection and verified world-to-map argument handling.
- `src/bin/wow_cli/gen_ui_maps.rs`: repository-CSV assignment generator, including `AreaID`.
- `src/bin/wow_cli/gen_area_table.rs`: reduced Retail AreaTable generator.
- `crates/sim-data-profile/data/ui_map_assignments_retail.rs`: generated assignment rows.
- `crates/sim-data-profile/data/area_table_retail.rs`: generated AreaTable ID/u16 lookup.

## Tests asserting this spec

- `tests/c_map_api.rs`: Mulgore point, off-center transform, overlap selection, outside extrapolation, signed-ID truncation, Usage errors, negative-MapID resolution, unresolved AreaID 0, and catalog-derived no-world Zones.
- `tests/patch_8_0_1_map_coordinates.rs`: explicit rectangle precedence and bounds.
- `src/c_api/c_map.rs`: optional Retail unit test for a resolved zero map number.

## Known gaps (current cycle)

- **INFERRED:** exact row parity between builds 12.1.5.70783 and 12.1.5.70575 is not verified.
- Native iterator row order is unknown. CSV order is only an **INFERRED** tie choice.
- The swapped/reversed transform is **INFERRED**. Exact floating-point operation precision is unknown.
- **INFERRED from R88x and the committed build data:** negative assignments with AreaID 0 cannot resolve from this AreaTable export. Native runtime data for build 12.1.5.70688 was not measured against the 70783 export.
- `GetMapPosFromWorldPos` native assignment ranking and parent promotion remain unmodeled.

## Out of scope

- Assignment tables for profiles without matching exports.
- Replacing existing non-Retail map-position behavior.
- Native world-to-map assignment search, compatibility ranking, or parent promotion.
