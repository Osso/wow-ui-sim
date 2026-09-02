# XML Template System

WoW UI definitions declared in XML are parsed into typed Rust structs, then converted to widgets by generating and executing Lua code. Virtual frames become reusable templates stored in a global registry and applied when frames inherit from them.

## XML Format and Parsing

Every WoW XML file has a `<Ui>` root deserializing into `UiXml { elements: Vec<XmlElement> }` via `quick_xml` serde. Tag names map to enum variants via `#[serde(rename_all = "PascalCase")]`.

**XmlElement** (30+ variants): Frame-like widgets (Frame, Button, CheckButton, EditBox, ScrollFrame, Slider, StatusBar, GameTooltip, ModelScene, etc.) all map to `FrameXml`. Regions: `Texture(TextureXml)`, `FontString(FontStringXml)`. File refs: `Script`, `Include` (both with lowercase variants). Font defs: `Font`, `FontFamily`. Container: `ScopedModifier` (transparent wrapper).

**FrameXml** key attributes: `name`, `parent`, `parentKey` (property on parent), `inherits` (comma-separated templates), `mixin`, `virtual/intrinsic` (template-only), `toplevel`, `hidden`, `alpha`, `setAllPoints`, `enableMouse`, `parentArray` (appends to parent array). Child elements via `FrameChildElement`: Size, Anchors, Layers, Frames, Scripts, Animations, button-specific textures, widget-specific fields. XML intrinsic widget types prepend their intrinsic template to the explicit `inherits` chain.

## Template Registry

Virtual/intrinsic frames are registered in a process-global `OnceLock<RwLock<HashMap<String, TemplateEntry>>>` and not instantiated:

```rust
pub struct TemplateEntry { pub name: String, pub widget_type: String, pub frame: FrameXml }
```

A separate registry holds virtual texture templates for mixin chain resolution.

## Inheritance Chain Resolution

`get_template_chain(names: &str) -> Vec<TemplateEntry>` splits comma-separated names and recursively follows each template's own `inherits`, depth-first with cycle detection. Returns base-to-derived order — for `inherits="A, B"` where A inherits C: chain is `[C, A, B]`.

Property resolution per template walk:
- **Size**: most-derived wins per dimension; frame's own overrides all. Partial XML sizes (`<Size x="..."/>` or `<Size y="..."/>`) apply only the declared dimension.
- **Anchors**: frame's own if present; otherwise most-derived template with anchors
- **Mixins**: accumulated base-to-derived, then frame's own (duplicates skipped)
- **KeyValues**: later values overwrite; frame's own applied last
- **Hidden**: first template with a value wins (break on hit)

## XML-to-Widget Conversion (`src/loader/xml_frame.rs`)

`create_frame_from_xml()` pipeline for each non-virtual frame:
1. Virtual/intrinsic check — register template and return early
2. Name resolution — `$parent` substitution, `__anon_{id}` for anonymous children
3. Build Lua code. Ordinary frames use `CreateFrame(type, name, parent, inherits)`; XML definitions for engine-created roots `UIParent` and `WorldFrame` reuse the existing global frame instead of creating a replacement. Both paths then append `Mixin()`, `SetSize()`, `SetPoint()`, `Hide()`, `EnableMouse()`, `SetScript()`, and event-registration configuration against that frame object.
4. Execute single `env.exec()` call
5. Recurse into `<Frames>` children, then `<Layers>` (textures/fontstrings)
6. Apply animation groups, button textures, button text
7. Fire lifecycle scripts: OnLoad, then OnShow if visible

The `inherits` parameter in `CreateFrame()` triggers `apply_templates_from_registry()` at runtime, so template children are created before the XML loader recurses into direct children. The root reuse branch is required because UIParent and WorldFrame are created before Blizzard XML loads; their XML scripts, event registrations, mixins, and lifecycle configuration must target the pre-created objects later observed through `_G.UIParent` and `_G.WorldFrame`. A duplicate object strands those behaviors on the original root while later code observes another object, removing UIParent startup handlers and blocking CombatLog runtime loading. The behavior is implemented in `src/loader/xml_frame_codegen.rs` (commit `e5089fbeb2`).

Top-level XML frames with `toplevel="true"` retain the implicit UIParent used during creation instead of being immediately orphaned. If their `OnLoad` calls `SetParent(UIParent)`, that is treated as a same-parent operation, so a non-fixed XML `frameStrata` such as `HIGH` survives the lifecycle callback. Without this preservation, the reparent resets the effective strata to `MEDIUM`; `SettingsPanel` is the covered retail case (commit `c2d26f5c5`).

## Lua-Side Template Application (`src/lua_api/globals/template/mod.rs`)

