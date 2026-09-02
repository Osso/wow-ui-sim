# Event System

The event system connects WoW-style game events and UI interactions to Lua script handlers. Events flow from Rust through two mechanisms: named game events dispatched to registered frames via OnEvent, and direct per-frame script handlers fired by specific Rust triggers.

## Event Queue and Types

`EventQueue` is a `Vec<Event>` with push/drain semantics. Most events are fired directly via `fire_event()` rather than queued.

```rust
pub struct Event { pub name: String, pub args: Vec<EventArg> }
pub enum EventArg { String(String), Number(f64), Boolean(bool), Nil }
```

Predefined constants include PLAYER_LOGIN, PLAYER_ENTERING_WORLD, ADDON_LOADED, VARIABLES_LOADED, UPDATE_BINDINGS, DISPLAY_SIZE_CHANGED, UNIT_HEALTH, UNIT_POWER_UPDATE, COMBAT_LOG_EVENT, BAG_UPDATE. Retail/PTR `RegisterEvent()` validates names against generated and epoch-specific strict tables; retail 12.1 includes `EXTERNAL_EVENT_LAUNCH_URL_FAILED` for current `Blizzard_GameMenu` loading. Classic profiles accept any non-empty name. Registerability does not model an event producer, payload, or `C_ExternalEventURL` behavior.

## Script Handler Types (36+, `src/event/mod.rs`)

Per-frame, per-type (at most one active handler): OnEvent, OnUpdate, OnPostUpdate, OnShow, OnHide, OnPostShow, OnPostHide, OnClick, OnPostClick, OnEnter, OnLeave, OnMouseDown, OnMouseUp, OnMouseWheel, OnDragStart, OnDragStop, OnReceiveDrag, OnSizeChanged, OnLoad, OnAttributeChanged, OnKeyDown, OnKeyUp, OnChar, OnEnterPressed, OnEscapePressed, OnTabPressed, OnSpacePressed, OnEditFocusGained/Lost, OnTextChanged, OnValueChanged, OnMinMaxChanged, OnTooltipCleared/SetItem/SetUnit/SetSpell.

## Handler Storage

Handlers stored in the `__scripts` global table keyed as `"{widget_id}_{handler_name}"` (e.g., `"42_OnClick"`). Frame userdata references stored as `__frame_{id}` globals for dispatch. A parallel Rust `ScriptRegistry` (`HashMap<u64, HashMap<ScriptHandler, i32>>`) lets Rust check handler existence without touching Lua.

`SetScript("OnUpdate", fn)` also inserts frame ID into `SimState.on_update_frames: HashSet<u64>`. `HookScript` appends to `__script_hooks` (stored but not auto-invoked during dispatch yet).

## Event Dispatch (`src/lua_api/env.rs`)

`fire_event(event_name)` flow:
1. `state.widgets.get_event_listeners(event)` — linear scan of all frames for `registered_events`
2. For each listener: look up `__scripts["{id}_OnEvent"]` and `__frame_{id}`, call `handler(frame, event_name, ...args)`
3. Errors logged per-frame; dispatch continues to remaining frames

`fire_script_handler(id, handler_type, args)` fires any non-OnEvent handler on a specific frame.

`FireEvent(event, ...)` — Lua-callable global that performs the same dispatch, useful for tests.

## OnUpdate Tick

`fire_on_update()` sequence each render tick:
1. Filter `on_update_frames` to visible-only
2. Call `__scripts["{id}_OnUpdate"](frame, elapsed)` for each
3. Fire OnPostUpdate handlers for the same set
4. Tick animation groups

OnUpdate errors suppress the frame on subsequent ticks (added to `on_update_errors` set). OnPostUpdate errors are logged but keep firing.

## Mouse and Key Input Flow

**Mouse** (`src/iced_app/update.rs`): hit-test determines frame under cursor → fires OnEnter/OnLeave on hover change → OnMouseDown on press → OnClick + OnMouseUp on release (if same frame). MouseWheel walks parent chain for first OnMouseWheel handler.

**Keys** (`src/lua_api/key_dispatch.rs`): Escape checks focused EditBox then toggles GameMenuFrame. Other keys check focused EditBox special handlers (OnEnterPressed, etc.), then fire OnKeyDown on focused frame, propagating up via `propagate_keyboard_input`.

## Startup Event Sequence

After all addons load, the simulator fires in order: `ADDON_LOADED("WoWUISim")`, `VARIABLES_LOADED`, `PLAYER_LOGIN`, `EDIT_MODE_LAYOUTS_UPDATED`, `TIME_PLAYED_MSG`, `PLAYER_ENTERING_WORLD(true, false)`, `UPDATE_BINDINGS`, `DISPLAY_SIZE_CHANGED`, `UI_SCALE_CHANGED`.

## XML Script Setup

Three handler forms: `function="X"` uses X directly, `method="X"` wraps as `self:X(...)`, inline body wraps as `function(self, ...) body end`. `inherit="prepend"` or `"append"` chains handlers with pcall wrapping.

## Sources

- [event-system.md](../../event-system.md) — EventQueue, ScriptHandler types, dispatch, OnUpdate, input flow, startup sequence

## See Also

- [[lua-api]] — SetScript, RegisterEvent, timer system
- [[widget-system]] — Frame.registered_events, on_update_frames set
- [[frame-data-flow]] — __scripts table layout, method lookup chain
