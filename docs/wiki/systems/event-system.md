# Event System

The event system connects WoW-style game events and UI interactions to Lua script handlers. Events flow from Rust through two mechanisms: named game events dispatched to registered frames via OnEvent, and direct per-frame script handlers fired by specific Rust triggers.

## Event Queue and Types

`EventQueue` is a `Vec<Event>` with push/drain semantics. Most events are fired directly via `fire_event()` rather than queued.

```rust
pub struct Event { pub name: String, pub args: Vec<EventArg> }
pub enum EventArg { String(String), Number(f64), Boolean(bool), Nil }
```

Predefined constants include PLAYER_LOGIN, PLAYER_ENTERING_WORLD, ADDON_LOADED, VARIABLES_LOADED, UPDATE_BINDINGS, DISPLAY_SIZE_CHANGED, UNIT_HEALTH, UNIT_POWER_UPDATE, COMBAT_LOG_EVENT, BAG_UPDATE. Retail/PTR `RegisterEvent()` validates names against generated and epoch-specific strict tables; retail 12.1 includes `EXTERNAL_EVENT_LAUNCH_URL_FAILED` for current `Blizzard_GameMenu` loading. Classic profiles accept any non-empty name. Forever otherwise uses a finite extension table over the generated mainline list. Source-documented additions include `CHAT_MSG_COLLECTED_APPEARANCE`, `UNIT_AURA_BLOCKED`, `CHAT_MSG_GUILD_DISCORD`, `PLAYER_SWING`, and `PLAYER_SWING_RANGE_UPDATE`; arbitrary unknown names, including undocumented `PLAYER_EQUIPED_SPELLS_CHANGED`, remain rejected. `PLAYER_SWING` and range-update registration have cleanup/payload coverage, an injected AppelSwingsForever GUI workflow, and independent targeted verification. This records registration and simulator-admin dispatch only; gameplay/range producers, payload secrecy, and native delivery remain unverified.

Registerability does not model an event producer, payload, or `C_ExternalEventURL` behavior.

## State-based removal source checkpoint — 2026-10-13

