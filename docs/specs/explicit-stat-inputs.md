# Explicit stat inputs for former constant producers — B88

Ten stat queries returned a wrapped constant or an unrelated proxy. Each now reads its own explicit host input. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) gives every one the single delta `+ SecretWhenUnitStatsRestricted`; wrapping a constant earned no credit under the [output-restriction contract](unit-stat-output-restriction.md), so the rows stayed pending.

| Source ID suffix | Query | Input | Returns |
|---|---|---|---|
| 422 | `GetBlockChance` | `player.stats.block_chance` | 1 |
| 478 | `GetShieldBlock` | `player.stats.shield_block` | 1 |
| 440 | `GetHitModifier` | `player.stats.hit_modifier` | 1 |
| 488 | `GetSpellHitModifier` | `player.stats.spell_hit_modifier` | 1 |
| 434 | `GetExpertise` | `player.stats.expertise` (main-hand, off-hand, ranged) | 3 |
| 436 | `GetExpertisePercent` | `player.stats.expertise_percent` (same order) | 3 |
| 452 | `GetModResilienceDamageReduction` | `player.stats.mod_resilience_damage_reduction` | 1 |
| 470 | `GetPvpPowerDamage` | `player.stats.pvp_power_damage` | 1 |
| 472 | `GetPvpPowerHealing` | `player.stats.pvp_power_healing` | 1 |
| 464 | `GetPetSpellBonusDamage` | `pet.spell_bonus_damage` (`None` → 0) | 1 |

Cached retail `PlayerScriptDocumentation.lua` declares each as a no-argument function with `SecretWhenUnitStatsRestricted = true` and nonnil number returns in the order above. Contract context, not native execution evidence.

## What it must do

- [ ] Each query returns exactly its declared number of results, read live from its own input. Unconfigured inputs return zero; the zero default is simulator policy.
- [ ] Inputs are independent: changing one never moves another. Expertise and expertise percent are separate triples with no conversion between them.
- [ ] `GetShieldBlock` no longer returns armor and no longer reads a unit argument; any argument is ignored.
- [ ] `GetPetSpellBonusDamage` is independent of player stats and does not require or create a pet.
- [ ] Inputs are per environment.
- [ ] Under explicit `unit_stats_restricted`, every result is a secret host number with value and arity preserved, plain again when the flag clears.
- [ ] Restricted results are opaque to tainted callers: unwrap and arithmetic fail, caller taint is unchanged.

## How it works

- [Lua API](../lua-api.md)
- [Unit-stat output restriction](unit-stat-output-restriction.md) — shared wrapper and restriction flag.

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — nine `CharacterStats` inputs.
- `src/lua_api/sim_substates/mod.rs` — `PetState.spell_bonus_damage`.
- `src/lua_api/globals/real/combat_stats.rs`, `src/lua_api/globals/unit_stats.rs`, `src/lua_api/globals/real/pet_stats.rs` — producers.

## Tests asserting this spec

- `tests/character_stats/explicit_inputs.rs` — nine cases: zero defaults, per-input values, live independent updates, shield block versus armor and unit argument, expertise triples, optional pet input, environment isolation, restriction toggle, tainted opacity.
- `tests/character_stats/stat_restriction_fixtures.lua` — `GetShieldBlock` fixture changes from the armor proxy `1234` to the unconfigured `0`.

## Known gaps (current cycle)

- [ ] Inputs only: producers still return constants or armor; no compiled RED or GREEN recorded yet.

## Out of scope

- Native formulas, rating conversions, gear or talent derivation: none exists; these are host-configured snapshot values. `CharacterStats::compute` builds a fresh snapshot, so configured inputs do not survive a recompute.
- Automatic restriction activation, secret-argument policy, native parity.
- `UnitAttackSpeed` (row 504): still a constant pair; it needs a per-unit input with a nullable off-hand and is handled separately.
- Older profiles share these producers and inputs but no older-profile execution or parity is claimed.
