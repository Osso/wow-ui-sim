# Shared unit-vitals lookup

`UnitHealth`, `UnitHealthMax`, `UnitPower`, `UnitPowerMax` and their derived percentage queries read modeled unit vitals. This contract covers the shared query surface in `src/lua_api/globals/utility_system_spell/spell_api.rs`; see [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Preserve player health, power, and supported player secondary resources.
- [ ] Read distinct modeled target, focus, party, and raid-alias values rather than player values.
- [ ] Return numeric zero for absent target, inactive party/raid, and unknown tokens in health/max, power/max, percentage, and explicit secondary-power queries. This absent-unit zero is simulator policy inferred from the numeric API declarations and `UnitExists` state, not native-verified behavior.
- [ ] Preserve existing power-type metadata defaults for absent units (`0`, `MANA`); do not infer native metadata semantics from the numeric vitals contract.
- [ ] Exclude targeting-only synthetic units such as `enemy1` from numeric queries when the unit-existence model says they do not exist.
- [ ] Return exactly one value from `UnitPowerMax`: the maximum resource amount. Both cached current `UnitDocumentation.lua` and Wrath Classic 3.4.3 source documentation declare only `maxPower`.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: shared lookup and power queries.
- `src/lua_api/globals/targeting_verbs.rs`: modeled snapshot resolver.
- `src/lua_api/globals/group_queries.rs`: existing party/raid active-unit policy.
- `src/lua_api/globals/unit_stats.rs`: unrelated combat-stat queries; no duplicate maximum-health/power registration.

## Tests asserting this spec

- `tests/admin_health_power_api.rs`: modeled target/focus, active-group, absent-token and targeting-only seed regressions; existing player/target primary and player secondary tests.
- `tests/unit_stats.rs`: maximum-health behavior and the documented single-value power maximum, including an explicitly selected secondary pool.

## Known gaps (current cycle)

- [ ] Confirm grouped integration GREEN on `gui,client-wrath` after fixture correction.

## Out of scope

Unit-identity expansion, new unit/resource models, native absent-unit or power-type metadata claims, profile-gated behavior changes, security changes.
