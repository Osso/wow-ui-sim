# Scenario unit criteria progress

`C_ScenarioInfo.GetUnitCriteriaProgressValues(unit)` reads modeled M+ Enemy Forces credit at the C API boundary (`src/c_api/c_scenario_info.rs`). Official cached retail `ScenarioInfoDocumentation.lua` lines 121–138 specifies a required UnitToken, `MayReturnNothing`, `SecretWhenUnitIdentityRestricted`, and secret arguments `AllowedWhenUntainted`. [Patch source](../../data/patch-api/sources/12.0.5-api-changes.txt) line 220 describes integer credit, floating percentage, and display percentage.

## What it must do

- [x] Publish the query in the default cumulative retail environment without replacing existing scenario methods. Registration uses `retail-12-0-5`; disabled-epoch exclusion is not exercised by this proof.
- [x] Return exactly three supplied values: integer credit as a Lua number, percentage as a Lua number, and the unchanged display string; do not derive a denominator or round/localize text.
- [x] Read independent unit-token rows, including explicit zero progress and subsequent row updates.
- [x] Inferred: absent scenario, absent unit row, and empty-default input return zero results, not nil or fabricated zeros.
- [x] Reject missing/nil/non-string unit arguments, including non-string secret payloads. Native exact diagnostics and extra-argument policy are unprobed.
- [x] Unwrap secret token input only through rilua's untainted-caller guard; tainted secret input must fail without disclosure.
- [x] Inferred input-boundary policy: each row explicitly classifies identity restriction; restricted rows wrap all three values with native rilua secrets. No party/raid-token heuristic or global string marking.
- [x] Identical public values remain public after restricted output; explicit classification updates affect subsequent reads.
- [ ] Plain-token queries from tainted addon callers return three opaque secret values for restricted rows, preserving caller taint and the secret-input/unwrap guards. Trusted typed Rust producers wrap modeled output only; they do not declassify Lua inputs.

## How it works

- [C API and Lua surface](../lua-api.md)
- [Retail secret-value contract](retail-secret-values.md)

## Implementation inventory

- `src/c_api/c_scenario_info.rs`: per-unit input type, guarded token reads, and typed host-produced secret results.
- Pinned rilua typed host number/string producers: output minting preserves caller taint; Lua-value wrapping and unwrapping remain guarded.
- `src/c_api/mod.rs`: public model module.
- `src/c_api/registration.rs`: patch-gated namespace registration.
- `src/lua_api/state_types/mythic_plus_scenario.rs`: empty-default unit-token map in ScenarioState.

## Tests asserting this spec

- `tests/scenario_unit_criteria.rs` in the existing grouped `integration` target, default cumulative retail.
- `tests/c_scenario_info_probes.rs`: existing scenario namespace controls with empty unit-credit fixtures.

Bounded development proof: implementation `f372687b6` plus fixture followup `72220958b`; unchanged Rust at docs descendant `e3537af64`. Initial absent-API RED ran two cases; the two later fixture cases did not independently execute RED. GREEN passed all four new cases and 11 existing controls. Ledger: `/tmp/patch-12.0.5-scenario-ledger.md`. This is simulator proof, not native parity or a broad acceptance gate. It predates the host-result correction: the updated tainted plain-token regression is RED against the old guarded output path (`/tmp/patch-12.0.5-scenario-host-secret-red.log`). Current host-result GREEN is pending.

## Known gaps (current cycle)

- [ ] Native probes: unknown/despawned unit, no scenario, scenario without unit credit, zero-credit creature, active M+ credited creature, token reassignment/aliases, and percentage/display localization.
- [ ] Native probes: public/restricted identity transitions, tainted calls with plain tokens, untainted/tainted secret tokens, and exact argument errors.
- [ ] No stronger existing identity model: both current unit-name helpers infer restriction from party/raid token syntax. Row classification is supplied input, not native-verified identity policy; replace when a modeled restriction resolver/native evidence exists.

## Out of scope

- Automatic M+ credit calculation, denominator/localization derivation, unit alias resolution, global identity-secrecy redesign, or native-client equivalence claims.
- Other client profiles and a generalized scenario API migration.
