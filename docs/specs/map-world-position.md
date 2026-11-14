# Map world-position conversion

Retail `C_Map.GetWorldPosFromMapPos` converts a UI-map point to a map number and world-position vector. Its assignment data comes from a profile-scoped `UiMapAssignment` export. `C_Map.GetMapPosFromWorldPos` uses explicit simulator rectangles; this contract covers only its verified argument and Usage behavior, not native assignment selection.

## What it must do

### Map to world

- **VERIFIED (Retail 12.1.5.70688):** accept a numeric UI map ID in signed 32-bit range and truncate fractions toward zero. Accept a plain table with readable numeric `x` and `y` fields.
- **VERIFIED:** argument errors include this byte-exact Usage string: `Usage: local continentID, worldPosition = C_Map.GetWorldPosFromMapPos(uiMapID, mapPosition)`.
- **VERIFIED:** return exactly two values after a successful conversion and zero values when no usable result exists.
- **VERIFIED:** select the eligible containing assignment nearest the point to its UI-rectangle center. If none contains the point, select the nearest-center outside candidate. Keep the first visited row on a tie.
- **VERIFIED:** include rectangle edges, reject an assignment when either UI span is below single-precision epsilon, do not require coordinates in `[0,1]`, and do not clamp the transform. Outside points can extrapolate.
- **INFERRED:** map X blends `Region_3` toward `Region_0` using the normalized map Y coordinate; world Y blends `Region_4` toward `Region_1` using normalized map X.
- **INFERRED:** a selected nonnegative `MapID` supplies the returned map number. A negative `MapID` instead requires `AreaID` to resolve `AreaTable.ContinentID`.
- An explicit `MapWorldRect` for a known map takes precedence over generated Retail assignment data.

### World to map argument boundary

- **VERIFIED (Retail 12.1.5.70688):** accept a numeric continent ID and optional override UI map ID in signed 32-bit range, truncating fractions toward zero. An absent or nil override is equivalent to zero. Accept a plain table with readable numeric `x` and `y` fields.
- **VERIFIED:** argument errors include this byte-exact Usage string: `Usage: local uiMapID, mapPosition = C_Map.GetMapPosFromWorldPos(continentID, worldPosition [, overrideUiMapID])`.
- Native assignment selection, inverse coordinate projection, and parent-map promotion are outside this contract.

## Data scope

- Use the current Retail 12.1.5 `UiMapAssignment` export. The export is from build 12.1.5.70783; parity with the selected 12.1.5.70575 pin is **INFERRED**.
- Store only `MapID`, `UiMin`, `UiMax`, and the X/Y world endpoints needed by the vector2 result. Preserve CSV row order as an **INFERRED** tie-visitation choice; native iterator order is unknown.
- Do not claim assignment-data support for PTR, Forever, Mists, Wrath, Era, or Anniversary without matching profile data.

## Implementation inventory

- `src/c_api/c_map.rs`: Retail assignment selection, forward conversion, and vector creation.
- `src/c_api/map_world_coordinates.rs`: explicit rectangle projection and verified world-to-map argument handling.
- `src/bin/wow_cli/gen_ui_maps.rs`: repository-CSV assignment generator.
- `crates/sim-data-profile/src/lib.rs`: current-Retail data gate.
- `crates/sim-data-profile/data/ui_map_assignments_retail.rs`: generated assignment rows.

## Tests asserting this spec

- `tests/c_map_api.rs`: Mulgore point, inferred off-center transform, overlap selection, outside extrapolation, signed-ID truncation, Usage errors, and negative-`MapID` gap.
- `tests/patch_8_0_1_map_coordinates.rs`: explicit rectangle precedence and bounds.

## Known gaps

- **INFERRED:** exact row parity between builds 12.1.5.70783 and 12.1.5.70575 is not verified.
- The simulator's `AreaInfo` has no `ContinentID`. The negative-`MapID` AreaTable fallback is not implemented; that selected row currently returns no values.
- Native iterator row order is unknown. CSV order is only the simulator's **INFERRED** tie choice.
- The formula is **INFERRED**. Exact floating-point operation precision is unknown.
- `GetMapPosFromWorldPos` native assignment ranking and parent promotion remain unmodeled.

## Out of scope

- Assignment tables for profiles without matching exports.
- Replacing the existing non-Retail map-position behavior.
- Native world-to-map assignment search, compatibility ranking, or parent promotion.