Called from `CreateFrame()` at runtime (no `LoaderEnv` access). `apply_single_template()` order: Mixin → Size → Anchors → SetAllPoints → KeyValues → Layers → button textures → child frames → Scripts. OnLoad for ALL template-created children is deferred until after the entire chain is applied. XML frame KeyValues are passed through the CreateFrame template initializer when present, so template child OnLoad handlers see those values before they fire.

## Partitioned 12.1 Mixins

PTR 12.1 adds AuraContainer XML that wraps frames in `<ScopedModifier useForbiddenObjectTable="true">` and applies secure mixins with `targetPartition`, `inboundPartition`, and `secureDelegates` attributes.

The simulator models this as two Lua object partitions per frame:
- **public** — the normal frame object exposed to addon/public code
- **forbidden** — a per-frame table returned by `GetForbiddenObjectTable(frame)`

When `useForbiddenObjectTable` is active during XML loading, frame-local `<KeyValues>` are written to the forbidden table rather than the public frame. A mixin with `targetPartition="public"` installs its fields on the public frame. A secure mixin without an explicit target inside that scope installs on the forbidden table. Function values with `inboundPartition="forbidden"` are exposed as delegates: calls through the public method replace `self` with the forbidden table before invoking the secure/source function. `secureDelegates="true"` uses the same delegate path for public secure-source methods.

This is intentionally simulator-side compatibility behavior, not a Blizzard source fallback. It keeps private AuraContainer state off the public frame while allowing public inbound methods to operate on the private/forbidden state.

## Inline Scripts

Three `ScriptBodyXml` forms:
- `function="X"` — uses X directly
- `method="X"` — binds the method function after XML composition, before lifecycle execution
- Inline body — wraps as `function(self, ...) <body> end`

For `method="X"`, live PTR 12.1 probing showed two separate stores: the object field (`frame.X`) and the script handler returned by `GetScript`. XML method binding installs the currently composed method function as the script handler. Later `frame.X = otherFunction` changes direct `frame:X()` calls but does not change `GetScript`; later `SetScript` changes `GetScript` but does not change `frame.X`. `__wow_bind_xml_method` resolves the public frame first, then the forbidden object table when `useForbiddenObjectTable="true"`; private handlers receive the forbidden self, whose missing frame methods forward to the public `FrameHandle`. This covers precompiled intrinsic `OnLoad` as well as ordinary XML handlers, matching `Blizzard_AuraContainer.xml`'s private `OnLoad_Intrinsic`/`OnEvent_Intrinsic` pattern.

Intrinsic default scripts use the precall binding; ordinary XML scripts use the normal binding unless `intrinsicOrder` requests precall or postcall. Dispatch visits precall, normal, then postcall, while `GetScript(name)` without a binding argument returns only the normal handler. This distinction is required when checking an intrinsic handler alongside a derived style handler.

`inherit="prepend"` or `"append"` chains new/existing handlers, both wrapped in `pcall`. Without `inherit`, new handler replaces old.

## Name Substitution and parentKey

`$parent` in frame names resolves to the actual parent name. `$parent.ScrollBox` in `relativeKey` resolves to `parent["ScrollBox"]`; `$parent.$parent.X` chains via `GetParent()`.

`parentKey="Title"` produces `parent.Title = frame`. `parentArray="Buttons"` appends to `parent.Buttons`. Both resolved via template inheritance.

## Sources

- [xml-template-system.md](../../xml-template-system.md) — XML types, registry, inheritance, conversion pipeline, inline scripts
- `src/lua_api/env_init/shared_bootstrap.lua` — `GetForbiddenObjectTable` and XML partition helper functions
- `src/loader/xml_frame_codegen.rs` — XML codegen for partition-aware Mixins, KeyValues initialization before template-child OnLoad, engine-root reuse, and top-level implicit-parent preservation
- `src/loader/helpers_anim.rs` — animation-group code generation; mixins are applied before XML method-script binding and OnLoad
- `src/xml/parse.rs` — parser handling that preserves sibling elements after inline `<Scripts>...</Scripts>` blocks
- `src/loader/addon.rs` — shared addon loading transaction used while XML files execute
- `src/loader/tests/runtime_template_misc.rs` — regression coverage for forbidden object tables, secure delegates, and XML `method=` binding timing

## See Also

- [[addon-loading]] — TOC parsing, per-file XML/Lua loading, and idempotent loaded-addon handling that feeds this system
- [[widget-system]] — WidgetType and Frame structs produced by XML conversion
- [[frame-data-flow]] — Mixin() application, __frame_fields storage, and script dispatch
- [[dropdown-intrinsic-script-chain]] — intrinsic dropdown binding and derived style-handler order
