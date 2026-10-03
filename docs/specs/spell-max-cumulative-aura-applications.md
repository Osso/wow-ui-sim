# Spell maximum cumulative aura applications — B95

`C_Spell.GetSpellMaxCumulativeAuraApplications(spellID)` reports the maximum stack count a spell's aura can reach. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 314–316 carry two deltas: `# arg1.Type number -> SpellIdentifier` (source ID `global api-C_Spell-GetSpellMaxCumulativeAuraApplications-315`) and `# SecretWhenSpellAuraRestricted -> SecretWhenUnitAuraRestricted` (`-316`). Before this work the simulator did not register the function.

Cached retail `SpellDocumentation.lua` lines 461–475 declare nonnil `spellID: SpellIdentifier`, one nonnil `cumulativeAura: number`, `SecretWhenUnitAuraRestricted` and `SecretArguments = "AllowedWhenTainted"`. Contract context, not native execution evidence.

## What it must do

### Identifier and value — row 315

- [ ] Resolve the argument through the shared alias-first spell identifier resolver: a public non-negative integral number, or a string that matches a seeded case-normalized alias. A seeded numeric alias takes precedence over numeric identity.
- [ ] Return exactly one number: the explicit `spell_max_cumulative_aura_applications` entry for the resolved spell, read live, independent of any currently applied aura stacks.
- [ ] Return one zero for an undeclared spell or an unseeded string. Inferred miss policy; the map is empty by default and per environment.
- [ ] Reject missing, nil, non-string/non-number, negative, fractional and non-finite identifiers. Inferred representation policy.

### Output restriction — row 316

- [ ] Under explicit `unit_auras_restricted` the result, including a miss, is a secret host number with the same payload; plain again when the flag clears. `cooldowns_restricted` and `unit_stats_restricted` have no effect. Results returned while restricted stay secret.
- [ ] Tainted callers get public results for public identifiers without their taint changing, and cannot unwrap or do arithmetic on a restricted result.

## How it works

- [Cooldown aura spell identifiers](cooldown-aura-spell-identifiers.md) — the same public identifier boundary.
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_spell_counts.rs` — producer.
- `src/c_api/c_spell.rs` — registration under `retail-12-0-5`.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — maximum map and `unit_auras_restricted`.

## Tests asserting this spec

- `tests/spell_max_cumulative_aura_applications.rs` — eight cases: identifiers, alias precedence, independence from applied stacks, misses and live map, invalid representations, restriction flag, tainted callers, secret identifier rejection.

## Known gaps (current cycle)

- [ ] Inputs only: the function is not registered and the state inputs do not exist; no compiled RED or GREEN recorded yet.

## Out of scope

- Native `AllowedWhenTainted`: secret identifiers are rejected for every caller, as for the other public-identifier queries; no secret is unwrapped.
- A `C_Secrets` predicate for unit-aura restriction: none is declared in the cached documentation, so none is added. Nothing sets `unit_auras_restricted` automatically, and other aura queries do not read it yet.
- Native maxima, a spell catalog, link parsing and unknown-spell behavior.
