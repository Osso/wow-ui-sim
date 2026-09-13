# Missing unit health

`UnitHealthMissing(unit, usePredicted=true)` returns `UnitHealthMax(unit) - UnitHealth(unit)`.

## What it must do

- [x] Retail 12.0.0+ exposes the query and returns exactly one number.
- [x] Player and target queries read existing current/max health, return zero at full health, and reflect subsequent vital-state changes without changing health themselves.
- [x] Preserve existing unit lookup behavior; do not create separate health storage.

## How it works

- [Patch audit model](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: query and registration beside existing health queries.

## Tests asserting this spec

- `tests/admin_health_power_api.rs`: two `unit_health_missing_*` tests. RED 0/2 at `ec5363a18`; GREEN 2/2 at `ab204a8cb`. Independent proof pending.

## Known gaps (current cycle)

- [ ] Prediction state is unmodeled. `usePredicted` uses the existing current-health view; only explicit false is credited as the bounded query contract.
- [ ] Other unit tokens, focus/unknown-unit lookup, coercion, validation, native availability, security and consumer semantics remain unproven.

## Out of scope

Prediction modeling, security/secret enforcement and unrelated corrections to existing unit lookup policies.
