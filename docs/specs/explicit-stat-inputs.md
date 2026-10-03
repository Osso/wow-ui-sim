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

- [x] Each query returns exactly its declared number of results, read live from its own input. Unconfigured inputs return zero; the zero default is simulator policy.
- [x] Inputs are independent: changing one never moves another. Expertise and expertise percent are separate triples with no conversion between them.
- [x] `GetShieldBlock` no longer returns armor and no longer reads a unit argument; any argument is ignored.
- [x] `GetPetSpellBonusDamage` is independent of player stats and reads only its optional input. That it neither requires nor creates a pet follows from the read-only producer, not from a direct assertion.
- [x] Inputs are per environment.
- [x] Under explicit `unit_stats_restricted`, every result is a secret host number with value and arity preserved, plain again when the flag clears.
- [x] Restricted results are opaque to tainted callers: unwrap and arithmetic fail, caller taint is unchanged.

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

## Development proof and independent bounded acceptance — 2026-10-03

| Stage | Revision | Result |
|---|---|---|
| Inputs RED | `be7c0cc2c` | `character_stats::` 33 PASS / 11 FAIL: all nine new cases plus the two restriction-matrix tests on the changed `GetShieldBlock` fixture. |
| Producers GREEN | `62d0ce70f` | `character_stats::`, `unit_stats::`, `pet_stats::` 79/79, cargo exit0, no warnings; `cargo fmt --check` exit0; startup `lua-errors` `[]`. |

Main accepts an independent GPT-6.1-sol review of both commits: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `2a7f0f1adae2b2b1aa40196fb3cc820f28d7f1edceda1b7414bb50e15d95db34`, scratchpad-only). It confirmed every producer reads its own field through `push_stat_number`, no producer is still constant, the removed zero helper had only these three callers, fields and producers are ungated alike, and each new test fails against the old producers except the zero-default case, which the old shield proxy alone fails. It reran `character_stats::` itself: **44 passed, exit0**. Retail `PaperDollFrame.lua` calls `GetShieldBlock()` with no arguments and feeds the value to armor-effectiveness helpers that handle zero; no zero-specific error was found by inspection.

Checked requirements are bounded simulator proof. Independence and environment isolation are sampled, not exhaustive over all ten inputs. Older-profile compilation or execution, paper-doll panel interaction and `cargo check` were not run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows422/434/436/440/452/464/470/472/478/488 `bounded-coverage` under new capability `explicit-stat-inputs`; **92 capabilities/362 IDs; 109 pending /209 bounded /11 partial /33 metadata**.

## B92 `UnitAttackSpeed` swing-time inputs — row 504

Source ID `global api-Unit UnitAttackSpeed-504`, delta `+ SecretWhenUnitStatsRestricted`. Cached `UnitDocumentation.lua` declares nonnil `unit: UnitToken`, returns nonnil `attackSpeed: number` and nilable `offhandAttackSpeed: number`. The provider returned the literal pair `2.0, 2.0` for every unit.

- [ ] Player results read `player.stats.attack_speed` and `player.stats.offhand_attack_speed` live. The base snapshot seeds `2.0` and `Some(2.0)`, keeping the previous player output; both are simulator defaults.
- [ ] A `None` off-hand returns a plain, non-secret nil as the second of exactly two results.
- [ ] Target, focus and party snapshots get a synthetic `2.0` main-hand time and no off-hand; unknown units return `0` and nil. Neither reads player inputs.
- [ ] Under explicit `unit_stats_restricted` each numeric result is a secret host number with its value preserved, plain again when the flag clears, and opaque to tainted callers without changing their taint.

Tests: `tests/unit_attack_speed.rs`, four cases. Unit-token handling is unchanged: a missing or non-string argument still falls back to `"player"`, and the cached `AllowedWhenUntainted` selector policy is not modeled.

## Known gaps (current cycle)

- [ ] B92 development proof: inputs `cb1fc14ab` RED 0 PASS / 4 FAIL; producer GREEN 73/73 across `unit_attack_speed::`, `character_stats::`, `unit_stats::`. The first GREEN attempt failed one case on a test-side `UnitExists('party1')` assertion, removed because party stat lookup does not depend on it. Independent verification and accounting pending.
- [ ] Default shield block is now zero instead of armor, which changes the block-mitigation number the paper doll shows until a host configures the input.

## Out of scope

- Native formulas, rating conversions, gear or talent derivation: none exists; these are host-configured snapshot values. `CharacterStats::compute` builds a fresh snapshot, so configured inputs do not survive a recompute.
- Automatic restriction activation, secret-argument policy, native parity.
- Older profiles share these producers and inputs but no older-profile execution or parity is claimed.
