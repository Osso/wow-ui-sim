# Event System & Script Dispatch

## Overview

The event system connects WoW-style game events (PLAYER_LOGIN, ADDON_LOADED, etc.) and UI interactions (mouse clicks, key presses, visibility changes) to Lua script handlers running inside addon code. Events flow from Rust to Lua through two mechanisms:

1. **Game events** -- Named string events (e.g., "PLAYER_LOGIN") dispatched to all frames that called `RegisterEvent()`. Routed through the frame's `OnEvent` handler.
2. **Script handlers** -- Direct per-frame callbacks (OnClick, OnShow, OnUpdate, etc.) fired by specific Rust-side triggers like mouse input or visibility changes.

Both mechanisms store handler functions in a shared `__scripts` Lua global table and look up frame references via `__frame_{id}` globals.

## Event Types and the Event Queue

**File:** `src/event/mod.rs:1-71`

### Event Struct

```rust
pub struct Event {
    pub name: String,           // e.g., "PLAYER_LOGIN"
    pub args: Vec<EventArg>,    // Optional payload
}

pub enum EventArg {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}
```

### EventQueue

A simple `Vec<Event>` with push/drain semantics (line 28-71). Used by `SimState` to buffer events before dispatch, though in practice most events are fired directly via `fire_event()` / `fire_event_with_args()` rather than queued.

```rust
pub struct EventQueue {
    pending: Vec<Event>,
}
```

Methods: `push()`, `push_simple()`, `drain()`, `is_empty()`.

### Predefined Event Constants

**File:** `src/event/mod.rs:6-24`

The `events` module defines constants for common WoW events:

| Constant | Event Name |
|----------|------------|
| `PLAYER_LOGIN` | Player login complete |
| `PLAYER_LOGOUT` | Player logging out |
| `PLAYER_ENTERING_WORLD` | Zone transition or login |
| `ADDON_LOADED` | An addon finished loading (arg: addon name) |
| `VARIABLES_LOADED` | SavedVariables loaded |
| `UPDATE_BINDINGS` | Key bindings changed |
| `DISPLAY_SIZE_CHANGED` | Window resized |
| `UI_SCALE_CHANGED` | UI scale changed |
| `PLAYER_TARGET_CHANGED` | Target changed |
| `UNIT_HEALTH` | Unit health updated |
| `UNIT_POWER_UPDATE` | Unit power (mana/energy) updated |
| `COMBAT_LOG_EVENT` | Combat log entry |
| `CHAT_MSG_CHANNEL` | Channel chat message |
| `CHAT_MSG_SAY` | Say chat message |
| `CHAT_MSG_WHISPER` | Whisper message |
| `BAG_UPDATE` | Inventory changed |
| `UPDATE_MOUSEOVER_UNIT` | Mouseover target changed |

These are string constants only -- the system accepts any arbitrary event name string, not just those listed here.

## Script Handler Types

**File:** `src/event/mod.rs:73-198`

The `ScriptHandler` enum defines all recognized handler types. Each frame can have at most one active handler per type (set via `SetScript`).

### Full List

