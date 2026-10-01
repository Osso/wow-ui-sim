# Unit-stat output restriction

The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) marks exactly 50 player/unit stat APIs `SecretWhenUnitStatsRestricted`. This bounded implementation covers secrecy for 40 existing concrete default-retail functions, not native parity of their underlying stat models.

## What it must do

- [ ] Under `retail-12-0-5`, an explicit `SimState.unit_stats_restricted` boolean defaults false. `C_Secrets.ShouldUnitStatsBeSecret()` returns that input as one plain boolean.
- [ ] Restriction marks every numeric result of the 40 supported queries secret, preserving modeled values, return count and order. Unrestricted calls remain plain; nil returns remain nil.
- [ ] Tainted callers receive opaque host-produced numeric results without changing caller taint; Lua secret-input checks and unwrap/arithmetic denial remain enforced.
- [ ] Queries outside the source block remain plain. Other profiles retain existing behavior and namespace feature gates.
- [ ] No automatic combat/aura activation. Native activation rules are unknown; explicit input is an approved simulator policy, not native-verified inference.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/{sim_state.rs,state.rs}` — explicit input and default.
- `src/c_api/{mod.rs,c_secrets.rs}` — feature-scoped predicate and namespace registration.

## Tests asserting this spec

- `tests/character_stats/stat_restriction.rs` — grouped predicate and output behavior.

## Known gaps (current cycle)

- [ ] Output wrapping and targeted post-change proof pending. Predicate pre-change RED: `/tmp/patch-12.0.5-stats-predicate-red.log` (1 failed, missing predicate); corrected shared build revision `a3ba2a23a`.
- [ ] Separately implement 10 unsupported default-retail base models: `GetMastery`, `GetOverrideAPBySpellPower`, `GetOverrideSpellPowerByAP`, `GetPetMeleeHaste`, `GetPowerRegen`, `GetPowerRegenForPowerType`, `GetSpellPenetration`, `GetSturdiness`, `PlayerEffectiveAttackPower`, `UnitWeaponAttackPower`. No new no-op stubs here.
- [ ] Future native probes: predicate transition across combat/encounter/aura contexts, absent-unit nil/zero behavior, all result positions and caller taint. Existing stat formulas/placeholders are not claimed native parity.

## Out of scope

- Automatic restriction producers and the 10 missing base models require separate implementation; the Forever-only aura flag is not this input.
