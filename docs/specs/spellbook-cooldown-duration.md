# Spellbook cooldown duration

`C_SpellBook.GetSpellBookItemCooldownDuration(slot, spellBank [, ignoreGCD])` snapshots the cooldown for an existing spellbook entry. Retained `data/patch-api/sources/12.0.5-api-changes.txt:323–324` adds arg3 `ignoreGCD`; interval and invalid-input policies below are explicit simulator inferences. See [duration core](../wiki/systems/duration-core.md) for the shared object model.

## What it must do

- [ ] Register the modeled producer on retail 12.0.0+ without replacing earlier-profile compatibility defaults.
- [ ] Resolve a positive integral slot in player bank 0 through existing spellbook data, not as a spell ID. Slot 5 resolves to Flash of Light, spell 19750.
- [ ] Return nil for missing/non-numeric/fractional/out-of-range slots, missing/unsupported banks, or absent entries. Pet entries are not modeled; do not fabricate them.
- [ ] Use the existing later-ending active spell/GCD interval for omitted/false. On retail 12.0.5+, true selects only the active individual interval. Earlier epochs ignore the argument.
- [ ] Return a zero-span duration for a valid entry without an active selected cooldown; retain rate one, runtime clock, and query-time snapshot independence.

## How it works

- [Duration core](../wiki/systems/duration-core.md)
- [Shared selection policy](spell-cooldown-duration.md#1205-ignoregcd-contract-green-pending)

## Implementation inventory

- `src/c_api/c_spell_book.rs`: gated producer, slot/bank lookup, shared snapshot construction.
- `src/c_api/cooldown_duration.rs`: duration-only selector and 12.0.5 argument gate.
- `src/lua_api/globals/spellbook_data.rs`: existing player slot entries.
- `src/lua_api/globals/lua_duration_object.rs`: existing zero/nonzero timed-duration factory.

## Tests asserting this spec

`tests/cooldown_probes/ignore_gcd.rs` covers slot 5/bank 0, rejected ID-as-slot and pet/unknown bank inputs, overlapping/individual/GCD-only/empty/expired state, omitted/false/true, snapshots and runtime clock. Actual RED at `9a50d8a5c`: 0/6 pass in `/tmp/patch-12.0.5-batch4-ignore-gcd-red.log`; six cases jointly exercise action/spell/spellbook, not six independent native contracts. Parent owns batched GREEN; checkboxes remain unchecked pending that proof.

## Development proof and independent bounded acceptance — 2026-10-03 (cooldown-ignore-gcd-authentication)

Commit `4d142e325`. RED: 2 PASS / 3 FAIL module. GREEN: 5/5 module. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-auras-nav.md) SHA256 `c810ea3b9a7e3468a21bae71c13d18bedfe71c13c51e4438eb1f8a280ef11b20`. Spell variant, coercion and consumers unproven. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-114 partial-development-green under capability `cooldown-ignore-gcd-authentication`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Post-change GREEN and earlier-profile/epoch preservation proof are pending.
- [ ] Native invalid-input, selection, coercion, secrecy, identity/lifetime, and real-consumer behavior remain unverified.

## Out of scope

- Pet book entries, macro spell resolution, dynamic spellbook redesign, numeric spellbook-cooldown modernization, charges/loss-of-control, secrecy/security, vendor behavior, and non-default rates.

## Authored follow-up for prose 2026-03-25-114 (not executed)

- [ ] Query-selected objects from action/spell/spellbook drive actual Cooldown widgets; removing the individual interval clears new ignoreGCD=true objects, while false retains GCD and old true snapshots remain usable.
- [ ] On retail12.0.5+, action/book ignoreGCD uses VM authentication per cached AllowedWhenUntainted: secure secret booleans select the same intervals as plain booleans; tainted secret flags error without clearing caller taint.
- [ ] Authenticate book ignoreGCD before invalid-entry nil return; secure invalid-entry queries still return nil.

Tests: `tests/retail_12_0_5_partial_104_114.rs`. INFERRED: retain the prior literal-true-only public conversion policy and earlier-epoch argument-ignore policy; this is not native type/coercion validation. Interval selection remains the existing inferred later-end/individual-only model. No additional SimState fields.

Scope is only the new flag for action/book and ordinary widget consumption for all three APIs. Action-slot and book slot/bank secret authentication remain unmodeled by these duration producers. Spell's distinct cached AllowedWhenTainted policy is unresolved and deliberately unchanged: applying the untainted-only helper there would misrepresent its contract. Restricted-output secrecy, pet book entries, native behavior, real Blizzard consumers, earlier-epoch proof and independent acceptance remain open. Row114 stays partial; these authored additions carry no execution credit.