| Handler | Trigger | Arguments (after self) |
|---------|---------|----------------------|
| `OnEvent` | Game event fires for a registered event | `event, ...args` |
| `OnUpdate` | Every render tick (visible frames only) | `elapsed` (seconds) |
| `OnPostUpdate` | After all OnUpdate handlers fire | `elapsed` (seconds) |
| `OnShow` | Frame becomes visible (Show() or creation) | none |
| `OnHide` | Frame becomes hidden (Hide()) | none |
| `OnPostShow` | After OnShow completes | none |
| `OnPostHide` | After OnHide completes | none |
| `OnClick` | Mouse down + up on same frame | `button, down` |
| `OnPostClick` | After OnClick completes | none |
| `OnEnter` | Mouse cursor enters frame bounds | none |
| `OnLeave` | Mouse cursor leaves frame bounds | none |
| `OnMouseDown` | Mouse button pressed on frame | `button` |
| `OnMouseUp` | Mouse button released on frame | `button` |
| `OnMouseWheel` | Scroll wheel on frame (propagates up) | `delta` |
| `OnDragStart` | Drag begins | none |
| `OnDragStop` | Drag ends | none |
| `OnReceiveDrag` | Frame receives a dragged item | none |
| `OnSizeChanged` | Frame dimensions change | none |
| `OnLoad` | Frame creation from XML completes | none |
| `OnAttributeChanged` | SetAttribute() called | `name, value` |
| `OnKeyDown` | Key pressed (propagates via keyboard input) | `key` |
| `OnKeyUp` | Key released | `key` |
| `OnChar` | Character input | `char` |
| `OnEnterPressed` | Enter key in EditBox | none |
| `OnEscapePressed` | Escape key in EditBox | none |
| `OnTabPressed` | Tab key in EditBox | none |
| `OnSpacePressed` | Space key in EditBox | none |
| `OnEditFocusGained` | EditBox gains focus | none |
| `OnEditFocusLost` | EditBox loses focus | none |
| `OnTextChanged` | EditBox text changes | none |
| `OnValueChanged` | Slider value changes | `value` |
| `OnMinMaxChanged` | Slider min/max changes | none |
| `OnTooltipCleared` | Tooltip content cleared | none |
| `OnTooltipSetItem` | Tooltip set to item | none |
| `OnTooltipSetUnit` | Tooltip set to unit | none |
| `OnTooltipSetSpell` | Tooltip set to spell | none |

Parsing from string is done via `ScriptHandler::from_str()` (line 116-156), which returns `None` for unrecognized handler names. This is a custom implementation, not the `FromStr` trait.

## Script Handler Registration and Storage

### The `__scripts` Global Table

**File:** `src/lua_api/frame/methods/methods_script.rs:16-22`

All script handler functions are stored in a single Lua global table named `__scripts`. The table is lazily created on first use:

```rust
fn get_or_create_scripts_table(lua: &mlua::Lua) -> mlua::Table {
    lua.globals().get("__scripts").unwrap_or_else(|_| {
        let t = lua.create_table().unwrap();
        lua.globals().set("__scripts", t.clone()).unwrap();
        t
    })
}
```

Keys are formatted as `"{widget_id}_{handler_name}"` -- for example, a frame with ID 42 and an OnClick handler is stored at key `"42_OnClick"`. Values are Lua function references.

### The `__frame_{id}` References

**File:** `src/lua_api/globals/create_frame.rs:159-179`

When a frame is created (via `CreateFrame` or XML loading), a global reference `__frame_{id}` is set pointing to the `FrameHandle` userdata:

```rust
let frame_key = format!("__frame_{}", frame_id);
lua.globals().set(frame_key.as_str(), ud.clone())?;
```

This reference is used by the event dispatch code to pass `self` as the first argument to handlers. Named frames are also set as globals under their name (e.g., `lua.globals().set("UIParent", ud)`).

### SetScript

**File:** `src/lua_api/frame/methods/methods_script.rs:25-58`

`frame:SetScript("OnClick", function)` does three things:

1. Stores the function in `__scripts["{id}_OnClick"]`
2. Records the handler in the Rust-side `ScriptRegistry` (`state.scripts.set(id, handler, 1)`)
3. For OnUpdate/OnPostUpdate: adds the frame ID to `state.on_update_frames` (a `HashSet<u64>`)

Passing `nil` as the function removes the handler from both `__scripts` and `ScriptRegistry`, and removes the frame from `on_update_frames` if applicable.

### SetOnClickHandler

**File:** `src/lua_api/frame/methods/methods_script.rs:60-73`

WoW 10.0+ convenience method. Equivalent to `SetScript("OnClick", func)`.

### GetScript

**File:** `src/lua_api/frame/methods/methods_script.rs:77-89`

Returns the handler function from `__scripts["{id}_{handler}"]`, or `nil` if none is set.

