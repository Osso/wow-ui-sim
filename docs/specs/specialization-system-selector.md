# Retail specialization-system selector

VERIFIED — Retail 12.1.5.70688 `GetSpecializationSystem` ignores Lua arguments and returns one numeric value with integer value 1 on its normal publication path.

## What it must do

- [x] Return exactly one non-nil Lua number with raw integer value 1 on the Retail normal publication path.
- [x] Ignore absent, nil, wrong-type, and extra arguments while keeping the same one-number return shape.
- [x] Allow the cached Retail `PlayerUtil.IsPlayerEffectivelyTank` consumer to call the getter.

## How it works

- [Specialization-system selector](../wiki/systems/specialization-system-selector.md)

## Implementation inventory

- `src/lua_api/globals/real/specialization_system.rs` registers and returns the verified selector result.
- `src/lua_api/globals/register.rs` registers the getter for the Retail profile.
- `src/ptr/compat_bootstrap.lua` no longer supplies the unsupported `Traits` guess.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/global_functions.rs`: the cached Retail consumer invokes the getter; calls with absent, nil, wrong-type, and extra arguments assert the raw result and return shape.

## Known gaps (current cycle)

- [ ] The separate internal guard-controlled native fault path is not modeled.

## Out of scope

- Profiles without the verified Retail behavior do not receive this implementation.
- The inferred enum-name mapping is recorded separately and is not asserted by tests.
- Secret argument behavior is not exercised by the simulator test.
