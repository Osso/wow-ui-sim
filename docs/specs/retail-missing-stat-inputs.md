# Retail stat inputs

Ten previously absent retail globals from the [12.0.5 source](../../data/patch-api/sources/12.0.5-api-changes.txt) expose explicit simulator inputs, with [unit-stat output restriction](unit-stat-output-restriction.md). Generated `PlayerScriptDocumentation.lua` and `UnitDocumentation.lua` in the profile cache establish return names, order, and secrecy, not native numeric formulas.

## What it must do

- [ ] `GetMastery()` returns the existing rating-derived mastery component: 520/1040 rating gives 4/8. **Guess:** this is not the total mastery effect or a native class-specific baseline.
- [ ] Conversion getters return independent configured percentages; `GetSpellPenetration` returns a configured amount; `GetSturdiness` returns an independent configured durability-loss reduction percentage. **Guesses:** percentage interpretation and zero defaults, not avoidance/armor aliases. Reuse existing Forever scalar backing without changing its handlers.
- [ ] `GetPetMeleeHaste` reads an optional independent pet percentage. **Guess:** absent input yields one zero; setting input does not create a unit or inherit player haste.
- [ ] Regen queries return distinct `{base, casting}` per-second inputs selected by active/requested power type. **Guess:** missing pools yield two zeros; no intellect/mana alias or invented non-mana regeneration formula.
- [ ] Optional effective AP returns main-hand, off-hand, ranged, base melee, base ranged in that order. **Guess:** absent input returns no values. Weapon AP returns three independent contributions keyed by existing unit GUID; same-GUID aliases share values. **Guess:** absent entity or unconfigured row yields three zeros; configured rows never create entities.
- [ ] Every numeric result uses existing stat secrecy publication, preserving values, all arities, restriction toggles and tainted caller context. Secret power/unit selectors are accepted only when untainted through VM-authenticated, API-local selector reads; no generic declassification. **Guess:** power type requires an exact i32; unit token requires UTF-8 text.
- [ ] Registrations and newly added state are retail-only; existing Forever functions retain behavior.

## How it works

- [Lua API](../lua-api.md)
- [Patch audit](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — explicit scalar, regen, effective AP and weapon AP input types.
- `src/lua_api/sim_substates/mod.rs` — independent optional pet haste.
- `src/lua_api/{state.rs,state/sim_state.rs}` — GUID-indexed weapon contributions and empty initial map.
- `src/lua_api/globals/{real/combat_stats.rs,real/combat_stats/retail_inputs.rs,real/pet_stats.rs,unit_stats.rs}` — bounded stat producers and registrations.

## Tests asserting this spec

- `tests/character_stats/missing_apis.rs` — concrete distinct values, updates, input selection, missing entities, return counts, secret input/output and tainted caller behavior.

## Known gaps (current cycle)

- [ ] Observed batch7 explicit-input fixtures PASS 13/13; broader independent final verification pending. B75 accepts only the three output annotations below, not this full fixture/API scope. Original RED at `e0a46d691`: ten missing-global invocation failures. Concrete fixture RED at `eac08bda3`: 0/13 pass, all fail on missing globals (`/tmp/patch-12.0.5-batch6-stats-fixture-red.log`); parent default integration compile succeeded after public input export correction. Bodies follow that RED.
- [ ] Future native probes: mastery across classes/specs/levels; conversion units and absent conversion; penetration/sturdiness scales and equipment effects; pet presence/inheritance; regen by power and casting state; effective AP absence and all five positions; weapon contributions for aliases and missing entities; secrecy of each position and secret selectors under tainted callers.

## Out of scope

Native numeric parity, automatic equipment/spell-derived input producers, automatic restriction activation, other-profile expansion, vendor changes, and generic secret-value unwrapping.

## B75 existing-model bounded acceptance — 2026-10-03

Main accepts independent632 for **EXACT466/468/496 `+ SecretWhenUnitStatsRestricted` output annotations only**: `GetPowerRegen`, `GetPowerRegenForPowerType`, `PlayerEffectiveAttackPower`. Supplied evidence: `/tmp/patch-12.0.5-b75-existing-model-proof.md` and `.json`. Existing explicit inputs supply two active/selected-pool outputs and five distinct optional AP outputs; accepted scope is flag-controlled publication, preserved values/arity and tainted-caller opacity, not native input/default/formula semantics.

**Six saved PASS**, compiled `71973af4b71a8768d2c447472b40196ac9f645c4`, under `character_stats::missing_apis::`:

| Exact test suffix | Bounded observed behavior |
|---|---|
| `active_power_regen_selects_pool_and_tracks_updates` | Active pool3 (19,13), pool0 (11,7), casting update (11,5), missing99 (0,0). |
| `requested_power_regen_selects_input_not_active_pool` | Requested0/3 independent of active3; updated3 (29,17); missing99 (0,0); omitted selector errors. |
| `effective_attack_power_preserves_five_distinct_fields_and_unavailable_shape` | (101,202,303,404,505), offhand-only212 update; absent input gives zero results under either flag. |
| `missing_stats_toggle_preserves_all_values_and_arities` | Nine scoped slots across false→true→false: value/secrecy/arity preserved. |
| `missing_stats_tainted_callers_receive_opaque_host_outputs` | Nine restricted slots secret/inaccessible; unwrap/arithmetic fail; caller taint preserved; plain typed selector works, tainted secret selector rejected. |
| `secret_selection_inputs_are_allowed_when_untainted` | Secure secret selector0 returns public (11,7), exact arity. |

Saved partition0079: exit0,100 PASS/0 FAIL/0 ignored/0 unobserved,60.44s; only these six count here. Historical build exit0,295.4167359889252s, diagnostics `[]`. Reported named-body equivalence through `9441f7c7c665142d022540329b5b1f9addcbbf6e` covers exactly `retail_inputs.rs`, `combat_stats.rs`, `c_secrets.rs`, `character_world.rs`, and `tests/character_stats/missing_apis.rs`; named working paths matched that HEAD. B74 changed `register.rs` registration and `real/mod.rs`; those files are **not equivalent**. No fresh current-process run, whole-tree/VM/dependency equivalence, or current-HEAD clearance claimed. Broad13 plus adjacent4 are not new PASS credit.

Main-supplied accounting `88362f9a6` promotes **exactly these three rows**, adds one capability: **172 audit-pending / 169 bounded-coverage / 14 partial-development-green / 7 metadata-only = 362 ordered IDs; 79 capabilities**. The report's175/166/14/7 ledger and proposed172/169 were pre-accounting snapshots. Independent accounting640 PASS39/39 verifies this exact checkpoint ([proof](../../../../../../../../tmp/patch-12.0.5-batch75-accounting-validation.md)); no extra row/API credit.

Native restriction activation, zero/absence defaults, formulas, nominal numeric type/domain parity and all-profile behavior remain unverified; guesses above remain guesses. Full-suite goal remains OPEN/red: report's saved run12021 PASS/60 FAIL/18 ignored/4 unobserved, two custom harness exit1 and three pending doctests is historical, not new suite acceptance; later summary remains separately recorded in the audit. B74's later [bounded acceptance](ambiguate-context.md#b74-independent-bounded-acceptance--2026-10-03) supersedes its pending gates separately; no B74 scope or source-row credit belongs to B75.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

`character_stats::missing_apis::` PASS 13/13 (`/tmp/patch-12.0.5-batch7-integration-0.log`): explicit scalar/pet/regen/AP inputs and bounded selector/secrecy fixtures. No automatic producers or native formulas.
