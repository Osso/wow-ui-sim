# Key-down state

Retail `IsKeyDown(keyOrMouseName, excludeBindingState?)` reads simulator keyboard, modifier, and mouse-button state. Keyboard dispatch is in `src/lua_api/key_dispatch.rs`; the API is registered with modeled modifier globals.

## What it must do

- [x] A fresh headless environment reports an unpressed key as false.
- [x] An existing key-down dispatch makes its key observable inside `OnKeyDown` and while held; key-up clears it.
- [x] Synthetic one-shot `send_key_press` leaves no key held after dispatch.
- [ ] GUI key-down and key-up events maintain held state across frames (routed but not yet exercised by an automated GUI event test).
- [ ] Existing modifier and mouse-button state is reflected by corresponding key names (modeled, not independently tested here).

## How it works

- [Keyboard dispatch](../event-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/key_dispatch.rs` — dispatch and physical-key transitions.
- `src/lua_api/state.rs`, `src/lua_api/state/sim_state.rs` — per-environment held keys.
- `src/lua_api/globals/real/modifier_keys.rs` — `IsKeyDown` query alongside modifier probes.
- `src/iced_app/view.rs`, `src/iced_app/mod.rs`, `src/iced_app/update.rs` — GUI key event routing.

## Tests asserting this spec

- `tests/key_dispatch.rs` — headless defaults, live dispatch observation, release, one-shot behavior.

## Known gaps (current cycle)

- [ ] Native behavior for unknown keys and `excludeBindingState` is unverified; the latter currently reads physical state without alternate binding-state resolution.

## Out of scope

- Native validation/secret-value semantics and additional keyboard layouts require client evidence. Empty names return nil as a simulator policy, not a native-proven rule.
