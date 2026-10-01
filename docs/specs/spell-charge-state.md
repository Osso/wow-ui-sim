# Spell charge state and duration queries

Explicit spell-keyed charge input backs `C_Spell.GetSpellCharges`, `C_ActionBar.GetActionCharges`, and Retail 12.0.5+ action/spell/spellbook charge durations. Source: `src/c_api/charge_state.rs`. See [duration core](duration-core.md) for existing time/rate semantics.

## What it must do

- [ ] Return no charge table for missing explicit spell charge state. Both table queries expose `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, and `chargeModRate` from the same input, including at maximum charges.
- [ ] On Retail 12.0.5+, missing charge state or unresolved identity returns no duration. Ordinary cooldown/GCD, action membership and spellbook membership do not establish charges.
- [ ] On Retail 12.0.5+, configured current charges at or above maximum return a fully elapsed zero-span duration at query time. This rule comes from the retained [12.0.5 source](../../data/patch-api/sources/12.0.5-api-changes.txt).
- [ ] Below maximum, snapshot the explicit recharge start/base duration/rate through existing duration-core semantics. This selection is a simulator inference; no native recharge/rate parity is claimed.
- [ ] Use existing numeric/seeded-alias spell identity, actual action-to-spell assignments and player spellbook slot/bank resolution. Reject unresolved banks/slots; do not invent pet or macro mappings.
- [ ] Existing duration snapshots retain their configured timing after input/mapping changes, while elapsed/remaining observations follow the simulator clock. New queries read current input.
- [ ] Preserve earlier spell/book duration publication boundaries. The already-published action duration query uses the shared explicit model globally, not an empty-object fallback; pre-12.0.5 maximum charges have no active recharge duration (inference). Table queries globally replace fabricated placeholders without changing their five-field shape.
- [ ] An explicit row with `max_charges == 0` does not configure a charge spell: all queries return no charge data. This is an input validity policy, not native behavior.
- [ ] Retail 12.0.5+ zero-span durations report expired and elapsed fraction one, remaining fraction zero; duration quantities remain zero. Default/reset/future-start zero spans follow this same core rule. Mapping “fully elapsed” to these two observations is explicit inference; HasStarted/IsActive remain unchanged.

## How it works

- [Duration core](duration-core.md)
- [Charge model](../wiki/systems/spell-charge-state.md)

## Implementation inventory

- `src/c_api/charge_state.rs`: public five-field input, shared table/duration producers.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: empty `spell_charges` map and default.
- `src/c_api/c_spell.rs`: existing identifier resolver and authoritative spell queries.
- `src/c_api/c_spell_book.rs`: existing slot/bank resolver and charge duration registration.
- `src/lua_api/globals/action_bar_api.rs`: actual action assignment and shared charge queries.
- `src/lua_api/globals/lua_duration_object.rs`: validated rate-aware duration snapshot constructor.
- `src/lua_api/workarounds/temporary/spell_static_defaults.rs`: obsolete charge fallback removed; unrelated defaults retained.

## Tests asserting this spec

`tests/cooldown_probes/charge_duration.rs` remains in the existing grouped integration binary. Initial actual RED at `e0a46d691`: 4 cases, 1 passing no-data spell control and 3 failures (`/tmp/patch-12.0.5-batch5-charge-red.log`). Type/input and concrete fixtures committed at `d1f2474e5`, strengthened nonnil assertions at `91a0bfb55`; concrete fixture RED at `eac08bda3` records 10 cases, 1 pass and 9 expected failures (`/tmp/patch-12.0.5-batch6-charge-red.log`; exact argv in `/tmp/patch-12.0.5-batch6-runs.json`). This includes direct shared zero-span failure before core changes. Producers and core correction follow that RED; batch7 post-change charge fixtures PASS 10/10; independent final gates remain parent-owned. Existing `tests/duration_core.rs` zero/reset/default expectations are epoch-aware; direct zero-span and zero-maximum input cases join the grouped charge fixtures.

The passing no-data spell control did not prove a modeled provider: `runtime_surface_bootstrap.lua:65–76` lazily installs a nil-returning closure for an unresolved namespace key. The authoritative explicit registration replaces that key, not the unrelated namespace policy.

## Known gaps (current cycle)

- [ ] Parent-batched GREEN, earlier-profile controls, format/check/startup and readability proof.
- [ ] Native rate interpretation, return arity, malformed input/error behavior, snapshots and charge transitions remain unprobed. Future native fixture: one configured charged spell at max and during recharge, compare all five table fields plus duration timing/rate before and after one charge use.

## Out of scope

- Automatic spending/replenishment, recharge ordering, spell metadata inference, pet-bank and macro association: no explicit model/evidence authorized.
- Secret behavior, vendor edits, deployment and native Forever probes: no security bypass or native parity claim.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

`cooldown_probes::charge_duration::` PASS 10/10 (`integration-2.log`); `duration_core::` PASS 27/27 (`integration-3.log`). Earlier-profile controls and automatic progression remain unproven.
