# Generated data crates

Generated Rust catalogs live under `crates/sim-data` and `crates/sim-data-profile`. The root library preserves existing catalog API paths. See [design](../wiki/design/generated-data-crates.md).

## What it must do

- [x] Preserve generated catalog bytes and observable lookup behavior (all 23 file hashes and bounded catalog tests pass).
- [ ] Compile common catalogs independently of client selection.
- [ ] Preserve atlas, global-string and aura attribute selection through forwarded features.
- [ ] Preserve root exactly-one-client validation.
- [x] Retail spell and area-name APIs return names from pinned DB2 exports and preserve missing-ID results.
- [ ] Generators and Docker builds use the moved files.

## How it works

- [Crate boundary](../wiki/design/generated-data-crates.md)

## Implementation inventory

- `crates/sim-data/`: common catalogs and profession item overrides.
- `crates/sim-data-profile/`: profile-selected catalogs.
- `src/lib.rs`, `src/c_api/c_secrets.rs`: compatibility re-exports.
- `Cargo.toml`, `Dockerfile`: dependency and build context wiring.
- `tools/gen*.py`: Python generator paths, including `tools/gen_mythic_plus_season_maps.py` for the pinned Retail 12.1.5.70391 `MythicPlusSeasonTrackedMap` export.
- `crates/sim-data/data/gen_spell_names.rs`, `crates/sim-data-profile/data/gen_area_table.rs`: Rust name-index generators.
- `src/bin/wow_cli/main.rs`: path-based generator module registration.
- `src/c_api/c_spell.rs`, `src/c_api/c_map.rs`: Retail public name lookups.
- `src/lua_api/workarounds/temporary/map_runtime_state.rs`: non-Retail area-name fallback.

## Tests asserting this spec

- `tests/c_item_probes.rs`, `tests/c_spell_probes.rs`, `tests/texture_atlas_regressions.rs`: 52 integration preservation cases pass.
- `tests/generated_data_refresh_coverage.rs`: one Lua refresh-manifest preservation case passes; this does not exercise Rust catalogs.
- `tests/spell_api.rs::test_retail_spell_name_overrides_match_db2_rows` and `test_spell_get_spell_name_unknown`: Retail SpellName values and missing-ID behavior.
- `src/lua_api/workarounds/temporary/map_runtime_state.rs::tests::retail_area_names_match_db2_rows`: AreaTable values and unknown/nil behavior.
- `crates/sim-data/data/*.rs`: 37 existing catalog unit tests pass; `mythic_plus_season_maps.rs` is checked through the original ChallengesFrame integration test.
- `crates/sim-data-profile/data/global_strings_retail.rs`: four existing selected-string unit tests pass.
- Six generator Python tests pass.

Other checklist requirements have static/build proof, not dedicated new regression tests: seven-profile metadata resolution, byte-identical root profile validation, copied original cfg expressions, cross-profile Fresh common unit and Docker COPY audit. See the [proof summary](../wiki/design/generated-data-crates.md#proof-status).

## Known gaps (current cycle)

No remaining gap in the requested static audit and bounded build/test gate. Unchecked requirements above lack dedicated regression tests; their static/build evidence is linked in the test section.

## Out of scope

Other crate cuts, test-harness restructuring, data regeneration, pushes and runtime changes.
