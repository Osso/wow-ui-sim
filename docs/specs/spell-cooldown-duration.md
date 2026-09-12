# Spell cooldown duration

`C_Spell.GetSpellCooldownDuration(spellIdentifier)` exposes modeled spell cooldown timing through a duration object. The pinned 12.0.0 register declares a string identifier and optional duration; numeric IDs, seeded aliases, and interval policies below are bounded simulator behavior, not native conformance.

## What it must do

- [x] On retail 12.0.0+, resolve numeric identifiers and seeded case-insensitive aliases through the same identifier model as `GetSpellIDForSpellIdentifier`.
- [x] Snapshot the existing spell/GCD selector's interval into a duration with rate 1 and the runtime clock, consistently with numeric `GetSpellCooldown` results.
- [x] Return zero timing for an inactive numeric identifier and nil for an unresolved string alias; these are explicit simulator policies.
- [x] Keep an earlier object's configured interval unchanged after cooldown or alias updates; new queries reflect current state.
- [x] Preserve the earlier-profile nil result rather than installing the new producer there.

## How it works

- [Duration core](duration-core.md)
- [Action cooldown duration](action-cooldown-duration.md)
- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/c_api/c_spell.rs`: profile-gated producer and shared spell-identifier lookup.
- `src/lua_api/globals/action_bar_api.rs`: existing spell/GCD interval selector.
- `src/lua_api/globals/lua_duration_object.rs`: shared duration snapshot construction.
- `src/c_api/c_action_bar.rs`: existing action producer reuses the same construction.

## Tests asserting this spec

`tests/cooldown_probes.rs::get_spell_cooldown_duration_*`: active timing/clock bounds, aliases/snapshots, later-ending GCD, inactive/unresolved identifiers and earlier-profile control. Initial retail RED: four failures because calls returned nil. An initial absence assertion was rejected by the observed Mists runtime; earlier profiles expose the generic nil-returning namespace method. `/tmp/verify-spell-cooldown-duration-ledger.json` passes 32/32 on retail 12.0.0, 12.0.5, and 12.0.7, and 29/29 on Mists. Format, default check/build, startup `[]`, validators, readability, and all 14,813 evidence references pass. Historical warnings remain 6/6/1/6; default checks have none. This proves ordinary simulator behavior only.

Existing `tests/c_spell_flyout_probes.rs` covers shared identifier lookup. Existing action duration cases cover the moved construction path.

## Known gaps (current cycle)

- [ ] Native identifier, unknown-spell, return arity, GCD/snapshot/identity/lifetime and error semantics remain unverified.
- [ ] Real Blizzard-consumer acceptance is not established by seeded API fixtures.

## Out of scope

- Later `ignoreGCD` option, real spell-name discovery beyond seeded aliases, charges/loss-of-control producers, non-default rates, secrets/security, and cooldown-engine redesign.
