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

- [ ] Parent-owned GREEN and final verification pending. Original RED at `e0a46d691`: ten missing-global invocation failures. Concrete fixture RED at `eac08bda3`: 0/13 pass, all fail on missing globals (`/tmp/patch-12.0.5-batch6-stats-fixture-red.log`); parent default integration compile succeeded after public input export correction. Bodies follow that RED.
- [ ] Future native probes: mastery across classes/specs/levels; conversion units and absent conversion; penetration/sturdiness scales and equipment effects; pet presence/inheritance; regen by power and casting state; effective AP absence and all five positions; weapon contributions for aliases and missing entities; secrecy of each position and secret selectors under tainted callers.

## Out of scope

Native numeric parity, automatic equipment/spell-derived input producers, automatic restriction activation, other-profile expansion, vendor changes, and generic secret-value unwrapping.
