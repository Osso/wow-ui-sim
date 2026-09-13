# Missing unit power

`UnitPowerMissing(unitToken, powerType?, unmodified=false)` returns `UnitPowerMax() - UnitPower()` for the requested pool.

## What it must do

- [x] Retail 12.0.0+ exposes the query and returns exactly one number.
- [x] Read existing player/target primary pools and supported player secondary pools through the same lookup as the power queries.
- [x] Reflect current/max state changes, return zero for full pools and leave resource state unchanged.

## How it works

- [Patch audit model](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: query and registration beside existing power queries.

## Tests asserting this spec

- `tests/admin_health_power_api.rs`: three `unit_power_missing_*` tests; RED 0/3 at `65873915e`. GREEN and independent proof pending.

## Known gaps (current cycle)

- [ ] Unmodified resource scaling is unmodeled; only `unmodified=false` ordinary queries are credited.
- [ ] Other unit tokens, target secondary pools, invalid/coerced inputs, negative values, native availability, security and consumer semantics remain unproven.

## Out of scope

New resource models, secret/security enforcement and corrections to existing unit/power lookup policies.
