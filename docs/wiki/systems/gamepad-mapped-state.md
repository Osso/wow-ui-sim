# Forever mapped-stick queries

`f774ce454` adds a bounded environment-local input and two Forever queries. [Contract and inference limits](../../specs/gamepad-mapped-state.md) own requirements, unknown native fields/signatures and incomplete acceptance.

## Backing representation

`c_api::c_game_pad::MappedStickSnapshot` contains an ordered `Vec<MappedStick>`; each entry contains a configuration name and x/y model inputs. `SimState.gamepad_mapped_sticks` is optional, initially None. It is separate from logical input style and widget gamepad-enable flags. No physical device record or full mapped DTO exists.

The getter clones the optional input and drops the state borrow before Lua allocation. It roots the result on the stack, attaches the stick array, then attaches each child before field-key interning. Selector lookup clones the chosen name before Lua string allocation. Neither path unwraps secrets or changes caller taint.

## Consumer and evidence boundary

Actual cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua` eagerly computes both `inputBindingAxisListener` centering fields during TOC loading. `tests/gamepad_mapped_state.rs` installs concrete input before loading that unchanged TOC's dependency closure; it asserts these source-owned fields as well as query results. No vendor overrides or callback bypasses.

Current `model-v2-ledger.json` and sibling stdout/stderr record build `5e15752` → run `74e6c8845` at rilua `6044544b`: 70 cases, 63 pass / 7 fail. Three direct-model tests pass (snapshot replacement/table independence, environment/input-style isolation, taint/opaque selectors). Three cached initializer cases and four AutoRoll cases fail before local addon/decision assertions at `C_GamePad.SetAllowHoverEventsWithFreeLook`, `FrameControlsManager.lua:830`. The earlier `1fbea8e70` namespace-gap RED is historical; cached initializer GREEN and AutoRoll decisions remain unproved.

The [comparison evidence](../investigations/forever-addon-comparison.md#mapped-stick-and-autoroll--current-pin-boundary) records 36 prior messaging/module/restricted/BugCapture passes and 24 loot/instance control passes on the current pin. Default fmt/check passed with unchanged source/config inputs in `/tmp/patch-12.0.5-batch11-rust-gates.json`; independent audit `20366` is pending. New hover-policy tests have implementation pending; no outcome is inferred. Native hover/controller and Reveal remain unmodeled. The retained SharedXML Reveal diagnostic changes debug setup, not native parity. No archive/matrix counter credit or overall completion follows.

## Sources

- [Mapped-stick contract](../../specs/gamepad-mapped-state.md) — requirements, proof artifact location, policies and exclusions.
- [Logical input style](../../specs/forever-input-interface-style.md) — separate current-style behavior.
- `src/c_api/c_game_pad.rs`, `tests/gamepad_mapped_state.rs` — committed implementation and behavioral fixtures; only the three direct mapped cases have GREEN in the cited run.

## See Also

- [[lua-api]] — runtime surface.
- [[forever-addon-comparison]] — separate addon workflow evidence.
