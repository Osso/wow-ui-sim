# Display metrics

The simulator distinguishes physical display pixels from its base layout canvas. `WowLuaEnv` owns these inputs in `src/lua_api/env_runtime.rs`; see [layout coordinates](../layout-system.md) for frame resolution.

## What it must do

- [x] Preserve `set_screen_size(width, height)` as an explicit one-to-one canvas/output input, including existing screen API behavior.
- [x] Accept physical display dimensions through `set_display_size(width, height)`, retain them for `GetPhysicalScreenSize`, and derive the base UI canvas using Blizzard PixelUtil's 768-unit reference height.
- [x] Apply UIParent scale separately: physical 3440×1440 at scale 0.8 must report approximately 2293.333×960 through `GetScreenWidth`, `GetScreenHeight`, and `UIParent:GetRect`, while `GetEffectiveScale` remains 0.8.
- [x] Resize physical displays without losing UI scale, and reject nonpositive/nonfinite dimensions without mutating screen state.
- [x] Convert pixels to UI units using `pixels * 768 / (physicalHeight * frameScale)`, matching Blizzard's `PixelUtilSecure.lua`.

## How it works

- [Layout system](../layout-system.md)
- [Live frame-position investigation](../wiki/investigations/frame-position-baseline-drift.md)

## Implementation inventory

- `src/lua_api/env_runtime.rs` — explicit canvas/display setters and screen globals.
- `src/lua_api/state/sim_state.rs` — separate physical dimensions and base canvas dimensions.
- `src/lua_api/state.rs` — initial one-to-one dimensions.

## Tests asserting this spec

- `tests/screen_mode.rs` — existing canvas contracts, captured physical/UI metrics, resize, and invalid-input behavior.

## Known gaps (current cycle)

- [ ] Complete captured EditMode replay and compare resulting frame values independently of display metadata.

## Out of scope

- Changing existing GUI/headless canvas callers in this input-model slice.
- Inferring operating-system DPI or changing `GetScreenDPIScale` without evidence.
- Assigning measured frame rectangles or anchors as simulator setup values.