[Removal owner](../../specs/frame-event-removal.md#current-source-checkpoint--2026-10-13) links [Current bounded gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md#current-bounded-runtime-gate--2026-10-13): independent Retail PASS; PTR independent PASS; selected retention PASS. Strict insertion/state-based removal are simulator policy; Classic preservation and native parity unproved, last requirement not fully checked.

## Script Handler Types (36+, `src/event/mod.rs`)

Per-frame, per-type (at most one active handler): OnEvent, OnUpdate, OnPostUpdate, OnShow, OnHide, OnPostShow, OnPostHide, OnClick, OnPostClick, OnEnter, OnLeave, OnMouseDown, OnMouseUp, OnMouseWheel, OnDragStart, OnDragStop, OnReceiveDrag, OnSizeChanged, OnLoad, OnAttributeChanged, OnKeyDown, OnKeyUp, OnChar, OnEnterPressed, OnEscapePressed, OnTabPressed, OnSpacePressed, OnEditFocusGained/Lost, OnTextSet, OnTextChanged, OnValueChanged, OnMinMaxChanged, OnTooltipCleared/SetItem/SetUnit/SetSpell.

## Handler Storage

Handlers stored in the `__scripts` global table keyed as `"{widget_id}_{handler_name}"` (e.g., `"42_OnClick"`). Frame userdata references stored as `__frame_{id}` globals for dispatch. A parallel Rust `ScriptRegistry` (`HashMap<u64, HashMap<ScriptHandler, i32>>`) lets Rust check handler existence without touching Lua.

`SetScript("OnUpdate", fn)` also inserts frame ID into `SimState.on_update_frames: HashSet<u64>`. `HookScript` appends to `__script_hooks`. `EditBox:SetFocus` and `ClearFocus` dispatch normal script bindings and hooks through the ordered normal-handler path; other trigger paths retain their own dispatch coverage.

## Event Dispatch (`src/lua_api/env.rs`)

`fire_event(event_name)` flow:
1. `state.widgets.get_event_listeners(event)` — linear scan of all frames for `registered_events`
2. For each listener: look up `__scripts["{id}_OnEvent"]` and `__frame_{id}`, call `handler(frame, event_name, ...args)`
3. Errors logged per-frame; dispatch continues to remaining frames

`fire_script_handler(id, handler_type, args)` fires any non-OnEvent handler on a specific frame. Changed EditBox `SetText`/`SetFormattedText` first commits text and clamps scalar cursor/selection state, then dispatches synchronous `OnTextSet(self)`. Source-only `c8d79f3bf` defers `OnTextChanged` until an early shared tick or pre-key-down dirty flush refreshes presentation and dispatches cursor notification first. Insertion `OnChar` stays synchronous; pending change/origin bits coalesce notification and are consumed before callbacks. Same-value assignment does not enqueue a change. [Owning EditBox contract](../../specs/editbox-text-position-selection.md#cursor-geometry-follow-up) retains historical RED, origin rules and uncompiled/unexecuted producer status. `EditBox:SetFocus` and `ClearFocus` use the normal ordered binding dispatcher directly: after committing focus state, a transfer calls former owner `OnEditFocusLost`, then current requested owner `OnEditFocusGained`; clearing calls loss only. Repeated focus/clear and clearing a non-owner are no-ops. Loss-handler reentry can replace the requested owner, suppressing its stale gained callback; handler errors are reported and later focus callbacks continue. This closes simulator omissions, not native-WoW skips. Mouse input focus behavior is unchanged.

`FireEvent(event, ...)` — Lua-callable global that performs the same dispatch, useful for tests.

## Explicit unit role notification

Source-inspected 2026-10-12: `globals/real/unit_roles_assignment.rs` commits changed GUID-backed role state, releases its state borrow, then calls existing `script_helpers::fire_named_event_state` for synchronous `ROLE_CHANGED_INFORM` before returning the setter boolean. This is not a new queue or dispatcher. [Role assignment contract](../../specs/unit-role-assignment.md) owns payload/caller requirements, modeled policies and pending runtime proof; no PASS or native-delivery claim here. [[lua-api#Global Functions]] links the group-role API inventory.

## OnUpdate Tick

`fire_on_update()` sequence each render tick (independently validated bounded saved runtime; not broader acceptance):

Before ordinary OnUpdate dispatch, flush EditBox presentation/cursor and pending text notifications; the same flush precedes key-down handling. Changed inputs notify; idle inputs/ticks are suppressed. The [EditBox owner](../../specs/editbox-text-position-selection.md#saved-retail-execution--2026-10-15) owns saved execution, stderr caveats, historical failures and unresolved genuine-consumer acceptance. No native callback-count claim.

1. Filter `on_update_frames` to visible-only
2. Call `__scripts["{id}_OnUpdate"](frame, elapsed)` for each
3. Fire OnPostUpdate handlers for the same set
4. Tick animation groups

OnUpdate errors suppress the frame on subsequent ticks (added to `on_update_errors` set). OnPostUpdate errors are logged but keep firing.

## Mouse and Key Input Flow

**Mouse** (`src/iced_app/update.rs`): hit-test determines frame under cursor → fires OnEnter/OnLeave on hover change → OnMouseDown on press → OnClick + OnMouseUp on release (if same frame). MouseWheel walks parent chain for first OnMouseWheel handler.

**Keys** (`src/lua_api/key_dispatch.rs`): Escape checks focused EditBox then toggles GameMenuFrame. Other keys check focused EditBox special handlers (OnEnterPressed, etc.), then fire OnKeyDown on focused frame, propagating up via `propagate_keyboard_input`.

## Startup Event Sequence

The [GUI startup contract](../../specs/gui-startup-yield.md) owns update-turn yielding and acceptance. [Startup mechanics](../../event-system.md#startup-event-sequence) owns the event order, first-canvas capture, IPC ordering, settling and resize dispatch; game login and glue startup have distinct sequences.

Wrath uses a two-chunk readiness callback for VARIABLES_LOADED, omits the synthetic WoWUISim addon completion, and delivers argless PLAYER_ENTERING_WORLD. A pending callback may complete after login/entry; other profiles keep their prior simulator-policy order. The [ReloadUI contract](../../specs/reload-ui.md#wrath-12340-requirements) separates profile-native facts, supplied readiness and bounded proof.

## XML Script Setup

Three handler forms: `function="X"` uses X directly, `method="X"` wraps as `self:X(...)`, inline body wraps as `function(self, ...) body end`. `inherit="prepend"` or `"append"` chains handlers with pcall wrapping.

## Sources

- [Unit role assignment](../../specs/unit-role-assignment.md) — role producer, original callers and proof limits

- [event-system.md](../../event-system.md) — EventQueue, ScriptHandler types, dispatch, OnUpdate, input flow, startup sequence
- [GUI startup contract](../../specs/gui-startup-yield.md) — update-turn yielding, IPC ordering and proof limits
- [EditBox focus callbacks](../../specs/editbox-focus-callbacks.md) — Lua focus-transition contract and limits
- [Forever finite constants](../../specs/forever-finite-constants.md) — source-backed finite event additions and limits
- `Blizzard_APIDocumentationGenerated/{ChatInfoDocumentation,UnitAuraDocumentation}.lua` in the pinned Forever cache — event literals

## See Also

- [[lua-api]] — SetScript, RegisterEvent, timer system
- [[client-profiles]] — Forever-specific finite registration boundary
- [[widget-system]] — Frame input/focus state, registered_events, and on_update_frames
- [[frame-data-flow]] — __scripts table layout, method lookup chain
