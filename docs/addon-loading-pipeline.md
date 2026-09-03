# Addon/XML Loading Pipeline

## Addon Discovery & TOC File Parsing

### Addon Discovery
**File:** `src/loader/mod.rs`

```rust
pub fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    // Priority: {AddonName}_Mainline.toc > {AddonName}.toc > first non-Classic .toc
}
```

Prefers Mainline variants for retail WoW compatibility. Game startup selects eligible non-LoD `Blizzard_*` roots, their hard TOC dependency closure, and explicit LoD startup roots. `Blizzard_Game` depends on `Blizzard_TimeManager`, `Blizzard_CooldownBroadcaster`, `Blizzard_BoostTutorial`, and `Blizzard_CombatLog`; CombatLog's declared base and processor dependencies load first, publishing `CombatLog_LoadUI` before `PLAYER_LOGIN`. `Blizzard_MacroUI` and `Blizzard_TrainerUI` are standalone Game-only LoD roots that publish `MacroFrame_LoadUI` and `ClassTrainerFrame_LoadUI`; they remain LoD and excluded from glue screens. `Blizzard_AchievementUI` is also a standalone Game-only LoD root: its full TOC publishes `AchievementFrame_LoadUI` before `AlertFrame` handles `ACHIEVEMENT_EARNED`, preserving the real achievement-toast queue path while remaining excluded from glue screens. `Blizzard_Transmog` is an explicit Game-only LoD root whose full TOC publishes `Transmog_LoadUI` from `Blizzard_Transmog_Bootstrap.lua` and registers the Transmogrifier interaction; this does not add a bootstrap-only pass. `Blizzard_RaidUI` is selected as an LoD startup root and depends on `Blizzard_RaidFrame`, so RaidFrame loads first. Other LoD addons—including non-Blizzard `Deprecated_PaperDoll`—remain excluded. `[Bootstrap]` remains an inline TOC annotation, not a discovery trigger or file-order override.

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
- Splits `[AllowLoadGameType]` values on commas or whitespace, then keeps files matching the active profile (for example, `vanilla tbc mainline` includes retail `mainline`)
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

### Removed API compatibility

Retail 12.1 removes public `GetInventorySlotInfo`. Current `Blizzard_TransmogShared` calls `C_PaperDollInfo.GetInventorySlotInfo` directly, so its loader needs no legacy-global compatibility scope. The legacy global remains unavailable to general addon code; see [[transmog-inventory-slot-scope]] for the retired stale-source workaround.

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

Blizzard startup discovery is root-and-closure based, then topologically sorted from TOC dependencies plus simulator implicit startup dependencies. Eligible non-LoadOnDemand cache addons whose directory names start with `Blizzard_` are roots, subject to screen/profile exclusions. The candidate pool also retains eligible LoadOnDemand Blizzard TOCs and eligible non-Blizzard TOCs so a selected root can pull its transitive hard `## Dependencies:` closure.

A non-Blizzard cache directory is not an eager root: it loads only when a retained Blizzard root requires it. Current retail `Blizzard_TutorialManager` demonstrates this: its `middleclass` dependency loads before the root. Conversely, unrelated non-Blizzard directories such as `Deprecated_PaperDoll` remain excluded. LoadOnDemand Blizzard addons remain out of the root set unless reached by the closure or named as an implicit startup-dependency key; such LoD keys are selected as startup roots and load complete TOCs in dependency order. `Blizzard_CombatLog` remains LoadOnDemand in TOC metadata, but `Blizzard_Game` selects it to publish `CombatLog_LoadUI` before `PLAYER_LOGIN`; `Blizzard_CombatLogBase` and `Blizzard_CombatLogProcessor` precede it through declared dependencies. Standalone Game-only root `Blizzard_AchievementUI` remains LoadOnDemand and glue-excluded, but its full TOC publishes `AchievementFrame_LoadUI` before `AlertFrame` receives `ACHIEVEMENT_EARNED` and enters the real achievement-toast queue. `[Bootstrap]` neither selects every LoD addon nor changes file order.

Foundational SharedXML addons are promoted to `LoadFirst` so templates exist before other Blizzard addons instantiate frames. Third-party addons load after this Blizzard startup pass.

### Secure-library replay