### HasScript

**File:** `src/lua_api/frame/methods/methods_script.rs:182-230`

Returns `true` if the handler name is in a hardcoded list of common script types. This checks whether the frame *supports* the handler type, not whether one is currently set. The check is case-insensitive.

### HookScript

**File:** `src/lua_api/frame/methods/methods_script.rs:92-116`

`frame:HookScript("OnClick", func)` appends to a separate `__script_hooks` global table. Hooks are stored as arrays keyed by `"{id}_{handler}"`:

```
__script_hooks["42_OnClick"] = { func1, func2, ... }
```

Note: The current implementation stores hooks but does not automatically invoke them during dispatch -- hooks are appended to the table for addon compatibility, but the dispatch code only looks at `__scripts`.

### ClearScripts

**File:** `src/lua_api/frame/methods/methods_script.rs:131-179`

Removes all entries from both `__scripts` and `__script_hooks` that start with `"{id}_"`, clears the frame from `ScriptRegistry`, and removes it from `on_update_frames`.

### ScriptRegistry (Rust Side)

**File:** `src/event/mod.rs:200-235`

A parallel Rust-side registry tracking which widgets have which handlers. Structure: `HashMap<u64, HashMap<ScriptHandler, i32>>`. The `i32` value is always `1` (a placeholder -- the actual function is in the Lua `__scripts` table).

This exists so Rust code can check handler existence without touching Lua. Methods: `set()`, `get()`, `remove()`, `remove_all()`.

## Event Registration on Frames

**File:** `src/lua_api/frame/methods/methods_event.rs:1-66`

Frames must explicitly register for game events to receive them via OnEvent.

### RegisterEvent / UnregisterEvent

Adds/removes event names from `frame.registered_events` (`HashSet<String>` on the Rust `Frame` struct, line 126 of `src/widget/frame.rs`).

### RegisterAllEvents

Sets `frame.register_all_events = true` (line 243 of `src/widget/frame.rs`), which causes `is_registered_for_event()` to return true for all event names.

### IsEventRegistered

Returns `frame.register_all_events || frame.registered_events.contains(event)`.

### UnregisterAllEvents

Clears `frame.registered_events` (does not reset `register_all_events`).

### RegisterUnitEvent

Accepts an event name plus variadic args (unit IDs in WoW, ignored here). Delegates to `register_event()`.

## Event Dispatch

### Finding Listeners

**File:** `src/widget/registry.rs:63-70`

When an event fires, listeners are found by scanning all widgets:

```rust
pub fn get_event_listeners(&self, event: &str) -> Vec<u64> {
    self.widgets
        .values()
        .filter(|w| w.is_registered_for_event(event))
        .map(|w| w.id)
        .collect()
}
```

This is a linear scan of all registered widgets. No index exists for event-to-frame mapping.

### fire_event / fire_event_with_args

**File:** `src/lua_api/env.rs:112-152`

The primary dispatch path from Rust to Lua:

1. Borrow `SimState`, call `get_event_listeners(event)` to get a `Vec<u64>` of widget IDs
2. Release the state borrow
3. For each widget ID:
   a. Look up `__scripts["{id}_OnEvent"]` in Lua globals
   b. Look up `__frame_{id}` to get the frame userdata
   c. Call the handler with arguments `(self, event_name, ...args)`
   d. On error: log to stderr with frame name and event name, continue to next frame

The call signature matches WoW's convention: `handler(self, event, arg1, arg2, ...)`.

### fire_event_collecting_errors

**File:** `src/lua_api/env.rs:155-192`

Same dispatch logic as `fire_event_with_args`, but collects error strings into a `Vec<String>` instead of printing to stderr. Used during startup to aggregate errors for test assertions.

### fire_script_handler

**File:** `src/lua_api/env.rs:197-223`

Fires a specific handler type (not OnEvent) on a specific widget:

