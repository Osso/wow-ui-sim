# Character-create blur profile

`C_CharacterCreation.SetBlurEnabled` stores the character-create camera-profile selection used by the Glue screen. The simulator models the Boolean state, but not the 3D visual effect. See the [model-scene catalogue](model-scene-catalogue.md) for the permanent 3D rendering gap.

## What it must do

### Registration and input

- [ ] Register `SetBlurEnabled` on Retail and Forever Glue screens only.
- [ ] Require argument 1. Missing or nil raises the standard bad-argument error with `Usage: C_CharacterCreation.SetBlurEnabled(enabled)`.
- [ ] Use Lua truthiness: only `false` disables the selected profile; `0`, `""`, and other non-nil values enable it.
- [ ] Return no values.

### State

- [ ] Store one Boolean for the selected camera profile. Do not add a getter.
- [ ] Keep the current PTR surface unchanged. PTR registration is UNKNOWN.

## How it works

- [Character-create altered form](character-create-altered-form.md)
- [Selected model-scene catalogue](model-scene-catalogue.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs` — Glue registration, required-argument behavior, truth conversion, and stored state.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — per-environment camera-profile Boolean.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs` — retains the existing PTR-only no-op surface and unrelated 3D placeholders.

## Tests asserting this spec

- `tests/character_create_altered_form.rs::set_blur_enabled_stores_required_lua_truthiness_without_a_getter` — checks screen registration, missing/nil errors, return arity, truthiness, and stored Boolean.
- The three blocked Forever Glue character-create filters — exercise the original consumer call; rerun after implementation.

## Known gaps (current cycle)

- [ ] PTR namespace registration is UNKNOWN; retain the existing PTR surface without inferring native availability.

## Out of scope

- Permanent 3D visual gap: do not render the blur, model, or camera profiles. R51 confirms native `SetCameraZoomLevel` reads the stored Boolean only to select a camera profile; the simulator stores the flag but does not render that effect. See the [model-scene catalogue](model-scene-catalogue.md).
- Modeling preview gear, dress state, camera zoom, or other 3D effects.
