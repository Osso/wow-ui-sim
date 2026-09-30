# Scenario unit criteria progress

`C_ScenarioInfo.GetUnitCriteriaProgressValues(unit)` reads modeled M+ Enemy Forces credit at the C API boundary (`src/c_api/c_scenario_info.rs`). Official cached retail `ScenarioInfoDocumentation.lua` lines 121–138 specifies a required UnitToken, `MayReturnNothing`, `SecretWhenUnitIdentityRestricted`, and secret arguments `AllowedWhenUntainted`. [Patch source](../../data/patch-api/sources/12.0.5-api-changes.txt) line 220 describes integer credit, floating percentage, and display percentage.

## What it must do

- [ ] Register under the cumulative `retail-12-0-5` gate without replacing other scenario methods.
- [ ] Return exactly three supplied values: integer credit as a Lua number, percentage as a Lua number, and the unchanged display string; do not derive a denominator or round/localize text.
- [ ] Read independent unit-token rows, including explicit zero progress and subsequent row updates.
- [ ] Inferred: absent scenario, absent unit row, and empty-default input return zero results, not nil or fabricated zeros.
- [ ] Reject missing/nil/non-string unit arguments, including non-string secret payloads. Native exact diagnostics and extra-argument policy are unprobed.
- [ ] Unwrap secret token input only through rilua's untainted-caller guard; tainted secret input must fail without disclosure.
- [ ] Inferred input-boundary policy: each row explicitly classifies identity restriction; restricted rows wrap all three values with native rilua secrets. No party/raid-token heuristic or global string marking.
- [ ] Identical public values remain public after restricted output; explicit classification updates affect subsequent reads.
- [ ] Current VM limitation: tainted callers querying restricted rows fail at guarded `wrap_secret`; no caller declassification bypass. Plain public rows remain callable while tainted.

## How it works

- [C API and Lua surface](../lua-api.md)
- [Retail secret-value contract](retail-secret-values.md)

## Implementation inventory

- `src/c_api/c_scenario_info.rs`: per-unit input type and guarded three-result query.
- `src/c_api/mod.rs`: public model module.
- `src/c_api/registration.rs`: patch-gated namespace registration.
- `src/lua_api/state_types/mythic_plus_scenario.rs`: empty-default unit-token map in ScenarioState.

## Tests asserting this spec

- `tests/scenario_unit_criteria.rs` in the existing grouped `integration` target, default cumulative retail.

## Known gaps (current cycle)

- [ ] Native probes: unknown/despawned unit, no scenario, scenario without unit credit, zero-credit creature, active M+ credited creature, token reassignment/aliases, and percentage/display localization.
- [ ] Native probes: public/restricted identity transitions, tainted calls with plain tokens, untainted/tainted secret tokens, and exact argument errors.
- [ ] No stronger existing identity model: both current unit-name helpers infer restriction from party/raid token syntax. Row classification is supplied input, not native-verified identity policy; replace when a modeled restriction resolver/native evidence exists.

## Out of scope

- Automatic M+ credit calculation, denominator/localization derivation, unit alias resolution, global identity-secrecy redesign, or native-client equivalence claims.
- Other client profiles and a generalized scenario API migration.
