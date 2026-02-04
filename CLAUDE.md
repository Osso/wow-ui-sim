# WoW UI Simulator

## WoW Game Files

- `~/Projects/wow/Interface` - code & art extract from WoW game files (2026-01-16)
- `~/Projects/wow/WTF` - SavedVariables from real WoW installation
- `~/Projects/wow/reference-addons/wow-ui-source` - Blizzard base UI (loaded before addons, not scanned as addon)

## Reference Implementations

- `~/Repos/wowless` - Headless WoW client Lua/XML interpreter (useful for understanding WoW API behavior)
  - `wowless/render.lua` - Frame-to-rect conversion with strata/level ordering
  - `wowless/modules/points.lua` - Anchor point system (SetPoint, ClearAllPoints)
  - `wowless/modules/loader.lua` - XML element handlers (anchors, texcoords, colors, gradients)
  - `data/products/wow/uiobjects.yaml` - Full Frame/Texture/Button API definitions
- `~/Repos/wow-ui-schema` - Official UI.xsd schema (62KB) for XML validation

## Rendering Order

**Frame Strata** (low to high): `BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < FULLSCREEN_DIALOG < TOOLTIP`

**Draw Layers** within frames: `BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT`
- Textures render first, then FontStrings (text always above textures in same layer)
- Overlapping textures in same layer have undefined order

**Texture Coordinates**: 8 values - `tlx, tly, blx, bly, trx, try, brx, bry` (top-left, bottom-left, top-right, bottom-right)

## Performance

Uses **Lua 5.1** via mlua (WoW's Lua version).

### Running the Simulator

**Always use `timeout 20` or less** when running the simulator to prevent hung processes:
```bash
WOW_SIM_NO_SAVED_VARS=1 WOW_SIM_NO_ADDONS=1 timeout 15 cargo run --bin wow-ui-sim
```

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

- `BetterWardrobe/ColorFilter.lua` has very large constant tables (works in WoW's patched LuaJIT)

## Textures

### Texture Sources (in order of priority)

1. `./textures` - Local WebP textures (fastest, smallest)
2. `~/Repos/wow-ui-textures` - wow-ui-textures repo (PNG versions of WoW textures)
3. `~/Projects/wow/Interface` - Extracted WoW game files (BLP format)
4. Addon directories - For addon-specific textures

### Texture Path Resolution

WoW paths like `Interface\\Buttons\\UI-Panel-Button-Up` are resolved by:
1. Normalizing backslashes to forward slashes
2. Trying extensions: webp, WEBP, PNG, png, TGA, tga, BLP, blp, jpg, JPG
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
