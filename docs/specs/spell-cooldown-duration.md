# Spell cooldown duration

`C_Spell.GetSpellCooldownDuration(spellIdentifier [, ignoreGCD])` exposes modeled spell cooldown timing through a duration object. Retail 12.0.0+ publishes the method; Forever 1.60.1.70707 also publishes it in Game, and R17 measured the same decisions on Retail 12.1.5. Numeric IDs, seeded aliases, and interval policies below are bounded simulator behavior, not native timing conformance.

## What it must do

- [x] On retail 12.0.0+, resolve numeric identifiers and seeded case-insensitive aliases through the same identifier model as `GetSpellIDForSpellIdentifier`.
- [x] Snapshot the existing spell/GCD selector's interval into a duration with rate 1 and the runtime clock, consistently with numeric `GetSpellCooldown` results.
- [x] Return zero timing for an inactive numeric identifier and nil for an unresolved string alias; these are explicit simulator policies.
- [x] Keep an earlier object's configured interval unchanged after cooldown or alias updates; new queries reflect current state.
- [x] Preserve the nil result on profiles earlier than Retail 12.0.0 rather than installing the new producer there; Forever is not an earlier profile for this API.
- [ ] On R17 Forever/Retail 12.1.5, reject missing and non-number/non-string required identifiers, return zero Lua values for valid unknown identifiers, and return a zero LuaDurationObject for a known supported spell with no active cooldown. `ignoreGCD` accepts nil/false/true and its true path may return the zero object. The general cooldown/GCD selection and error-message text remain bounded simulator behavior unless separately proven.

## 12.0.5 ignoreGCD contract (GREEN pending)

Retained `data/patch-api/sources/12.0.5-api-changes.txt:306–307` adds arg2 `ignoreGCD`. Selection semantics below are explicit simulator inferences, not native-verified behavior.

- [ ] On retail 12.0.5+, true excludes GCD and snapshots the active individual spell interval; expired/missing individual cooldowns produce a zero-span object.
- [ ] Omitted/false retains the existing later-ending spell/GCD selector, including GCD for unknown numeric IDs; earlier profiles ignore the new argument.
- [ ] Aliases, unresolved-string/unsupported-identifier nil results, rate one, runtime clock, and snapshot independence remain unchanged.

`tests/cooldown_probes/ignore_gcd.rs` asserts these ordinary-value policies across all three producers. Actual RED at `9a50d8a5c`, `/tmp/patch-12.0.5-batch4-ignore-gcd-red.log`: 0/6 pass. Implementation committed; parent owns batched GREEN and older-profile regression proof. No secrecy or native conformance claim.

## How it works

- [Duration core](duration-core.md)
- [Action cooldown duration](action-cooldown-duration.md)
- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/c_api/c_spell.rs`: profile-gated producer and shared spell-identifier lookup.
- `src/lua_api/globals/action_bar_api.rs`: existing spell/GCD interval selector.
- `src/c_api/cooldown_duration.rs`: shared duration-only selection and 12.0.5 argument gate.
- `src/lua_api/globals/lua_duration_object.rs`: shared duration snapshot construction.
- `src/c_api/c_action_bar.rs`: existing action producer reuses the same construction.

## Tests asserting this spec

`tests/cooldown_probes.rs::get_spell_cooldown_duration_*` covers the modeled 12.0.0 behavior and earlier-profile control. `tests/cooldown_probes/forever_duration.rs::forever_spell_cooldown_duration_distinguishes_missing_inactive_and_active` covers the R17 Forever surface, unknown result arity, invalid argument types, active supplied timing, and supported inactive object. Existing evidence for earlier Retail/Mists remains historical and does not prove native 12.1.5 parity.

Existing `tests/c_spell_flyout_probes.rs` covers shared identifier lookup. Existing action duration cases cover the moved construction path.

## Known gaps (current cycle)

- [ ] Native cooldown interval selection, GCD precedence, snapshot clock/lifetime, secrecy, exact error rendering, spell-name aliases, and all spell-link forms remain unverified. R17 covers parser types and normal-result categories, not every resolver or timing branch.
- [ ] Real Blizzard-consumer acceptance is not established by seeded API fixtures.

## Out of scope

- Real spell-name discovery beyond seeded aliases, charges/loss-of-control producers, non-default rates, secrets/security, and cooldown-engine redesign.
