# WoW UI Simulator - Plan

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
