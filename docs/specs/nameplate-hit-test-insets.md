# Nameplate hit-test inset configuration

Retail 12.0.5 adds per-`Enum.NamePlateType` hit-test offsets. The cached official `Blizzard_APIDocumentationGenerated/NamePlateManagerDocumentation.lua` specifies the signatures and security annotations; [patch notes](../../data/patch-api/sources/12.0.5-api-changes.txt) announce the offset override. This contract models configuration only, not world nameplate interaction. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [ ] Publish `C_NamePlateManager.GetNamePlateHitTestInsets(type)` and `SetNamePlateHitTestInsets(type, left, right, top, bottom)` under `retail-12-0-5`. Official `NamePlateConstantsDocumentation.lua` publishes Friendly=0 and Enemy=1.
- [ ] Return exactly four `uiUnit` numbers in left/right/top/bottom order; independently retain each type's complete tuple. Setting `(2.5, -1.25, 3, 4)` preserves fractions and negative offsets; replacement replaces all four values and returns no values.
- [ ] Default each tuple to `(0, 0, 0, 0)`, explicitly inferred simulator configuration, not a native-observed default. Reject unsupported/fractional/non-numeric types and missing/non-numeric/nonfinite inset values without mutation.
- [ ] Honor getter `SecretArguments=AllowedWhenUntainted` through existing VM authentication; reject all setter secret arguments (`NotAllowed`). Infer the setter's `HasRestrictions` policy as an ordinary untainted-caller guard, without declassifying callers; failed writes leave all configuration unchanged.
- [ ] Leave `C_NamePlate.GetNamePlateForUnit` nil and `GetNamePlates` empty. Do not invent plate frames or clamp stored offsets against fabricated bounds.

Official prose says positive offsets decrease and negative offsets increase the hit area, with actual hit testing clamped to plate bounds. Configuration retains both signs. No plate-bounds model exists, so this change does not implement or claim that clamp.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_nameplate_manager.rs`: typed per-type inset storage, query/write validation, and namespace registration.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: gated namespace publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment configuration and inferred zero default.
- `tests/nameplate_hit_test_insets.rs`: grouped public-query and security behavior tests.

## Tests asserting this spec

- `tests/nameplate_hit_test_insets.rs`: both published types, exact four-value arity, fractional/negative round trip, replacement/isolation, invalid inputs preserving state, secret/restricted callers, and no fabricated plates.

## Known gaps (current cycle)

- [ ] Native default probe: before any setter, call the getter twice each with Friendly=0 and Enemy=1; record all four results and client build.
- [ ] Native allowed-caller probe: attempt setter `(0, 2.5, -1.25, 3, 4)` and `(1, 2.5, -1.25, 3, 4)` from ordinary addon code out of combat and in combat, then from an available trusted caller. Record errors and getter results; the ordinary restriction is inferred until this evidence exists.
- [ ] Future real-bounds probe: once actual plate bounds are modeled, compare hit testing for `(2.5, -1.25, 3, 4)` and `(-1000, -1000, -1000, -1000)` at points just inside/outside all four plate edges. Native documentation establishes clamping, not an invented numeric bound.

## Out of scope

World hit-grid/rendered interaction, plate catalogs/creation, camera queries, simplified/size setters, guessed bounds, other-profile publication, native conformance, vendor changes, and broad verification gates.
