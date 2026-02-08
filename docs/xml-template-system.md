# XML Parsing & Template System

## Overview

WoW UI definitions are declared in XML files with a `<Ui>` root element. The simulator parses these files using `quick_xml` serde deserialization into Rust structs, then generates and executes Lua code to create the corresponding widgets. Virtual frames (templates) are stored in a global registry and applied when frames inherit from them via the `inherits` attribute or when `CreateFrame()` is called with a template name.

The system spans four layers:

1. **XML parsing** -- `src/xml/` deserializes XML text into typed Rust structs
2. **Template registry** -- `src/xml/template.rs` stores virtual frames for later instantiation
3. **XML-to-widget loader** -- `src/loader/xml_*.rs` walks parsed XML and emits Lua code during addon loading
4. **Lua-side template application** -- `src/lua_api/globals/template/` applies templates when `CreateFrame()` is called at runtime

---

## XML File Format

### Ui Root Element
**File:** `src/xml/types.rs:11-16`

Every WoW XML file has a `<Ui>` root element containing a flat list of top-level elements:

```xml
<Ui>
    <Script file="MyAddon.lua"/>
    <Frame name="MyFrame" parent="UIParent" inherits="BackdropTemplate">
        <Size x="200" y="100"/>
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame>
</Ui>
```

```rust
#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "Ui")]
pub struct UiXml {
    #[serde(rename = "$value", default)]
    pub elements: Vec<XmlElement>,
}
```

### XmlElement Enum
**File:** `src/xml/types.rs:26-95`

Top-level elements inside `<Ui>` are deserialized into `XmlElement` variants. The enum uses `#[serde(rename_all = "PascalCase")]` for XML tag name matching. Key variant categories:

- **Frame-like widgets** (30+ variants): `Frame`, `Button`, `CheckButton`, `EditBox`, `ScrollFrame`, `Slider`, `StatusBar`, `GameTooltip`, `ModelScene`, etc. -- all map to `FrameXml`
- **Regions**: `Texture(TextureXml)`, `FontString(FontStringXml)`
- **File references**: `Script(ScriptXml)` / `Include(IncludeXml)` -- both have case-insensitive lowercase variants (`ScriptLower`, `IncludeLower`)
- **Font definitions**: `Font(FontXml)`, `FontFamily(FontFamilyXml)`
- **Containers**: `ScopedModifier(ScopedModifierXml)` -- transparent wrapper that allows grouping elements
- **Text content**: `Text(String)` -- catches malformed XML or comments

All frame-like widget types share the same `FrameXml` struct. The XML tag name (e.g., `<Button>` vs `<Frame>`) determines the `WidgetType` used during creation.

### FrameElement Enum
**File:** `src/xml/types_elements.rs:186-234`

Child frames inside `<Frames>` containers use a parallel enum `FrameElement` with the same variants. This exists because serde needs distinct types for elements inside `<Ui>` (which also contain `Script`/`Include`) vs elements inside `<Frames>` (which only contain frame-like widgets).

---

## FrameXml: The Core Parsed Structure

**File:** `src/xml/types.rs:98-304`

`FrameXml` represents any frame-like widget parsed from XML. It captures both XML attributes and child elements.

### XML Attributes

