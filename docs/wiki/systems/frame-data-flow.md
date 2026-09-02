# Frame Data Flow

The simulator maintains two parallel systems that stay in sync through a shared `id: u64`. Rust owns layout and rendering state; Lua frame-ref tables own mixin methods, script handlers, and custom properties.

## Parallel Systems

| System | Storage | Contents |
|--------|---------|----------|
| Rust (`SimState`) | `WidgetRegistry` HashMap | Frame geometry, visibility, event registrations |
| Lua frame refs | `__frame_refs`, `__scripts` | Mixin methods, handlers, custom properties |

Each frame ref is a backed Lua table whose backing data encodes the widget ID. Everything else is looked up at call time.

## Global Lua Tables

| Table | Key structure | Purpose |
|-------|--------------|---------|
| `__frame_refs[frame_id]` | id → backed table | Lua frame refs, including mixin methods and custom properties |
| `__scripts["{frame_id}_{handler}"]` | string | Script handler functions |
| `_G["FrameName"]` | named globals | Named frame references |

## Method Lookup Order (`__index`)

When Lua reads `frame.SomeKey`:

1. **rilua method table** — Rust-registered methods (`SetSize`, `SetPoint`, etc.); cannot be overridden from Lua
2. **`children_keys`** — Rust HashMap for child frame refs (`frame.Text`, `frame.NormalTexture`)
3. **frame table fields** — mixin methods and custom properties
4. **nil**

Critical implication: Rust methods shadow mixin methods. `self.SetShownBase = self.SetShown` stores nil because native methods are not exposed as callable values through ordinary `__index` lookup. `EditModeSystemMixin` is handled explicitly during mixin application so its native aliases are available before lifecycle scripts run.

## Property Storage (`__newindex`)

When Lua assigns `frame.Key = value`:
- child frame ref → stored in `children_keys` (Rust) and on the frame table
- any other value → stored on the frame table

`debug.getfenv(frame)[1]` is a live compatibility view onto the frame's per-instance field table. It must not consume raw numeric key `1` on the frame itself, because addons such as oUF use frames as array-like containers (`element[1]`, `table.insert(element, button)`). A previous implementation stored the hidden field table at raw `frame[1]`, causing ElvUI/oUF aura updates to treat the field table as an aura button and fail at `button:SetSize`.

## Mixin Application

`Mixin(target, MixinTable)` iterates the mixin table and assigns each key to the target via `__newindex`, landing on the frame table. Applied at two points: inside `apply_templates_from_registry()` (before frame name available in Lua) and again after `CreateFrame()` returns in xml_frame.rs (redundant but harmless).

Template chain order: `[TemplateBase, TemplateA, TemplateB]` (depth-first, parents before children). Within each template: Mixin → Size → Anchors → KeyValues → Layers → Children → Scripts.

### EditModeSystemMixin base aliases

`EditModeSystemMixin.OnSystemLoad` normally saves aliases for methods that it overrides. A template with `inherit="prepend"` can run an `OnLoad` handler before `OnSystemLoad`, so the simulator seeds these callable native aliases immediately when applying the mixin:

- `SetScaleBase`, `SetPointBase`, `ClearAllPointsBase`
- `SetShownBase`, `ShowBase`, `HideBase`, `IsShownBase`

The aliases are stored on the frame's per-instance field table before template scripts execute. Missing native methods are reported as template-application errors rather than becoming nil aliases. This ordering supports action-bar templates whose prepended handlers call the base visibility methods.

## Event Dispatch Flow

```
fire_event("PLAYER_LOGIN")
  ├─ Rust: state.widgets.get_event_listeners("PLAYER_LOGIN") → Vec<u64>
  └─ For each id:
       ├─ Lua: __scripts["{id}_OnEvent"] → handler function
       ├─ Lua: __frame_refs[id] → frame ref table
       └─ Call: handler(frame, "PLAYER_LOGIN", ...args)
            └─ self:OnEvent(...)  →  __index lookup for "OnEvent"
                 └─ frame.OnEvent  ← from Mixin
```

## Addon Loading Context

