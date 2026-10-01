# Forever mapped-stick queries

`f774ce454` adds a bounded environment-local input and two Forever queries. [Contract and inference limits](../../specs/gamepad-mapped-state.md) own requirements, unknown native fields/signatures and pending parent verification.

## Backing representation

`c_api::c_game_pad::MappedStickSnapshot` contains an ordered `Vec<MappedStick>`; each entry contains a configuration name and x/y model inputs. `SimState.gamepad_mapped_sticks` is optional, initially None. It is separate from logical input style and widget gamepad-enable flags. No physical device record or full mapped DTO exists.

The getter clones the optional input and drops the state borrow before Lua allocation. It roots the result on the stack, attaches the stick array, then attaches each child before field-key interning. Selector lookup clones the chosen name before Lua string allocation. Neither path unwraps secrets or changes caller taint.

## Consumer and evidence boundary

Actual cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua` eagerly computes both `inputBindingAxisListener` centering fields during TOC loading. `tests/gamepad_mapped_state.rs` installs concrete input before loading that unchanged TOC's dependency closure; it asserts these source-owned fields as well as query results. No vendor overrides or callback bypasses.

Existing `1fbea8e70` RED is four AutoRoll tests stopping before addon load on this cached namespace dependency, with 24 prior controls passing. It is not AutoRoll decision RED. The exact retained SharedXML Reveal diagnostic changes debug setup and is documented in the contract. New tests have not been compiled/run by this author; parent owns GREEN and independent verification against the approved pin.

## Sources

- [Mapped-stick contract](../../specs/gamepad-mapped-state.md) — requirements, proof artifact location, policies and exclusions.
- [Logical input style](../../specs/forever-input-interface-style.md) — separate current-style behavior.
- `src/c_api/c_game_pad.rs`, `tests/gamepad_mapped_state.rs` — committed implementation and unrun behavioral fixtures.

## See Also

- [[lua-api]] — runtime surface.
- [[forever-addon-comparison]] — separate addon workflow evidence.