1. Look up `__scripts["{id}_{handler_name}"]`
2. Look up `__frame_{id}`
3. Call with `(self, ...extra_args)`

Used for OnClick, OnEnter, OnLeave, OnMouseDown, OnMouseUp, OnMouseWheel, OnKeyDown, etc.

### FireEvent (Lua Global)

**File:** `src/lua_api/globals/system_api.rs:156-195`

A simulator utility function exposed to Lua as `FireEvent(event, ...)`. Performs the same dispatch as `fire_event_with_args` but from within Lua code. Useful for testing.

### has_script_handler

**File:** `src/lua_api/env.rs:226-237`

Checks if `__scripts["{id}_{handler_name}"]` contains a function. Used for OnMouseWheel propagation to determine whether to stop walking up the parent chain.

## Error Handling During Script Dispatch

### OnEvent Errors

Errors in OnEvent handlers are caught per-frame and logged. The frame name (or "(anonymous)") and event name are included in the error message. Dispatch continues to subsequent frames -- one frame's error does not prevent other frames from receiving the event.

### OnUpdate Error Suppression

**File:** `src/lua_api/env.rs:26-27, 533-551`

`WowLuaEnv` maintains an `on_update_errors: RefCell<HashSet<u64>>` set. When an OnUpdate handler errors, its frame ID is added to this set. On subsequent ticks, frames in this set are skipped entirely. This prevents a single broken OnUpdate handler from flooding stderr with repeated stack traces every frame.

OnPostUpdate handlers do NOT have this suppression -- they log errors but continue firing on subsequent ticks (line 557-579).

### Lifecycle Script Errors (OnLoad, OnShow)

**File:** `src/loader/xml_frame.rs:582-631`

OnLoad and OnShow handlers fired during XML frame creation are wrapped in `pcall()`. Errors are caught and printed via `print()` (which goes to `console_output`), matching WoW's C++ engine behavior where script errors during frame creation are displayed but not propagated.

### fire_handler_returns_truthy

**File:** `src/lua_api/key_dispatch.rs:104-118`

Used for EditBox special key handlers (OnEscapePressed, OnEnterPressed, etc.). Fires the handler and checks if the return value is truthy (not nil and not false). If truthy, the key event is consumed and not propagated further.

## Input Event Flow (Rust to Lua)

### Mouse Events

**File:** `src/iced_app/update.rs:55-158`

The iced app translates canvas messages into script handler calls:

**MouseMove** (line 65-87):
1. Hit-test to find the frame under the cursor
2. If hovered frame changed: fire `OnLeave` on old frame, `OnEnter` on new frame
3. Update `self.hovered_frame`

**MouseDown** (line 90-113):
1. Hit-test to find frame under cursor
2. Check if frame is enabled (via `__enabled` attribute)
3. Fire `OnMouseDown` with `"LeftButton"` argument
4. Record `mouse_down_frame` and `pressed_frame`

**MouseUp** (line 115-149):
1. Hit-test to find frame under cursor
2. If released on same frame as mouse-down:
   - Toggle CheckButton state if applicable
   - Fire `OnClick` with `("LeftButton", false)` arguments
3. Fire `OnMouseUp` with `"LeftButton"` argument
4. Clear `mouse_down_frame` and `pressed_frame`

**MouseWheel** (line 160-205):
1. Hit-test to find frame under cursor
2. Walk up the parent chain looking for a frame with an OnMouseWheel handler
3. Fire the handler on the first frame that has one (with delta value)
4. If no handler found, fall back to scroll offset adjustment

### Key Events

**File:** `src/lua_api/key_dispatch.rs:1-174`

Key dispatch follows WoW's priority chain:

**Escape** (line 24-34):
1. If an EditBox is focused and has OnEscapePressed that returns truthy: consumed
2. `CloseSpecialWindows()`: iterate `UISpecialFrames` table, hide visible ones
3. Toggle `GameMenuFrame` visibility (show fires `fire_on_show_recursive`)

