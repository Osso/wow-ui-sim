# XML Template System

WoW UI definitions declared in XML are parsed into typed Rust structs, then converted to widgets by generating and executing Lua code. Virtual frames become reusable templates stored in a global registry and applied when frames inherit from them.

## XML Format and Parsing

Every WoW XML file has a `<Ui>` root deserializing into `UiXml { elements: Vec<XmlElement> }` via `quick_xml` serde. Tag names map to enum variants via `#[serde(rename_all = "PascalCase")]`.

**XmlElement** (30+ variants): Frame-like widgets (Frame, Button, CheckButton, EditBox, ScrollFrame, Slider, StatusBar, GameTooltip, ModelScene, etc.) all map to `FrameXml`. Regions: `Texture(TextureXml)`, `FontString(FontStringXml)`. File refs: `Script`, `Include` (both with lowercase variants). Font defs: `Font`, `FontFamily`. Container: `ScopedModifier` (transparent wrapper).

**FrameXml** key attributes: `name`, `parent`, `parentKey` (property on parent), `inherits` (comma-separated templates), `mixin`, `virtual/intrinsic` (template-only), `toplevel`, `hidden`, `alpha`, `setAllPoints`, `enableMouse`, `parentArray` (appends to parent array). Child elements via `FrameChildElement`: Size, Anchors, Layers, Frames, Scripts, Animations, button-specific textures, widget-specific fields. XML intrinsic widget types prepend their intrinsic template to the explicit `inherits` chain.

## Registered intrinsic-tag declarations

[Owning contract](../../specs/xml-intrinsic-tags.md) covers same-name cached intrinsic declarations at top-level and nested XML tag sites. Previously, `UIThemeContainerFrame` was mapped to plain Frame without inheriting its cached mixin/preload; original GossipFrame consequently lacked `UpdateTheme`. Tag resolution now consults the declaration and uses the existing inheritance pipeline. Owning contract records focused original-cache RED→GREEN, 170 template preservation PASS with one pre-existing ignored TODO, and separate clean Retail startup at saved `f963b6b18`. No vendor behavior was patched.

## XML widget ranges and default values

[Owning contract](../../specs/xml-widget-values.md) covers shared StatusBar/Slider `minValue`, `maxValue`, `defaultValue` parsing and inherited application before scripts. Ignored attributes left the original tracker bar at0..1, clamping a correctly supplied50 to1 while its label displayed50%. Generic direct/runtime template initialization now consumes those attributes; owning contract records two Retail RED→GREEN cases, XML170/Slider18/original-StatusBar-UI6 preservation, one existing ignored tooltip TODO, and separate clean Retail startup at their saved scopes.

## Declarative click registration

