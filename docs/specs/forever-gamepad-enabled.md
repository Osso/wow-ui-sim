# Retail and Forever gamepad queries

`src/c_api/c_game_pad.rs` publishes a bounded Retail device-query surface and the existing Forever `C_GamePad.IsEnabled()` capability. [System notes](../wiki/systems/gamepad-mapped-state.md) describe the separate mapped-stick model and the evidence boundary.

## What it must do

- [x] Forever `C_GamePad.IsEnabled()` returns one boolean from the stored `GamePadEnable` CVar, as declared by Forever 1.60.1.70532 `Blizzard_APIDocumentationGenerated/GamePadDocumentation.lua:222-229`.
- [x] Retail `GetAllDeviceIDs()` returns a non-nil table and `GetDeviceRawState(deviceID)` returns a nilable raw-state record, as declared by `GamePadDocumentation.lua:126-131,174-185`. **INFERRED (simulator mapping):** without physical-device input, the table is empty and raw state is nil.
- [x] Retail `IsEnabled()` returns one boolean, as declared by `GamePadDocumentation.lua:213-219`. **INFERRED (simulator mapping):** it reads the existing `GamePadEnable` CVar.
- [x] `GamePadEnable` value `"1"` means enabled; its existing default `"0"` means disabled in the simulator. This is a state projection, not proof of native CVar/device equivalence or hardware detection.
- [x] Keep the bounded mapped-stick and hover-policy queries on PTR and Forever. Retail device queries do not read `SimState.gamepad_mapped_sticks`.
- [x] Forever's original Game-screen restricted-addon wrapper observes disabled/enabled/disabled through `CallRestrictedClosure` and `IsGamePadEnabled`.

The Retail generated `GamePadDocumentation.lua` declares these members. The simulator has no physical-device input, so Retail exposes only the empty device list and nil raw-state result; connection, disconnection and device events are not modeled.

## How it works

- [Mapped-state system and capability boundary](../wiki/systems/gamepad-mapped-state.md).
- [Consumer selection and RED requirements](../wiki/investigations/consumer-selection-and-red-proof.md).

## Implementation inventory

- `src/c_api/c_game_pad.rs` — Retail empty device queries and the bounded PTR/Forever projections.
- `src/c_api/mod.rs`, `src/c_api/registration.rs` — profile publication boundaries.
- `src/cvars.yaml`, `src/cvars.rs` — existing default and stored boolean input; unchanged by this lane.

## Tests asserting this spec

`tests/forever_gamepad_enabled.rs` covers the original Forever restricted wrapper. `tests/retail_dialogue_ui_gamepad_surface.rs` checks the Retail API boundary used by DialogueUI.

## Known gaps

- Native enablement rules and hardware detection remain UNKNOWN. Retail has no simulated device input; the empty-device result is a simulator default, not a native hardware claim.

## Out of scope

Native enablement rules/default, physical-device discovery and hotplug lifecycle, enablement events, temporary-CVar behavior, the deferred Gamepad SetTempCVar test and module-import initialization remain out of scope. PTR `IsEnabled()` remains unpublished.
