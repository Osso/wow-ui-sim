# OnUpdate Handlers and render_dirty

## Problem

`handle_process_timers()` in `update.rs:363-369` blanket-discards `render_dirty` after firing OnUpdate handlers:

```rust
self.fire_on_update();
self.env.borrow().state().borrow().widgets.take_render_dirty(); // discard
```

This was added because some handlers call `get_mut()` every tick without actually changing visual state (e.g., SetText with the same value). But it also suppresses legitimate visual changes — notably the cast bar's `SetValue()` calls, which update `statusbar_value` but never trigger a quad rebuild.

## Root Cause

`WidgetRegistry::get_mut()` unconditionally sets `render_dirty = true` (registry.rs:48). Any Lua method that calls `get_mut()` marks dirty even when writing the same value. The blanket discard was a workaround for this.

## Measured OnUpdate Handlers (37 visible at startup)

Data from instrumented dump-tree run with per-handler dirty tracking. `*` = set render_dirty.

### Noisy (set dirty without meaningful visual change)

| Frame | Type | Lua Source | What triggers dirty | Why noisy |
|-------|------|------------|---------------------|-----------|
| ActionBarButtonUpdateFrame | Frame | ActionButton.lua:354 | Iterates registered buttons, calls `UpdateState()` → `SetChecked()` on each | `stateDirty`/`flashDirty` cleared after first tick; subsequent ticks call `CheckNeedsUpdate()` which unregisters idle buttons. Noisy only on first few ticks after state change. |
| MainMenuMicroButton | Button | MainMenuBarMicroButtons.lua:1795 | Calls `SetNormalAtlas`, `SetPushedAtlas`, `SetDisabledAtlas`, `SetHighlightAtlas` every 1s | Status is always 0 (no streaming in sim). Atlas calls go through `get_mut()` even when setting identical values. |
| QueueStatusButton | Button | QueueStatusFrame.lua:162 | `self.Eye.texture:Show()` or `:Hide()` every tick | In static mode, calls `Show()` every tick on an already-shown texture. Show/Hide use `get_mut()` unconditionally. |
| UIParent | Frame | UIParent.lua:208 | Calls `FCF_OnUpdate` → fade logic, `ButtonPulse_OnUpdate`, `AnimatedShine_OnUpdate` | `FCF_OnUpdate` evaluates cursor position and may call `FCF_FadeOutChatFrame` → `UIFrameFadeOut` → `SetAlpha`. Pulse/Shine tables are usually empty at idle but the fade path triggers on first few ticks. |
| __anon_22331LeaveInstanceGroupButton | Button | (inherited OnUpdate) | Button update logic | Sets button textures/state even when unchanged. |
| Various StatusBars (__anon_20031, __anon_20006, etc.) | StatusBar | UnitFrame.lua:821 (health), UnitFrame.lua:952 (mana) | `UnitFrameHealthBar_OnUpdate` / `UnitFrameManaBar_OnUpdate` | Health bar is properly guarded (`currValue ~= self.currValue`), but `AnimatedLossBar:UpdateLossAnimation()` may still call `get_mut()`. Mana bar calls `SetValue` each tick. |
| PetFrameManaBar | StatusBar | UnitFrame.lua:952 | `UnitFrameManaBar_OnUpdate` → `SetValue` | Calls `SetValue` every tick even when value unchanged. |

### Legitimate (should trigger redraws)

| Frame | Type | Lua Source | What triggers dirty | Why legitimate |
|-------|------|------------|---------------------|----------------|
| PlayerCastingBarFrame | StatusBar | CastingBarFrame.lua:501 | `SetValue(self.value)` and `UpdateCastTimeText()` → `SetText()` every tick | Bar fill genuinely changes every frame during a cast. Text changes every ~frame. Both are real visual updates. |
| PlayerFrame | Button | PlayerFrame.lua:195 | `SetAlpha()` on StatusTexture (combat pulse) | Alpha oscillates smoothly between 0.22 and 1.0 on a 0.5s cycle. Every frame produces a different value. Only fires when StatusTexture is shown (combat). |
| Action button flash textures | Button | ActionButton.lua:1233-1253 | `Show()`/`Hide()` toggling flash texture | Flash animation toggles visibility at `ATTACK_BUTTON_FLASH_TIME` intervals. Each toggle is a real visual change. |

### Inert (no dirty, no visual work)

| Frame | Type | Notes |
|-------|------|-------|
| ChatFrame1 | MessageFrame | No mutation in OnUpdate handler at idle |
| ChatFrame1EditBox | EditBox | Cursor blink handled by iced, not Lua |
| WorldFrame | Frame | OnUpdate is a no-op in sim |
| GameTimeFrame | Button | Only updates when time changes or tooltip is shown |
| ModelScene frames (6x) | ModelScene | No-op OnUpdate |
| PartyMemberFrame buttons (4x) | Button | Guarded by combat/status checks, inert at idle |
| PetFrame | Button | Combat feedback update, inert at idle |
| DispatcherFrame, id:22438, etc. | Frame | No-op or empty handlers |

## Fix Strategy

The blanket discard exists because `get_mut()` is too coarse — it marks dirty for any mutable access, not just actual visual changes. Two approaches:

### Option A: Same-value guards in Rust methods

Make `SetValue`, `SetText`, `SetAlpha`, `Show`, `Hide`, `SetChecked`, `SetNormalAtlas` etc. skip `get_mut()` when the new value equals the current value. Read with `get()` first, only call `get_mut()` if changed.

**Pros**: Fixes the root cause. Noisy handlers stop producing false dirty flags. Blanket discard can be removed entirely.
**Cons**: Requires touching many Lua API methods. Must be careful with methods that have side effects beyond the value change.

### Option B: Track dirty per-frame instead of globally

Replace the single `render_dirty: bool` with a set of dirty frame IDs. OnUpdate discard can then selectively preserve dirty from frames with known-legitimate changes (StatusBars during active casts, frames with active animations).

**Pros**: Surgical. Doesn't require changing every setter method.
**Cons**: More complex tracking logic. Still doesn't fix the underlying `get_mut()` problem.

### Option C: Dedicated "visual dirty" flag on StatusBar

After `fire_on_update()`, check if any StatusBar's `statusbar_value` actually changed from its pre-tick value. If so, set `quads_dirty`. This is minimal and targets the cast bar specifically.

**Pros**: Smallest change. Directly fixes the cast bar lag.
**Cons**: Doesn't fix the general problem. Each new legitimate OnUpdate visual change needs another special case.
