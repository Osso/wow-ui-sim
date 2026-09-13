# Creature GUID identifiers

`C_CreatureInfo.GetCreatureID` extracts identifiers from GUID strings in `src/c_api/c_creature_info.rs`. See [API audit](../wiki/investigations/patch-12-0-0-api-audit.md) for evidence boundaries.

## What it must do

- [ ] Retail 12.0.0+ returns exactly one numeric identifier for the tested Creature GUIDs.
- [ ] Player GUIDs and empty strings return exactly one nil under the existing simulator parser policy.
- [ ] Legacy `UnitCreatureID` and existing creature namespace helpers retain their behavior.

## How it works

- [C API audit model](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_creature_info.rs`: GUID parser and namespace method.
- `src/c_api/mod.rs`: module registration.
- `src/lua_api/globals/missing_surface/creature_info.rs`: existing namespace wiring.
- `src/lua_api/globals/unit_misc.rs`: legacy unit-token query.

## Tests asserting this spec

- `tests/c_system_api.rs`: explicit GUID results and return counts.

## Known gaps (current cycle)

- [ ] Independent profile and legacy regression verification.

## Out of scope

Complete GUID-format validation, coercion/error compatibility, native malformed-input policy, and secret enforcement require separate evidence. No creature database is implied by identifier extraction.
