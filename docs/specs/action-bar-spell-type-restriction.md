# Action-bar spell type restriction

`C_ActionBar.SpellMatchesActionSpellTypeRestriction` is a PTR action-slot query. **VERIFIED:** PTR API documentation at `ActionBarFrameDocumentation.lua:1031–1048` declares required `spellID` and `actionID` inputs and a Boolean result. **VERIFIED:** native behavior returns zero Lua values for converted slots outside the valid range. The modeled restriction and spell-label inputs are host supplied; the simulator does not ship `SpellLabel` data.

## What it must do

### API and arguments

- [ ] Publish the method only for PTR.
- [x] Read required `spellID` then `actionID`; accept finite numbers and numeric strings, and truncate fractions toward zero.
- [x] Reject tested out-of-range spell and action IDs with their verified range errors.
- [x] For an action slot converted outside `[0, 408)`, return zero values without an argument error.

### Restriction result

- [x] For a valid slot and an empty restriction map, return one `true` value before checking whether `spellID` is zero.
- [x] With a non-empty map, return `false` for spell ID zero; a missing or zero restriction permits a nonzero spell.
- [x] Key restrictions by zero-based 12-slot action group and compare the required label ID with host-supplied spell labels by exact equality.
- [ ] Keep restriction-map and spell-label inputs empty by default; do not ship label records or assert real spell membership.

## How it works

- [Lua API architecture](../lua-api.md)
- [Consumer-selection and RED proof](../wiki/investigations/consumer-selection-and-red-proof.md)

## Implementation inventory

- `src/c_api/c_action_bar_spell_type_restriction.rs` — PTR API publication, argument conversion, and result selection.
- `src/c_api/action_button_hover.rs` — shared action-slot conversion used by the PTR API.
- `src/lua_api/globals/action_bar_api/registration.rs` — action-bar API registration.
- `src/lua_api/state/sim_state.rs` — host-supplied group-to-label restrictions and spell-label inputs.
- `src/lua_api/state.rs` — empty-state construction for those inputs.

## Tests asserting this spec

- `tests/action_bar_spell_type_restriction.rs` — public result arity, conversions, errors, profile publication, and host-supplied label matching.
- `tests/action_bar_drag.rs` — selected PTR drag consumer under an empty restriction map.

## Known gaps (current cycle)

- [ ] Automatic WoWHack restriction-map initialization and the native `SpellLabel` database are not modeled.
- [ ] Game-only versus Glue publication is inferred, so this simulator contract does not claim or enforce it.
- [ ] Cached API documentation promises a non-nil Boolean for valid slots; the verified native invalid-slot path returns zero values, so the implementation follows the native path.

## Out of scope

- Shipping spell-label records or claiming whether a specific real spell matches a restricted slot.
- Guessing screen-specific registration behavior or native traceback text beyond verified error payloads.
