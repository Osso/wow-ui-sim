# Widget System

## Frame Struct: Core Widget Representation

**File:** `src/widget/frame.rs:104-340`

The `Frame` struct is the fundamental data structure representing any widget in the UI, with ~140 fields organized by purpose.

### Core Identity & Hierarchy

- **`id: u64`** -- Unique widget ID, auto-generated via `next_widget_id()` atomic counter
- **`widget_type: WidgetType`** -- Discriminates between Frame, Button, Texture, FontString, etc.
- **`name: Option<String>`** -- Global name for lookup (e.g., "UIParent"); None = unnamed
- **`parent_id: Option<u64>`** -- Parent frame ID; None = root/unparented
- **`children: Vec<u64>`** -- Direct child IDs, maintains insertion order
- **`children_keys: HashMap<String, u64>`** -- Named child references (e.g., "Text" -> FontString ID)

### Visibility & Rendering Order

- **`visible: bool`** (default: true) -- Frame's own visibility flag (not parent-dependent)
- **`frame_strata: FrameStrata`** -- Major draw order (BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < TOOLTIP)
- **`frame_level: i32`** (default: 0) -- Draw order within strata
- **`has_fixed_frame_strata: bool`** (default: false) -- Whether strata was explicitly set
- **`has_fixed_frame_level: bool`** (default: false) -- Whether level was explicitly set
- **`alpha: f32`** (default: 1.0) -- Transparency (0.0-1.0)
- **`scale: f32`** (default: 1.0) -- Scale factor
- **`draw_layer: DrawLayer`** (default: Artwork) -- For regions: BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT
- **`draw_sub_layer: i32`** (default: 0) -- Fine-grained ordering within layer
- **`clips_children: bool`** (default: false) -- Mask child rendering to parent bounds

### Positioning & Anchoring

- **`width: f32`** / **`height: f32`** (default: 0.0) -- Explicit size; can be calculated from anchors
- **`anchors: Vec<Anchor>`** -- Anchor points defining position/size

### Input & Interaction

- **`mouse_enabled: bool`** (default: false) -- Respond to mouse clicks
- **`mouse_motion_enabled: bool`** (default: false) -- Fire OnEnter/OnLeave events
- **`keyboard_enabled: bool`** (default: false) -- Respond to keyboard
- **`propagate_keyboard_input: bool`** (default: false) -- Bubble key events to parent
- **`movable: bool`** / **`resizable: bool`** (default: false) -- Drag support
- **`clamped_to_screen: bool`** (default: false) -- Bound movement to screen
- **`user_id: i32`** (default: 0) -- User-defined ID (from SetID())

### Events

- **`registered_events: HashSet<String>`** -- Event names this frame listens for
- **`register_all_events: bool`** (default: false) -- Overrides event filtering

### Text & Font (FontString Widgets)

- **`text: Option<String>`** -- Text content
- **`font: Option<String>`** -- Font file path
- **`font_size: f32`** (default: 14.0) -- Font size in pixels
- **`font_outline: TextOutline`** -- None / Outline (1px) / ThickOutline (2px)
- **`text_color: Color`** (default: gold {1.0, 0.8, 0.2, 1.0})
- **`shadow_color: Color`** / **`shadow_offset: (f32, f32)`** -- Shadow rendering
- **`justify_h: TextJustify`** / **`justify_v: TextJustify`** -- Alignment
- **`word_wrap: bool`** (default: true) / **`max_lines: u32`** (default: 0 = unlimited)

### Textures (Texture & Button Widgets)

- **`texture: Option<String>`** -- File path for Texture widget
- **`color_texture: Option<Color>`** -- Solid color (from SetColorTexture)
- **`vertex_color: Option<Color>`** -- Tint color
- **`normal_texture`** / **`pushed_texture`** / **`highlight_texture`** / **`disabled_texture`** -- Button state textures (each with `_tex_coords` UV companion)
- **`checked_texture`** / **`disabled_checked_texture`** -- CheckButton variants
- **`tex_coords: Option<(f32, f32, f32, f32)>`** -- Final UV coordinates (left, right, top, bottom)
- **`atlas_tex_coords`** -- Atlas sub-region (SetTexCoord remaps relative to these)
- **`atlas: Option<String>`** -- Atlas name if set via SetAtlas
- **`horiz_tile: bool`** / **`vert_tile: bool`** -- Tiling flags
- **`blend_mode: BlendMode`** (default: Alpha)
- **`nine_slice_layout`** / **`nine_slice_atlas`** -- Nine-slice rendering
- **`is_mask: bool`** (default: false) -- Don't render (mask textures)

