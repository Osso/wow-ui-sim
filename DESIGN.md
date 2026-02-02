# WoW UI Simulator - Design

## Goals

- **Primary**: Run WoW addons outside the game for testing and development
- **Secondary**: Visual preview of addon UI using iced for rendering
- Load BlizzardInterfaceCode (wow-ui-source) as base, then user addons on top

## Non-Goals

- Full game emulation (combat, spells, inventory, etc.)
- Network/server connectivity
- Audio playback
- Secure execution (taint system is stubbed as always-secure)

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Rust Host                        │
├──────────────┬──────────────┬──────────────────────┤
│  lua_api/    │   widget/    │      render/         │
│  - globals   │   - Frame    │   - iced UI          │
│  - WoW APIs  │   - Registry │   - texture loading  │
│              │   - Anchor   │   - text rendering   │
├──────────────┴──────────────┴──────────────────────┤
│                  mlua (Lua 5.1)                    │
├────────────────────────────────────────────────────┤
│              Addon Lua Code                        │
│  - BlizzardInterfaceCode (base)                    │
│  - User addons (loaded on top)                     │
└────────────────────────────────────────────────────┘
```

## Key Concepts

### Widget System
- `Frame` is the base widget type (also Button, Texture, FontString, etc.)
- Widgets have anchors for positioning (SetPoint, ClearAllPoints)
- Parent-child hierarchy with UIParent as root
- Frame strata and levels for z-ordering

### Lua Environment
- mlua embeds Lua 5.1 (WoW's Lua version)
- Global aliases for compatibility (strlen, tinsert, bit.band, etc.)
- Mixin system for OOP patterns (Mixin, CreateFromMixins)
- Event system for addon communication

### Load Order (target)
1. Blizzard_SharedXMLBase (Mixin, utilities)
2. Blizzard_SharedXML (more utilities)
3. Blizzard_FrameXML (UI templates)
4. User addons (via TOC files)

## Phases

### Phase 1: Core Lua Environment [COMPLETE]
- [x] Embed Lua 5.1 via mlua
- [x] CreateFrame and basic widget methods
- [x] Event registration (RegisterEvent, SetScript)
- [x] Global aliases (string, math, table, bit)
- [x] Mixin system intrinsics
- [x] Security stubs (issecure, etc.)

### Phase 2: Widget API Expansion [COMPLETE]
- [x] Frame properties (alpha, strata, level, mouse)
- [x] Texture and FontString creation
- [x] Anchor system (SetPoint, GetPoint, SetAllPoints)
- [x] Parent-child relationships

### Phase 3: Addon Loading [COMPLETE]
- [x] TOC file parser
- [x] XML template parser
- [x] Load Blizzard_SharedXMLBase (100%)
- [x] Load Blizzard_SharedXML (100%)
- [ ] Load Blizzard_FrameXML (requires more APIs - Phase 5)

### Phase 4: Rendering [COMPLETE]
- [x] iced integration for visual output
- [x] Widget tree to iced elements (canvas-based)
- [x] Strata/level z-ordering with DrawLayer support
- [x] Text rendering (FontString with WoW fonts)
- [x] Texture loading (PNG, BLP formats)
- [x] Button texture rendering (normal/pushed/highlight)
- [x] Anchor cycle detection (prevents infinite recursion)

### Phase 5: Real Addon Testing [IN PROGRESS]
- [x] Load 127 addons (Ace3, WeakAuras, Details, BigWigs, etc.)
- [x] Load SavedVariables from WTF folder
- [ ] Implement missing APIs (many addons load with errors)
- [ ] Event simulation (combat log, unit info, etc.)
