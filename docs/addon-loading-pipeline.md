# Addon/XML Loading Pipeline

## Addon Discovery & TOC File Parsing

### Addon Discovery
**File:** `src/loader/mod.rs:24-54`

```rust
pub fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    // Priority: {AddonName}_Mainline.toc > {AddonName}.toc > first non-Classic .toc
}
```

Prefers Mainline variants for retail WoW compatibility.

### TOC File Parsing
**File:** `src/toc.rs:63-120`

```rust
pub struct TocFile {
    pub addon_dir: PathBuf,
    pub name: String,
    pub metadata: HashMap<String, String>,  // ## Key: Value pairs
    pub files: Vec<PathBuf>,                 // Load order (relative paths)
}
```

**Metadata** (lines 123-193):
- `Interface`: Version numbers (comma-separated)
- `Title`: Display name (defaults to directory name)
- `Dependencies` / `RequiredDeps`: Required addons
- `OptionalDeps`: Optional dependencies
- `LoadOnDemand`: Set to "1" for load-on-demand addons
- `SavedVariables`: Account-wide persistent variables
- `SavedVariablesPerCharacter`: Per-character persistent variables

**File Processing** (lines 69-104):
- Skips `#` comment lines
- Strips `[AllowLoadTextLocale]` annotations (only loads enUS)
- Skips `[AllowLoadGameType]` files unless "mainline" or "standard"
- Replaces placeholders: `[Family]` -> "Mainline", `[Game]` -> "Standard"
- Normalizes backslashes, strips inline annotations

**Case-Insensitive Path Resolution** (lines 196-240): Resolves addon file paths with case-insensitive matching for Windows/macOS compatibility.

---

## Addon Loading Flow

### Main Orchestration
**File:** `src/loader/mod.rs:91-118`

```rust
pub fn load_addon(env: &LoaderEnv<'_>, toc_path: &Path) -> Result<LoadResult>
pub fn load_addon_with_saved_vars(env, toc_path, saved_vars_mgr) -> Result<LoadResult>
```

Returns `LoadResult` with timing breakdown:
```rust
pub struct LoadResult {
    pub name: String,
    pub lua_files: usize,
    pub xml_files: usize,
    pub timing: LoadTiming,
    pub warnings: Vec<String>,
}

pub struct LoadTiming {
    pub io_time: Duration,
    pub xml_parse_time: Duration,
    pub lua_exec_time: Duration,
    pub saved_vars_time: Duration,
}
```

### Addon Context & Internal Loading
**File:** `src/loader/addon.rs:16-124`

```rust
pub struct AddonContext<'a> {
    pub name: &'a str,
    pub table: Table,           // Private Lua table for addon
    pub addon_root: &'a Path,
}
```

**File Loading Process** (lines 59-124):
1. Initialize SavedVariables (WTF first, then JSON fallback)
2. Create addon private Lua table
3. Iterate through TOC file list in order
4. For each file:
   - Check local overlay first (`./Interface/AddOns/{addon}/{file}`)
   - Fall back to addon root
   - Load `.lua` via `load_lua_file()`
   - Load `.xml` via `load_xml_file()`
   - Apply C++ mixin stubs after each `.lua` file
5. Return accumulated results and warnings

**C++ Mixin Stubs** (lines 126-152): After each `.lua` file, injects empty stubs for C++-only mixins (`ModelSceneControlButtonMixin.OnLoad`, etc.) and guards `PetActionBarMixin.Update`.

---

## Lua File Loading
**File:** `src/loader/lua_file.rs:12-42`

1. Read file with lossy UTF-8 conversion
2. Strip UTF-8 BOM if present
3. Transform path to WoW-style for debugstack: `@Interface/AddOns/...`
4. Execute with varargs: `env.exec_with_varargs(code, chunk_name, addon_name, addon_table)`
5. Time execution separately

Each addon file receives `...` = `(addonName, addonTable)`. Addons unpack as: `local E, L, C = select(2, ...):unpack()`

---

## XML File Loading & Processing

### XML Parsing
**File:** `src/xml/parse.rs:6-14`