### Backdrop

- **`backdrop: Backdrop`** -- Contains: enabled, bg_file, edge_file, bg_color, border_color, edge_size, insets

### Widget-Specific Fields

**Slider:** `slider_value`, `slider_min/max`, `slider_step`, `slider_orientation`, `slider_obey_step_on_drag`, `slider_steps_per_page`

**StatusBar:** `statusbar_value`, `statusbar_min/max`, `statusbar_color`, `statusbar_texture_path`, `statusbar_bar_id`, `statusbar_fill_style`, `statusbar_reverse_fill`, `statusbar_orientation`

**EditBox:** `editbox_cursor_pos`, `editbox_max_letters/bytes`, `editbox_multi_line`, `editbox_auto_focus`, `editbox_numeric`, `editbox_password`, `editbox_blink_speed`, `editbox_history`, `editbox_text_insets`

**ScrollFrame:** `scroll_child_id`, `scroll_horizontal`, `scroll_vertical`

**Cooldown:** `cooldown_start`, `cooldown_duration`, `cooldown_reverse`, `cooldown_draw_swipe/edge/bling`, `cooldown_hide_countdown`, `cooldown_paused`

### Attributes

- **`attributes: HashMap<String, AttributeValue>`** -- Custom properties (for secure/unit frames)

---

## WidgetType Enum

**File:** `src/widget/mod.rs:22-92`

```rust
pub enum WidgetType {
    Frame,          // Container only
    Button,         // Clickable with state-driven textures
    FontString,     // Text rendering
    Texture,        // Image/solid-color
    EditBox,        // Text input
    ScrollFrame,    // Viewport with scrollable child
    Slider,         // Value control with thumb
    CheckButton,    // Button with checked/unchecked state
    StatusBar,      // Filled bar (health, mana)
    Cooldown,       // Timer with swipe animation
    Model,          // 3D model (stub)
    ModelScene,     // 3D scene (stub)
    PlayerModel,    // Cinematic model variant
    ColorSelect,    // Color picker
    MessageFrame,   // Scrolling text
    SimpleHTML,     // Rich text (stub)
    GameTooltip,    // Tooltip
    Minimap,        // Minimap (stub)
}
```

**Type conversion:** `from_str()` maps WoW names ("ItemButton" -> Button, "ScrollingMessageFrame" -> MessageFrame). `as_str()` reverses to canonical WoW names.

---

## WidgetRegistry

**File:** `src/widget/registry.rs:7-126`

```rust
pub struct WidgetRegistry {
    widgets: HashMap<u64, Frame>,
    names: HashMap<String, u64>,
    render_dirty: Cell<bool>,
}
```

### Operations

- **`register(widget)`** -- Adds/overwrites, updates names index
- **`get(id)`** / **`get_mut(id)`** -- Lookup (get_mut sets render_dirty)
- **`get_by_name(name)`** / **`get_id_by_name(name)`** -- Name lookup
- **`add_child(parent_id, child_id)`** -- Appends to parent's children vec
- **`get_event_listeners(event)`** -- Returns IDs registered for an event
- **`would_create_anchor_cycle(frame_id, relative_to_id)`** -- BFS cycle detection
- **`take_render_dirty()`** -- Atomically check and clear dirty flag

---

## Default Children

**File:** `src/lua_api/globals/create_frame.rs:181-380`

### Button/CheckButton (lines 205-246)

Creates 4 child Textures (NormalTexture, PushedTexture, HighlightTexture, DisabledTexture) + 1 FontString (Text). All textures anchored TOPLEFT+BOTTOMRIGHT to fill button. Button has `mouse_enabled = true`.

### Slider (lines 258-270)

Creates Low, High, Text FontStrings + ThumbTexture.

### GameTooltip (lines 249-255)

Inserts tooltip data into `SimState.tooltips`. Sets strata to TOOLTIP.

### ItemButton (lines 305-362)

Creates child Textures (icon, searchOverlay, IconBorder, IconOverlay, IconOverlay2, ItemContextOverlay) and FontStrings (Count, Stock) with specific anchors and draw layers.

### Helper: `add_fill_parent_anchors()` (lines 273-291)

Sets TOPLEFT and BOTTOMRIGHT anchors to fill parent (equivalent to SetAllPoints).

---

## Frame Strata & Level System

