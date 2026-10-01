# Unit-stat output restriction

The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) marks exactly 50 player/unit stat APIs `SecretWhenUnitStatsRestricted`. This bounded implementation covers secrecy for 40 existing concrete default-retail functions, not native parity of their underlying stat models.

## What it must do

- [ ] Under `retail-12-0-5`, an explicit `SimState.unit_stats_restricted` boolean defaults false. `C_Secrets.ShouldUnitStatsBeSecret()` returns that input as one plain boolean.
- [ ] Restriction marks every numeric result of the 40 supported queries secret, preserving modeled values, return count and order. Unrestricted calls remain plain; nil returns remain nil.
- [ ] Tainted callers receive opaque host-produced numeric results without changing caller taint; Lua secret-input checks and unwrap/arithmetic denial remain enforced.
- [ ] Queries outside the source block remain plain. Other profiles retain existing behavior and namespace feature gates.
- [ ] No automatic combat/aura activation. Native activation rules are unknown; explicit input is an approved simulator policy, not native-verified inference.

## How it works

- [Bounded audit and implementation notes](../wiki/investigations/patch-12-0-5-api-audit.md#bounded-unit-stat-output-secrecy-verified-2026-10-01)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — explicit input and default.
- `src/c_api/{mod.rs,c_secrets.rs}` — feature-scoped predicate, namespace registration, and trusted numeric output producer.
- `src/lua_api/globals/unit_stats.rs` — eight unit queries and seven player queries.
- `src/lua_api/globals/real/combat_stats.rs` — 21 player queries, including aliases; unrelated zero-return queries stay plain.
- `src/lua_api/globals/{cooldown_probes.rs,real/pet_stats.rs,real/unit_speed.rs}` — two spell-power queries, pet spell bonus, and four-value speed query.

## Tests asserting this spec

- `tests/character_stats/stat_restriction.rs` — grouped predicate, all-result value/arity/order/toggle, tainted caller, secret-input, absent-unit, and unrelated-query behavior.
- `tests/character_stats/stat_restriction_fixtures.lua` — concrete current-model expectations for 39 queries; Rust setup adds `GetUnitSpeed` as the 40th under default retail.

### Exact default-retail secrecy coverage

All rows below are implemented; targeted post-change GREEN is pending. Counts describe APIs, not independent native models.

| Source APIs | Count |
|---|---:|
| GetAttackPowerForStat, GetBlockChance, GetDodgeChance, GetDodgeChanceFromAttribute, GetParryChance, GetParryChanceFromAttribute, GetShieldBlock | 7 |
| GetAvoidance, GetCombatRating, GetCombatRatingBonus, GetCritChance, GetExpertise, GetExpertisePercent, GetHaste, GetHitModifier, GetLifesteal, GetManaRegen, GetMasteryEffect, GetMeleeHaste, GetModResilienceDamageReduction, GetPvpPowerDamage, GetPvpPowerHealing, GetRangedCritChance, GetRangedHaste, GetSpeed, GetSpellCritChance, GetSpellHitModifier, GetVersatilityBonus | 21 |
| GetPetSpellBonusDamage, GetSpellBonusDamage, GetSpellBonusHealing | 3 |
| UnitArmor, UnitAttackPower, UnitAttackSpeed, UnitDamage, UnitRangedAttackPower, UnitRangedDamage, UnitSpellHaste, UnitStat, GetUnitSpeed | 9 |

## Known gaps (current cycle)

- [ ] Targeted post-change GREEN and parent-owned final gate pending. Predicate pre-change RED: `/tmp/patch-12.0.5-stats-predicate-red.log` (1 failed, missing predicate); corrected shared build revision `a3ba2a23a`. Output RED at `9a50d8a5c`: `/tmp/patch-12.0.5-batch4-stats-output-red.log`, predicate passes / three output cases fail, including `GetAttackPowerForStat secrecy 1`. Fresh evidence verified 2026-10-01; earlier retained provenance dates remain unchanged.
- [ ] Separately implement 10 unsupported default-retail base models: `GetMastery`, `GetOverrideAPBySpellPower`, `GetOverrideSpellPowerByAP`, `GetPetMeleeHaste`, `GetPowerRegen`, `GetPowerRegenForPowerType`, `GetSpellPenetration`, `GetSturdiness`, `PlayerEffectiveAttackPower`, `UnitWeaponAttackPower`. No new no-op stubs here.
- [ ] Future native probes: predicate transition across combat/encounter/aura contexts, absent-unit nil/zero behavior, all result positions and caller taint. Existing stat formulas/placeholders are not claimed native parity.

## Out of scope

- Automatic restriction producers and the 10 missing base models require separate implementation; the Forever-only aura flag is not this input.
