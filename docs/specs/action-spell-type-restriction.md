# PTR Action Spell Type Restriction

`C_ActionBar.SpellMatchesActionSpellTypeRestriction` decides whether a spell can be placed on an action slot in the PTR Game environment. Its client behavior comes from R33; the PTR ActionButton call site is described in the [cursor model system](../wiki/systems/cursor-model.md).

## What it must do

### Registration and arguments

- [ ] Register only in the PTR Game environment; do not publish the member on Retail.
- [ ] Require `spellID` followed by `actionID`; parse signed 32-bit and unsigned 32-bit values, accept numeric strings, and truncate fractions toward zero.
- [ ] Report the R33 usage and range errors for missing, nil, nonnumeric, non-finite, and out-of-range arguments.

### Results

- [ ] Return zero Lua values when the converted action slot is outside 1–408; otherwise return one boolean.
- [ ] In normal mode, return true for every valid slot, including `spellID == 0`.
- [ ] In WoW Hack mode, return false for `spellID == 0`; allow nonzero spells in unrestricted slots; and require the slot group's SpellLabel in restricted slots.
- [ ] Read SpellLabel IDs from host-supplied per-spell data, with no records by default.

## How it works

- [Cursor model system](../wiki/systems/cursor-model.md) — the original PTR ActionButton drag path and cursor transitions.

## Implementation inventory

- `src/c_api/c_action_bar.rs` — PTR API registration, argument parsing, and restriction results.
- `src/lua_api/state/sim_state.rs` — host-supplied spell labels.
- `src/lua_api/state.rs` — empty default for spell labels.
- `tests/action_bar_drag.rs` — PTR API behavior and Retail absence assertions.

## Tests asserting this spec

- `tests/action_bar_drag.rs`

## Known gaps (current cycle)

None.

## Out of scope

- Retail does not publish this function.
- Native enum companions that the original ActionButton call path does not read are not modeled here.