### FrameStrata Enum (`src/widget/frame.rs:557-602`)

```
World(0) < Background(1) < Low(2) < Medium(3) < High(4) < Dialog(5) < Fullscreen(6) < FullscreenDialog(7) < Tooltip(8)
```

### Inheritance

- Child inherits `parent.frame_strata` if `has_fixed_frame_strata = false`
- Child inherits `parent.frame_level + 1` if `has_fixed_frame_level = false`
- `SetFrameStrata()` marks as fixed, preventing further inheritance

---

## Visibility System

**File:** `src/lua_api/frame/methods/methods_core.rs:264-334`

- **`Show()`** -- Sets `visible = true`, fires `OnShow` recursively on visible children
- **`Hide()`** -- Sets `visible = false`
- **`IsVisible()`** -- True only if this frame AND all ancestors are visible (walks parent chain)
- **`IsShown()`** -- Returns `frame.visible` only (ignores parents)
- **`SetShown(bool)`** -- Equivalent to `if shown then Show() else Hide() end`

A frame can be `IsShown() = true` but `IsVisible() = false` if parent is hidden.

---

## Widget-Type-Specific Rendering

**File:** `src/iced_app/render.rs:467-512`

| Type | Text Alignment | Notes |
|------|---------------|-------|
| Frame/StatusBar | N/A | Backdrop, nine-slice |
| Button | Center/Center, no wrap | State-driven textures |
| Texture | N/A | Skipped if is_mask |
| FontString | justify_h/justify_v, word_wrap | Full text control |
| CheckButton | Left/Center, x+20 offset | Checkbox reserved space |
| EditBox | Left/Center, 4px padding | |

---

## Event Registration

**File:** `src/event/mod.rs` and `src/lua_api/frame/methods/methods_event.rs`

### Methods

- `RegisterEvent(event)` / `UnregisterEvent(event)` / `UnregisterAllEvents()`
- `RegisterAllEvents()` -- Receives ALL events
- `IsEventRegistered(event)` -- True if register_all_events OR event in set

### Script Handlers

OnEvent, OnUpdate, OnShow, OnHide, OnClick, OnEnter, OnLeave, OnMouseDown, OnMouseUp, OnDragStart, OnDragStop, OnReceiveDrag, OnMouseWheel, OnSizeChanged, OnLoad, OnAttributeChanged, OnKeyDown, OnKeyUp, OnChar, OnEnterPressed, OnEscapePressed, OnTabPressed, OnSpacePressed, OnEditFocusGained/Lost, OnTextChanged, OnValueChanged, OnMinMaxChanged, OnTooltipCleared/SetItem/SetUnit/SetSpell, and Post-handlers (OnPostUpdate, OnPostShow, OnPostHide, OnPostClick).

---

## Parent-Child Hierarchy

### CreateFrame
**File:** `src/lua_api/globals/create_frame.rs:12-48`

1. Parse args; default parent to UIParent
2. Register frame, link parent-child, inherit strata/level
3. Create default children per widget type
4. Return FrameHandle userdata

### SetParent
**File:** `src/lua_api/frame/methods/methods_hierarchy.rs:31-40`

Removes from old parent's children list, sets new parent, re-inherits strata/level if not fixed.

### Named Children (children_keys)

`SetParentKey(key)` stores in `parent.children_keys[key] = this.id`. Allows `parent.key = child` in Lua via `__newindex` metamethod.

---

## Summary Table

| Aspect | File | Key Items |
|--------|------|-----------|
| Core Frame | `src/widget/frame.rs` | Frame struct (~140 fields) |
| Widget Types | `src/widget/mod.rs` | WidgetType enum (18 types) |
| Registry | `src/widget/registry.rs` | WidgetRegistry (HashMap, names index, dirty flag) |
| Default Children | `src/lua_api/globals/create_frame.rs` | Button textures, slider parts, tooltip state |
| Strata/Levels | `src/widget/frame.rs`, `methods_core.rs` | FrameStrata enum, inheritance rules |
| Visibility | `methods_core.rs` | Show/Hide/IsVisible/IsShown, parent chain |
| Rendering | `src/iced_app/render.rs` | Widget type dispatch, quad emission |
| Events | `src/event/mod.rs`, `methods_event.rs` | EventQueue, ScriptHandler, registration |
| Hierarchy | `methods_hierarchy.rs` | SetParent, children_keys, inheritance |
| Anchoring | `anchor.rs`, `methods_anchor.rs` | Anchor struct, cycle detection |
