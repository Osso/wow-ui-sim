# Display metrics

The simulator distinguishes physical display pixels from its base layout canvas. `WowLuaEnv` owns these inputs in `src/lua_api/env_runtime.rs`; see [layout coordinates](../layout-system.md) and [[layout-system]] for frame resolution.

## What it must do

- [x] Preserve `set_screen_size(width, height)` as an explicit one-to-one canvas/output input, including existing screen API behavior.
- [x] Accept physical display dimensions through `set_display_size(width, height)`, retain them for `GetPhysicalScreenSize`, and derive the base UI canvas using Blizzard PixelUtil's 768-unit reference height.
- [x] Apply UIParent scale separately: physical 3440×1440 at scale 0.8 must report approximately 2293.333×960 through `GetScreenWidth`, `GetScreenHeight`, and `UIParent:GetRect`, while `GetEffectiveScale` remains 0.8.
- [x] Resize physical displays without losing UI scale, and reject nonpositive/nonfinite dimensions without mutating screen state.
- [x] Convert pixels to UI units using `pixels * 768 / (physicalHeight * frameScale)`, matching Blizzard's `PixelUtilSecure.lua`.

### Captured layout replay

- [x] Reproduce the captured Ultrawide configuration's selected frame anchors/rectangles from physical size, scale, and EditMode cache inputs, without assigning measured outputs. Default-only layout repairs must not replace custom EditMode anchors or height.

## How it works

- [Layout system](../layout-system.md)
- [Live frame-position investigation](../wiki/investigations/frame-position-baseline-drift.md)

## Implementation inventory

- `src/lua_api/env_runtime.rs` — explicit canvas/display setters and screen globals.
- `src/lua_api/state/sim_state.rs` — literal `screen_width`/`screen_height` base-canvas fields and separate `physical_screen_width`/`physical_screen_height` pixel fields.
- `src/lua_api/state.rs` — initial one-to-one dimensions.
- `src/lua_api/workarounds/temporary/post_event_frame_layout.rs` — default-only ObjectiveTracker repair, leaving custom layout state to Blizzard.
- `src/startup.rs` — startup orchestration without a duplicate unconditional tracker-height override.

## Tests asserting this spec

- `tests/screen_mode.rs` — existing canvas contracts, captured physical/UI metrics, resize, and invalid-input behavior.
- `tests/frame_position_replay.rs` — captured configuration replay through real Blizzard startup.

## Known gaps (current cycle)

- [ ] Independently verify replay and reconcile the two legacy frame-position expectations using that evidence.

## Out of scope

- Changing existing GUI/headless canvas callers in this input-model slice.
- Inferring operating-system DPI or changing `GetScreenDPIScale` without evidence.
- Assigning measured frame rectangles or anchors as simulator setup values.
