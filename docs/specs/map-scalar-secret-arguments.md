# Map scalar secret arguments

`C_Map.GetMapInfo(uiMapID)` and `C_Map.GetMapArtID(uiMapID)` must accept authentic secret-number selectors from untainted callers in current Retail/PTR while preserving existing ordinary-input and historical-profile behavior. Their source lives in `src/c_api/c_map.rs`; see the [existing map investigation](../wiki/investigations/map-display-info-model.md) for related secret-input behavior and evidence boundaries. Implementation is user-approved; bounded runtime proof and remaining limits are recorded below.

Cached Retail and PTR `AddOns/Blizzard_APIDocumentationGenerated/MapDocumentation.lua` both declare these two functions with required numeric `uiMapID`, `MayReturnNothing = true`, and `SecretArguments = "AllowedWhenUntainted"`. This is declaration evidence, not native parity. `client-retail` enables `retail-12-1-0`; `client-ptr` enables `retail-12-1-5`, which includes `retail-12-1-0`. Historical builds without that capability retain their existing behavior; this contract does not infer historical native secret policy.

## What it must do

Saved d97 epoch `20261010T214515Z` reaches six tests: two authentic untainted-secret parser failures, four compatibility PASS. Unchanged producer `fae775099c3cd8b4f412547b6dc70cb09d89c20e` and unchanged d97 tests receive independent bounded Retail93/PTR91 PASS, including all six target tests in each. Historical `profile-retail,retail-12-0-7` passes only the two ordinary tests; current secret tests are capability-excluded. See [scalar proof](../wiki/investigations/map-display-info-model.md#scalar-map-selectors--bounded-retailptr-green) for exact scopes and retained reports. Existing MapDisplay proof is separate.

### Current Retail/PTR secret arguments

- [x] Bounded fixture coverage: both `*_untainted_secret_returns_one_fixture_*` tests assert authentic host-secret map84, one result and distinctive art620104/name; ordinary fixture tests assert corresponding values/count. Arbitrary integer IDs/native equivalence remain unproved.
- [x] Both `*_tainted_addon_accepts_ordinary_and_rejects_secret` tests assert ordinary fixture output/count and authentic secret rejection under `pcall`.
- [x] Four current secret/taint tests assert rooted-wrapper identity/secrecy and caller taint preservation; addon tests also assert parent restoration.

### Existing compatibility

- [ ] PARTIAL: art ordinary compatibility test covers one fixture result and zero returns for absent9999/999999; current secret test covers present84 only. Secret absent-map branches are not directly tested.
- [ ] PARTIAL: info ordinary compatibility test covers fixture84, generic9999 fields and zero returns for999999; current secret test covers fixture identity/name/count only. Full table equality and secret generic/absent branches are not directly tested.
- [x] Both ordinary compatibility tests assert fractional/string/missing/boolean/table and both i32-overflow rejection diagnostics in Retail, PTR, historical12.0.7 and saved Wrath/Mists/Era/Anniversary/Forever executions.
- [ ] PARTIAL: historical Retail12.0.7 ordinary2 PASS plus five other profiles ordinary2 each (10 executions of two distinct tests) independently admitted. Current-secret tests are capability-excluded; other historical epochs and native secret policy remain unproved.

## How it works

- [Map display model and bounded evidence](../wiki/investigations/map-display-info-model.md)
- [Related MapDisplay contract](map-display-info.md)
- [Lua API architecture](../lua-api.md)
- [Client profile architecture](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_map.rs` — existing target registrations, scalar selector parsing, catalog lookups, generic map details, and related MapDisplay secret-aware precedent.
- `src/lua_bridge/from_stack.rs` — unchanged i32 conversion and diagnostics extracted into a pure value conversion used by stack extraction and the two current-epoch selectors.
- `src/lua_bridge/mod.rs` — crate-private conversion re-export gated to its `retail-12-1-0` consumer; internal ordinary conversion remains available in every profile. Ungated re-export produced a Mists unused-import warning; no warning suppression or new public API.
- `Cargo.toml` — current Retail/PTR capability selection and historical epoch feature boundaries.
- `tests/c_map_probes.rs` — existing ordinary map controls and authentic host-secret/tainted-caller precedent.
- `tests/map_scalar_secret_inputs.rs` — four current-epoch secret-input tests and two ungated ordinary compatibility tests, executed under the generated `integration` target as `map_scalar_secret_inputs::*`; saved pre-producer RED reaches all six; unchanged tests pass all six in current Retail/PTR and only two ordinary tests in historical Retail12.0.7.

## Tests asserting this spec

`tests/map_scalar_secret_inputs.rs` supplies new target coverage. Existing `tests/c_map_probes.rs` ordinary target tests and MapDisplay security tests are controls/precedent, not admitted RED/GREEN for this slice. The saved pre-producer RED executes six actual target tests: two untainted-secret FAIL and four compatibility PASS. Final independent reports admit changed-producer Retail93/PTR91 PASS, all six targets included; historical12.0.7 admits two ordinary PASS only.

Required main-thread evidence must use a deliberately seeded map row with distinctive art ID/name and an authentic host-VM secret numeric selector, not a Lua lookalike. For both APIs, assert exact result count and fixture values, untainted secret acceptance, tainted ordinary acceptance, tainted secret rejection, and wrapper/caller preservation. Retain known/absent-map controls, `GetMapInfo` generic-range controls, ordinary invalid-selector controls, and historical-profile compatibility coverage. Fixture values are simulator test data, not native catalog facts.

## Known gaps (current cycle)

- [x] Authentic RED2FAIL/4PASS preserved; unchanged d97 target assertions reach GREEN6/6 under current Retail and PTR.
- [x] Bounded historical Retail12.0.7 ordinary preservation reaches2/2; not all historical profiles.
- [x] Saved five-profile ordinary preservation: Wrath/Mists/Era/Anniversary/Forever each2/2 PASS, five compile/execution exits0, independent source/artifact/feature admission. [Retained report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-scalar-five-profile-ordinary/final-report.md) distinguishes ten executions from two distinct tests and differing submission revisions.
- [ ] Wider other-profile runtime/native acceptance and full handoff remain OPEN. [Saved714 suite](../wiki/investigations/fullsuite-714-2026-10-10.md) FAIL15,017PASS/4FAIL/19skips includes allsix scalarPASS; later b061 export-only correction passes seven profilecompilerchecks, separate from five-profile ordinary runtime proof. Older956FAIL3/b048FAIL4 unchanged; no broad rerun for docs.

## Out of scope

Native parity and historical native secret-policy claims; guessed or refreshed map catalogs; parser-validation changes; changes to established generic/default/zero-return behavior; other `C_Map` members or other APIs; generic bridge redesign. This docs-only stage runs no builds/tests; bounded simulator PASS is not full implementation, native or handoff completion.
