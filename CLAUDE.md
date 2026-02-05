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

## Lua + Rust Architecture

WoW frames exist in **two parallel systems** that must stay in sync:

### Rust Side (rendering)
- `widget::Frame` struct with `WidgetRegistry` HashMap
- Used for layout computation and rendering
- Parent-child via `children: Vec<u64>` and `children_keys: HashMap<String, u64>`

### Lua Side (WoW API)
- `FrameHandle` userdata with metatables
- Used for running actual addon Lua code
- Parent-child via Lua table properties: `parent.TitleContainer = frame`

### How They Connect

Each `FrameHandle` stores an `id: u64` pointing to the Rust `Frame`. Method calls like `:SetText()` use this ID to update Rust state:

```rust
methods.add_method("SetText", |_, this, text: String| {
    let mut state = this.state.borrow_mut();
    state.widgets.get_mut(this.id).text = Some(text);  // Updates Rust via ID
});
```

### Automatic Sync via `__newindex`

When Lua assigns a frame to a property (`parent.Child = frame`), the `__newindex` metamethod automatically syncs to Rust `children_keys`:

```rust
// In FrameHandle's __newindex metamethod (globals.rs)
if let Value::UserData(child_ud) = &value {
    if let Ok(child_handle) = child_ud.borrow::<FrameHandle>() {
        parent_frame.children_keys.insert(key, child_handle.id);
    }
}
```

This allows Rust methods like `SetTitle()` to find child frames via fast HashMap lookup instead of querying Lua. Test: `test_lua_property_syncs_to_rust_children_keys`.

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
- `WOW_SIM_DEBUG_ELEMENTS=1` - Show debug visualization: red borders around elements and green dots at anchor points
- `WOW_SIM_DEBUG_BORDERS=1` - Show only red borders around elements
- `WOW_SIM_DEBUG_ANCHORS=1` - Show only green dots at anchor points

### Timing Breakdown

Each addon shows timing: `(total: io=X xml=X lua=X sv=X)`
- `io` - File I/O (~3%)
- `xml` - XML parsing (~1%)
- `lua` - Lua execution (~78%)
- `sv` - SavedVariables loading (~18%)

### Known Issues

