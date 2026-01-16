# WoW UI Simulator - Plan

## GTK4 Migration

### Why Migrate from iced

The current iced canvas renderer has fundamental limitations:
- No text measurement APIs - centering text requires guessing character widths
- Manual layout calculations for everything
- No proper widget system - reimplementing basic UI primitives
- Fighting the framework instead of leveraging it

GTK4/relm4 provides:
- Proper text rendering with Pango (measurement, wrapping, ellipsis)
- Layout containers (Box, Grid, Overlay) that handle positioning
- Real widget system with CSS styling
- Same Elm-like architecture as iced (via relm4)

### Migration Strategy

**Phase 1: Scaffold GTK App** ✅
- [x] Add gtk4, relm4, libadwaita dependencies
- [x] Create basic window with relm4 Application
- [x] Port console/command input (bottom panel)
- [x] Port event buttons (ADDON_LOADED, etc.)
- [x] Port frames list sidebar
- [x] Integrate gtk-layout-inspector for debugging
- [x] Add WoW-style CSS theming
- [x] Cairo rendering for WoW frames

**Phase 2: WoW UI Canvas**
- [ ] Create custom GtkDrawingArea for WoW frame rendering
- [ ] Port nine-slice texture rendering to Cairo
- [ ] Port texture/image rendering
- [ ] Implement proper text rendering with Pango
- [ ] Mouse event handling (hover, click)

**Phase 3: Widget Mapping**
- [ ] Map WoW Button → GTK rendering with proper text centering
- [ ] Map WoW FontString → Pango text layout
- [ ] Map WoW Frame → Cairo rectangle with backdrop
- [ ] Map WoW Texture → Cairo image surface
- [ ] Map WoW EditBox → GTK Entry overlay or custom drawing

**Phase 4: Cleanup**
- [ ] Remove iced dependencies
- [ ] Remove render/ui.rs (1400 lines of manual layout)
- [ ] Update iced-layout-inspector → gtk-layout-inspector

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│ relm4 Application                                       │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────┐ ┌─────────────────┐ │
│ │ WoW Canvas (GtkDrawingArea)     │ │ Frames Sidebar  │ │
│ │ - Cairo rendering               │ │ - GtkListView   │ │
│ │ - Pango text                    │ │                 │ │
│ │ - Mouse events                  │ │                 │ │
│ └─────────────────────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────┤
│ Event Buttons (GtkBox with GtkButtons)                  │
├─────────────────────────────────────────────────────────┤
│ Command Input (GtkEntry) │ Console Output (GtkTextView) │
└─────────────────────────────────────────────────────────┘
```

### Files to Modify

| Current (iced) | New (GTK4) |
|----------------|------------|
| src/render/ui.rs (1400 lines) | src/gtk_app.rs - relm4 component |
| src/render/nine_slice.rs | src/render/cairo_nine_slice.rs |
| src/main.rs | src/main.rs - gtk4 app init |
| Cargo.toml (iced deps) | Cargo.toml (gtk4, relm4, adw) |

### Keep Unchanged

These modules are renderer-agnostic and stay the same:
- `src/lua_api/` - Lua bindings (mlua)
- `src/loader.rs` - Addon loading
- `src/widget/` - Frame tree data structures
- `src/xml/` - XML parsing
- `src/toc.rs` - TOC file parsing
- `src/saved_variables.rs` - SavedVariables persistence
- `src/event/` - Event system

### Dependencies

```toml
[dependencies]
# Remove
# iced = ...
# iced_runtime = ...
# iced-layout-inspector = ...

# Add
relm4 = { version = "0.10", features = ["libadwaita"] }
gtk = { version = "0.10", package = "gtk4" }
adw = { version = "0.8", package = "libadwaita" }
gtk-layout-inspector = { path = "...", features = ["server"] }
```

### Reference Implementation

See `/home/osso/Projects/apps/reddit-desktop-gtk` for relm4 patterns:
- TypedListView for frame list
- CSS styling
- gtk-layout-inspector integration
- Async operations with tokio

---

## Current State

- Basic rendering with WoW-style textures and 9-slice
- Mouse interaction (hover, click, OnEnter/OnLeave/OnClick)
- Event system (ADDON_LOADED, PLAYER_LOGIN, PLAYER_ENTERING_WORLD)
- Saved variables persistence (JSON in ~/.local/share/wow-ui-sim/)
- Loading real addons: Ace3, SharedXML, GameMenu, WeakAuras, DBM, Details, Plater

### Load Statistics
- Ace3: 43 Lua, 15 XML, 0 warnings
- SharedXMLBase: 34 Lua, 2 XML
- SharedXML: 155 Lua, 72 XML, 0 warnings
- GameMenu: 3 Lua, 1 XML, 1 warning
- WeakAuras: 97 Lua, 4 XML, 1 warning
- DBM-Core: 43 Lua, 0 XML, 33 warnings
- Details: 117 Lua, 2 XML, 42 warnings
- Plater: 13 Lua, 1 XML, 42 warnings

## High Impact

### Fix Addon Errors
- Details PlayerInfo.lua:852 - `'for' initial value must be a number`
- Investigate missing APIs causing 30+ warnings per addon
- Add stubs or implementations as needed

### More Frame Types
- ScrollFrame - scrollable content areas
- EditBox - text input fields
- Slider - value sliders
- CheckButton - checkboxes
- DropDownMenu - dropdown menus

### Keyboard Input
- EditBox text entry
- Keybinding system (SetBinding, GetBindingKey)
- Focus management

## Visual Improvements

### Font Rendering
- Load WoW fonts (FRIZQT__, ARIALN, etc.)
- FontString:SetFont() implementation
- Text alignment and wrapping

### Tooltip System
- GameTooltip frame
- SetOwner, AddLine, Show/Hide
- Anchor positioning

### Frame Dragging/Resizing
- StartMoving/StopMovingOrSizing
- SetMovable/SetResizable
- Clamp to screen

## Functionality

### Slash Commands
- Register handlers via SlashCmdList
- Parse and dispatch /command input
- Console input for typing commands

### API Coverage
Priority APIs by addon usage:
- C_Timer (After, NewTimer, NewTicker)
- C_ChatInfo (SendAddonMessage)
- C_Covenants, C_Soulbinds
- Combat log events
- Unit auras (UnitAura, UnitBuff, UnitDebuff)

## Reference Addons

Located at `~/Projects/wow/reference-addons/`:
- `Ace3/` - Popular addon framework
- `DeadlyBossMods/` - Raid encounter alerts
- `Details/` - Damage meter
- `Plater/` - Nameplate addon
- `WeakAuras2/` - Custom display framework
- `wow-ui-source/` - Blizzard's official UI code