| Attribute | Field | Type | Purpose |
|-----------|-------|------|---------|
| `name` | `name` | `Option<String>` | Global name, supports `$parent` substitution |
| `parent` | `parent` | `Option<String>` | Parent frame name (default: `UIParent`) |
| `parentKey` | `parent_key` | `Option<String>` | Property name on parent (e.g., `parent.Title = frame`) |
| `inherits` | `inherits` | `Option<String>` | Comma-separated template names |
| `mixin` | `mixin` | `Option<String>` | Comma-separated mixin table names |
| `secureMixin` | `secure_mixin` | `Option<String>` | Secure mixin (combined with `mixin`) |
| `virtual` | `is_virtual` | `Option<bool>` | Template-only, not instantiated |
| `intrinsic` | `intrinsic` | `Option<bool>` | Engine intrinsic template (treated like virtual) |
| `hidden` | `hidden` | `Option<bool>` | Start hidden |
| `alpha` | `alpha` | `Option<f32>` | Initial alpha |
| `setAllPoints` | `set_all_points` | `Option<bool>` | Fill parent |
| `enableMouse` | `enable_mouse` | `Option<bool>` | Accept mouse input |
| `text` | `text` | `Option<String>` | Button text (localization key or literal) |
| `parentArray` | `parent_array` | `Option<String>` | Append to parent's array property |
| `id` | `xml_id` | `Option<i32>` | Numeric ID (SetID) |

### Child Elements via FrameChildElement
**File:** `src/xml/types.rs:307-375`

Child elements are collected into `children: Vec<FrameChildElement>` using serde's `$value` pattern. The `FrameChildElement` enum captures:

- **Layout**: `Size`, `Anchors`
- **Content**: `Layers` (textures/fontstrings), `Frames` (child frames), `Scripts`, `Animations`
- **Data**: `KeyValues`, `Attributes`
- **Button-specific**: `NormalTexture`, `PushedTexture`, `DisabledTexture`, `HighlightTexture`, `CheckedTexture`, `DisabledCheckedTexture`, `ButtonText`, `NormalFont`, `HighlightFont`, `DisabledFont`
- **Widget-specific**: `ScrollChild`, `ThumbTexture`, `BarTexture`, `BarColor`, `Backdrop`, `ResizeBounds`, `HitRectInsets`, `TextInsets`, `PushedTextOffset`
- **Catch-all**: `Unknown` via `#[serde(other)]`

`FrameXml` provides accessor methods for each child type: `size()`, `anchors()`, `scripts()`, `layers()`, `all_frame_elements()`, `normal_texture()`, `button_text()`, etc.

---

## TextureXml and FontStringXml

### TextureXml
**File:** `src/xml/types_elements.rs:56-100`

Represents `<Texture>` elements inside layers or as button texture children. Key attributes:

| Attribute | Field | Purpose |
|-----------|-------|---------|
| `file` | `file` | Texture file path (e.g., `Interface\Buttons\UI-Panel-Button-Up`) |
| `atlas` | `atlas` | Atlas name (e.g., `RedButton-Exit`) |
| `useAtlasSize` | `use_atlas_size` | Auto-size from atlas dimensions |
| `horizTile` / `vertTile` | `horiz_tile` / `vert_tile` | Tiling mode |
| `alphaMode` | `alpha_mode` | Blend mode (e.g., `ADD`) |
| `hidden` | `hidden` | Start hidden |
| `inherits` | `inherits` | Inherit from virtual texture template |
| `mixin` | `mixin` | Mixin table names |
| `virtual` | `is_virtual` | Template-only |

Also contains nested `Size`, `Anchors`, `Color`, `Animations`, and `Scripts` elements.

### FontStringXml
**File:** `src/xml/types_elements.rs:102-140`

Represents `<FontString>` elements. Key attributes: `name`, `parentKey`, `inherits`, `text`, `justifyH`, `justifyV`, `hidden`, `wordwrap`, `maxLines`. Contains nested `Size`, `Anchors`, `Color`, `Shadow`, and `Scripts`.

---

## XML Parsing

### Parse Functions
**File:** `src/xml/parse.rs:1-44`

Parsing is thin -- it delegates entirely to `quick_xml` serde deserialization:

```rust
pub fn parse_xml(xml: &str) -> Result<UiXml, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

pub fn parse_xml_file(path: &Path) -> Result<UiXml, XmlLoadError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_xml(&contents)?)
}
```

`XmlLoadError` wraps both `std::io::Error` and `quick_xml::DeError`.

