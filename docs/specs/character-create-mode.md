# Character-Create Mode Input

`C_CharacterCreation.SetInCharacterCreate` supplies the character-creation mode flag used by the Glue character-create screen. The cached Forever consumer is `Blizzard_CharacterCreate/Camelot/Blizzard_CharacterCreate.lua` in `CharacterCreateMixin:OnShow` and `OnHide`.

## What it must do

- [ ] Forever registers the setter through the Glue API installer chain; the Game installer chain does not register it. Glue-only visibility is INFERRED because cross-environment Lua table copying is UNKNOWN.
- [ ] The setter requires one argument and uses Lua truthiness: `false` clears the mode; `true`, zero, strings, and tables set it. Missing or nil raises `Usage: C_CharacterCreation.SetInCharacterCreate(inCharacterCreate)`.
- [ ] The setter returns no values, raises for inaccessible secret arguments, and fires no named event.
- [ ] The setter changes only the modeled `in_character_create` flag and has no effect when the requested state is already active.
- [ ] Retail uses the same handler behavior. The simulator exposes the modeled setter on Glue screens only.

R27 native probe: Forever's Glue registration, argument truthiness, error behavior, return count, mode-flag mutation, repeated-state no-op, and absence of named events are VERIFIED for build 1.60.1.70707. Retail's handler pattern is VERIFIED for build 12.1.5.70688. PTR's handler exists, but its registration location is UNKNOWN; the simulator retains its prior user-approved PTR registration.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs`: implements the mode setter and Glue-screen registration.
- `src/lua_api/env_runtime.rs`: updates the C API surface when the selected screen changes.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs`: retains the existing PTR registration path while using the shared setter.

## Tests asserting this spec

- `tests/glue_character_select.rs::character_create_mode_is_glue_only_and_uses_lua_truthiness`: checks screen-specific publication, accepted truthy values, no return values, errors, state changes, and no event.
- `tests/glue_character_select.rs::character_select_screen_shows_character_select_not_login`: loads the original Glue consumers, including the CharacterCreate screen transition.

## Known gaps (current cycle)

- [ ] Native entry/exit resets also touch creation scene, selection, and customization state and call model/customization helpers. Those reset targets are UNKNOWN and have no matching simulator state; do not invent them.
- [ ] PTR registration location is UNKNOWN; keep the existing user-approved registration path.
- [ ] Targeted Forever tests remain pending slot 3 admission.

## Out of scope

- Modeling the 3D creation scene, selected race/class preview state, customization choices, or helper side effects without corresponding simulator inputs.
