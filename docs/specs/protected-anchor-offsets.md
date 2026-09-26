# Protected anchor offset mutation

`AdjustPointsOffset` and `SetPointsOffset` modify anchor offsets on script regions. They follow the existing protected-frame write policy; see [protected-frame enforcement](../protected-frame-enforcement.md).

## What it must do

- [x] Insecure combat calls on protected frames or frames anchored to protected state leave all offsets unchanged and emit `ADDON_ACTION_BLOCKED` naming the blocked method.
- [x] Insecure combat calls on plain frames preserve normal offset behavior without a blocked event.
- [x] Secure combat and insecure out-of-combat calls on protected frames preserve additive `AdjustPointsOffset` and absolute `SetPointsOffset` behavior without blocked events.

## How it works

- [Protected-frame enforcement](../protected-frame-enforcement.md)
- [Anchor resolution](../anchor-resolution.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs` — `AdjustPointsOffset` and existing anchor gate.
- `src/lua_api/frame/methods/misc/bounds.rs` — `SetPointsOffset`.
- `src/lua_api/frame/methods/methods_helpers.rs` — shared permission and event emission.

## Tests asserting this spec

- `tests/protected_frame_enforcement.rs` — offset denial, protected relation, plain-frame and allowed-caller controls.

## Known gaps (current cycle)

None for this bounded change.

## Out of scope

`ClearPointsOffset` reset semantics, changes to offset arithmetic, other APIs, and broader native-client conformance.