`FrameXml.register_for_clicks: Option<String>` feeds the shared inherited/instance setter before scripts in direct XML and Lua-created template paths (`33d62d705`). Comma-separated tokens are trimmed; later declarations replace inherited registration, omission preserves it, and later Lua mutation remains authoritative. Existing mouse dispatch policy is unchanged. [Contract](../../specs/xml-button-click-registration.md), [implementation details](../../xml-template-system.md#declarative-click-registration) and [bounded default-Retail GREEN SSOT](../investigations/integrated-source-and-factory-proof-2026-10-09.md#xml-click-registration--bounded-default-retail-green) separate current simulator proof from native grammar/parity and all-profile acceptance. Earlier RED/zero-test history remains in the [3.3.0 audit](../investigations/patch-3-3-0-api-audit.md#current-bounded-reconciliation).

## Template Registry

Virtual/intrinsic frames are registered in a process-global `OnceLock<RwLock<HashMap<String, TemplateEntry>>>` and not instantiated:

```rust
pub struct TemplateEntry { pub name: String, pub widget_type: String, pub frame: FrameXml }
```

A separate registry holds virtual texture templates for mixin chain resolution.

## Texture-template anchor inheritance

VERIFIED: PTR 70493 `Blizzard_PlayerSpells/Mainline/SpellBook/Blizzard_SpellBookTemplates.xml:41-51` supplies half-page anchors through texture templates. `resolve_texture_inheritance` discarded those anchors by restoring the instance's absent anchor field after merging, so both book halves received parent-fill anchors. The resolver now merges anchor declarations and lets explicit instance anchors replace inherited anchors. Both XML-loaded and runtime-created frame children share this resolution. The [contract](../../specs/texture-template-anchors.md) records targeted geometry proof and its native-evidence limits.

## Inheritance Chain Resolution

`get_template_chain(names: &str) -> Vec<TemplateEntry>` splits comma-separated names and recursively follows each template's own `inherits`, depth-first with cycle detection. Returns base-to-derived order — for `inherits="A, B"` where A inherits C: chain is `[C, A, B]`.

Property resolution per template walk:
- **Size**: most-derived wins per dimension; frame's own overrides all. Partial XML sizes (`<Size x="..."/>` or `<Size y="..."/>`) apply only the declared dimension.
- **Anchors**: frame's own if present; otherwise most-derived template with anchors
- **Mixins**: accumulated base-to-derived, then frame's own (duplicates skipped)
- **KeyValues**: later values overwrite; frame's own applied last
- **Hidden**: first template with a value wins (break on hit)
- **EditBox `letters` / `bytes`**: resolve independently; an explicit instance value, including zero, wins, otherwise the most-derived template declaration applies literally.

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

Called from `CreateFrame()` at runtime (no `LoaderEnv` access). `apply_single_template()` order: Mixin → Size → Anchors → SetAllPoints → KeyValues → Layers → button textures → child frames → Scripts. Direct XML properties, including independently resolved EditBox `letters` / `bytes`, apply before lifecycle scripts; this also covers nested template children. OnLoad for ALL template-created children is deferred until after the entire chain is applied. XML frame KeyValues are passed through the CreateFrame template initializer when present, so template child OnLoad handlers see those values before they fire. [EditBox text position and selection](../../specs/editbox-text-position-selection.md) defines the bounded contract; post-change independent verification of `547907429` remains pending.

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

Intrinsic default scripts use the precall binding; ordinary XML scripts use the normal binding unless `intrinsicOrder` requests precall or postcall. Dispatch visits precall, normal, then postcall, while `GetScript(name)` without a binding argument returns only the normal handler. `1c66baffc` parses and registers `OnHorizontalScroll`, `OnVerticalScroll`, and `OnScrollRangeChanged` in ordinary and runtime-template construction; `c82ed91af` proves both paths at runtime. `0c913d2ed` adds locals before an inline body only: `offset` for horizontal/vertical scroll, `xrange`/`yrange` for range changes, and `delta` for wheel callbacks, preserving `...` and globals; `function=` and `method=` bindings are unchanged. Historical tests-only `6b63be464` was RED in three wrong-value cases. The [inline-scroll contract](../../specs/xml-scroll-arguments.md) now retains saved three-case PASS, separate from source equivalence and the enclosing suite FAIL. The three new alias cases are error-clean; the earlier 46 behavioral passes carried five FauxScrollFrame diagnostics and are not an all-errors-clean result. `/tmp/scroll-error-isolated-attribution.md` attributes them to `function=""` resolving `_G` to a table, not shared environment or ScrollBox state. `3e67e7b6e` clears ordinary normal and explicitly selected runtime precall/postcall bindings; bounded default-profile proof passes 47 test instances plus format/check/integration compilation. Its root cause was literal `FastHandlerRef::Function("")`: empty path resolution returns `_G`, then invokes that table—not a `NoOp`. A separate `NoOp` installer bug removed normal rather than the selected intrinsic slot. The pre-fix RED did not independently reach postcall/retained assertions. Intrinsic-default selection, whitespace-only classification, and `method=` precedence are source-inspected only; native/full-profile behavior is unclaimed. Cached `FauxScrollFrameTemplateLight` uses `offset`/`delta`; Mists `CharacterCreate` uses `yrange`. `4d42cb803` adds `OnDoubleClick` to the optimized collector, preventing an empty successful collection from bypassing general XML compilation. Tests-only `0d052b4b2` is RED in three ordinary/runtime method-and-inline cases; its `OnEnter` control passes. [Inline XML scroll arguments](../../specs/xml-scroll-arguments.md), [XML empty script-function clearing](../../specs/xml-empty-script-clearing.md), [XML double-click script registration](../../specs/xml-double-click-scripts.md), and [ScrollFrame script bindings](../../specs/scrollframe-script-bindings.md) are the bounded contracts.

`inherit="prepend"` or `"append"` chains new/existing handlers, both wrapped in `pcall`. Without `inherit`, new handler replaces old.

## Inline function calls resolve globals at invocation

The function-family builders emit parser-validated Lua call expressions inside the installed handler instead of capturing resolved globals. This covers plain/dotted callbacks, literal/global arguments, ancestor fields, method/function-result arguments and checked-assignment callbacks. The handler's Lua environment owns lookup, so later definitions, replacements and `hooksecurefunc` wrappers remain visible; nil-call diagnostics come from Lua itself. XML `function="X"` remains a direct value binding, distinct from the inline body `X(self)`.

The unchanged-producer fixtures at `6b03106f9` failed all six cases, including original Wrath FrameXML TOC order followed by GameMenu Show→tick→Hide→tick. That run recorded missing `UpdateMicroButtons`, `Disable_BagButtons`, `Enable_BagButtons` and `VoiceChat_Toggle` despite their later definitions being callable, plus four `cursorOffset`-nil occurrences. At producer `c1e98b4cc`, Wrath passes6/6 and Retail5/5, with all32 function-family and all4 checked environment shapes completed under each profile. Original GameMenu warnings/errors are now both empty, including the cursor errors, without an EditBox change. Both full-feature startups exit0 with byte-identical committed `[]` baselines: no startup error was added or removed and no baseline refresh is needed. Follow-up producer and bounded preservation results at `fb2e3961d` are recorded below. Evidence: private handoff `fast-handler-late-bind`, `green-<profile>*` receipts/stdout/stderr and `primary-<profile>-baseline-diff.json`. This is not a performance, native-parity, rendered, full-suite or current-descendant acceptance claim. Tests: [`fast_handler_late_bind.rs`](../../../tests/fast_handler_late_bind.rs); implementation: [`builder_function_family`](../../../src/lua_api/globals/create_frame/template_chain/builder_function_family/mod.rs).

### Conditional global predicates

Global no-argument predicates and predicates receiving a global getter result now use the ordinary Lua handler compiler, including surrounding sequences. Their capture templates, enum variants and parser dispatch were removed. Original Wrath KeyRing and Retail ChatConfig OnClick bodies provide four behavioral REDs: later predicate/getter definitions and secure hooks were bypassed. The former ChatConfig optimization-selection assertion is replaced by those original-body behavior tests; no behavior test is ignored. All four original-body cases are GREEN at `fb2e3961d` under their matching profiles.

### LFG branch callbacks

The `GetLFGMode` branch special case also uses the ordinary Lua compiler; its capture template, parser and enum variant were removed. Four original LFD/RaidFinder OnClick-body regressions independently show frozen join/leave callbacks bypassing replacements and secure hooks, with both category-only and category+slot argument checks. These behavior tests replace the old parser-selection assertion. All four original-body cases are GREEN at `fb2e3961d` under Retail.

### Tooltip formatter callbacks

The global-method function-result/three-number builder now emits a Lua method call containing the formatter expression, instead of capturing its function, target and first global argument. Original CharacterFrameTab1 OnEnter supplies a behavioral RED after `MicroButtonTooltipText` replacement+hook; the literal-first changed-environment fixture already passes as a control and does not establish fast-path selection. Both cases are GREEN at `fb2e3961d` under Retail. Other object/value capture sites without runtime RED remain untouched and listed in the handoff audit.

### Bounded final proof — source `fb2e3961d`

| Profile | Lane fixtures | lib `template_chain` | XML templates | Script bindings | Template OnLoad | Startup |
|---|---:|---:|---|---|---:|---|
| Retail | 13 PASS | 33 PASS | 170 PASS, 1 existing ignored TODO | 5 PASS | 3 PASS | exit0, `[]` |
| Wrath | 8 PASS | 33 PASS | 169 PASS, 1 bootstrap FAIL, 1 existing ignored TODO | 4 PASS, 1 bootstrap FAIL | 3 PASS | exit0, `[]` |

Wrath failures are `xml_templates::inline_advanced::empty_script_overrides::cached_faux_scrollframe_empty_function_disables_inherited_auto_range` and `scroll_widgets::script_bindings::cached_event_scrollframe_callbacks_precede_normal_scripts_once`. Both stop at `tests/common/mod.rs:193`: the Retail `Blizzard_SharedXMLBase` TOC is absent under the Wrath cache, before consumer handler construction. Fixture files, shared helper, TOC discovery (`src/loader/mod.rs`), profile resolver and committed Wrath manifest have identical Git blobs at base `cef922007`, producer `fb2e3961d` and master `00fdef62357bdca093c5bcec095bd55240671170`. This is current runtime plus boundary-source/cache classification, not a full base-binary replay. Main explicitly defers profile-gating of those Retail-only fixtures to a separate lane before I1; no ignore, gate or bootstrap workaround was added here. **Full Wrath preservation is not green.**

The existing ignored XML case is the color-table `AddLine` expansion TODO; no new ignored test was introduced. Both startup outputs remain byte-identical to committed baselines, with zero added/removed startup errors. Final original Wrath GameMenu warnings/errors, including cursorOffset-nil, are empty. Every runtime case used a fresh child CVar copy; original stat/hash remained unchanged. Two profile builds exited0; Wrath lib-test compilation retains a base-existing unused-import warning already fixed on master. Saved `acceptance-summary.json` and `wrath-bootstrap-classification.json` in the private handoff bind commands, source revisions, binary hashes and limits. No native, rendered, full-suite or other-profile acceptance claim.

## Name Substitution and parentKey

`$parent` in frame names resolves to the actual parent name. `$parent.ScrollBox` in `relativeKey` resolves to `parent["ScrollBox"]`; `$parent.$parent.X` chains via `GetParent()`.

`parentKey="Title"` produces `parent.Title = frame`. `parentArray="Buttons"` appends to `parent.Buttons`. Both resolved via template inheritance.

## Texture XML self-key source checkpoint — 2026-10-13

[`480b9f934` policy contract](../../specs/xml-self-relative-key.md) owns the approved, user-correlated simulator behavior: a Texture XML direct `$parent.<ownParentKey>` published single-segment self-key targets its parent. Direct Lua validation, shared Frame/FontString generation, sibling and missing-key handling remain unchanged; this is not native WoW parity. Executed pre-producer Mists `a90f84c5d` RED is 1 PASS / 1 FAIL, actual exit101: real cached ModelScene/ProductChoice prerequisites reach the layer-child self-anchor failure, while direct Lua rejection preserves two points and the distinct-target control passes. Producer GREEN, normal startup and named prefork acceptance remain pending.

## Texture XML bounded runtime checkpoint — 2026-10-13

[Owning acceptance](../../specs/xml-self-relative-key.md#known-gaps-current-cycle) and [evidence index](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/xml-self-relative-key-2026-10-13/README.md) separate focused Mists GREEN, cached normal startup, shared anchor controls and named prefork proof from publication integrity. Historical source-only pending labels remain above. Store fixture mismatches retain both FAILs; actual sibling identity/card bounds, native/rendered and full-parent acceptance remain unproved.

## Sources

- [xml-template-system.md](../../xml-template-system.md) — XML types, registry, inheritance, conversion pipeline, inline scripts
- `src/lua_api/env_init/shared_bootstrap.lua` — `GetForbiddenObjectTable` and XML partition helper functions
- `src/loader/xml_frame_codegen.rs` — XML codegen for partition-aware Mixins, KeyValues initialization before template-child OnLoad, engine-root reuse, and top-level implicit-parent preservation
- `src/loader/helpers_anim.rs` — animation-group code generation; mixins are applied before XML method-script binding and OnLoad
- `src/xml/parse.rs` — parser handling that preserves sibling elements after inline `<Scripts>...</Scripts>` blocks
- `src/loader/addon.rs` — shared addon loading transaction used while XML files execute
- `src/loader/tests/runtime_template_misc.rs` — regression coverage for forbidden object tables, secure delegates, and XML `method=` binding timing
- [XML double-click script registration](../../specs/xml-double-click-scripts.md) — optimized `OnDoubleClick` collection boundary and pending proof
- [Inline XML scroll arguments](../../specs/xml-scroll-arguments.md) — inline scroll locals, preserved varargs, diagnostic attribution, and bounded saved proof
- [XML empty script-function clearing](../../specs/xml-empty-script-clearing.md) — bounded selected-binding proof; source-inspected gaps

## See Also

- [[addon-loading]] — TOC parsing, per-file XML/Lua loading, and idempotent loaded-addon handling that feeds this system
- [[widget-system]] — WidgetType and Frame structs produced by XML conversion
- [[frame-data-flow]] — Mixin() application, __frame_fields storage, and script dispatch
- [[dropdown-intrinsic-script-chain]] — intrinsic dropdown binding and derived style-handler order
