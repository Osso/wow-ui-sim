# Area POI 12.0.5 boolean publication

`C_AreaPoiInfo.GetAreaPOIInfo` must publish the two AreaPOIInfo additions in [retained 12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 627–629. Cached retail `Blizzard_APIDocumentationGenerated/AreaPoiInfoDocumentation.lua`, lines 177–178, declares both fields nonnil booleans and `isLocked` default false. It does not declare an `isSuppressible` default. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] On mainline Retail/PTR with `retail-12-0-5` or a cumulative later epoch, populated POI results publish exact boolean `isSuppressible` and `isLocked` values.
- [ ] Existing populated Stormwind Portal Room (POI 7000, map 84) retains ID, map, name, description, atlas, position and event/glow fields. Both new fields are false; suppression false is an inferred simulator initialization policy, not native-verified behavior.
- [ ] Map mismatch and unknown POI keep returning no values. Nil map lookup still selects the populated record.
- [ ] Earlier retail epochs and non-mainline profiles retain absence of both new keys and existing lookup/content behavior.
- [ ] Explicit test-local input values, not POI identity or metadata, determine each new output boolean.

## How it works

- [Lua API architecture](../lua-api.md).

## Implementation inventory

- `src/lua_api/state_types/collections.rs`: existing `AreaPoiInfo` record; new input fields are not implemented in this tests-only slice.
- `src/lua_api/state/defaults/area_pois.rs`: existing populated POIs; no edits or invented locked game records.
- `src/lua_api/globals/missing_surface/area_poi.rs`: registered state-backed producer; currently omits both new fields.

## Tests asserting this spec

- `tests/area_poi_patch_12_0_5.rs`: independent default-publication assertions and legacy absence control, each querying real populated runtime state with identity/lookup checks. Grouped automatically into `integration`.

## Known gaps (current cycle)

- [ ] Run actual RED in parent's next freshly generated integration batch. No Cargo command or behavioral proof in this slice; the pending build may predate test-file discovery.
- [ ] Proposed input stage: add explicit `is_suppressible: bool` and `is_locked: bool` to `AreaPoiInfo`; preserve existing seeded content with false initialization. Publish only at the mainline 12.0.5+ gate.
- [ ] After input stage, extend this same grouped file with test-local populated rows 91237/map 88007 (`is_suppressible=true`, `is_locked=false`) and 91409/map 88007 (`false`, `true`). Assert exact booleans and distinct IDs/names/positions via registered matching-map and nil-map queries. Never replace the query provider or alter production records to fabricate true values.

## Out of scope

- POI suppression/locking transitions, native parity and actual game eligibility rules: source fields alone establish no such behavior.
- Other structure deltas, production edits, build/test execution and test-runner rewiring: this slice supplies pending tests and the bounded input proposal only.
