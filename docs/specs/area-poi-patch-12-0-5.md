# Area POI 12.0.5 boolean publication

`C_AreaPoiInfo.GetAreaPOIInfo` must publish the two AreaPOIInfo additions in [retained 12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 627–629. Cached retail `Blizzard_APIDocumentationGenerated/AreaPoiInfoDocumentation.lua`, lines 177–178, declares both fields nonnil booleans and `isLocked` default false. It does not declare an `isSuppressible` default. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Implement exact boolean `isSuppressible` and `isLocked` publication on mainline Retail/PTR with `retail-12-0-5` or a cumulative later epoch. Bounded batch9 GREEN is recorded below.
- [x] Preserve existing populated Stormwind Portal Room (POI 7000, map 84) ID, map, name, description, atlas, position and event/glow fields. Both new inputs initialize false; suppression false is inferred simulator policy, not native-verified behavior.
- [x] Preserve map-mismatch/unknown-POI zero-return behavior, nil-map lookup, map enumeration and seconds-left operations without changing their bodies.
- [x] Gate new publication away from earlier retail epochs and non-mainline profiles; legacy behavioral proof remains pending.
- [x] Read each new output from explicit input values, never POI identity or metadata. Contrasting true/false inputs exist only in tests.

## How it works

The record and its owning provider live in `src/c_api/c_area_poi_info.rs`. Existing public state-type paths reexport `AreaPoiInfo`; namespace registration retains its original load position. The obsolete Lua miscellaneous provider is removed, not wrapped.

## Implementation inventory

- `src/c_api/c_area_poi_info.rs`: `AreaPoiInfo` with explicit `is_suppressible: bool` / `is_locked: bool`, and the three existing registered operations. Only table serialization adds behavior, under `all(retail-12-0-5, any(profile-retail, client-ptr))`.
- `src/lua_api/state_types/collections.rs`: public record reexport, preserving `lua_api::state::AreaPoiInfo`.
- `src/lua_api/state/defaults/area_pois.rs`: all five existing seeded rows initialize both inputs false; no invented locked or suppressible production records.
- `tests/c_area_poi_probes.rs`: existing constructor supplies false inputs; unrelated behavior unchanged.

## Tests asserting this spec

`tests/area_poi_patch_12_0_5.rs` stays grouped into `integration`: two mainline tests, or one legacy control under the inverse publication gate. Before the serializer edit, existing tests were extended with test-local rows 91237/map 88007 (`true`, `false`) and 91409/map 88007 (`false`, `true`). Real matching-map and nil-map queries assert exact IDs, names, positions, return arity and booleans; mismatched maps return nothing. Map enumeration asserts both distinct IDs. No provider replacement.

## Proof ledger

- Actual batch8 RED at `693883c77003589b24b9a555141ea51a3e71e1e9`: 0/2 pass, exit 101; `isLocked must be a boolean` and `isSuppressible must be a boolean`. Log: `/tmp/patch-12.0.5-batch8-new-model-red-0.log`; exact command/revision/binary attribution: `/tmp/patch-12.0.5-batch8-new-model-runs.json`.
- RED command: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c area_poi_patch_12_0_5:: --nocapture --test-threads=1`. This predates the contrasting fixture extension and implementation; it proves the existing populated serialization failure boundary, not those new fixtures.
- Model and publisher implemented; changed Rust paths formatted directly with rustfmt. No Cargo, test/check/readability, broad acceptance, push or deployment in this implementation slice.

## Known gaps (current cycle)

- [x] Batch9 `4f9e1607c`: `area_poi_patch_12_0_5::` 2/2 PASS and `c_area_poi_probes::` 8/8 PASS. Saved build/run attribution: `/tmp/patch-12.0.5-batch9-build-result.json`, `/tmp/patch-12.0.5-batch9-runs.json`; logs `integration-0.log` and `integration-1.log` under `/tmp/patch-12.0.5-batch9-`. Queries exercise the moved C API provider, contrasting inputs, populated serialization and existing controls, not an unexecuted legacy provider.
- [ ] Independent producer audit 119: `/tmp/patch-12.0.5-batch9-independent-proof.md` pending. Final current-default fmt/check and startup after new query producers remain pending.
- [ ] Legacy proof: same patch filter selects 1 control on an earlier retail epoch or non-mainline profile. Mainline GREEN alone does not prove field absence there.
- [ ] Parent-owned applicable compilation/check/acceptance gates remain pending; formatting is not compilation proof.
- [ ] Future native probes: observe suppression/locking transitions and populated eligibility cases, including combinations of flags, map/nil-map queries and field defaults. No native transition semantics inferred from field declarations.

## Out of scope

POI suppression/locking transitions, native parity, actual game eligibility rules, other namespaces/structure deltas, vendor edits, new production content, runner rewiring, push and deployment.
