# Forever input interface style

## Contract

- Forever exposes `C_InputInterfaceStyle.GetCurrentStyle()` and `Enum.InputDeviceInterfaceType` (`Mkb=0`, `Gamepad=1`).
- Style belongs to one simulator environment. Initial Mkb is a simulator keyboard/mouse policy, not an observed native default.
- Rust simulator input control changes style before queuing `INPUT_DEVICE_INTERFACE_TRANSITION(newStyle, oldStyle)`. Selecting the same style queues nothing (simulator policy).
- Blizzard `InputUtil.lua` owns callback registration, initialization and transition dispatch. No C API callback methods are added.
- No host gamepad integration, new Admin API, native conformance or arbitrary input-style coercion is implied.

## Sources and proof

Authenticated Forever 1.60.1.69913 `InputInterfaceStyleDocumentation.lua`, `InputConstantsDocumentation.lua`, and `InputDocumentation.lua` specify the API, enum and event. `Mainline/InputUtil.lua` and shared `InputUtil.lua` are behavioral test consumers.

`tests/wowforever_input_style.rs` covers initial callback initialization, source-owned setup/uninit/init callbacks, exact new/old queued payload, state-before-callback, same-style suppression and independent environments. Final independent verification remains pending.
