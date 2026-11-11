# Character-create camera and preview state

`C_CharacterCreation` Glue APIs store the character-create camera zoom and validate model-preview inputs. The simulator does not render the character model or its camera, dress, and gear visuals. See the [model-scene catalogue](model-scene-catalogue.md) for the permanent 3D gap.

## What it must do

### Registration

- [ ] Register `GetCurrentCameraZoom`, `SetCameraZoomLevel`, `SetModelDressState`, and `SetSelectedPreviewGearType` on Retail and Forever Glue screens only.
- [ ] Keep the current PTR surface unchanged. PTR registration is UNKNOWN for these APIs.

### Camera zoom

- [ ] `SetCameraZoomLevel` requires a signed 32-bit integer `zoomLevel`, truncates it toward zero, then clamps it to 0 through 100.
- [ ] `keepCustomZoom` is optional: missing or nil is false; other ordinary values use Lua truthiness.
- [ ] Store the clamped zoom. `GetCurrentCameraZoom` returns exactly one number from that same stored value.
- [ ] When the internal custom-zoom flag is true and `keepCustomZoom` is truthy, preserve the stored zoom. When a different zoom is stored, clear that flag. An unchanged zoom does not clear it.
- [ ] Use `Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom])` for the required-argument error.

### Model dress and preview gear

- [ ] `SetModelDressState` requires one non-nil argument. Ordinary non-nil values use Lua truthiness. It has no modeled 3D effect.
- [ ] `SetSelectedPreviewGearType` converts numeric inputs, truncates toward zero, then validates the low byte as 0 or 1. Other low bytes raise the usage error. It has no modeled 3D effect.
- [ ] On Forever, numeric strings are accepted and failed ordinary non-nil conversions map to 0.
- [ ] Use `Usage: C_CharacterCreation.SetModelDressState(dressedState)` and `Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)` for their argument errors.
- [ ] The simulator dispatches no events and writes no CVars; native indirect effects are UNKNOWN.

## How it works

- [Character-create blur profile](character-create-blur.md)
- [Character-create mode input](character-create-mode.md)
- [Selected model-scene catalogue](model-scene-catalogue.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs` — Glue registration, argument conversion, camera zoom storage/readback, and preview-input validation.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — per-environment zoom and custom-zoom flag.
- `src/lua_api/workarounds/temporary/character_create_defaults.rs` — retains the prior PTR-only Lua fallbacks.

## Tests asserting this spec

- `tests/character_create_visual_state.rs` — Game/Glue registration, camera zoom conversion/clamping/readback, custom-zoom truthiness, required-argument errors, low-byte enum validation, Forever-only conversion acceptance, getter arity, and simulator event/CVar effects.
- `tests/character_create_altered_form.rs::set_blur_enabled_stores_required_lua_truthiness_without_a_getter` — R51 blur state and registration.
- The three blocked Forever Glue character-create filters — exercise the original consumer calls.

## Known gaps (current cycle)

- [ ] PTR registration is UNKNOWN for these APIs; retain existing PTR fallback behavior and do not publish new Rust registrations there.
- [ ] The native initial camera-zoom and custom-zoom flag values are UNKNOWN. The simulator initializes zoom to 0 and the custom flag to false; no native custom-zoom producer is modeled.
- [ ] Retail numeric-string and failed-value conversion behavior is UNKNOWN; its matching simulator implementation does not prove native parity.
- [ ] Acceptance and conversion details for non-finite numbers, secret values, and restricted stack states are UNKNOWN.
- [ ] **INFERRED:** Successful R57 setter calls return no values; the measured business paths and separate zero-result tails do not prove every intervening guard path end to end.
- [ ] Only the checked native setter paths are known to omit explicit named events and CVar writes; indirect event/CVar effects are UNKNOWN.
- [ ] The native catalogs declare no dedicated model-dress or selected-gear getter. Indirect readback is UNKNOWN.

## Out of scope

- Permanent 3D visual gap: do not render character-model camera profiles, blur, dress state, or preview gear. Keep the zoom value because `GetCurrentCameraZoom` exposes it. See the [model-scene catalogue](model-scene-catalogue.md).
