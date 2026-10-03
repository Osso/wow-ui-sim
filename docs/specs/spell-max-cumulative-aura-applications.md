# Spell maximum cumulative aura applications — B95

`C_Spell.GetSpellMaxCumulativeAuraApplications(spellID)` reports the maximum stack count a spell's aura can reach. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 314–316 carry two deltas: `# arg1.Type number -> SpellIdentifier` (source ID `global api-C_Spell-GetSpellMaxCumulativeAuraApplications-315`) and `# SecretWhenSpellAuraRestricted -> SecretWhenUnitAuraRestricted` (`-316`). Before this work the simulator did not register the function.

Cached retail `SpellDocumentation.lua` lines 461–475 declare nonnil `spellID: SpellIdentifier`, one nonnil `cumulativeAura: number`, `SecretWhenUnitAuraRestricted` and `SecretArguments = "AllowedWhenTainted"`. Contract context, not native execution evidence.

## What it must do

### Identifier and value — row 315

- [x] Resolve the argument through the shared alias-first spell identifier resolver: a public non-negative integral number, or a string that matches a seeded case-normalized alias. A seeded numeric alias takes precedence over numeric identity.
- [x] Return exactly one number: the explicit `spell_max_cumulative_aura_applications` entry for the resolved spell, read live, independent of any currently applied aura stacks.
- [x] Return one zero for an undeclared spell or an unseeded string. Inferred miss policy; the map is empty by default and per environment.
- [x] Reject missing, nil, non-string/non-number, negative, fractional and non-finite identifiers. Inferred representation policy.

### Output restriction — row 316

- [x] Under explicit `unit_auras_restricted` the result, including a miss, is a secret host number with the same payload; plain again when the flag clears. `cooldowns_restricted` and `unit_stats_restricted` have no effect. Results returned while restricted stay secret.
- [x] Tainted callers get public results for public identifiers without their taint changing, and cannot unwrap or do arithmetic on a restricted result.

## How it works

- [Cooldown aura spell identifiers](cooldown-aura-spell-identifiers.md) — the same public identifier boundary.
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_spell_counts.rs` — producer.
- `src/c_api/c_spell.rs` — registration under `retail-12-0-5`.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — maximum map and `unit_auras_restricted`.

## Tests asserting this spec

- `tests/spell_max_cumulative_aura_applications.rs` — eight cases: identifiers, alias precedence, independence from applied stacks, misses and live map, invalid representations, restriction flag, tainted callers, secret identifier rejection.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 0 PASS / 8 FAIL. GREEN: 8/8; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `c82a251027147d0fa6068c968bb8769cc466fb4634ffd683a37d2c7fe111561b`, scratchpad-only), own rerun 8/8 exit0. Secret identifiers are rejected for every caller; the cached declaration says `AllowedWhenTainted`. Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows 315, 316 under new capability `spell-max-cumulative-aura-applications`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] Older profiles and a full Blizzard UI load were not executed; infinities are not individually tested.

## Out of scope

- Native `AllowedWhenTainted`: secret identifiers are rejected for every caller, as for the other public-identifier queries; no secret is unwrapped.
- A `C_Secrets` predicate for unit-aura restriction: none is declared in the cached documentation, so none is added. Nothing sets `unit_auras_restricted` automatically, and other aura queries do not read it yet.
- Native maxima, a spell catalog, link parsing and unknown-spell behavior.