`__secureenv` is separate from public `_G`, so selected Blizzard libraries are re-executed there after normal loading instead of generically mirroring globals. The allowlist includes `Blizzard_FrameXMLUtil`: secure `Blizzard_AuraContainer` needs its `AuraUtil.DefaultAuraCompare` and `AuraUtil.UnitFrameDebuffComparator`. Before commit `93761fdb4`, public-only `AuraUtil` left secureenv stale, aborted TargetFrame aura initialization, and prevented subsequent `FocusFrame` creation. It also includes `Blizzard_RaidWarning`: secure `Blizzard_PrivateAurasUI` resolves its declared dependency's `RaidWarningUtil` through `__secureenv` for secure XML `RaidWarningUtil.MessageType` access. Focused coverage: `loader::tests::lua_loading::blizzard_frame_xml_util_replays_aura_comparators_into_secure_environment` and `tests/blizzard_private_auras_ui_loads.rs::blizzard_private_auras_ui_reads_raid_warning_util_and_publishes_mixins_into_secure_env`.

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

Recoverable loader/XML/Lua failures are returned in `LoadResult.warnings`; fatal loader errors return `Err(LoadError)`. Regular nil-global observations and missing `C_*` requirements use separate typed `LoadResult` fields.

### Nil-Symbol Diagnostic Reconciliation

Implementation: `src/loader/addon/nil_symbol_reports.rs`, `src/lua_api/globals/compat_overrides.rs`, and `src/lua_api/globals/create_frame/helpers_shared.rs`.

Nil-symbol diagnostics remain strict and typed. A missing regular global reached through direct `GETGLOBAL`/slot fallback becomes a `NilSymbolObservation` in `LoadResult.nil_symbol_observations`; every missing `C_*` namespace or member becomes a `MissingRequirement` in `LoadResult.missing_requirements`. Both retain addon, source, line, and public/secure environment attribution and are deduplicated by environment and symbol kind. They remain visible through loader tracing and `WOW_SIM_DEBUG_NIL_GLOBALS`, but do not make otherwise-successful startup unhealthy. `LoadResult.warnings` is reserved for actual loader/XML/Lua/runtime failures.

Explicit `_G.name` and `_G[name]` reads of missing regular globals are ordinary optional probes: they do not create a nil-symbol record or enter the `__wow_logged_nil_symbols` dedup cache. Explicit `_G` access does not relax `C_*` diagnostics; missing `C_*` namespaces and members remain typed requirements through their namespace/member paths. A non-`C_*` global read as nil is reconciled only when that same addon explicitly publishes the name later into the same environment through ordinary Lua assignment or a named XML frame, and the final value is non-nil. Public and secure publications use separate ledgers and final-table checks: secure Lua assignments and Rust secure frame exports record the stable addon index in the secure ledger, so a secure publication cannot resolve a public lookup or vice versa. A nested `C_AddOns.LoadAddOn` publication belongs to the nested addon and does not resolve the outer addon's observation. Globals later cleared remain observations. Both publication ledgers are cleaned up with the `LoadingAddonGuard` transaction lifecycle.

rilua tracks lookup origin in VM execution state. `debug.isglobalindex()` is a read-only query that returns true only while `_G.__index` handles a syntactic global load and false for explicit table reads or calls outside that lookup; the state restores correctly across nested lookups, errors, and coroutine swaps.

Generated lifecycle helpers avoid synthetic observations: precompiled OnLoad/OnShow dispatch snapshots and restores raw `_G.self`, while post-cleanup runtime-surface restoration reads raw `_G.C_StoreSecure` before merging the namespace. This keeps helper state restoration and simulator-owned namespace repair out of client nil-symbol attribution. Nested runtime-addon loads finalize all three diagnostic channels under the nested addon, then forward them exactly once to the immediate parent `LoadResult`; forwarding is transitive for nested-nested loads and does not reprocess raw nil-access records. Top-level runtime addon diagnostics remain in a typed SimState ledger until startup/test collectors drain them exactly once. The publication recorder used by the global assignment hook is captured in a bootstrap-local upvalue and removed from `_G` before addon code runs, so addon Lua cannot forge publication-ledger entries; ordinary Lua assignments, secure assignments, and named XML frame publications remain tracked.

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
2. **Blizzard Addons**: Discover eligible `Blizzard_*` non-LoD roots, explicit LoD startup roots, and their transitive hard TOC dependencies from the candidate pool; load every selected TOC in dependency order
3. **Third-Party Addons**: Scan `./Interface/AddOns`, load alphabetically
4. **Post-Load Scripts**: Execute global initialization and reconcile replacement `_G.SettingsPanel`/`Settings` surfaces before category registration/opening
5. **Startup Events**: Fire `ADDON_LOADED`, hide runtime-hidden frames
6. **GUI/Dump/Screenshot**: Launch interactive UI, dump frame tree, or render screenshot
