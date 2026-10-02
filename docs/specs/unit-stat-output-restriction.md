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

All 40 existing APIs below are implemented; batch7 restriction fixtures PASS 4/4. Ten additional models separately PASS 13/13; this is not native stat parity. Counts describe APIs, not independent native models.

| Source APIs | Count |
|---|---:|
| GetAttackPowerForStat, GetBlockChance, GetDodgeChance, GetDodgeChanceFromAttribute, GetParryChance, GetParryChanceFromAttribute, GetShieldBlock | 7 |
| GetAvoidance, GetCombatRating, GetCombatRatingBonus, GetCritChance, GetExpertise, GetExpertisePercent, GetHaste, GetHitModifier, GetLifesteal, GetManaRegen, GetMasteryEffect, GetMeleeHaste, GetModResilienceDamageReduction, GetPvpPowerDamage, GetPvpPowerHealing, GetRangedCritChance, GetRangedHaste, GetSpeed, GetSpellCritChance, GetSpellHitModifier, GetVersatilityBonus | 21 |
| GetPetSpellBonusDamage, GetSpellBonusDamage, GetSpellBonusHealing | 3 |
| UnitArmor, UnitAttackPower, UnitAttackSpeed, UnitDamage, UnitRangedAttackPower, UnitRangedDamage, UnitSpellHaste, UnitStat, GetUnitSpeed | 9 |

## Batch52 exact-row annotation audit — 2026-10-01

Scope: five retained IDs only. Literal deltas in the [retained GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) are quoted below; line numbers identify the added annotation. Each adds output restriction only, not a signature, ordinary value, argument policy, or native activation rule.

| Exact retained ID | Literal source delta | Existing concrete fixture (ordered results) | Model limits, separate from annotation mechanism |
|---|---|---|---|
| `global api-Unit UnitStat-514` | `Unit UnitStat` / `+ SecretWhenUnitStatsRestricted` | `UnitStat('player', 1)` → `{300, 300, 0, 0}` | Repeated base/effective value and zero modifiers do not prove native decomposition; other indexes and synthetic nonplayer snapshots are not covered by this fixture. |
| `global api-Unit UnitArmor-500` | `Unit UnitArmor` / `+ SecretWhenUnitStatsRestricted` | `UnitArmor('player')` → `{1234, 1234, 1234, 0, 0}` | Single armor value repeated across components, zero modifiers; no native component parity. |
| `global api-PlayerScript GetCombatRating-424` | `PlayerScript GetCombatRating` / `+ SecretWhenUnitStatsRestricted` | `GetCombatRating(9)` → `{360}` | Exact restriction fixture covers index9; four state-backed rating categories do not establish other-index zero placeholders as native values. |
| `global api-PlayerScript GetHaste-438` | `PlayerScript GetHaste` / `+ SecretWhenUnitStatsRestricted` | `GetHaste()` → `{3}` from haste rating510 | Simulator conversion expectation, not native formula proof. |
| `global api-Unit GetUnitSpeed-498` | `Unit GetUnitSpeed` / `+ SecretWhenUnitStatsRestricted` | `GetUnitSpeed('player')` → `{9, 9, 14, 4}` | Rust adds this fixture under `client-retail`; configured player identity/speeds only, nonplayer zero placeholders not native parity. |

### Existing behavioral coverage; no new execution

The existing [Lua fixture loop](../../tests/character_stats/stat_restriction_fixtures.lua) and [Rust restriction tests](../../tests/character_stats/stat_restriction.rs) substantively exercise **all five** exact APIs, not merely registration or source text:

- `stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle` runs the shared fixtures plain → restricted → plain. `assertStatOutputs` checks exact arity, each ordered position's value, secrecy and access status. Unrelated queries, numeric literals and the predicate remain plain. This exercises the existing explicit restriction input and numeric secret wrapper without requiring producer/model rewrites.
- `stat_restriction_tainted_callers_receive_opaque_host_results` checks every fixture's arity and every result's secret/inaccessible status, denied unwrap/arithmetic, and unchanged `StatRestrictionProbe` caller taint. Exact values are checked by the trusted fixture loop, not by unwrapping inside the tainted caller.
- `stat_restriction_absent_unit_shapes_and_constant_outputs` checks secret zeroes and arities4/5/4 for missing-unit `UnitStat`/`UnitArmor`/`GetUnitSpeed`. These assertions establish simulator behavior only. `stat_restriction_predicate_defaults_plain` checks the plain single-boolean predicate and explicit toggles. Secret-selector rejection for rating/speed is existing extra behavior, not a requirement added by these output-only rows.

No uncovered annotation boundary is demonstrated for the five concrete seeded cases. Broader index/unit combinations lack exact per-case restriction evidence; native activation, output/access policy, stat decomposition and inferred formulas remain unverified. These gaps must not be converted into native parity or bulk row credit. No duplicate tests, fabricated RED, code changes, or current execution proof are introduced by this audit.

### Acceptance and history remain open

This supplement does not narrow or replace the prior40-API matrix, batch7 observed4/4 restriction proof, separate ten-model13/13 proof, or historical RED/pending claims below. Those are historical observations, not batch52 current acceptance. Parent current execution and independent validation remain required before any acceptance/checkbox/source-accounting change. All five exact rows remain pending; no capability, page-coverage, data, wiki or PLAN accounting changes. All selected patch history and the broader goal remain open.

## Known gaps (current cycle)

- [ ] Batch7 restriction GREEN observed 4/4; parent-owned independent final gate pending. Predicate pre-change RED: `/tmp/patch-12.0.5-stats-predicate-red.log` (1 failed, missing predicate); corrected shared build revision `a3ba2a23a`. Output RED at `9a50d8a5c`: `/tmp/patch-12.0.5-batch4-stats-output-red.log`, predicate passes / three output cases fail, including `GetAttackPowerForStat secrecy 1`. Fresh evidence verified 2026-10-01; earlier retained provenance dates remain unchanged.
- [ ] Batch7 observed GREEN 13/13 for the ten separately implemented [retail stat input models](retail-missing-stat-inputs.md); concrete input, order, update and security fixtures are RED at `eac08bda3`. No new no-op stubs here.
- [ ] Future native probes: predicate transition across combat/encounter/aura contexts, absent-unit nil/zero behavior, all result positions and caller taint. Existing stat formulas/placeholders are not claimed native parity.

## Out of scope

- Automatic restriction producers remain outside this contract; the ten additional base models have their [own contract](retail-missing-stat-inputs.md). The Forever-only aura flag is not this input.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

`character_stats::stat_restriction::` PASS 4/4 (`/tmp/patch-12.0.5-batch7-integration-1.log`) retains the 40-API matrix; separate ten-model fixtures PASS 13/13. No native activation/formula claim.
