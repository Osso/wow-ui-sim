# Forever input interface style

## Contract

- Forever exposes `C_InputInterfaceStyle.GetCurrentStyle()` and `Enum.InputDeviceInterfaceType` (`Mkb=0`, `Gamepad=1`).
- Style belongs to one simulator environment. Initial Mkb is a simulator keyboard/mouse policy, not an observed native default.
- Forever `IsUsingGamepad()` returns whether that same current style is `Gamepad`. It changes with the existing model before transition callbacks; neither `C_GamePad.IsEnabled()` capability nor per-frame gamepad enablement selects the current style.
- Rust simulator input control changes style before queuing `INPUT_DEVICE_INTERFACE_TRANSITION(newStyle, oldStyle)`. Selecting the same style queues nothing (simulator policy).
- Blizzard `InputUtil.lua` owns callback registration, initialization and transition dispatch. No C API callback methods are added.
- No host gamepad integration, new Admin API, native conformance or arbitrary input-style coercion is implied.

## Sources and proof

Authenticated Forever 1.60.1.69913 `InputInterfaceStyleDocumentation.lua`, `InputConstantsDocumentation.lua`, and `InputDocumentation.lua` specify the API, enum and event. `Mainline/InputUtil.lua` and shared `InputUtil.lua` are behavioral test consumers.

`tests/wowforever_input_style.rs` covers initial callback initialization, source-owned setup/uninit/init callbacks, exact new/old queued payload, state-before-callback, same-style suppression and independent environments. Final independent verification remains pending.

The `IsUsingGamepad` extension uses `src/lua_api/globals/real/input_interface.rs`; no duplicate input state or new input-event routing is introduced. Two additional tests cover model transitions/environment isolation, capability/frame-flag independence, and state observed inside existing Blizzard transition callbacks. Frozen `b8f0982be` reproduces the missing global after successfully reading the existing Mkb style and enabling frame flags (`/tmp/forever-addon-audit/is-using-gamepad-red-gqtwjpbd/ledger.json`). Compiled GREEN and actual Angleur replay remain pending. Other profiles retain their existing global surface.