All XML structure knowledge lives in the serde annotations on the type definitions. The `@` prefix marks XML attributes (`@name`), `$value` collects mixed child elements, and `$text` captures text content. `rename_all = "PascalCase"` maps Rust variants to XML tag names.

### Processing Flow
**File:** `src/loader/xml_file.rs:17-73`

After parsing, `load_xml_file()` iterates the `UiXml.elements` list and dispatches each element:

1. `Script` / `ScriptLower` -- Load a Lua file from `file` attribute, or execute inline code from text content
2. `Include` / `IncludeLower` -- Recursively load another XML or Lua file
3. `Font` -- Create a Lua font object table with `GetFont`/`SetFont` methods
4. `FontFamily` -- Create a Lua font family object table
5. `ScopedModifier` -- Recursively process wrapped child elements
6. **All others** -- Dispatch to `resolve_frame_element()` which maps the `XmlElement` variant to a `(FrameXml, widget_type)` pair, then calls `create_frame_from_xml()`

---

## Template System

### Virtual Frames and Intrinsics

A frame with `virtual="true"` or `intrinsic="true"` is not instantiated. Instead, it is registered in the global template registry for later use.

**File:** `src/loader/xml_frame.rs:19-25`

```rust
if frame.is_virtual == Some(true) || frame.intrinsic == Some(true) {
    if let Some(ref name) = frame.name {
        crate::xml::register_template(name, widget_type, frame.clone());
    }
    return Ok(None);
}
```

The same applies to virtual textures:

**File:** `src/loader/xml_texture.rs:121-127`

```rust
if texture.is_virtual == Some(true) {
    if let Some(ref name) = texture.name {
        register_texture_template(name, texture.clone());
    }
    return Ok(());
}
```

### Template Registry (Frame Templates)
**File:** `src/xml/template.rs:1-135`

The frame template registry is a process-global `HashMap<String, TemplateEntry>` behind `OnceLock<RwLock<...>>` for thread-safe lazy initialization:

```rust
fn template_registry() -> &'static RwLock<HashMap<String, TemplateEntry>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, TemplateEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}
```

Each entry stores:

```rust
pub struct TemplateEntry {
    pub name: String,
    pub widget_type: String,   // e.g., "Frame", "Button"
    pub frame: FrameXml,       // Full parsed XML, including children
}
```

Key operations:
- `register_template(name, widget_type, frame)` -- Insert a template (line 22)
- `get_template(name)` -- Lookup by name, returns `Option<TemplateEntry>` cloned (line 35)
- `get_template_chain(names)` -- Resolve full inheritance chain (line 94)
- `get_template_info(name)` -- Get type + resolved size for `C_XMLUtil.GetTemplateInfo` (line 48)
- `clear_templates()` -- Reset for testing (line 132)

### Texture Template Registry
**File:** `src/xml/template.rs:137-184`

A separate registry for virtual textures, used to resolve mixin chains:

```rust
fn texture_template_registry() -> &'static RwLock<HashMap<String, TextureXml>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, TextureXml>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}
```

`collect_texture_mixins(texture)` walks the texture's `inherits` chain through this registry, collecting all mixin names from parent templates plus the texture's own `mixin` attribute.

---

## Template Inheritance Chains

### How Inheritance is Resolved
**File:** `src/xml/template.rs:94-128`

The `inherits` attribute is a comma-separated list of template names. `get_template_chain()` resolves the full chain by recursively following each template's own `inherits`:

```rust
pub fn get_template_chain(names: &str) -> Vec<TemplateEntry> {
    // For each comma-separated name, recursively collect parent templates first
    // Returns base-to-derived order (most base first, most derived last)
}
```

`collect_template_chain()` handles the recursion with cycle detection via a `HashSet<String>`:

```rust
fn collect_template_chain(name: &str, chain: &mut Vec<TemplateEntry>, visited: &mut HashSet<String>) {
    if visited.contains(name) { return; }
    visited.insert(name.to_string());
    if let Some(entry) = get_template(name) {
        // First: recurse into parent templates
        if let Some(ref inherits) = entry.frame.inherits {
            for parent in inherits.split(',') {
                collect_template_chain(parent.trim(), chain, visited);
            }
        }
        // Then: add this template
        chain.push(entry);
    }
}
```

