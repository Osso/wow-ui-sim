# Character-Create Default Facing

`C_CharacterCreation.GetDefaultCharacterCreateFacing` supplies the default model rotation used by the Glue character-create screen. `GetCharacterCreateFacing` and `SetCharacterCreateFacing` read and write the current rotation in degrees. Their backing state lives in `SimState`; see the [character-create mode system](../wiki/systems/character-create-mode.md).

## What it must do

- [ ] Retail and Forever expose the default-facing query on Glue screens and remove it on Game screens.
- [ ] Keep the existing PTR default-facing surface. Native PTR registration is UNKNOWN and is not asserted.
- [ ] The query takes no arguments and returns exactly one number.
- [ ] The host may supply a facing in degrees for a race ID and sex ID pair. The current selection selects the pair.
- [ ] If the host has no facing for the current race and sex, return `0`.
- [ ] Do not add guessed per-race values.
- [ ] The original `CharacterCreateMixin:ResetSubjectRotation` consumer passes the returned facing to `RotateSubjectToTarget`.

## Current facing

- [ ] Retail and Forever expose Get/Set only on Glue screens; both are absent on Game screens.
- [ ] Get takes no arguments and returns one number in degrees from the stored radians. The initial value is 0.0.
- [ ] Set requires a number or numeric string, converts degrees to radians, and wraps the value into [−π, +π]. It returns no values and stores the result when no model exists.
- [ ] Missing or nil input raises the wrapped usage error. Non-numeric and finite out-of-range inputs raise the wrapped range error. NaN and infinity raise the wrapped finite-number error.
- [ ] The original CharacterCreate reset consumer loads the host-supplied default for the selected race/sex; absent host data selects 0 degrees.
- [ ] Keep the existing PTR current-facing surface without adding or asserting PTR registration claims.

## How it works

- [Character-create mode input](character-create-mode.md)
- [Character-create mode system](../wiki/systems/character-create-mode.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs`: registers the Glue-facing defaults and current-facing APIs and reads selected race/sex input.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: store and initialize current radians and host-supplied default facings.
- `src/lua_api/env_runtime.rs`: updates the API when the screen changes.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs`: retains the existing PTR current-facing fallback.

## Tests asserting this spec

- `tests/glue_character_select.rs::character_create_default_facing_uses_host_race_sex_data_in_blizzard_consumer`: checks R34 conversion/errors and the original reset consumer.

## Known gaps (current cycle)

- [ ] The full character-create scene and 3D model rendering remain outside scope.

## Out of scope

- Guessed model facings, camera/scene behavior, and character customization state.
