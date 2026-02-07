# Frame Data Flow: Mixin, Events, and Metamethods

How frame methods are stored via Mixin(), looked up via `__index`, and dispatched via events.

## Architecture Overview

The simulator maintains two parallel systems:

| System | Storage | Purpose |
|--------|---------|---------|
| **Rust** (`SimState`) | `WidgetRegistry` HashMap | Layout, rendering, event listener tracking |
| **Lua** | Global tables (`__frame_fields`, `__scripts`) | Mixin methods, script handlers, custom properties |

Each `FrameHandle` userdata stores an `id: u64` that links the two systems.

## Global Lua Tables

| Table | Key Structure | Purpose |
|-------|--------------|---------|
| `__frame_fields[frame_id][key]` | Nested: numeric frame ID → string key | Mixin methods, custom properties |
| `__scripts["{frame_id}_{handler}"]` | String: `"7623_OnEvent"` | Script handler functions |
| `__frame_{frame_id}` | Individual globals | Frame userdata references for event dispatch |
| `_G["FrameName"]` | Named globals | Named frame references |

## Method Lookup Order (`__index`)

When Lua accesses `frame.SomeKey`:

1. **mlua method table** — Rust-registered methods (`SetSize`, `SetPoint`, `SetShown`, `GetName`, etc.)
   - These take absolute priority and **cannot be overridden** via Lua assignment
2. **`children_keys`** — Rust HashMap lookup for child frames (`frame.Text`, `frame.NormalTexture`)
3. **`__frame_fields[frame_id]["SomeKey"]`** — Lua table lookup for mixin methods and custom properties
4. **Fallback methods** — Hardcoded stubs (`Clear`, `Lower`, `Raise`)
5. **`nil`** — Key not found

Source: `src/lua_api/frame/methods/methods_meta.rs:133-172`

### Critical Implication

Rust methods shadow mixin methods. If a mixin defines `SetShown`, it gets stored in `__frame_fields` but is never reached because the Rust `SetShown` method is found first (step 1).

This means patterns like `self.SetShownBase = self.SetShown` store a **nil** value (Rust methods are not accessible as values through `__index`), unless the method is specifically handled to return a callable.

## Property Storage (`__newindex`)

When Lua assigns `frame.SomeKey = value`:

1. If `value` is a `FrameHandle` userdata:
   - Stores in Rust `children_keys` HashMap (fast child lookup)
   - Also stores in `__frame_fields[frame_id]["SomeKey"]`
2. If `value` is anything else (function, number, string, table):
   - Stores **only** in `__frame_fields[frame_id]["SomeKey"]`

Source: `src/lua_api/frame/methods/methods_meta.rs:175-219`

## Mixin Application Flow

### Mixin() Function (Lua)

```lua
function Mixin(object, ...)
    for i = 1, select("#", ...) do
        local mixin = select(i, ...)
        if mixin then
            for k, v in pairs(mixin) do
                object[k] = v  -- triggers __newindex on userdata
            end
        end
    end
    return object
end
```

Source: `src/lua_api/globals/utility_api.rs:574-584`

### When Mixin() is Called

Mixin happens at **two points** during frame creation:

1. **Inside `CreateFrame()`** → `apply_templates_from_registry()` → `apply_single_template()` → `apply_mixin()`
   - Uses the global frame name: `local f = FrameName; Mixin(f, SomeMixin)`
   - Source: `src/lua_api/globals/template/mod.rs:267-304`

2. **After `CreateFrame()` returns** → `append_mixins_code()` in xml_frame.rs
   - Uses the local `frame` variable: `Mixin(frame, SomeMixin)`
   - Source: `src/loader/xml_frame.rs:147-180`

Both apply the same mixins (redundant but harmless — second call overwrites same values).

### Template Application Order (`apply_single_template`)

Within a single template, properties are applied in this order:

1. **Mixin** — `Mixin(frame, SomeMixin)` copies methods to `__frame_fields`
2. Size
3. Anchors
4. SetAllPoints
5. KeyValues
6. Layers (textures, fontstrings)
7. Button textures
8. Children (child frames)
9. **Scripts** — `SetScript("OnEvent", handler)` stores in `__scripts`

Source: `src/lua_api/globals/template/mod.rs:88-142`

### Template Chain Order

For `inherits="TemplateA, TemplateB"` where TemplateA itself inherits from TemplateBase:

Chain = `[TemplateBase, TemplateA, TemplateB]` — depth-first, parents before children.

Each template in the chain is processed via `apply_single_template` in order.

Source: `src/xml/template.rs:94-128`

## Script Handler Setup

### Template Script Application

When a template defines `<OnEvent method="OnEvent"/>`:

1. `build_handler_expr()` generates: `function(self, ...) self:OnEvent(...) end`
2. `append_script_handler()` generates: `frame:SetScript("OnEvent", <handler>)`
3. `SetScript` stores the handler in `__scripts["{frame_id}_OnEvent"]`

Source: `src/loader/helpers.rs:385-402, 440-450`

### Script Chaining (`inherit` attribute)

| Attribute | Behavior |
|-----------|----------|
| `inherit="prepend"` | New handler runs first, then old handler (both wrapped in pcall) |
| `inherit="append"` | Old handler runs first, then new handler |
| (none) | New handler **replaces** old handler entirely |

Source: `src/loader/helpers.rs:385-402`

## Event Dispatch Flow

### Registration

`frame:RegisterEvent("DISPLAY_SIZE_CHANGED")` → stores event name in Rust `Frame.registered_events` set.

Source: `src/lua_api/frame/methods/methods_event.rs:14-66`