**Resolution order**: base-to-derived. For `inherits="A, B"` where A inherits C:
- Chain = `[C, A, B]`
- Properties are applied in this order, so most-derived values win

### Property Resolution Pattern

Throughout the codebase, properties are resolved by walking the chain and keeping the last value seen (most-derived wins). Some properties use "first found" (e.g., hidden), others accumulate (e.g., mixins). The pattern in `src/loader/xml_frame.rs`:

**Size** (line 194-214): Walk chain base-to-derived, update width/height. Frame's own size applied last.

**Anchors** (line 230-243): Frame's own anchors take priority. If none, the most-derived template with anchors is used (chain walked in reverse).

**Hidden** (line 246-263): First template with a hidden value wins (chain walked forward, `break` on first hit).

**Mixins** (line 159-180): Accumulated from all templates (base first), then frame's own mixins appended. Duplicates are skipped.

**KeyValues** (line 333-340): Applied from all templates (base first), then frame's own values. Later values overwrite earlier ones for the same key.

---

## XML-to-Widget Conversion

### Frame Creation Pipeline
**File:** `src/loader/xml_frame.rs:12-73`

`create_frame_from_xml()` is the main entry point for converting a parsed `FrameXml` into a live widget. It returns the created frame's name (or `None` for virtual/skipped frames).

**Step-by-step flow:**

1. **Virtual check** (line 20-25): If `virtual` or `intrinsic`, register as template and return early
2. **Name resolution** (line 27-30): Apply `$parent` substitution, generate anonymous name if needed
3. **Build Lua code** (line 37-51): Generate a string of Lua statements:
   - `CreateFrame(type, name, parent, inherits)` call
   - `parentKey` assignment on parent
   - `Mixin()` calls from template chain + frame
   - `SetSize()` from template chain + frame
   - `SetPoint()` anchors
   - `Hide()`, `SetAlpha()`, `EnableMouse()`, `SetAllPoints()`
   - `KeyValue` property assignments
   - `SetAttribute()` calls from `<Attributes>`
   - `SetID()` from `id` attribute
   - `SetScript()` calls from `<Scripts>`
4. **Execute Lua code** (line 56-58): Single `env.exec()` call runs the accumulated Lua string
5. **Create child frames** (line 61): Recursively call `create_frame_from_xml()` for each `<Frames>` child
6. **Create layer children** (line 62): Create textures and fontstrings from `<Layers>`
7. **Apply animation groups** (line 63): From frame and inherited templates
8. **Apply button textures** (line 65): `NormalTexture`, `PushedTexture`, etc. from XML
9. **Apply button text** (line 66): From `text` attribute
10. **Init action bar tables** (line 68): Pre-create `actionButtons` table for action bar frames
11. **Fire lifecycle scripts** (line 70): Execute OnLoad, then OnShow if visible

### Name Resolution
**File:** `src/loader/xml_frame.rs:76-94`

```rust
fn resolve_frame_name(frame: &FrameXml, parent_override: Option<&str>) -> Option<String> {
    // Named frames: replace "$parent" with actual parent name
    // Anonymous child frames: generate "__anon_{id}" name
    // Anonymous top-level frames: return None (treated as templates)
}
```

The `$parent` placeholder is WoW's mechanism for creating uniquely-named children: `name="$parentTitle"` under parent "MyFrame" becomes "MyFrameTitle".

### CreateFrame Lua Code Generation
**File:** `src/loader/xml_frame.rs:97-118`

```rust
fn build_create_frame_code(widget_type, name, parent, inherits) -> String {
    // Special case: if name == parent, reuse existing engine frame (e.g., UIParent)
    // Otherwise: CreateFrame("type", "name", parent, "inherits")
}
```

