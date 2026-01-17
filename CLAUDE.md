# WoW UI Simulator

## WoW Game Files

- `~/Projects/wow/Interface` - code & art extract from WoW game files (2026-01-16)
- `~/Projects/wow/WTF` - SavedVariables from real WoW installation
- `~/Projects/wow/reference-addons/wow-ui-source` - Blizzard base UI (loaded before addons, not scanned as addon)

## Performance

Uses **LuaJIT** (same as retail WoW) for ~30% faster loading vs standard Lua 5.1.

### Environment Variables

- `WOW_SIM_NO_SAVED_VARS=1` - Skip loading WTF SavedVariables for faster startup (~18% of load time)
- `WOW_SIM_NO_ADDONS=1` - Skip loading third-party addons (for faster texture testing)

### Timing Breakdown

Each addon shows timing: `(total: io=X xml=X lua=X sv=X)`
- `io` - File I/O (~3%)
- `xml` - XML parsing (~1%)
- `lua` - Lua execution (~78%)
- `sv` - SavedVariables loading (~18%)

### Known Issues

- `BetterWardrobe/ColorFilter.lua` exceeds LuaJIT's 65536 constant limit (works in WoW's patched LuaJIT)

## Textures

### Texture Sources (in order of priority)

1. `~/Repos/wow-ui-textures` - wow-ui-textures repo (PNG versions of WoW textures)
2. `~/Projects/wow/Interface` - Extracted WoW game files (BLP format)
3. Addon directories - For addon-specific textures

### Texture Path Resolution

WoW paths like `Interface\\Buttons\\UI-Panel-Button-Up` are resolved by:
1. Normalizing backslashes to forward slashes
2. Trying extensions: PNG, png, TGA, tga, BLP, blp
3. Case-insensitive directory matching

### Button Textures

Standard WoW button textures in `Interface/Buttons/`:
- `UI-Panel-Button-Up.PNG` - Normal state (128x32, dark red gradient with gold border)
- `UI-Panel-Button-Down.PNG` - Pressed state
- `UI-Panel-Button-Highlight.PNG` - Hover state
- `UI-Panel-Button-Disabled.PNG` - Disabled state

### Current Goal: Render WoW Buttons

Making button textures render correctly via:
- `SetNormalTexture("Interface\\Buttons\\UI-Panel-Button-Up")`
- `SetPushedTexture("Interface\\Buttons\\UI-Panel-Button-Down")`
- `SetHighlightTexture("Interface\\Buttons\\UI-Panel-Button-Highlight")`

These are stored in Frame fields: `normal_texture`, `pushed_texture`, `highlight_texture`, `disabled_texture`
