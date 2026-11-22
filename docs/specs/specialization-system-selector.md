# Retail specialization-system selector

VERIFIED — Retail 12.1.5.70688 `GetSpecializationSystem` ignores Lua arguments and returns one numeric value with integer value 1 on its normal publication path.

## What it must do

- [x] Return exactly one non-nil Lua number with raw integer value 1 on the Retail normal publication path.
- [x] Ignore absent, nil, wrong-type, and extra arguments while keeping the same one-number return shape.
- [x] Allow the cached Retail `PlayerUtil.IsPlayerEffectivelyTank` consumer to call the getter.
- [x] INFERRED — On Forever, return the supplied `SpecializationSystem` value so the cached UnitUtil tank-role consumer can select its specialization path.

## How it works

- [Specialization-system selector](../wiki/systems/specialization-system-selector.md)

## Implementation inventory

- `src/lua_api/globals/real/specialization_system.rs` registers and returns the verified selector result.
- `src/lua_api/globals/register.rs` registers the getter for the Retail profile.
- INFERRED — `src/c_api/specialization_system.rs` registers the state-backed Forever getter and `A_Admin.SetSpecializationSystem`; Retail registration is unchanged.
- `src/ptr/compat_bootstrap.lua` no longer supplies the unsupported `Traits` guess.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/global_functions.rs`: the cached Retail consumer invokes the getter; calls with absent, nil, wrong-type, and extra arguments assert the raw result and return shape.
- `tests/unit_threat_consumer.rs`: the real TargetFrame threat indicator follows two supplied specialization-system values and two active specs through the original UnitUtil consumer.

## Known gaps (current cycle)

- [ ] The separate internal guard-controlled native fault path is not modeled.

## Out of scope

- Profiles other than Retail and Forever do not receive this implementation.
- INFERRED — Forever starts with `SpecializationSystem.TalentTab` (0); `A_Admin.SetSpecializationSystem` supplies either documented value for scenario tests. The Forever API declares the enum and the no-argument numeric getter in `Blizzard_APIDocumentationGenerated/SpecializationSharedDocumentation.lua:75-109`.
- INFERRED — For Retail 12.1.5.70688, raw value 1 maps to `Enum.SpecializationSystem.ChrSpecialization` based on local Retail documentation-cache metadata for 12.1.5.70783; this does not prove the exact 70688 UI payload used the same enum name. Tests do not assert this enum identity.
- Secret argument behavior is not exercised by the simulator test.