The `inherits` parameter in `CreateFrame()` triggers `apply_templates_from_registry()` in the Lua-side template module. This means template children (frames, textures, fontstrings) are already created by the time `create_frame_from_xml()` continues to step 5. The XML loader then only creates children defined directly on the frame, not in the template.

### Texture Creation
**File:** `src/loader/xml_texture.rs:114-197`

`create_texture_from_xml()` generates Lua code calling `parent:CreateTexture(name, drawLayer)`, then applies:
- Mixin calls from texture template chain
- `SetTexture()` / `SetAtlas()` from file/atlas attributes
- `SetSize()`, `SetVertexColor()`, `SetHorizTile()`, `SetVertTile()`
- `SetAllPoints()` -- implicitly applied if no anchors defined (WoW behavior)
- `SetPoint()` from anchors
- `Hide()` if hidden
- `SetBlendMode()` from alphaMode
- MaskTexture flag (`frame.is_mask = true`) for renderer skip
- Animation groups

### FontString Creation
**File:** `src/loader/xml_fontstring.rs:91-158`

`create_fontstring_from_xml()` generates Lua code calling `parent:CreateFontString(name, drawLayer, inherits)`, then applies:
- `SetText()` with global string resolution (localization keys like `ADDON_FORCE_LOAD` resolve to "Load out of date AddOns")
- `SetJustifyH()` / `SetJustifyV()`
- `SetTextColor()`
- `SetSize()`
- `SetWordWrap()` / `SetMaxLines()`
- `SetAllPoints()`, anchors, `Hide()`
- Direct Rust state sync for text and auto-height

---

## Lua-Side Template Application (CreateFrame)

### Entry Point: CreateFrame
**File:** `src/lua_api/globals/create_frame.rs:12-48`

When `CreateFrame("type", "name", parent, "template")` is called from Lua:

1. Parse arguments (type, name, parent userdata, template string)
2. Create Rust `Frame` struct in widget registry
3. Create default children for widget type (Button gets NormalTexture/PushedTexture/etc., Slider gets ThumbTexture, etc.)
4. Create `FrameHandle` userdata and register in Lua globals
5. If template specified, call `apply_templates_from_registry(lua, name, template)`

### Template Application
**File:** `src/lua_api/globals/template/mod.rs:67-85`

```rust
pub fn apply_templates_from_registry(lua: &Lua, frame_name: &str, template_names: &str) {
    let chain = get_template_chain(template_names);
    // Apply each template in order (base to derived)
    for entry in &chain {
        let child_names = apply_single_template(lua, frame_name, entry);
        all_child_names.extend(child_names);
    }
    // Fire OnLoad for ALL children after ALL templates applied
    for child_name in &all_child_names {
        fire_on_load(lua, child_name);
    }
}
```

OnLoad firing is deferred until after the entire template chain is applied because child OnLoad handlers may depend on KeyValues or mixins from later templates in the chain.

### Single Template Application
**File:** `src/lua_api/globals/template/mod.rs:88-142`

`apply_single_template()` processes a template entry in this order:

1. **Mixin** -- `Mixin(frame, MixinTable)` calls, with special initialization for `ActionBarMixin` and `EditModeSystemMixin`
2. **Size** -- `SetSize()` from template
3. **Anchors** -- `SetPoint()` from template
4. **SetAllPoints** -- `SetAllPoints(true)` if flagged
5. **KeyValues** -- Property assignments (`frame.key = value`)
6. **Layers** -- Create textures and fontstrings from `<Layers>`
7. **Button textures** -- `NormalTexture`, `PushedTexture`, etc. via setter methods
8. **BarTexture** / **ThumbTexture** -- Widget-specific textures
9. **ButtonText** / **EditBox FontString** -- Text region children
10. **Child frames** -- Recursively create `<Frames>` children
11. **ScrollChild children** -- Create scroll child frames
12. **Scripts** -- `SetScript()` for OnLoad, OnEvent, OnClick, etc.