**Other Keys** (line 37-52):
1. If an EditBox is focused: check for special key handlers (ENTER -> OnEnterPressed, TAB -> OnTabPressed, SPACE -> OnSpacePressed). If handler returns truthy: consumed
2. Fire `OnKeyDown` on focused or keyboard-enabled frame

**OnKeyDown Propagation** (line 55-101):
1. Find the starting frame: focused frame, or first frame with `keyboard_enabled && visible`
2. Fire OnKeyDown with the key name
3. If `frame.propagate_keyboard_input` is true, walk up to parent and repeat

## Lifecycle Script Dispatch

### OnLoad

**File:** `src/loader/xml_frame.rs:585-608` and `src/lua_api/globals/template/mod.rs:306-330`

Fired at the end of frame creation from XML or template application. The handler lookup checks two sources:

1. `frame:GetScript("OnLoad")` -- handler set via `SetScript` (from XML `<Scripts><OnLoad>`)
2. `frame.OnLoad` -- function property set by a Mixin (e.g., `ButtonStateBehaviorMixin.OnLoad`)

Both are wrapped in `pcall()` to match WoW behavior.

### OnShow

**File:** `src/lua_api/frame/methods/methods_core.rs:264-289` and `src/loader/xml_frame.rs:610-631`

Fired when a frame transitions from hidden to visible. The `Show()` method checks `was_hidden` before firing:

```rust
methods.add_method("Show", |lua, this, ()| {
    let was_hidden = /* check frame.visible == false */;
    /* set frame.visible = true */
    if was_hidden {
        fire_on_show_recursive(lua, &this.state, this.id)?;
    }
    Ok(())
});
```

### fire_on_show_recursive

**File:** `src/lua_api/frame/methods/methods_core.rs:223-262`

Fires OnShow on a frame, then recursively on all visible children. This ensures child frames that were already visible get their OnShow handlers called when a parent becomes visible.

1. Look up `__scripts["{id}_OnShow"]`
2. Call with `(self)` if found
3. Collect visible children IDs
4. Recurse on each child

### OnHide

Currently fires only the `Hide()` method which sets `frame.visible = false`. The OnHide handler dispatch is simpler than OnShow -- no recursive propagation to children.

## OnUpdate Tick Mechanism

**File:** `src/lua_api/env.rs:517-585`

### Per-Frame OnUpdate

OnUpdate fires every render tick for frames that:
1. Have an OnUpdate or OnPostUpdate handler registered (tracked in `state.on_update_frames: HashSet<u64>`)
2. Are currently visible (`frame.visible == true`)

The tick sequence:

1. Collect frame IDs from `on_update_frames`, filtering to visible-only
2. For each frame, look up `__scripts["{id}_OnUpdate"]` and call with `(self, elapsed)`
3. After ALL OnUpdate handlers: fire OnPostUpdate handlers for the same frames
4. Tick animation groups via `tick_animation_groups()`

### Tick Timing

**File:** `src/iced_app/update.rs:297-311, 332-340`

The iced app drives the tick via `handle_process_timers()`:

1. Update FPS counter
2. Run any pending `--exec-lua` code
3. Clear the render dirty flag
4. Process C_Timer callbacks (`process_timers()`)
5. Fire OnUpdate with elapsed time since last tick
6. Check if widgets changed and invalidate the render cache

The elapsed time is computed from `Instant::now() - last_on_update_time` and passed as seconds (f64).

### on_update_frames Set

**File:** `src/lua_api/state.rs:74`

A `HashSet<u64>` in `SimState` tracking which frames have active OnUpdate/OnPostUpdate handlers. Modified by:

- `SetScript("OnUpdate", func)` -- inserts ID (line 39 of methods_script.rs)
- `SetScript("OnUpdate", nil)` -- removes ID (line 53 of methods_script.rs)
- `ClearScripts()` -- removes ID (line 175 of methods_script.rs)

This avoids scanning all widgets every tick to find OnUpdate handlers.

## Startup Event Sequence

