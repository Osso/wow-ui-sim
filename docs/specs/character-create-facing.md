# Character-Create Default Facing

`C_CharacterCreation.GetDefaultCharacterCreateFacing` supplies the model rotation used by the Glue character-create screen. Its backing state lives in `SimState`; see the [character-create mode system](../wiki/systems/character-create-mode.md).

## What it must do

- [ ] Retail, PTR, and Forever expose the query on Glue screens and remove it on Game screens.
- [ ] The query takes no arguments and returns exactly one number.
- [ ] The host may supply a facing in degrees for a race ID and sex ID pair. The current selection selects the pair.
- [ ] If the host has no facing for the current race and sex, return `0`.
- [ ] Do not add guessed per-race values.
- [ ] The original `CharacterCreateMixin:ResetSubjectRotation` consumer passes the returned facing to `RotateSubjectToTarget`.

## How it works

- [Character-create mode input](character-create-mode.md)
- [Character-create mode system](../wiki/systems/character-create-mode.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs`: registers the Glue-only query and reads the selected race/sex input.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: store and initialize host-supplied facing values.
- `src/lua_api/env_runtime.rs`: updates the API when the screen changes.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs`: no longer supplies a constant facing.

## Tests asserting this spec

- `tests/glue_character_select.rs::character_create_default_facing_uses_host_race_sex_data_in_blizzard_consumer`

## Known gaps (current cycle)

- [ ] The full character-create scene and 3D model rendering remain outside scope.

## Out of scope

- Guessed model facings, camera/scene behavior, and character customization state.
