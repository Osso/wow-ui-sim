# Display metrics

The simulator distinguishes physical display pixels from its base layout canvas. `WowLuaEnv` owns these inputs in `src/lua_api/env_runtime.rs`; see [layout coordinates](../layout-system.md) and [[layout-system]] for frame resolution.

## What it must do

- [x] Preserve `set_screen_size(width, height)` as an explicit one-to-one canvas/output input, including existing screen API behavior.
- [x] Accept physical display dimensions through `set_display_size(width, height)`, retain them for `GetPhysicalScreenSize`, and derive the base UI canvas using Blizzard PixelUtil's 768-unit reference height.
- [x] Apply UIParent scale separately: physical 3440×1440 at scale 0.8 must report approximately 2293.333×960 through `GetScreenWidth`, `GetScreenHeight`, and `UIParent:GetRect`, while `GetEffectiveScale` remains 0.8.
- [x] Resize physical displays without losing UI scale, and reject nonpositive/nonfinite dimensions without mutating screen state.
- [x] Convert pixels to UI units using `pixels * 768 / (physicalHeight * frameScale)`, matching Blizzard's `PixelUtilSecure.lua`.

### Captured layout replay

- [x] Reproduce selected frames from the captured physical `3440×1440` display, effective scale `0.8`, and raw saved `Ultrawide` EditMode cache through normal Blizzard startup, without assigning measured outputs. Default-only layout repairs must not replace custom EditMode anchors or height.
- [x] Keep the post-event ObjectiveTracker repair default-only and remove duplicate headless height assignment, so saved custom anchors and height remain Blizzard-driven.

## How it works

- [Layout system](../layout-system.md)
- [Live frame-position investigation](../wiki/investigations/frame-position-baseline-drift.md) — causal replay diagnosis and parity limits

## Implementation inventory

- `src/lua_api/env_runtime.rs` — explicit canvas/display setters and screen globals.
- `src/lua_api/state/sim_state.rs` — literal `screen_width`/`screen_height` base-canvas fields and separate `physical_screen_width`/`physical_screen_height` pixel fields.
- `src/lua_api/state.rs` — initial one-to-one dimensions.
- `src/lua_api/workarounds/temporary/post_event_frame_layout.rs` — default-only ObjectiveTracker repair, leaving custom layout state to Blizzard.
- `src/startup.rs` — startup orchestration without a duplicate unconditional tracker-height override.

## Evidence

- `tests/screen_mode.rs` — existing canvas contracts, captured physical/UI metrics, resize, and invalid-input behavior; 9/9 passed for commit `b8d098059`.
- `tests/frame_position_replay.rs` — one captured-configuration replay passed for commit `1e79fca4f`, using the existing one-UI-unit tolerance. This is selected-frame causal replay, not bitwise-exact UI parity: the empty right managed container is width `0` in simulation versus about `1` live.

## Known gaps (current cycle)

- [ ] Independently audit and reconcile the two legacy `frame_positions` expectations using replay evidence.
- [ ] Establish source/build parity or explain differences between cache `12.1.0.69497` and live capture `12.1.0.69587`.
- [ ] Do not infer whole-UI or 78-addon parity from this selected-frame replay.

## Out of scope

- Changing existing GUI/headless canvas callers in this input-model slice.
- Inferring operating-system DPI or changing `GetScreenDPIScale` without evidence.
- Assigning measured frame rectangles or anchors as simulator setup values.