**File:** `src/iced_app/app.rs:53-105` (GUI mode) and `src/main.rs:434-478` (headless mode)

After all addons are loaded, the simulator fires startup events in this order:

1. `ADDON_LOADED` with arg `"WoWUISim"` -- signals all addons are loaded
2. `VARIABLES_LOADED` -- SavedVariables are available
3. `PLAYER_LOGIN` -- player data available
4. `EDIT_MODE_LAYOUTS_UPDATED` -- action bar layout info (if EditMode addon loaded)
5. `TIME_PLAYED_MSG` -- via `RequestTimePlayed()` global
6. `PLAYER_ENTERING_WORLD` with args `(true, false)` for initial login
7. `UPDATE_BINDINGS` -- key bindings ready
8. `DISPLAY_SIZE_CHANGED` -- screen dimensions set
9. `UI_SCALE_CHANGED` -- UI scale applied

Individual addons also receive per-addon `ADDON_LOADED` events during the loading phase, fired by `fire_addon_loaded()` in `src/lua_api/globals/addon_api.rs:550-558`.

## XML Script Handler Setup

**File:** `src/loader/helpers.rs:384-451`

When XML frames define script handlers via `<Scripts>`, the loader generates Lua code that calls `SetScript()`:

```xml
<Scripts>
    <OnLoad method="OnLoad"/>
    <OnEvent function="MyAddon_OnEvent"/>
    <OnClick>self:DoSomething()</OnClick>
</Scripts>
```

Three forms are supported:

1. **`method="X"`** -- generates `function(self, ...) self:X(...) end`
2. **`function="X"`** -- uses `X` directly as the handler function reference
3. **Inline body** -- generates `function(self, ...) <body> end`

### Script Inheritance (prepend/append)

**File:** `src/loader/helpers.rs:393-437`

Templates can specify `inherit="prepend"` or `inherit="append"` on script handlers:

- **prepend**: New handler runs first, then existing handler
- **append**: Existing handler runs first, then new handler

Both are wrapped in `pcall()` so errors in one don't prevent the other from running. The chained handler replaces the old one via `SetScript()`.

## Architecture Diagram

```
                    ┌─────────────────┐
                    │  iced App       │
                    │  (update.rs)    │
                    └───────┬─────────┘
                            │ mouse/key events
                            ▼
                    ┌─────────────────┐
                    │  WowLuaEnv      │
                    │  (env.rs)       │
                    ├─────────────────┤
                    │ fire_event()    │──── game events ────┐
                    │ fire_script_    │                     │
                    │   handler()     │── direct handlers ──┤
                    │ fire_on_update()│── per-tick ─────────┤
                    └───────┬─────────┘                     │
                            │                               │
                    ┌───────▼─────────┐             ┌──────▼──────────┐
                    │ SimState        │             │ Lua globals     │
                    │ (state.rs)      │             │                 │
                    ├─────────────────┤             │ __scripts = {   │
                    │ widgets:        │             │   "42_OnEvent"  │
                    │   WidgetRegistry│             │     = function  │
                    │ on_update_frames│             │   "42_OnClick"  │
                    │   : HashSet<u64>│             │     = function  │
                    │ scripts:        │             │ }               │
                    │   ScriptRegistry│             │                 │
                    │ events:         │             │ __frame_42 =    │
                    │   EventQueue    │             │   FrameHandle   │
                    └─────────────────┘             └─────────────────┘
```

### Key Data Flow

1. **Event fires** (Rust side): `fire_event("PLAYER_LOGIN")`
2. **Find listeners**: `widgets.get_event_listeners("PLAYER_LOGIN")` scans all frames for matching `registered_events`
3. **Look up handler**: `__scripts["42_OnEvent"]` in Lua
4. **Look up frame**: `__frame_42` in Lua globals
5. **Call handler**: `handler(frame, "PLAYER_LOGIN", ...args)`
6. **Error handling**: catch error, log with frame name, continue to next listener