### Firing (`fire_event_with_args`)

```
fire_event_with_args("DISPLAY_SIZE_CHANGED", [])
  │
  ├─ 1. Query Rust: state.widgets.get_event_listeners("DISPLAY_SIZE_CHANGED")
  │     → Returns Vec<u64> of frame IDs
  │
  └─ For each widget_id:
       │
       ├─ 2. Look up handler: __scripts["{widget_id}_OnEvent"]
       │
       ├─ 3. Look up frame: _G["__frame_{widget_id}"]
       │
       ├─ 4. Call: handler(frame, "DISPLAY_SIZE_CHANGED", ...args)
       │     │
       │     └─ Handler body: function(self, ...) self:OnEvent(...) end
       │           │
       │           ├─ self:OnEvent triggers __index for "OnEvent"
       │           ├─ mlua methods (no match) → children_keys (no match)
       │           ├─ __frame_fields[frame_id]["OnEvent"] → found (from Mixin)
       │           └─ Calls the mixin's OnEvent function
       │
       └─ 5. On error: log "[EVENT] handler error on FrameName (id=N): ..."
```

Source: `src/lua_api/env.rs:110-145`

## Frame Creation Sequence (xml_frame.rs)

Complete order for a frame defined in XML:

```
1. CreateFrame(type, name, parent, inherits)
   ├─ register_new_frame() → assigns frame_id
   ├─ create_widget_type_defaults() → button textures, slider parts, etc.
   ├─ create_frame_userdata() → sets _G["name"] and _G["__frame_{id}"]
   └─ apply_templates_from_registry() → processes template chain
       ├─ For each template: apply_single_template()
       │   (mixin → size → anchors → keyValues → layers → children → scripts)
       └─ fire_on_load() for template-created child frames

2. append_parent_key_code() → parent.Key = frame
3. append_mixins_code() → Mixin(frame, ...) again (redundant)
4. append_size/anchors/hidden/etc. → frame's own XML attributes
5. append_scripts_code() → frame's own <Scripts> block

6. create_child_frames() → non-template child frames
7. create_layer_children() → textures and fontstrings from <Layers>
8. apply_animation_groups()
9. apply_button_textures() → NormalTexture, PushedTexture, etc.
10. apply_button_text()

11. fire_lifecycle_scripts() → OnLoad, then OnShow if visible
```

Source: `src/loader/xml_frame.rs:13-71`

## Known Patterns and Pitfalls

### C++ Engine Mixin Stubs

Some Lua mixins (`ModelSceneControlButtonMixin = {}`) are intentionally empty tables where the C++ engine provides methods like OnLoad. The simulator patches these after each .lua file load:

```rust
fn apply_cpp_mixin_stubs(env: &WowLuaEnv) {
    // ModelSceneControlButtonMixin.OnLoad = function() end
    // PerksModelSceneControlButtonMixin.OnLoad = function() end
}
```

Source: `src/loader/addon.rs:126-144`

### EditModeSystemMixin "Base" Aliases

`EditModeSystemMixin.OnSystemLoad` normally saves method aliases (`SetShownBase = self.SetShown`). Frames with `inherit="prepend"` OnLoad handlers may call these aliases before OnSystemLoad runs. The simulator pre-initializes them during mixin application:

```rust
if name == "EditModeSystemMixin" {
    post_init.push_str("f.SetScaleBase = f.SetScale ");
    // ... etc
}
```

Source: `src/lua_api/globals/template/mod.rs:284-295`

### Script Chaining Order Issue

With `inherit="prepend"`, the new handler runs **before** the existing one. If the new handler depends on state initialized by the existing handler, it will fail. Example:

- `ActionBarTemplate` sets `<OnLoad method="ActionBar_OnLoad"/>`
- `EditModeActionBarTemplate` replaces with `<OnLoad method="EditModeActionBar_OnLoad"/>`
- `StanceBar` prepends `<OnLoad method="OnLoad" inherit="prepend"/>`
- Result: StanceBarMixin:OnLoad runs first → calls SetShowGrid → needs SetShownBase → not yet initialized

### Rust Method Shadow Problem

Rust-registered methods cannot be captured as values. This code:
```lua
self.SetShownBase = self.SetShown
```
Stores **nil** because `self.SetShown` goes through `__index`, which finds the Rust method in step 1 (mlua method table) — but mlua returns the method result only when it's called with arguments, not when accessed as a value.

Workaround: The simulator pre-initializes these aliases explicitly during mixin application, using Lua wrappers or by storing method references from the Lua side.

### `__frame_{id}` Global Namespace Collision (FIXED)

Anonymous template children were named `__frame_{rand_id()}` where `rand_id()` is a sequential counter starting at 1. Frame widget IDs (`next_widget_id()`) also start at 1. This caused collisions:

1. Frame with widget_id=974 is created → `_G["__frame_974"]` = FrameHandle(id=974)
2. Template creates anonymous child named `__frame_974` → `CreateFrame("Frame", "__frame_974", ...)`
3. `create_frame_userdata` sets `_G["__frame_974"]` = FrameHandle(id=8517) — **overwrites the original!**
4. Event fires for widget_id=974 → `fire_event_with_args` looks up `_G["__frame_974"]` → gets wrong frame
5. Handler calls `self:OnEvent(...)` → `__index` looks in `__frame_fields[8517]` → empty → nil error

**Fix**: Changed anonymous template child prefix from `__frame_` to `__tpl_` in `template/mod.rs:359`. The `__frame_{id}` namespace is now reserved exclusively for event dispatch references.