Uses `quick_xml` (serde deserialize) to parse WoW XML files into typed structures.

### XML Element Processing
**File:** `src/loader/xml_file.rs:17-73`

**Top-Level Elements:**

| Category | Elements |
|----------|----------|
| **File Refs** | `Script`, `Include` |
| **Frames** | `Frame`, `Button`, `CheckButton`, `EditBox`, `ScrollFrame`, `Slider`, `StatusBar`, `GameTooltip`, `Model`, `ModelScene`, `MessageFrame`, `Minimap`, etc. |
| **Regions** | `Texture`, `FontString`, `LayerTexture` |
| **Containers** | `ScopedModifier` (transparent wrapper) |
| **Fonts** | `Font`, `FontFamily` |
| **Animations** | `AnimationGroup`, `Actor` |

**Processing Order** (lines 38-73):
1. Script/Include -> load file or execute inline code
2. Font/FontFamily -> create font object
3. ScopedModifier -> recurse on children
4. Everything else -> `process_frame_element()`

---

## Template System

### Template Registry
**File:** `src/xml/template.rs:7-38`

```rust
pub struct TemplateEntry {
    pub name: String,
    pub widget_type: String,
    pub frame: FrameXml,
}
```

Global static `OnceLock<RwLock<HashMap<String, TemplateEntry>>>`. Thread-safe, populated during addon loading.

### Template Inheritance Chain Resolution
**File:** `src/xml/template.rs:92-128`

`get_template_chain(names: &str) -> Vec<TemplateEntry>`

1. Split on commas, trim whitespace
2. For each template: recursively collect parent templates first (depth-first)
3. Return chain from most base to most derived

Example: Template A inherits B, B inherits C -> `get_template_chain("A")` = `[C, B, A]`

### Texture Template System (lines 138-185)

Separate registry for virtual texture templates with `register_texture_template()` and `collect_texture_mixins()`.

---

## Frame Creation from XML
**File:** `src/loader/xml_frame.rs:13-72`

### Main Flow

1. **Register Virtual/Intrinsic** (line 20-25): If `virtual="true"` or `intrinsic="true"`, register in template registry and return
2. **Resolve Frame Name** (line 27-30): Apply `$parent` substitution, generate anonymous names
3. **Build CreateFrame Code** (line 37)
4. **Append Configuration** (lines 39-49): Parent key, mixins, size, anchors, hidden, EnableMouse, SetAllPoints, KeyValues, attributes, frame ID, script handlers
5. **Execute CreateFrame** (line 55-57): Note: CreateFrame with inherits already calls `apply_templates_from_registry`
6. **Create Children** (line 60-62): Child frames, layer children, animation groups
7. **Apply Button/StatusBar Elements** (line 64-65)
8. **Fire Lifecycle Scripts** (line 69): OnLoad and other startup handlers

### Template Resolution in Frame Creation (lines 157-262)

- **Mixins**: Collected from inherited templates (base -> derived), then from frame itself
- **Size**: Traverse template chain, most derived wins, frame overrides all
- **Anchors**: Frame's own if present, otherwise most derived template with anchors
- **Other**: Hidden, EnableMouse, SetAllPoints, KeyValues -- all via template chain with frame override

### Parent Key Handling (lines 119-155)

`{parent}.{parentKey} = frame` makes frame accessible as sibling property. Also handles `parentArray` for collection access.

---

## Template Application (After CreateFrame)
**File:** `src/lua_api/globals/template/mod.rs:63-125`

`apply_templates_from_registry()` is called automatically by CreateFrame when an inherits parameter is provided.

For each template in chain: apply mixin, size, anchors, SetAllPoints, KeyValues, layers, button textures, StatusBar/Slider textures, child frames. OnLoad fired on all created children after all templates applied.

---

## Intrinsic Frames & Engine Frames

### Built-in Engine Frames
**File:** `src/lua_api/builtin_frames.rs:64-128`

Created at startup: `UIParent` (screen-sized), `WorldFrame`, `ErrorFrame`.

