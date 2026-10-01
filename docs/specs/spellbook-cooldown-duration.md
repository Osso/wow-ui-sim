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

## Known gaps (current cycle)

- [ ] Post-change GREEN and earlier-profile/epoch preservation proof are pending.
- [ ] Native invalid-input, selection, coercion, secrecy, identity/lifetime, and real-consumer behavior remain unverified.

## Out of scope

- Pet book entries, macro spell resolution, dynamic spellbook redesign, numeric spellbook-cooldown modernization, charges/loss-of-control, secrecy/security, vendor behavior, and non-default rates.