Direct and runtime addon loading share one loading transaction owned by `SimState`. Runtime `C_AddOns.LoadAddOn()` enters that transaction before foundation and TOC dependency traversal, then loads Lua/XML files and applies post-load workarounds. The owner commits `loaded` only after those steps complete; `ADDON_LOADED` is dispatched after that commit.

Nested re-entry sees the active transaction: `C_AddOns.LoadAddOn()` returns success without re-entering the addon, while `C_AddOns.IsAddOnLoaded()` reports `(true, false)` until the owner commits, then `(true, true)`. RAII cleanup removes the transaction and restores the previous loading owner context on success or failure.

## Frame Creation Order (xml_frame.rs)

```
1. Create or reuse the frame
   ├─ ordinary names call CreateFrame(type, name, parent, inherits)
   │  ├─ register_new_frame() → assigns frame_id
   │  ├─ create_widget_type_defaults() → button textures etc.
   │  ├─ set _G["name"] and cache frame ref in __frame_refs
   │  └─ apply_templates_from_registry()
   │      └─ for each template: mixin → size → anchors → keyValues → layers → children → scripts
   └─ engine roots UIParent and WorldFrame reuse their pre-created frame refs; XML mixins, scripts, event registrations, and lifecycle configuration target those same refs

2. append_parent_key_code() → parent.Key = frame
3. append_mixins_code() → Mixin(frame, ...) again
4. append_size/anchors/hidden/EnableMouse/scripts from frame's own XML

5. create_child_frames(), create_layer_children()
6. apply_animation_groups(), apply_button_textures()
7. fire_lifecycle_scripts() → OnLoad, then OnShow if visible
```

## Known Pitfalls

**Rust method shadow**: `self.SetShownBase = self.SetShown` stores nil. The simulator resolves and stores seven native `EditModeSystemMixin` aliases during mixin application, before prepended lifecycle handlers can run.

**`__frame_{id}` namespace**: anonymous template children must use `__tpl_` prefix (not `__frame_`), since `__frame_{id}` is reserved for event dispatch. Historical collision caused wrong frame to be dispatched for events.

**Engine-created roots**: `UIParent` and `WorldFrame` exist before Blizzard XML loads. Their XML definitions configure those existing objects; they must not call `CreateFrame` again. XML mixins, scripts, event registrations, and lifecycle configuration therefore remain attached to the object later observed through the global. A duplicate replacement leaves those behaviors on the original root while later code observes another object. This broke UIParent startup handlers and prevented CombatLog runtime state from loading. The XML code generator now special-cases both names (`src/loader/xml_frame_codegen.rs`, commit `e5089fbeb2`).

**Script chaining order**: `inherit="prepend"` runs new handler before old. If new handler depends on state the old handler sets up, it will fail on first call.

## Sources

- [frame-data-flow.md](../../frame-data-flow.md) — parallel systems, __index order, Mixin flow, event dispatch, creation sequence, pitfalls
- [methods.rs](../../../src/lua_api/methods.rs) — frame ref cache and frame-field compatibility behavior
- [shared_bootstrap.lua](../../../src/lua_api/env_init/shared_bootstrap.lua) — `debug.getfenv(frame)` compatibility view
- [xml_frame_codegen.rs](../../../src/loader/xml_frame_codegen.rs) — reuses pre-created UIParent/WorldFrame objects for their XML definitions
- [c_addons_runtime.rs](../../../src/c_api/c_addons_runtime.rs) — enters runtime dependency loads through the shared loading transaction
- [addon.rs](../../../src/loader/addon.rs) — owns loading transaction commit and RAII cleanup
- [helpers.rs](../../../src/lua_api/globals/create_frame/helpers.rs) — applies mixins and seeds EditMode base aliases
- [template_chain.rs](../../../src/lua_api/globals/create_frame/template_chain.rs) — propagates mixin-application errors through template chains
- [state.rs](../../../src/lua_api/state.rs) — reports whether an addon is currently loading

## See Also

- [[lua-api]] — FrameHandle, method submodules, __index/__newindex implementation
- [[event-system]] — fire_event dispatch, __scripts table population
- [[xml-template-system]] — template chain resolution and application order