**Stub frames** for not-yet-loaded addons: BuffFrame, DebuffFrame, etc.

**Critical Rule:** Only engine-created frames or not-yet-loaded addon stubs belong here. Frames from BLIZZARD_ADDONS must NOT be duplicated.

### Virtual/Intrinsic Registration

When `virtual="true"` or `intrinsic="true"` is encountered during XML loading: register in template registry (not as a widget), return without creating an actual frame. Later inheritance applies the template.

---

## SavedVariables Loading
**File:** `src/saved_variables.rs:19-150`

### Priority

1. **WTF Loading** (primary): `WTF/Account/{account}/SavedVariables/{addon}.lua` and per-character variant
2. **JSON Fallback** (secondary): Initialize from TOC and store in simulator directory

### SavedVariablesManager (lines 72-150)

```rust
pub struct SavedVariablesManager {
    storage_dir: PathBuf,
    character_name: String,
    realm_name: String,
    registered: HashMap<String, Vec<String>>,
    registered_per_char: HashMap<String, Vec<String>>,
    wtf_config: Option<WtfConfig>,
    wtf_loaded: HashMap<String, bool>,
}
```

Default storage: `~/.local/share/wow-sim/SavedVariables/`

---

## Blizzard Addon Loading Order
**File:** `src/main.rs:229-278`

27 addons in hardcoded dependency order:

```
Foundation:
  SharedXMLBase -> Colors -> SharedXML -> SharedXMLGame ->
  UIPanelTemplates -> FrameXMLBase

Core:
  LoadLocale -> Fonts_Shared -> HelpPlate -> AccessibilityTemplates ->
  ObjectAPI -> UIParent -> TextStatusBar -> MoneyFrame -> POIButton ->
  Flyout -> StoreUI -> MicroMenu -> EditMode -> GarrisonBase ->
  GameTooltip -> UIParentPanelManager -> Settings_Shared ->
  SettingsDefinitions_Shared -> SettingsDefinitions_Frame ->
  FrameXMLUtil -> ItemButton -> QuickKeybind -> FrameXML

UI Panels:
  UIPanels_Game -> MapCanvasSecureUtil -> MapCanvas ->
  SharedMapDataProviders -> WorldMap -> ActionBar -> GameMenu ->
  UIWidgets -> Minimap -> AddOnList -> TimerunningUtil -> Communities
```

Third-party addons loaded alphabetically after Blizzard addons.

---

## Error Handling
**File:** `src/loader/error.rs:4-35`

```rust
pub enum LoadError {
    Io(std::io::Error),
    Toc(std::io::Error),
    Xml(crate::xml::XmlLoadError),
    Lua(String),
}
```

**Non-fatal warnings** returned in `LoadResult.warnings`. **Fatal errors** return `Err(LoadError)`.

**Path resolution fallback** (helpers.rs:52-79): Tries case-sensitive relative to XML, case-insensitive relative to XML, case-sensitive relative to addon root, case-insensitive relative to addon root.

---

## XML Types
**File:** `src/xml/types.rs`

```rust
pub struct FrameXml {
    pub name: Option<String>,
    pub parent: Option<String>,
    pub parent_key: Option<String>,
    pub inherits: Option<String>,
    pub mixin: Option<String>,
    pub hidden: Option<bool>,
    pub is_virtual: Option<bool>,
    pub intrinsic: Option<bool>,
    pub children: Vec<FrameChildElement>,
}
```

Accessors: `size()`, `anchors()`, `scripts()`, `layers()`, `all_frame_elements()`, `key_values()`.

---

## Complete Load Sequence

1. **Startup** (`main.rs`): Apply resource limits, create `WowLuaEnv`, set addon base paths, configure SavedVariables
2. **Blizzard Addons**: Load in hardcoded dependency order
3. **Third-Party Addons**: Scan `./Interface/AddOns`, load alphabetically
4. **Post-Load Scripts**: Execute global initialization
5. **Startup Events**: Fire `ADDON_LOADED`, hide runtime-hidden frames
6. **GUI/Dump/Screenshot**: Launch interactive UI, dump frame tree, or render screenshot