### Template Element Creation
**File:** `src/lua_api/globals/template/elements.rs:1-461`

This module contains functions that create child widgets (textures, fontstrings) from template XML definitions using generated Lua code. Unlike the loader-side code (which has access to `LoaderEnv`), these functions only have access to `&Lua` and must work within the Lua runtime context.

Key functions:

- `create_texture_from_template()` (line 9) -- Creates `parent:CreateTexture(name, layer)`, applies mixins, file/atlas, anchors, implicit SetAllPoints
- `create_fontstring_from_template()` (line 165) -- Creates `parent:CreateFontString(name, layer, inherits)`, applies text, justification, color, shadow
- `create_button_texture_from_template()` (line 375) -- Reuses existing button texture child or creates new one, then calls setter method (e.g., `parent:SetNormalTexture(tex)`)
- `create_bar_texture_from_template()` (line 292) -- Creates texture and calls `parent:SetStatusBarTexture(bar)`
- `create_thumb_texture_from_template()` (line 326) -- Creates texture and calls `parent:SetThumbTexture(thumb)`

---

## Inline Script Handling

### ScriptsXml Structure
**File:** `src/xml/types.rs:489-529`

The `<Scripts>` element contains handler elements for different events:

```xml
<Scripts>
    <OnLoad function="MyFrame_OnLoad"/>
    <OnShow method="OnShow"/>
    <OnClick>
        self:DoSomething()
    </OnClick>
</Scripts>
```

```rust
pub struct ScriptsXml {
    pub on_load: Vec<ScriptBodyXml>,
    pub on_event: Vec<ScriptBodyXml>,
    pub on_update: Vec<ScriptBodyXml>,
    pub on_click: Vec<ScriptBodyXml>,
    pub on_show: Vec<ScriptBodyXml>,
    pub on_hide: Vec<ScriptBodyXml>,
    // Animation group scripts:
    pub on_play: Vec<ScriptBodyXml>,
    pub on_finished: Vec<ScriptBodyXml>,
    pub on_stop: Vec<ScriptBodyXml>,
    pub on_loop: Vec<ScriptBodyXml>,
    pub on_pause: Vec<ScriptBodyXml>,
}
```

### ScriptBodyXml: Three Handler Forms
**File:** `src/xml/types.rs:517-529`

```rust
pub struct ScriptBodyXml {
    pub body: Option<String>,          // Inline Lua code
    pub function: Option<String>,      // Global function reference
    pub method: Option<String>,        // self:Method() call
    pub inherit: Option<String>,       // "prepend" or "append"
    pub intrinsic_order: Option<String>,
}
```

Handler resolution in `build_handler_expr()` (`src/loader/helpers.rs:440-451`):
- `function="MyFunc"` -- Uses the function directly: `frame:SetScript("OnLoad", MyFunc)`
- `method="OnLoad"` -- Wraps as: `function(self, ...) self:OnLoad(...) end`
- Inline body `<OnLoad>code</OnLoad>` -- Wraps as: `function(self, ...) code end`

### Script Inheritance (prepend/append)
**File:** `src/loader/helpers.rs:393-437`

The `inherit` attribute controls how a handler interacts with an existing handler:

- **No inherit** (default): Replaces the existing handler via `SetScript()`
- **`inherit="prepend"`**: New handler runs first, then the existing handler. Both wrapped in `pcall()`
- **`inherit="append"`**: Existing handler runs first, then the new handler

This is critical for templates: a derived template can prepend its own OnLoad while preserving the base template's OnLoad handler.

### Lifecycle Script Firing
**File:** `src/loader/xml_frame.rs:585-634`

After a frame is fully configured from XML, lifecycle scripts fire:

1. **OnLoad**: Checks `GetScript("OnLoad")` first (set via `SetScript`), then `frame.OnLoad` (set via mixin). Wrapped in `pcall()` to catch errors without propagating.
2. **OnShow**: Only fires if `frame:IsVisible()` is true. Same check pattern as OnLoad.

