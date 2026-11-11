# Character-create altered-form selection

`C_CharacterCreation.SetViewingAlteredForm` and `IsViewingAlteredForm` store the altered-model selection for the Glue character-create screen. The host supplies which race IDs have an altered model. See [character-create mode input](character-create-mode.md) and [the character-create mode system](../wiki/systems/character-create-mode.md).

## What it must do

### Registration and values

- [ ] Register both functions on Retail and Forever Glue screens. Keep both unregistered on PTR because native PTR registration is UNKNOWN.
- [ ] `SetViewingAlteredForm` requires a non-nil argument. Missing or nil raises `Usage: C_CharacterCreation.SetViewingAlteredForm(isViewingAlteredForm)`.
- [ ] Lua truthiness applies: only `false` selects false; other non-nil values select true.
- [ ] The setter returns no values. It does nothing when the selected race has no host-supplied altered model, except that it stores false.
- [ ] `IsViewingAlteredForm` returns exactly one Boolean: a host-supplied model exists for the selected race and the stored choice is true. It returns false when no race or model is selected.
- [ ] The stored choice starts false. Setting it to its current value has no effect. Barber Shop altered-form state remains separate.

### Lifecycle

- [ ] Entering the CharacterCreate screen and entering character-create mode clear the stored choice.
- [ ] A real race change clears the choice; selecting the current race does not.
- [ ] `SetSelectedSex`, `SetSelectedClass` and `ResetCharCustomize` rebuilds keep the choice when the selected race has a host-supplied altered model. They clear it when the race has no such model.
- [ ] Do not emit a direct event. Indirect event behavior is UNKNOWN.

## How it works

- [Character-create mode input](character-create-mode.md)
- [Character-create default facing](character-create-facing.md)
- [Character-create mode system](../wiki/systems/character-create-mode.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs` — registration, argument behavior, host-model lookup and stored state.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — per-environment model IDs and stored choice.
- `src/lua_api/env_runtime.rs` — selected-screen registration and CharacterCreate initialization.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs` — maps simulated race, sex, class and `ResetCharCustomize` setters to the native state transitions.

## Tests asserting this spec

- `tests/character_create_altered_form.rs` — registration, truthiness, return arity, model presence, race changes, sex/class/customization rebuilds, and screen/mode resets for Retail and Forever.

## Known gaps (current cycle)

- [ ] PTR registration is UNKNOWN; do not register or assert these methods on PTR.
- [ ] No camera, model, animation, or other 3D presentation effects are modeled.
- [ ] Indirect Lua event behavior is UNKNOWN and is not asserted.

## Out of scope

- Native model/camera side effects, pending customization side effects and Barber Shop state.
