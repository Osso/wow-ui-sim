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

## Batch52 saved parent observed proof — 2026-10-02

[Combined batch51/52 proof SSOT](action-spell-slot-identifiers.md#reconciled-combined-batch5152-parent-green--2026-10-02) owns artifacts, revisions, hashes, commands and timings. Annotation audit `3d357dd10` changed docs only; existing provider and fixtures unchanged, no fabricated RED. Saved corrected combined dirty-source execution observes the four existing restriction tests PASS, not independent acceptance or clean-revision proof.

Scope remains exactly **514 UnitStat, 500 UnitArmor, 424 GetCombatRating, 438 GetHaste, 498 GetUnitSpeed**. Concrete player fixtures above prove meaningful explicit inputs and the existing numeric wrapper: ordered values/arity plain → restricted → plain, tainted opaque outputs and retained caller taint. Repeated components, zero placeholders, nonplayer snapshots, other indexes, inferred formulas and native activation/access/secrecy remain unknown; no native model parity follows.

Prior40-API matrix, batch7 history and separate ten-model proof remain intact. Historical current-execution-pending wording above is superseded only by this saved parent observation. Verifier414, independent acceptance/accounting and every unchecked requirement remain pending; no row/page/capability credit or whole-goal closure.

## Exact-five annotation acceptance — 2026-10-02

Parent accepted independent414 for **514 UnitStat,500 UnitArmor,424 GetCombatRating,438 GetHaste,498 GetUnitSpeed only**. [Combined acceptance SSOT](action-spell-slot-identifiers.md#independent-bounded-acceptance--2026-10-02) owns gate results,122 uniquePASS/startup0 `[]`, dirty provenance, exact source accounting and limits. Four existing restriction tests substantively exercise all five player fixtures with ordered value/arity preservation, every-result wrapping, plain/restricted/plain transitions and tainted opacity. Existing providers unchanged/alreadyGREEN; no fabricated RED or unnecessary rewrite.

Prior40-API partial capability, other unaccepted source rows and broad unchecked requirements stay unchanged. Components/modifier placeholders, other indexes/nonplayer snapshots, speed combinations, simulator formulas, native restriction activation/access and all-profile execution remain unproved. Earlier pending wording is historical; this section supersedes it for exactly these five annotation boundaries, not full models or whole-page/goal acceptance.

## Batch54 exact-eight annotation supplement — 2026-10-02

Scope: numeric output wrapping on the current explicit player outputs for exactly eight retained rows. Full subject blocks in the [retained source](../../data/patch-api/sources/12.0.5-api-changes.txt) and full [register](../../data/patch-api/sources/12.0.5-register.json) records were inspected: each contains only the literal delta `+ SecretWhenUnitStatsRestricted`, with the exact added-line ID below. No signature, formula, selector policy or activation change is present in these eight source blocks. All eight [page-coverage records](../../data/patch-api/sources/12.0.5-page-coverage.json) remain `audit-pending`, with empty capabilities.

The cached retail `Blizzard_APIDocumentationGenerated/{PlayerScript,Unit}Documentation.lua` declarations were also inspected in full for these functions. All eight declare `SecretWhenUnitStatsRestricted = true`. `GetCombatRatingBonus` takes nonnil `ratingIndex: luaIndex` and declares a nullable numeric result; `GetVersatilityBonus` takes nonnil `combatRating: luaIndex` and declares a nonnil numeric result. Both, and the two Unit queries taking nonnil `unit: UnitToken`, declare `SecretArguments = "AllowedWhenUntainted"`. That existing declaration is not an additional delta in these retained rows or proof of selector compliance. The other four player functions declare no arguments. Ordered return names below come from those declarations; all results except the rating-bonus result are declared nonnil numbers. Cached declarations are contract context, not native execution evidence.

### Exact providers, ordered fixtures and wrappers

Every listed result position uses `push_stat_number`; the melee-haste function reaches it through `get_haste`. The [combat provider](../../src/lua_api/globals/real/combat_stats.rs), [unit provider](../../src/lua_api/globals/unit_stats.rs), [Rust setup](../../tests/character_stats/stat_restriction.rs) and [ordered Lua fixtures](../../tests/character_stats/stat_restriction_fixtures.lua) establish the following local cases. Setup seeds level10, strength300, agility200, intellect500, crit rating360, haste rating510, mastery rating520 and versatility rating1025. Percent conversions below describe observed current-model outputs, not native divisors.

| Exact retained ID; literal delta | Provider fields/formula and per-position wrapping | Ordered fixture; arity and declared return order |
|---|---|---|
| `global api-PlayerScript GetCombatRatingBonus-426`; `+ SecretWhenUnitStatsRestricted` | `get_combat_rating_bonus(9)` selects `player.stats.crit_pct()` from explicit crit rating360; one `push_stat_number(bonus)`. | `GetCombatRatingBonus(9)` → `{2}`; arity1, `result`. |
| `global api-PlayerScript GetCritChance-428`; `+ SecretWhenUnitStatsRestricted` | `get_crit_chance`: `player.stats.crit_pct() + 5.0`; one wrapped result. | `GetCritChance()` → `{7}`; arity1, `result`. |
| `global api-PlayerScript GetMasteryEffect-448`; `+ SecretWhenUnitStatsRestricted` | `get_mastery_effect`: `mastery = player.stats.mastery_pct()` from rating520; separately wraps `8.0 + mastery`, then `mastery`. | `GetMasteryEffect()` → `{12, 4}`; arity2, `masteryEffect, bonusCoefficient`. Local second output is the rating-derived percent; its native coefficient meaning is unproved. |
| `global api-PlayerScript GetMeleeHaste-450`; `+ SecretWhenUnitStatsRestricted` | `get_melee_haste` delegates to `get_haste`, which wraps `player.stats.haste_pct()` from rating510. | `GetMeleeHaste()` → `{3}`; arity1, `result`. Shared getter is legitimate local annotation backing, not native equivalence with accepted row438. |
| `global api-Unit UnitDamage-506`; `+ SecretWhenUnitStatsRestricted` | `unit_damage` resolves player through `stats_for`; `AP = strength + agility + level*10 = 600`; bounds `50 + level*5 + AP*0.1 = 160`, `80 + level*7 + AP*0.1 = 210`. Seven separate wraps: bounds, four zero placeholders, multiplier1. | `UnitDamage('player')` → `{160, 210, 0, 0, 0, 0, 1}`; arity7, `minDamage, maxDamage, offhandMinDamage, offhandMaxDamage, posBuff, negBuff, percent`. |
| `global api-Unit UnitRangedDamage-510`; `+ SecretWhenUnitStatsRestricted` | `unit_ranged_damage` uses the same player bounds; six separate wraps: fixed speed2, bounds160/210, two zero modifiers, multiplier1. | `UnitRangedDamage('player')` → `{2, 160, 210, 0, 0, 1}`; arity6, `speed, minDamage, maxDamage, posBuff, negBuff, percent`. |
| `global api-PlayerScript GetManaRegen-444`; `+ SecretWhenUnitStatsRestricted` | `get_mana_regen`: `base = 1 + max(intellect,0)/500 = 2`; non-Forever total equals base; separately wraps total and `base*0.5`. | `GetManaRegen()` → `{2, 1}`; arity2, `baseManaRegen, castingManaRegen`. Local values do not establish native component formulas. |
| `global api-PlayerScript GetVersatilityBonus-494`; `+ SecretWhenUnitStatsRestricted` | `get_versatility_bonus` wraps `player.stats.versatility_pct()` from rating1025; provider does not inspect the supplied index. | `GetVersatilityBonus(14)` → `{5}`; arity1, `result`. Index-dependent native behavior is not established. |

### Reused substantive proof; no new execution or gate

Saved `/tmp/patch-12.0.5-batch53-green-run-8.stdout` reports all four `character_stats::stat_restriction::` tests PASS, zero failures, 9598 filtered out, test duration0.84s. `/tmp/patch-12.0.5-batch53-green-runs.json` binds this run to revision `eb52b92c034818eb1b09017230b2c8f90f0f93a8`, dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, integration executable SHA256 `f40591e7e73256837796fddba1dd77b09fadf7530db91bfe7fdbe05bbbc1c80e`, exit0 and wall duration0.8640741010894999s. Saved argv is `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c character_stats::stat_restriction:: --test-threads=1`.

Scoped comparison against `eb52b92c0` shows both providers and both restriction fixture/test files unchanged. Batch53 fixture correction affects BreakUpLargeNumbers tests/spec, not these stat inputs/providers. This is reusable dirty-source-bound development proof, not clean-revision, native or independent batch54 acceptance. No tests/checks were rerun.

The all-output test calls every exact fixture plain → restricted → plain. Its `pack` stores `select('#', ...)`, so arity is checked independently of Lua table length; each ordered position checks exact value within `1e-10`, secrecy and access, using trusted unwrap only when restricted. The tainted-caller test separately checks arity, every position secret/inaccessible, failed unwrap and arithmetic, and preserved `StatRestrictionProbe` taint. Predicate/default and absent-unit tests are controls; their missing-unit cases are not UnitDamage/UnitRangedDamage evidence. No candidate annotation gap is demonstrated by these seeded outputs; no producer rewrite, fabricated RED or duplicate fixture is warranted.

### Eight known limits; acceptance/accounting unchanged

1. Rating-bonus426: only index9/value2; other rating categories, index policy, nullable outcomes and native conversion unproved.
2. Crit428: seeded result7 only; base5, conversion, mutation and native decomposition unproved.
3. Mastery448: both nonzero positions wrapped; native base8, coefficient semantics and component mutation unproved.
4. Melee-haste450: direct shared-getter wrapping proved locally; weapon/modifier behavior and native alias equivalence unproved.
5. Damage506: synthetic bounds only; offhand and four zero slots do not establish native weapons, modifiers or formula.
6. Ranged-damage510: fixed speed and repeated synthetic bounds; separate ranged model, zero modifiers and native formula unproved.
7. Mana444: explicit intellect-derived pair only; native regeneration/component semantics, Spirit/profile differences unproved.
8. Versatility494: explicit rating result5 only; ignored index, damage-taken distinction, modifiers and native formula unproved.

Across all eight, nonplayer cases, other indexes, changing inputs, native activation and access policy remain unproved. These are evidence limits, not speculative new modeling requirements or reasons to defer the known output-annotation mechanism behind native formula parity.

The candidate map's final count paragraph is wrong: it omits accepted424/438 from the accepted stat-annotation five. Accepted514/500/424/438/498 remain unchanged; do not adopt that paragraph's accounting. Prior40 partial history remains intact. These eight receive **no source credit** until independent validation and parent accounting; parent may extend existing443 to this bounded unchanged-source evidence, with no shadow gate here. No coverage/data/wiki/PLAN, requirements checkbox, source/test or broader acceptance changes.

## Exact-eight annotation acceptance — 2026-10-02

Parent accepted independent443's separate exact8 verdict: **426/428/444/448/450/494/506/510**, all21 numeric result positions from the concrete player fixtures above. [Combined acceptance SSOT](break-up-large-numbers.md#independent-bounded-acceptance--2026-10-02) owns gate results,110 uniquePASS/startup0 `[]`, dirty provenance and accounting. Four unchanged restriction tests are reused, not eight new tests or duplicated executions. Existing meaningful providers remain unchanged/alreadyGREEN; no fabricated RED or unnecessary rewrite.

Prior40 partial capability, accepted514/500/424/438/498 and broad unchecked requirements remain intact. Only these eight output-annotation boundaries promote; nullable/other-index/ignored-selector behavior, synthetic damage bounds, offhand/modifier placeholders, nonplayer cases, native formulas/activation/access and all-profile execution remain unproved. Earlier pending statements are historical; no complete stat-model, native or whole-page/goal acceptance follows.

## Tertiary annotation extension — 2026-10-02

[Exact420/442/480 acceptance SSOT](tertiary-stat-inputs.md#independent-bounded-acceptance--2026-10-02) records meaningful existing rating fields and inferred conversion after the omitted shared lookup/bonus arms were repaired. Independent450 accepts14 focused cases plus105 controls; no zero-placeholder-only credit or new state. Prior40 partial and exact-five/eight annotation scopes remain unchanged; supporting raw/bonus indices receive no additional row credit. Native conversion/acquisition/profile/activation and remaining model gaps stay explicit.

## B76 existing-model bounded acceptance — 2026-10-03

Main accepts independent642 for **exact430/432/458/460 output annotations only**, backed by four saved existing stat-restriction tests PASS. Concrete computed outputs are **7 / 2 / 8 / 3**, respectively. These are existing simulator-model values; privacy behavior and guessed coefficients do not establish native formulas or activation. No new provider/model, fabricated RED, duplicate execution, current-process proof, input-secret parity or all-profile credit follows.

Main-supplied data commit `ec7386f86` promotes these four rows/adds one capability: **167 pending / 174 bounded / 14 partial / 7 metadata, 362 ordered IDs / 81 capabilities**. Verifier647's retained terminal report records **31/31 PASS**, including unchanged 358 other rows, 80 prior capabilities and source hashes. Its named `/tmp/patch-12.0.5-batch76-accounting-validation.{md,json}` artifacts were unavailable during checkpoint reconciliation; no raw-artifact revalidation or rerun is claimed. Earlier accounting remains historical; prior annotation acceptances and partial-model limits remain intact. Full-suite GREEN is not claimed; broader audit remains open.

[Audit checkpoint](../wiki/investigations/patch-12-0-5-api-audit.md#batch76--existing-model-output-annotations-accepted) links this bounded acceptance. Evidence is supplied by main, not newly executed here.

## B83 exact486 annotation supplement — 2026-10-03

Scope: **`global api-PlayerScript GetSpellCritChance-486` only**. [Retained source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 485–486 read `PlayerScript GetSpellCritChance` / `+ SecretWhenUnitStatsRestricted`; the [register](../../data/patch-api/sources/12.0.5-register.json) record is `kind: delta`, `source_lines: [486]`. The delta adds an output annotation only: no signature, argument, formula or activation change.

Cached retail `Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua` lines 895–903 declare `SecretWhenUnitStatsRestricted = true` and one nonnil `result: number`. The declaration has **no `Arguments` and no `SecretArguments`**, so B82's original-school authentication guard has no counterpart here and is not copied. Cached declarations are contract context, not native execution evidence.

| Provider | Ordered fixture | Wrapping |
|---|---|---|
| [`get_spell_crit_chance`](../../src/lua_api/globals/real/combat_stats.rs) converts stack slot 1 as a fallible optional `i32` (nil/absent accepted; non-number, fractional or out-of-range values error before any output), discards the parsed value, and delegates to `get_crit_chance`: `player.stats.crit_pct() + 5.0`. | `GetSpellCritChance(2)` → `{7}` from seeded crit rating360; arity1, `result`. | One `push_stat_number` position: secret host number under explicit `unit_stats_restricted`, plain otherwise. |

Existing tests exercising this fixture, unchanged since the prior40 matrix:

- `stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle` — exact arity, value7, secrecy and access plain → restricted → plain.
- `stat_restriction_tainted_callers_receive_opaque_host_results` — arity, secret/inaccessible result, denied unwrap and arithmetic, retained `StatRestrictionProbe` caller taint.

The other two restriction tests are controls, not spell-crit evidence. No annotation gap is demonstrated for this seeded case: no producer change, new test or fabricated RED is warranted.

### Reused proof; no new execution

[B82 acceptance](spell-bonus-stat-security.md#independent-bounded-acceptance--2026-10-03) records main-accepted716 with these four restriction tests refreshed PASS at `d21d4a208`. `git diff d21d4a208 HEAD -- src tests Cargo.toml Cargo.lock` is empty at `684670add`; the sole dirty file is unowned `src/c_api/aura_duration.rs`, outside this provider chain. Reuse is bound to that committed record. **The underlying `/tmp/patch-12.0.5-*` artifacts are not readable on the host where this supplement was written**; no raw-artifact revalidation, rerun or current-process proof is claimed. B82's qualified check protocol carries over unchanged.

### Limits

- Value7 is the melee crit model plus simulator base5; no spell-specific or per-school crit, no native formula.
- The fixture passes a superfluous `2`; the no-argument call appears only in a `tests/character_stats.rs` smoke check (number, nonnegative), with no exact value, arity or restriction assertion. Wrong-type, nil and secret extra-argument handling are unverified against the no-argument declaration.
- Automatic restriction activation, older profiles, native/GUI parity and full-suite readiness remain unverified. `GetSpellCritChanceFromIntellect` stays plain and is not this row.

At supplement time row486 was `audit-pending`; the acceptance below supersedes that for this one boundary.

## B83 independent bounded acceptance — 2026-10-03

Main accepts an independent GPT-6.1-sol read-only audit of the supplement at `49c667ad4`: **ACCEPT WITH QUALIFICATIONS**, eight items PASS, for **exact486 output annotation only**. It confirmed the literal delta and register record, the cached no-argument declaration, the registered provider and single wrapped result, `crit_pct() = crit_rating / 180` giving computed `360/180 + 5 = 7`, the assertions of the two exercising tests, the empty `d21d4a208..HEAD` diff over `src tests Cargo.toml Cargo.lock`, and the unowned aura diff lying outside the provider chain. Its two wording qualifications are applied above.

Evidence class: fresh source inspection plus the **committed** B82 record, which states an aggregate "4 existing restriction controls" PASS at `d21d4a208`, not four individually named raw result lines. No test, build, check, formatter or startup ran for B83. Raw `/tmp/patch-12.0.5-*` artifacts were absent on the audit host, so historical PASS, timings and hashes are not independently re-established. The audit report (SHA256 `d82dd28128ae506dc3cbcd74e43a07a368f6150711e3eb72ee2894b869551517`) lives in a session scratchpad and is not durable; this section is the retained record.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): row486 `bounded-coverage` under new capability `spell-crit-restricted-output`; **87 capabilities/362 ordered IDs; 164 pending/180 bounded/11 partial/7 metadata**. Other 361 rows and prior 86 capabilities unchanged. Limits above, B82's qualified check protocol and all prior partial/annotation scopes stay as recorded. No neighboring row, native, older-profile or full-suite credit; audit remains in progress.

## B84 exact474/476 annotation supplement — 2026-10-03

Scope: **`global api-PlayerScript GetRangedCritChance-474` and `global api-PlayerScript GetRangedHaste-476` only**. [Retained source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 473–476 give each subject the single delta `+ SecretWhenUnitStatsRestricted`; both [register](../../data/patch-api/sources/12.0.5-register.json) records are `kind: delta` with `source_lines` `[474]` and `[476]`. Output annotations only.

Cached retail `PlayerScriptDocumentation.lua` lines 745–753 and 755–763 declare `SecretWhenUnitStatsRestricted = true` and one nonnil `result: number` each, with no `Arguments` and no `SecretArguments`. Contract context, not native execution evidence.

| Exact ID | Provider in [`combat_stats.rs`](../../src/lua_api/globals/real/combat_stats.rs) | Ordered fixture | Wrapping |
|---|---|---|---|
| 474 | `get_ranged_crit_chance` delegates to `get_crit_chance`: `player.stats.crit_pct() + 5.0`, `crit_pct = crit_rating / 180`. | `GetRangedCritChance()` → `{7}` from seeded crit rating360; arity1. | One `push_stat_number`. |
| 476 | `get_ranged_haste` delegates to `get_haste` outside `client-wowforever`: `player.stats.haste_pct()`, `haste_pct = haste_rating / 170`. | `GetRangedHaste()` → `{3}` from seeded haste rating510; arity1. | One `push_stat_number`. |

Both fixtures run in `stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle` (arity, value, secrecy, access plain → restricted → plain) and `stat_restriction_tainted_callers_receive_opaque_host_results` (arity, opacity, denied unwrap and arithmetic, retained caller taint). The other two restriction tests are controls. No annotation gap is demonstrated; no producer change, new test or fabricated RED.

Proof reuse is identical to [B83](#b83-independent-bounded-acceptance--2026-10-03): the committed B82 aggregate record of four restriction controls PASS at `d21d4a208`, with `git diff d21d4a208 HEAD -- src tests Cargo.toml Cargo.lock` empty at `559d47720`. No rerun; raw `/tmp/patch-12.0.5-*` artifacts unreadable on this host; B82's qualified check protocol carries over.

### Limits

- Both values are the melee getters reused: no ranged-specific crit or haste model, no native formula; base5 and divisors 180/170 are simulator choices. Shared getters are local annotation backing, not native equivalence with accepted rows428/438/450.
- Under `client-wowforever`, `get_ranged_haste` returns two **plain** values (haste, quiver) and bypasses `push_stat_number`. Not a 12.0.5 claim; older-profile behavior is unverified here.
- Outside `client-wowforever`, other calls appear only in a `tests/character_stats.rs` smoke check (number, nonnegative). `tests/forever_character_remaining_stats.rs` asserts the Forever two-plain-value `GetRangedHaste` shape; that profile cannot be combined with a retail epoch and adds no 12.0.5 credit. Activation, native/GUI parity and full suites remain unverified.

At supplement time rows474/476 were `audit-pending`; the acceptance below supersedes that for these two boundaries.

## B84 independent bounded acceptance — 2026-10-03

Main accepts an independent GPT-6.1-sol read-only audit of the supplement at `092b6efab`: **ACCEPT WITH QUALIFICATIONS for each row**, **exact474/476 output annotations only**. It confirmed both literal deltas and register records, both no-argument declarations, the registered providers and single wrapped results, computed `360/180 + 5 = 7` and `510/170 = 3`, the assertions of the two exercising tests (value within `1e-10`), the empty `d21d4a208..HEAD` diff over `src tests Cargo.toml Cargo.lock`, and the unowned aura diff lying outside both chains. From `Cargo.toml` and `src/client_profile.rs` it derived that default builds and `profile-retail,retail-12-0-5` compile the one-result haste branch and that Forever cannot combine with a retail epoch; that is source-derived, not compiled.

Its one failed sub-item was the supplement's repository-wide "only" claim about other `GetRangedHaste` callers, falsified by the Forever tests; corrected in the limits above. No behavioral claim changed.

Evidence class: fresh source inspection plus the committed B82 aggregate record, as for [B83](#b83-independent-bounded-acceptance--2026-10-03). No test, build, check, formatter or startup ran for B84; raw `/tmp/patch-12.0.5-*` artifacts absent. The audit report (SHA256 `0775c67b49eb6e3a491795fc65e046fa96afeac62b506d07eb0d973a28c11d7f`) lives in a session scratchpad and is not durable; this section is the retained record.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows474/476 `bounded-coverage` under new capability `ranged-crit-haste-restricted-outputs`; **88 capabilities/362 ordered IDs; 162 pending/182 bounded/11 partial/7 metadata**. Other 360 rows and prior 87 capabilities unchanged. No ranged-specific model, native, older-profile or full-suite credit; audit remains in progress.

## B85 exact502/508/512 annotation supplement — 2026-10-03

Scope: **`global api-Unit UnitAttackPower-502`, `global api-Unit UnitRangedAttackPower-508` and `global api-Unit UnitSpellHaste-512` only**. [Retained source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 501–502, 507–508 and 511–512 give each subject the single delta `+ SecretWhenUnitStatsRestricted`. Output annotations only.

Cached retail `UnitDocumentation.lua` function declarations (lines 654–670, 2855–2871, 3023–3037) carry `SecretWhenUnitStatsRestricted = true`, nonnil `unit: UnitToken` and `SecretArguments = "AllowedWhenUntainted"`. That argument policy is an existing declaration, not a delta in these rows and not proof of selector compliance. Returns: `attackPower, posBuff, negBuff` (three nonnil numbers) for both attack-power queries; one nonnil `result` for spell haste. Same-named `Type = "Event"` entries in that file are unrelated.

| Exact ID | Provider in [`unit_stats.rs`](../../src/lua_api/globals/unit_stats.rs) | Ordered fixture | Wrapping |
|---|---|---|---|
| 502 | `unit_attack_power`: `stats_for` resolves the unit; player `attack_power = strength + agility + level*10` truncated to integer, then two literal zeros. | `UnitAttackPower('player')` → `{600, 0, 0}` from strength300, agility200, level10; arity3. | Three `push_stat_number` positions. |
| 508 | `unit_ranged_attack_power` delegates to `unit_attack_power`. | `UnitRangedAttackPower('player')` → `{600, 0, 0}`; arity3. | Same three positions. |
| 512 | `unit_spell_haste`: `rating_to_percent(stats.haste_rating)` = `haste_rating / 180`. | `UnitSpellHaste('player')` → `{510/180}` from haste rating510; arity1. | One `push_stat_number`. |

All three fixtures run in `stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle` and `stat_restriction_tainted_callers_receive_opaque_host_results`. `stat_restriction_absent_unit_shapes_and_constant_outputs` additionally asserts three secret zeroes and arity3 for `UnitAttackPower('missing')`; it does not call the other two. Proof reuse is identical to [B83](#b83-independent-bounded-acceptance--2026-10-03): committed B82 aggregate record at `d21d4a208`; `git diff d21d4a208 HEAD -- tests Cargo.toml Cargo.lock` is empty and the only `src` change since is `608d52558` in `src/c_api/aura_duration.rs`, outside these chains. No rerun; raw artifacts unreadable on this host.

### Limits

- Attack power is a synthetic sum, not a native formula; `posBuff`/`negBuff` are literal zeros, wrapped but not modeled. Ranged attack power is the melee value reused.
- Spell haste divides by 180 while `GetHaste`/`GetMeleeHaste`/`GetRangedHaste` divide by 170: the simulator's two haste conversions disagree for the same rating. Neither is native-verified.
- Only `'player'` is fixture-covered; target/focus/party snapshots, `'pet'` aliasing to player stats, absent units for 508/512, secret or tainted unit selectors, activation, older profiles and native parity are unverified.

### Deferred from this batch: no credit

- **478 `GetShieldBlock`**: provider returns `stats.armor` (fixture1234) and reads stack slot 1 as a unit token although the cached declaration takes no arguments. Armor is not a shield-block model; wrapping alone does not make this meaningful behavior.
- **504 `UnitAttackSpeed`**: provider pushes literal `2.0, 2.0` regardless of unit or state; constant shim.

Rows502/508/512 remain `audit-pending` with no capability until independent validation and main accounting.
