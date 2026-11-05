# EditMode Layout Fixes

## Problem

4 frame position regressions in the `frame_positions` test after commit 73e6032 (which reordered `__index` to check Lua fields before Rust methods, matching real WoW behavior).

| Frame | Expected | Actual | Root Cause |
|-------|----------|--------|------------|
| FocusFrame | (1190, 926) | (1320, 860) | `SetScaleOverride` adjusts anchor offsets |
| ObjectiveTrackerFrame | (1335, 271) | (1340, 0) | `SetPointOverride` misparses 3-arg `SetPoint` |
| UIParentRightManagedFrameContainer | (1335, 260) | (1235, 260) | `GetRightActionBarWidth` returns ~100px |
| PetFrame | (361, 967) | (740, 0) | Hidden frame (no pet), removed from test |

## Root Cause: EditMode Method Overrides

`EditModeSystemMixin:OnSystemLoad` (EditModeSystemTemplates.lua:3-30) replaces three core methods on all 43 registered system frames:

```lua
self.SetScaleBase = self.SetScale
self.SetScale = self.SetScaleOverride     -- adjusts anchor offsets on scale change
self.SetPointBase = self.SetPoint
self.SetPoint = self.SetPointOverride      -- expects 5 explicit args
self.ClearAllPointsBase = self.ClearAllPoints
self.ClearAllPoints = self.ClearAllPointsOverride
```

These overrides are stored in the frame's per-instance fenv table (accessed via `debug.getfenv(frame)[1]`). After commit 73e6032, `__index` checks this table BEFORE Rust methods, so all `:SetPoint()` / `:SetScale()` calls now hit the Lua overrides.

### SetScaleOverride Impact (FocusFrame)

When `SetSmallSize(true)` calls `:SetScale(0.75)`, `SetScaleOverride` adjusts existing anchor offsets: `newOffset = oldOffset * oldScale / newScale`. FocusFrame's offset changes from 520 to 693.3, shifting it from x=1190 to x=1320.

### SetPointOverride Impact (ObjectiveTrackerFrame)

`VerticalLayoutMixin:LayoutChildren` positions frames using the 3-arg form `child:SetPoint("TOPRIGHT", -offset, -topOffset)`. When this hits `SetPointOverride(point, relativeTo, relativePoint, offsetX, offsetY)`, the numbers are mapped as `relativeTo=-0, relativePoint=-11` instead of offsets. `offsetX` and `offsetY` default to nil → 0.

### GetRightActionBarWidth Impact (Container)

`EditModeUtil.GetRightActionBarWidth()` checks if MultiBarLeft/Right are visible+initialized+in default position. On master, at least one passes all checks, returning ~100px. This shifts the managed container from x=1335 to x=1235.

## Fix (workarounds_editmode.rs)

### `clear_edit_mode_overrides`

After `UpdateSystems` (which calls `OnSystemLoad` → sets up overrides), clear all three overrides from every system frame's fenv:

```lua
function clear_frame_overrides(frame)
    local env = debug.getfenv(frame)
    if not env or not env[1] then return end
    rawset(env[1], "SetPoint", nil)
    rawset(env[1], "SetScale", nil)
    rawset(env[1], "ClearAllPoints", nil)
end
```

Then re-apply preset anchors via the now-unoverridded Rust `SetPoint` for non-managed frames.

Note: `rawset(frame, key, nil)` does NOT work — FrameRef is a userdata, not a table. Must access fenv[1] via `debug.getfenv()`.

### `GetActionBarToggles` fix (replaces `fix_right_action_bar_width`)

The `GetActionBarToggles` stub returned `(1,1,1,1)` (all bars enabled), causing `MultiActionBar_Update()` to show MultiBarLeft/Right via `SetShown(true)`. This made `GetRightActionBarWidth()` return ~100px, shifting the managed container 100px left.

Fixed by returning `(false, false, false, false)` — the default for a character that hasn't enabled optional bars. This makes `MultiActionBar_Update()` call `SetShown(false)`, keeping the bars hidden. `GetRightActionBarWidth` naturally returns 0.

### `reposition_managed_frames`

Re-run `UpdateManagedFrames()` on both containers after all fixes. This triggers `VerticalLayoutMixin:Layout()` which correctly positions children now that `SetPointOverride` is cleared.

### `patch_update_systems` / `patch_init_anchors`

Skip `isManagedFrame` frames in both `InitSystemAnchors` and `UpdateSystems` — their position comes from the container layout, not preset anchors.

## Key Insight

`rawset`/`rawget` on FrameRef userdata fails with "table expected, got userdata". Per-frame fields must be accessed through `debug.getfenv(frame)[1]`, which is the table checked by the patched `__index` function in `metatable.rs`.

## Files Modified

- `src/lua_api/workarounds_editmode.rs` — all layout fixes
- `src/lua_api/globals/c_stubs_api.rs` — `GetActionBarToggles` returns false (was 1)
- `tests/frame_positions.rs` — removed PetFrame (hidden), updated count to 27
- `tests/action_bar.rs` — added `test_right_action_bars_hidden_by_default`
- `AGENTS.md` — documented rawset/rawget limitation
