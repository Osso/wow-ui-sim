# `IsUsingGamepad`

`IsUsingGamepad()` reports whether the active input is gamepad input. The global is declared in Retail and PTR `Blizzard_APIDocumentationGenerated/InputDocumentation.lua` and uses profile-specific state; see the [input-interface model](../wiki/systems/gamepad-mapped-state.md).

## What it must do

### Retail and PTR

- [x] VERIFIED (Retail 12.1.5.70688): return a non-nil boolean. The gamepad-input timestamp starts at zero, so return false until its writer runs.
- [ ] VERIFIED (Retail 12.1.5.70688): read gamepad-input time, not mouse time or `C_GamePad.IsEnabled()` state.
- [ ] INFERRED: return true while unsigned 32-bit elapsed milliseconds are less than `GamePadOverlapMouseMs`; the exact boundary behavior is UNKNOWN.
- [ ] Do not invent timestamp writers, input events, or mouse-input resets; the native writer's conditions and lifecycle remain UNKNOWN.

### Forever

- [ ] Preserve the existing `InputInterfaceStyle`-backed result.

## How it works

- [Input-interface declarations](../lua-api.md)
- [Input style and mapped gamepad state](../wiki/systems/gamepad-mapped-state.md)

## Implementation inventory

- `src/lua_api/globals/real/input_interface.rs` — profile-specific `IsUsingGamepad` implementation.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs` — profile-gated module and global registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — Retail/PTR timestamp seam with empty default.

## Tests asserting this spec

- `src/lua_api/globals/real/input_interface.rs` — Retail default-false unit test.
- `tests/retail_ellesmere_core_gamepad_consumer.rs` — original Retail addon consumer call.
- `tests/wowforever_input_style.rs` — preserves the Forever style-backed contract.

## Known gaps (current cycle)

- [ ] No simulator-side writer updates the Retail/PTR timestamp. Native writer conditions and the event relationship are UNKNOWN.
- [ ] The `GamePadOverlapMouseMs` comparison is an explicit inference, not verified native behavior.

## Out of scope

- Hardware discovery, `C_GamePad` state, input-event delivery, and mouse/keyboard reset behavior; the available native evidence does not establish those transitions.