The same pattern is used in the template module's `fire_on_load()` (`src/lua_api/globals/template/mod.rs:310-332`), but only fires OnLoad (not OnShow) for template-created children.

---

## Key Architectural Details

### Lua Code Generation Strategy

Both the XML loader and the template module work by generating Lua code strings and executing them. This keeps the Lua and Rust sides in sync -- all widget creation goes through the same `CreateFrame()` / `CreateTexture()` / `CreateFontString()` Lua functions that update both the Lua-side `FrameHandle` userdata and the Rust-side widget registry.

### Two Paths for Template Application

Templates can be applied from two contexts with slightly different implementations:

| Context | Module | Has `LoaderEnv` | Creates via |
|---------|--------|-----------------|-------------|
| XML loading | `src/loader/xml_frame.rs` | Yes | `env.exec()` |
| `CreateFrame()` | `src/lua_api/globals/template/` | No | `lua.load().exec()` |

Both paths resolve the same template chain, but the template module must operate without `LoaderEnv` because `CreateFrame()` can be called from any Lua code at runtime, not just during addon loading.

### $parent Name Substitution

The `$parent` placeholder appears in frame names, anchor `relativeTo`, and `relativeKey` expressions:

- **Frame names**: `$parentTitle` under parent "MyFrame" becomes "MyFrameTitle" (`src/loader/xml_frame.rs:80-84`, `src/lua_api/globals/create_frame.rs:96-112`)
- **Anchor relativeTo**: `relativeTo="$parent"` resolves to the parent frame (`src/loader/helpers.rs:152-170`)
- **Anchor relativeKey**: `relativeKey="$parent.ScrollBox"` resolves to `parent["ScrollBox"]` via `resolve_relative_key()` (`src/loader/helpers.rs:128-145`). Chained `$parent.$parent.X` produces `parent:GetParent()["X"]`

### Anonymous Frame Naming

When a frame has no `name` attribute:
- **Top-level in XML** (no parent override): Returns `None`, treated as virtual/template (`src/loader/xml_frame.rs:90`)
- **Child frame** (has parent override): Generates `__anon_{counter}` name (`src/loader/xml_frame.rs:88`)
- **Template child frame**: Generates `__tpl_{counter}` name (`src/lua_api/globals/template/mod.rs:359`)
- **Template texture**: Generates `__tex_{counter}` (`src/lua_api/globals/template/elements.rs:20`)
- **Template fontstring**: Generates `__fs_{counter}` (`src/lua_api/globals/template/elements.rs:175`)

All use the shared atomic counter in `src/loader/helpers.rs:93-97`.

### parentKey and parentArray

`parentKey` assigns the child as a named property on its parent:
```lua
parent.Title = frame    -- from parentKey="Title"
```

`parentArray` appends the child to an array on the parent:
```lua
parent.Buttons = parent.Buttons or {}
table.insert(parent.Buttons, frame)  -- from parentArray="Buttons"
```

Both are resolved through template inheritance (`resolve_inherited_field()` in `src/lua_api/globals/template/mod.rs:441-458`): if the frame itself does not define a parentKey/parentArray, its inherited templates are checked.

### Widget Type Defaults vs Template Children

Frame creation creates two categories of children:

1. **Widget type defaults** (`src/lua_api/globals/create_frame.rs:183-202`): Intrinsic children created in Rust for every instance of a type. Button always gets NormalTexture, PushedTexture, HighlightTexture, DisabledTexture, and Text FontString slots. These exist before any template is applied.

2. **Template children**: Additional children defined in the template's XML (`<Frames>`, `<Layers>`). These are created by `apply_templates_from_registry()` after the widget type defaults.

This means a Button created with `inherits="MyButtonTemplate"` first gets its five default texture/text children, then gets any additional children defined in `MyButtonTemplate`.
