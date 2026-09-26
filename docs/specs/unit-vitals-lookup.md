# Shared unit-vitals lookup

`UnitHealth`, `UnitHealthMax`, `UnitPower`, `UnitPowerMax` and their derived percentage queries read modeled unit vitals. This contract covers the shared query surface in `src/lua_api/globals/utility_system_spell/spell_api.rs`; see [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Preserve player health, power, and supported player secondary resources.
- [ ] Read distinct modeled target, focus, party, and raid-alias values rather than player values.
- [ ] Return numeric zero for absent target, inactive party/raid, and unknown tokens in health/max, power/max, percentage, and explicit secondary-power queries. This absent-unit zero is simulator policy inferred from the numeric API declarations and `UnitExists` state, not native-verified behavior.
- [ ] Preserve existing power-type metadata fallback for absent units (`0`, `MANA`); do not infer native metadata semantics from the numeric vitals contract.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: shared lookup and power queries.
- `src/lua_api/globals/targeting_verbs.rs`: modeled snapshot resolver.
- `src/lua_api/globals/group_queries.rs`: existing party/raid active-unit policy.

## Tests asserting this spec

- `tests/admin_health_power_api.rs`: `unit_vitals_resolve_present_target_focus_and_absent_tokens`, `unit_vitals_only_use_active_party_members` (GREEN pending); existing player/target primary and player secondary tests.

## Known gaps (current cycle)

- [ ] Confirm grouped integration GREEN on `gui,client-wrath` after fixture correction.

## Out of scope

Unit-identity expansion, new unit/resource models, native absent-unit or power-type metadata claims, profile-gated behavior changes, security changes.