- `BetterWardrobe/ColorFilter.lua` has very large constant tables (works in WoW's patched LuaJIT)

### Button Texture Rendering

WoW buttons are **transparent by default** — `build_button_quads` renders nothing when `normal_texture` is None. Visuals come from:
- `SetNormalTexture`/`SetPushedTexture` etc. → stored in `normal_texture`/`pushed_texture` fields, state-dependent
- Child Texture widgets with custom parentKeys → render independently via `build_texture_quads`, NOT state-dependent

`SetAtlas` on a child texture propagates to the parent button's fields ONLY for standard parentKeys: `NormalTexture`, `PushedTexture`, `HighlightTexture`, `DisabledTexture`. Custom parentKeys do NOT propagate — the child renders independently.

**Button texture patterns in Blizzard UI:**

| Pattern | Example | How textures work |
|---------|---------|-------------------|
| Standard slots | UIPanelCloseButton | `<NormalTexture atlas="RedButton-Exit"/>` → propagates to button's `normal_texture` field |
| Single child texture | MinimalScrollBar Back/Forward | `<Texture parentKey="Texture"/>` → atlas set via `ButtonStateBehaviorMixin.OnLoad` → renders as child |
| Three-slice | SharedButtonSmallTemplate (Enable All) | `<Texture parentKey="Left/Right/Center"/>` → atlas set via `ThreeSliceButtonMixin.InitButton()` → children render independently |

### ThreeSliceButtonTemplate

Three-part horizontally-stretched button used for most Blizzard UI action buttons.

**Template chain:** `SharedButtonSmallTemplate` → `BigRedThreeSliceButtonTemplate` → `ThreeSliceButtonTemplate`

**Structure:** 3 child Texture widgets (parentKey "Left", "Right", "Center") + FontString for text. Center has `horizTile=true`.

**Mixin:** `ThreeSliceButtonMixin` sets atlas on children via `InitButton()` (OnLoad) and `UpdateButton()` (state changes). Atlas naming convention: `"atlasName-Left"`, `"atlasName-Right"`, `"_atlasName-Center"` (center has underscore prefix). State suffixes: `""` (normal), `"-Pressed"`, `"-Disabled"`.

**Scale logic:** `UpdateScale()` calculates scale from `buttonHeight / leftAtlasInfo.height`, applies to Left/Right, and uses `SetTexCoord()` to crop edges when button is too narrow for both.

**Key insight:** These buttons have NO `NormalTexture` set — all visuals come from child textures. The button itself must be transparent for the children to show through.

### Dump Limitations

- `--filter` matches frame **names** only, not parentKey names
- Anonymous frames (parentKey-only) show as `(anonymous)` and won't match filters
- Debug script hook: `src/main.rs` loads `/tmp/debug-scrollbox-update.lua` before GUI starts (not available in dump command)

## CLI Tools

The `wow-sim` binary provides CLI tools for interacting with a running simulator.

### Lua REPL

```bash
wow-sim lua                    # Interactive Lua REPL
wow-sim lua -e "print('hi')"   # Execute code and exit
wow-sim lua -l                 # List running servers
```

### Screenshot (Standalone)

Render the UI to an image file without starting the GUI (headless GPU, same shader pipeline as the live renderer). Text is not rendered — this is for debugging frame layout and textures.

```bash
wow-sim screenshot                                          # Render to screenshot.png (1024x768)
wow-sim screenshot -o frame.png --filter AddonList          # Render only AddonList subtree
wow-sim screenshot --width 1920 --height 1080               # Custom resolution
wow-sim screenshot --no-addons --no-saved-vars              # Fast: skip extras
```

### Dump Frame Tree (Standalone)

Load UI and dump the frame tree without starting the GUI (for debugging):

```bash
wow-sim dump --no-addons --no-saved-vars           # Fast: skip addons and saved vars
wow-sim dump --filter ScrollBar                    # Filter by name substring
wow-sim dump --visible-only                        # Show only visible frames
wow-sim dump                                       # Full load with all addons
```

Output shows frame hierarchy with dimensions:
```
AddonList [Frame] (600x550) hidden
  AddonListBg [Texture] (0x0) visible
  AddonListCloseButton [Button] (24x24) visible
```

### Dump Frame Tree (Connected)

Dump the rendered frame tree from a running simulator:

```bash
wow-sim dump-tree                      # Dump all frames
wow-sim dump-tree --filter Button      # Filter by name substring
wow-sim dump-tree --visible-only       # Show only visible frames
```

Output shows frame hierarchy with absolute screen coordinates and dimensions:
```
AddonList [Button] (x=50, y=400, w=80, h=22) visible
  CancelButton [Button] (x=430, y=508, w=80, h=22) visible
    Text [FontString] (x=430, y=508, w=80, h=22) visible text="Cancel"
```

### Convert Texture (BLP to WebP)

Convert a single BLP texture to WebP format:

```bash
wow-sim convert-texture ~/Projects/wow/Interface/BUTTONS/redbuttons.BLP -o ./textures/buttons/redbuttons.webp
```

### Extract Textures (Batch)

Extract all textures referenced by addons to WebP format:

```bash
wow-sim extract-textures                    # Use default paths
wow-sim extract-textures --output ./tex     # Custom output directory
```

This scans addon XML/Lua files for texture references and converts them from BLP to WebP.

## Textures

### Texture Sources (in order of priority)

1. `./textures` - Local WebP textures (fastest, smallest, ~45MB for ~1740 files)
2. `~/Projects/wow/Interface` - Extracted WoW game files (BLP format, fallback)
3. Addon directories - For addon-specific textures

### Adding Missing Textures

**Always convert from BLP game files**, never from `~/Repos/wow-ui-textures`. The `wow-ui-textures` PNG repo has textures at different resolutions than the BLP game files (e.g. `UIFrameMetal2x` is 1024x1024 in the PNG repo but 512x512 in the BLP). Using the wrong resolution causes nine-slice pieces to sample incorrect regions, rendering the wrong part of the texture.

```bash
# Find the BLP file (case-insensitive)
find ~/Projects/wow/Interface -iname "UIFrameMetal2x.BLP"

# Convert to webp (lowercase filename, matching existing convention)
wow-sim convert-texture ~/Projects/wow/Interface/FrameGeneral/UIFrameMetal2x.BLP \
  -o ./textures/framegeneral/uiframemetal2x.webp
```

Naming convention: lowercase directory and filename, no hyphens where the original BLP name has none (e.g. `UIFrameMetal2x.BLP` → `uiframemetal2x.webp`).

### Texture Path Resolution

WoW paths like `Interface\\Buttons\\UI-Panel-Button-Up` are resolved by:
1. Normalizing backslashes to forward slashes
2. Stripping `Interface/` prefix
3. Trying extensions: webp, WEBP, PNG, png, TGA, tga, BLP, blp, jpg, JPG
4. Case-insensitive directory matching as fallback

### Button Textures

Standard WoW button textures in `Interface/Buttons/`:
- `UI-Panel-Button-Up` - Normal state (128x32, dark red gradient with gold border)
- `UI-Panel-Button-Down` - Pressed state
- `UI-Panel-Button-Highlight` - Hover state
- `UI-Panel-Button-Disabled` - Disabled state
