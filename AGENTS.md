# WoW UI Simulator

## Important Rules

- **NEVER modify files in `Interface/AddOns/Wowless/`** — this is an external test suite, not our code.
- **NEVER modify files in `Interface/AddOns/WowlessData/`** — regenerate with `python3 tools/gen_wowless_data.py` (reads from `~/Repos/wowless/data/`). Update the source repo first: `cd ~/Repos/wowless && git pull`.
- Blizzard UI runtime files live in the profile-scoped cache at `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`, populated from the committed manifest with `wow-cli casc sync-blizzard-ui`. Do not rely on `Interface/BlizzardUI` or repo-local `vendor/wow-ui-source-*` checkouts for runtime loading.
- **NEVER modify files in `Interface/BlizzardUI/`** — this legacy symlink tree is no longer the runtime source. Run `./scripts/setup-blizzard-ui.sh` or `./scripts/init-worktree.sh` to sync the active profile cache.
- **NEVER override, monkey-patch, or otherwise change Blizzard/vendor Lua behavior as a performance optimization.** Blizzard Lua is the compatibility target. For perf work, optimize simulator-side primitive/method/dirty/dispatch costs (`SetAlpha`, `SetFormattedText`, `SetPoint`, `SetFontObject`, etc.) instead. Only patch Blizzard/vendor behavior when matching real WoW semantics/correctness, never as a performance shortcut.

## Debugging Priorities

- Do not jump to local UI or output-layer patches just because they are the fastest visible fix.
- Treat render/layout/UI mismatches as evidence first, not as proof that the render/layout/UI layer is the right fix target.
- If a framework or Blizzard/vendor path is behaving badly, prefer fixing the simulator/model/input state first unless evidence shows the downstream layer itself is wrong.

## Wiki

LLM-maintained knowledge base at `docs/wiki/`. See `docs/wiki/SCHEMA.md` for conventions and workflows. Read `docs/wiki/index.md` first when answering questions about the project.

### Wiki Maintenance

After completing work that produces knowledge worth preserving, update the wiki:

- **Investigations/debugging**: Create or update a page in `investigations/` with root cause, symptoms, and fix.
- **New systems or major changes**: Create or update a page in `systems/` describing how the system works.
- **Architecture decisions**: Record rationale in `design/`.

Follow the workflow in `SCHEMA.md`: check existing pages first (update > create), maintain cross-references, update `index.md` and `log.md`. Skip wiki updates for routine bug fixes, config tweaks, and cosmetic changes.

## Architecture Docs

- `docs/layout-system.md` - Anchor/layout system: AnchorPoint, single vs multi-anchor resolution, cycle detection, coordinate systems
- `docs/texture-atlas-system.md` - Texture loading, atlas lookup (~50K entries), nine-slice kits, tiling, UV remapping
- `docs/addon-loading-pipeline.md` - TOC parsing, XML/Lua loading, template registry, SavedVariables, Blizzard addon load order
- `docs/rendering-pipeline.md` - QuadBatch, WGSL shaders, tiered GPU atlas, glyph atlas, strata sorting, hit testing, alpha propagation
- `docs/widget-system.md` - Frame struct (~140 fields), WidgetType enum (18 types), WidgetRegistry, default children, visibility
- `docs/lua-api.md` - WowLuaEnv, FrameHandle, 200+ globals, 300+ frame methods, C_* namespaces, animations, implementation status
- `docs/event-system.md` - Event queue, script handler types (36 handlers), __scripts table, dispatch flow, OnUpdate tick, startup sequence
- `docs/xml-template-system.md` - XML parsing, template registry, inheritance chains, XML-to-widget conversion, inline scripts
- `docs/frame-data-flow.md` - Mixin, events, metamethods, script dispatch
- `docs/button-text-rendering.md` - Button text draw order, three-slice text-behind-background bug
- `docs/glow-plan.md` - Glow/shine effect plan
- `docs/anchor-resolution.md` - Anchor resolution walkthrough: data structures, core functions, single/multi-anchor, SetPoint API, coordinate inversion

## C API Boundary

- Treat the WoW `C_*` API surface as a first-class simulator subsystem, not as "Lua globals" or miscellaneous glue.
- `c_api` belongs at the same architectural level as `lua_api`, because it is a compatibility contract exposed to Lua, not a subcategory of Lua itself.
- Put code in `src/c_api/` when it implements a `C_*` namespace, `Enum.*` data used by those namespaces, or a backing model whose public contract is exposed through `C_*` calls. Legacy non-`C_*` globals may call into that model, but they should not make the model look like Lua glue.
- Put code in `src/lua_api/globals/real/` only when it implements real modeled non-`C_*` Lua globals or mixins backed by simulator state/behavior (for example `GetNetStats`, modifier-key probes, or legacy globals that wrap an existing model).
- Put unmodeled compatibility defaults, no-op behavior, and "returns plausible placeholder values so Blizzard Lua continues" code under `src/lua_api/workarounds/temporary/` or `src/lua_api/workarounds/permanent/`, not under `lua_api/globals/real/`.
- Use `temporary` workarounds for gaps that should be replaced by a modeled system; name the missing backing system in comments or tests. Use `permanent` workarounds only for intentionally unsupported domains with a documented rationale.
- Do not add new modules to the base `src/lua_api/globals/` directory as a classification escape hatch. Before moving anything out of `c_api`, decide whether it is real modeled Lua API, temporary workaround, or permanent unsupported-scope workaround.
- When a `C_*` function is missing or wrong, default to implementing the backing system or state model. Do not reach for shims just to satisfy a failing test.
- Use **temporary shims** only as explicit stopgaps with a clear retirement path. Keep them isolated and named as temporary.
- Use **permanent shims** only for intentionally unsupported domains with a documented rationale. The existing 3D/model gap is the baseline example, not the default pattern.
- Do not hide the importance of the C API behind file placement, wrappers, or "just move it under globals" refactors. Preserve the architectural boundary in both module layout and language.

## Docker

CI image for running `run-tests` in addon CI pipelines. Published to `ghcr.io/osso/wow-ui-sim`.

### GitHub Action

```yaml
- uses: osso/wow-ui-sim@12.0.5
  with:
    addon: MyAddon
```

Action refs match the WoW interface version (e.g. `@12.0.5`). The
action resolves `github.action_ref` (passed through `env:` because
composite actions don't expose it to `run:` directly) and pulls the
matching image tag, so `@12.0.5` runs `ghcr.io/osso/wow-ui-sim:12.0.5`.

When pinning the action to a branch or SHA, pass `image-tag`
explicitly to choose which image to run:

```yaml
- uses: osso/wow-ui-sim@master
  with:
    addon: MyAddon
    image-tag: 12.0.5
```

Without a resolvable version ref and no `image-tag`, the action
fails fast rather than silently pulling `:latest`.

End-to-end example: [Osso/test-wow-addon](https://github.com/Osso/test-wow-addon)
— a minimal addon (TOC + one Lua file + `tests/`) and a workflow that
calls this action. Use it as a template or as a smoke test when
changing the action / Docker image.

### Docker

```bash
# Run addon tests
docker run --rm -v ./MyAddon:/app/Interface/AddOns/MyAddon ghcr.io/osso/wow-ui-sim run-tests MyAddon

# Run with all Blizzard addons loaded
docker run --rm -v ./MyAddon:/app/Interface/AddOns/MyAddon ghcr.io/osso/wow-ui-sim --no-saved-vars run-tests MyAddon
```

### Building Locally

```bash
docker build -t wow-ui-sim .
docker run --rm wow-ui-sim --no-addons --no-saved-vars run-tests Wowless
```

The image is optimized for headless test commands (`run-tests`, `self-test`, `lua-errors`). It excludes audio (built with `--no-default-features`), textures, and GPU drivers to keep the image small (~220MB). The `screenshot` command is not supported in the Docker image.

## Addon Directories

- `./Interface/AddOns/` - Addons bundled with the simulator (Admin, Blizzard_FrameXML, SimCommands, TestFramework). This is the simulator's own addon directory, NOT the WoW game extract.

## WoW Game Files

- `~/.cache/wow-ui-sim/blizzard-ui` - Blizzard UI source cache used by profile runtime loading. Populate with `wow-cli casc sync-blizzard-ui`; the active profile's file list comes from `data/blizzard-ui-files/<profile>.txt`.
- `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns` - Profile-scoped Blizzard UI runtime cache. Populate with `wow-cli casc sync-blizzard-ui` or `./scripts/init-worktree.sh`; the active Cargo `client-*` feature selects both the cache subdirectory (`retail`, `ptr`, `wrath`, `mists`, `era`, `anniversary`) and the matching manifest in `data/blizzard-ui-files/`.
- WoW install (default `/syncthing/World of Warcraft`, override via `WOW_INSTALL_PATH` or `WOW_DATA_PATH`; `asset_resolver::wow_install_path()` also tries common Linux/Wine/Lutris/WSL/macOS paths). The simulator reads textures and fonts directly from CASC via the `asset-resolver` crate (gated behind the `casc` feature, on by default). Set `WOW_SIM_CASC=0` to disable.
- `~/Projects/wow/WTF` - SavedVariables from real WoW installation

## Reference Implementations

- `~/Repos/wowless` - Headless WoW client Lua/XML interpreter. Useful as a reference for WoW API behavior but less complete than wow-ui-sim (no rendering, limited widget support, no layout engine). Use for cross-referencing specific API semantics, not as an implementation target.
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

### Method/Property Lookup on UserData

All frame methods (`Hide`, `Show`, `IsVisible`, etc.) are registered once on `FrameRef` via the shared rilua method/metatable path, so runtime dispatch resolves them for ALL widget types regardless of `WidgetType`. The per-type method registry (`is_method_allowed`) only gates `getmetatable()` results, not actual method calls. Verified by `test_message_frame_has_global_methods` in `src/loader/tests/mod.rs`. If runtime errors report global methods as nil, the problem is the Lua value not being a `FrameRef` (e.g., overwritten by a table), not a missing method registration.

### UserData vs Table: rawset/rawget

`FrameRef` is a UserData that `type()` reports as `"table"` (custom metamethod), but it is NOT a Lua table. `rawset(frame, key, val)` and `rawget(frame, key)` will **fail** with "table expected, got userdata". To access or clear per-frame fields (children, mixin overrides, custom properties), use `debug.getfenv(frame)[1]`:

```lua
-- Clear a field from a frame's per-instance table
local env = debug.getfenv(frame)
if env and env[1] then rawset(env[1], "SetPoint", nil) end
```

This is the table checked by `__index` (in `PATCH_INDEX_LUA` in `metatable.rs`). EditMode overrides like `SetPointOverride`, `SetScaleOverride`, `ClearAllPointsOverride` are stored here by `OnSystemLoad` and shadow the Rust methods.

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

### XML Script Inheritance (`inherit="prepend"` / `inherit="append"`)

The `inherit` attribute on `<Scripts>` elements describes the **inherited** (template) handler's position, not the new instance handler:

- `inherit="prepend"` → inherited (old) runs **first**, instance (new) runs second
- `inherit="append"` → instance (new) runs **first**, inherited (old) runs second

This is counterintuitive — "append" means the inherited handler is appended (runs after), so the new code runs first. Confirmed via wowless (`wowless/modules/loader.lua:201-210`). Implementation: `src/loader/helpers.rs` `emit_chained_handler`.

## Taint System

The simulator uses **rilua** — a pure-Rust Lua 5.1 VM that bakes Elune-style taint tracking directly into the interpreter. There is no C runtime: taint state lives on each `CallInfo` frame and is read/written through rilua's native debug API.

### What rilua provides (native, in `/home/osso/Repos/rilua/src/stdlib/taint.rs`)

`debug.setobjecttaint`, `debug.getstacktaint`, `debug.setstacktaint`, `debug.settaintmode`. Stack taint propagates through call frames so tainted callers make every callee tainted on the way down.

### What the simulator provides

- **Rust** (`src/lua_api/globals/security.rs`): `securecallmethod`, `issecurevariable` overrides; permissive stubs for `issecretvalue`, `canaccessvalue`, `canaccessallvalues`, `canaccesstable`; SecureHandler / state-driver stubs.
- **Rust** (`src/lua_api/env_init/mod.rs`): `forceinsecure` registration.
- **Lua bootstrap** (`runtime_surface_bootstrap.lua`, `shared_bootstrap.lua`): Lua-level fallbacks for `issecure`, `hooksecurefunc`, and `secureexecuterange` (rilua's C-level `secureexecuterange` is a no-op stub, so the Lua impl is installed unconditionally).

`issecure()` checks `debug.getstacktaint()` — returns true when stack taint is nil (Blizzard code), false when tainted (addon code). The Lua fallback in `runtime_surface_bootstrap.lua` matches this contract; Wowless tests verify it.

### Per-addon taint stamping

- **Loading** (`loader/lua_file.rs`): `debug.setobjecttaint(compiled_func, addon_name)` on each addon's compiled closure. Blizzard UI code is NOT tainted.
- **Script dispatch** (`env.rs`): `debug.setobjecttaint(handler, addon_name)` before calling frame script handlers, using the frame's owner addon.
- **Effect**: When tainted code runs, rilua sets stack taint to the addon name. Variables set by tainted code are marked. `issecurevariable(table, key)` returns `false, "AddonName"`.

### What is NOT enforced

- **Protected frame restrictions**: `SetAttribute` does NOT check `issecure()`. In real WoW, protected frames block attribute changes from insecure code. We don't enforce this — known gap.
- **`SetForbidden`/`IsForbidden`**: Implemented as flags but not enforced on any method.

## Intentional Gaps

- **No 3D rendering**: Model, ModelScene, PlayerModel, DressUpModel frames have stub-only implementations for camera, lighting, transform, animation, and mesh methods (`widget_model.rs`). The simulator renders 2D UI only — 3D model display is out of scope. These ~38 stubs are permanent and should not be converted to real implementations.

## Client Profiles

The simulator builds for one of six client profiles per cargo build (`client-retail` default; `client-ptr`, `client-wrath`, `client-mists`, `client-era`, `client-anniversary` selectable via `--no-default-features --features "sound,gui,casc,client-<profile>"`). Exactly one profile feature must be enabled — `src/client_profile.rs` enforces this with a `compile_error!`. See `docs/wiki/systems/client-profiles.md` for the full architecture.

### Where stubs go

When a Blizzard global or frame method is referenced by vendor sources but not implemented in the simulator's runtime surface, add a stub. The placement rule is **scope, not convenience**:

| Scope | Location | Mechanism |
|-------|----------|-----------|
| Wrath only | `src/wrath/compat_bootstrap.lua` (or `frame_methods.rs` for methods on the frame metatable, or `compat_frame_proxies.lua` for frame-handle proxies) | `if rawget(_G, "X") == nil then ... end` guarded |
| Mists only | `src/mists/compat_bootstrap.lua` | rawget-guarded |
| Era only or Era+Anniversary (vanilla content) | `src/era/compat_bootstrap.lua` (shared between both) | rawget-guarded |
| Single third-party addon | `tools/classic-addon-compat/<addon>/<shim>/<shim>.lua` with `## LoadFirst: 1` | rawget-guarded |
| Cross-profile / runtime surface | `src/lua_api/env_init/runtime_surface_bootstrap.lua` (loaded under all profiles) | rawget-guarded |

**Promotion rule:** start narrow. A new gap belongs in the per-addon shim first; if the same gap surfaces across multiple addons under one profile, promote it to that profile's bootstrap; if it's needed across multiple profiles, promote it to `runtime_surface_bootstrap.lua`. **Never** add cross-profile content to a per-profile module — and never add per-profile content to the runtime surface.

### The cfg(any) refactor signal

When `#[cfg(any(feature = "client-X", feature = "client-Y"))]` (or the equivalent on a single helper / module reference) starts repeating in two or more places, that's the signal to extract a shared module.

The current shared modules:

- **`src/wrath/`** — compiled under `client-wrath` + `client-mists` + `client-era` + `client-anniversary` (gated in `src/lib.rs`). Provides `frame_methods::register_all` (no-op stubs for backdrop / depth / player-texture methods every non-retail profile needs) and `is_registerable_event` (permissive — accepts any non-empty event name, since the mainline `events.yaml` strict list doesn't cover pre-Cata content).
- **`src/era/`** — compiled under `client-era` + `client-anniversary` (vanilla content; the underlying API gaps are near-identical).
- **`src/mists/`** — compiled only under `client-mists` (no profile shares it).

If you find yourself writing `#[cfg(any(...))]` on a third site to gate the same set of profiles, **extract a function or module** with that cfg(any) at its declaration so individual call sites stay clean. Don't copy-paste cfg(any) lists — they always fall out of sync.

### When NOT to add per-profile code

- **Profile-aware behavior in `src/loader/` or `src/toc/`** belongs in helpers like `active_profile_toc_suffix()` / `other_profile_toc_suffixes()` that match on `crate::client_profile::ACTIVE`, not in `#[cfg]` walls. Use the runtime const so `cargo check` validates all five arms in one pass; cfg gates would silo arms behind feature flags and let drift accumulate.
- **Compat shims for retail** are usually a sign of a real bug or a missing namespace. Fix the underlying simulator surface; don't add a `src/retail/` (it doesn't exist).

### Vendor pinning

The Blizzard UI runtime cache is populated from the committed per-profile manifests and records a provenance file beside the completion marker. Manifest or listfile-source changes are deliberate changes — they tend to surface new gaps (or fix old ones) in the matching baseline at `docs/baselines/<profile>-lua-errors.json`. Re-capture the baseline after such a change and commit both together.

## Performance

Uses **Lua 5.1 semantics** on the `rilua` runtime.

### Rilua Hot Paths

- Do **not** default to blaming generic "Lua bridge cost" or old mlua userdata overhead when a handler is slow.
- The current simulator uses rilua stack-level dispatch and direct hot Rust calls for frame methods / globals.
- `WOW_SIM_LOG_HANDLER_TIMINGS` measures the handler body itself. Treat slow `OnUpdate`/event timings as real work in the handler, downstream Rust API calls, string formatting, table churn, or GC until proven otherwise.

### Running the Simulator

**CRITICAL: Always compile separately from running.** Never use `cargo run` directly with a timeout — compilation alone can take 30s+ and the timeout kills the build mid-compile, wasting all progress. Always build first, then run:
```bash
# Step 1: Build (no timeout — let it finish)
cargo build --bin wow-sim

# Step 2: Run (with timeout — prevents hung processes)
WOW_SIM_NO_SAVED_VARS=1 WOW_SIM_NO_ADDONS=1 timeout 90 cargo run --bin wow-sim
```
**Always use `timeout 90` or less** on the run step to prevent hung processes. Never put a timeout on the build step.

### Release Builds

Use the xtask-backed Cargo aliases for release binaries:

```bash
cargo release-windows
cargo release-linux
cargo release-current
```

These aliases build both `wow-sim` and `wow-cli` with `--release --no-default-features --features sound,gui`. This intentionally excludes the `fast-build` feature so release builds use non-dynamic iced. The `gui` feature requires `casc`, so release GUI builds always include CASC asset loading.

### CLI Arguments

- `--exec-lua "code"` - Execute Lua code after startup events. Works in all modes: GUI (after first frame render), screenshot, and dump-tree. Prefix with `@` to load from file (e.g., `--exec-lua @/tmp/debug.lua`)
- `--no-addons` / `--no-saved-vars` - Same as environment variables below
- `--delay <ms>` - Delay in milliseconds after firing startup events (for dump-tree/screenshot). The sleep is followed by one OnUpdate tick that reports the slept time as `elapsed` and by the timers that came due, and it runs before `--exec-lua`, so an addon that defers its setup with `C_Timer.After(n)` is initialised by the time the script runs

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

### Bilinear Atlas Bleed

GPU texture atlas uses `ClampToEdge` + `Linear` (bilinear) filtering. UVs are remapped to atlas slot space in `resolve_and_scale_quads` (`primitive.rs:210-214`) with **no half-pixel inset**, so the sampler bleeds into adjacent content at slot boundaries. Symptoms: thin bright lines at tile seams, colored fringe around icons/buttons.

**Partial fix applied**: `@crop:` paths isolate sub-region textures (nine-slice pieces, tiled textures) into their own atlas slots, preventing bleed from adjacent source texture content. See `crop_path_for_subregion()` in `tiling.rs` and `crop_piece()` in `nine_slice.rs`.

**Remaining**: Half-pixel UV inset needed in `resolve_and_scale_quads` to prevent bleed at atlas slot boundaries globally. Button textures (`emit_button_texture`, `emit_button_highlight` in `quad_builders.rs`) also pass raw atlas UVs without `@crop:`. See `PLAN.md` for details.

### Frame Re-creation and Orphaned Children

Several frames are pre-created in Rust before XML addons load (UIParent, WorldFrame, GameTooltip, etc.). When a Blizzard XML addon later defines a frame with the same name, `CreateFrame` creates a NEW frame with a new ID, orphaning the old one. Any children that were parented to the old frame become invisible because `collect_ancestor_visible_ids` can't reach them — the old frame is hidden and disconnected from the tree.

This is load-order dependent: if addon A creates a child of UIParent, and addon B (loaded later) re-creates UIParent via XML, the child is stranded on the old UIParent. Example: Blizzard_GameTooltip loads before Blizzard_UIParent, so GameTooltip ends up parented to the old UIParent.

**Fix**: `register_new_frame` in `create_frame.rs` calls `migrate_children_to_new_frame()` which reparents all children (updating both `parent_id` and the new parent's `children` Vec / `children_keys` HashMap). When debugging invisible frames, check for duplicate IDs via name lookup — if a frame's parent points to an orphaned/hidden frame, this migration didn't cover it.

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

- `--filter` matches frame **display names** (global name or `.parentKey`) per-frame, printing only matching frames
- `--filter-key` matches display names and prints the **full subtree** of matching frames — use this to explore a frame's children
- Debug script hook: `src/main.rs` loads `/tmp/debug-scrollbox-update.lua` before GUI starts (not available in dump command)

## CLI Tools

Both `wow-sim` and `wow-cli` provide CLI subcommands. `wow-sim` subcommands use the same loading as the GUI; `wow-cli` subcommands either load standalone or connect to a running server.

### Lua Errors

Check for Lua errors during startup. Outputs unique errors as JSON to stdout (all other output to stderr). **Always run this after adding/modifying stubs or API registrations.**

```bash
wow-sim --no-addons --no-saved-vars lua-errors 2>/dev/null   # Check for errors (JSON output)
wow-sim lua-errors 2>/dev/null                                # Full load with addons
```

Empty output (no JSON array) means zero errors. Non-empty output lists unique error messages with counts.

### Self-Test (Wowless Suite)

**Do NOT run during normal development** — takes 60s+ and hangs waiting for async test completion. Only run when specifically debugging Wowless compatibility.

```bash
wow-sim --no-saved-vars self-test                    # Run Wowless tests (exit 0=pass, 1=fail, 2=timeout)
wow-sim --no-saved-vars self-test --max-ticks 20000  # Increase timeout
```

### Rust Tests

**Primary test location.** Write new tests as Rust integration tests in `tests/`. These test the Lua API surface via `WowLuaEnv::eval()` and internal Rust logic directly. Run with `cargo test`.

```bash
cargo test test_character_stats     # Run specific test
cargo test                          # Run all tests (slow — loads Blizzard UI per test)
```

### Run Tests (Addon Test Runner)

Lua-level tests in `Interface/AddOns/<name>/tests/`. Use for addon-facing API integration tests. **Avoid for internal logic — use Rust tests instead.**

**Do NOT run `run-tests Wowless` or `run-tests WowBehaviorTest` during normal development** — they load the full Wowless/WowBehaviorTest suites which take 60s+ and may hang on async tests. Use `cargo test` instead.

```bash
wow-sim --no-addons --no-saved-vars run-tests Wowless    # Slow — only for Wowless compat debugging
wow-sim --no-saved-vars run-tests Wowless                # Full load — even slower
```

**Test file syntax** (`Interface/AddOns/<name>/tests/something.lua`):

```lua
test("frame name matches", function()
    local f = CreateFrame("Frame", "MyFrame")
    assertEquals("MyFrame", f:GetName())
end)

async_test("timer fires callback", function(done)
    C_Timer.After(0, function()
        done(function()
            assertTrue(true)
        end)
    end)
end)
```

**Available assertions** (provided by `Interface/AddOns/TestFramework/`):

| Function | Description |
|---|---|
| `assertEquals(expected, actual)` | Strict equality (`~=`) |
| `assertNotEquals(expected, actual)` | Not equal |
| `assertTrue(value)` / `assertFalse(value)` | Truthy/falsy |
| `assertNil(value)` / `assertNotNil(value)` | Nil checks |
| `assertError(fn)` | Function throws an error |
| `assertType(expected, value)` | `type(value) == expected` |
| `assertAlmostEquals(expected, actual, tolerance?)` | Float comparison (default 0.001) |
| `assertContains(haystack, needle)` | String substring or table value |
| `assertStartsWith(str, prefix)` | String prefix |
| `assertEndsWith(str, suffix)` | String suffix |
| `assertMatches(str, pattern)` | Lua pattern match |
| `assertCount(expected, table)` | Table element count |
| `assertTableEquals(expected, actual)` | Deep table equality |
| `assertTableContains(table, subset)` | Table contains subset (deep) |

**Async tests**: Use `async_test("name", function(done) ... end)`. The runner ticks OnUpdate/timers until `done()` is called or 500 ticks timeout. Pass an optional assertion function to `done()` for final checks.

### Screenshot

Render the UI to an image file without starting the GUI (headless GPU, same shader pipeline as the live renderer). Text is not rendered — this is for debugging frame layout and textures.

```bash
wow-sim --no-addons --no-saved-vars screenshot                       # Render to screenshot.webp (1024x768, lossy q65)
wow-sim screenshot -o frame.webp --filter AddonList                  # Render only AddonList subtree
wow-sim screenshot --width 1920 --height 1080                        # Custom resolution
wow-sim --no-addons --no-saved-vars screenshot -o fast.webp           # Fast: skip extras
```

Always saves as lossy WebP at quality 65. Extension is forced to `.webp` regardless of what's passed to `-o`.

Also available via `wow-cli screenshot` (standalone loading, same options).

### Dump Frame Tree

Load UI and dump the frame tree without starting the GUI (for debugging):

```bash
wow-sim --no-addons --no-saved-vars dump-tree                # Fast: skip addons and saved vars
wow-sim dump-tree --filter ScrollBar                         # Filter by name substring
wow-sim dump-tree --filter-key SpellBookFrame                # Filter by name, print full subtree
wow-sim dump-tree --visible-only                             # Show only visible frames
wow-sim dump-tree                                            # Full load with all addons
```

Output shows frame hierarchy with dimensions:
```
AddonList [Frame] (600x550) hidden
  AddonListBg [Texture] (0x0) visible
  AddonListCloseButton [Button] (24x24) visible
```

Also available via `wow-cli dump` (standalone loading, same options).

### Lua REPL

```bash
wow-cli lua                    # Interactive Lua REPL
wow-cli lua -e "print('hi')"   # Execute code and exit
wow-cli lua -l                 # List running servers
```

### Dump Frame Tree (Connected)

Dump the rendered frame tree from a running simulator:

```bash
wow-cli dump-tree                      # Dump all frames
wow-cli dump-tree --filter Button      # Filter by name substring
wow-cli dump-tree --visible-only       # Show only visible frames
```

Output shows frame hierarchy with absolute screen coordinates and dimensions:
```
AddonList [Button] (x=50, y=400, w=80, h=22) visible
  CancelButton [Button] (x=430, y=508, w=80, h=22) visible
    Text [FontString] (x=430, y=508, w=80, h=22) visible text="Cancel"
```

### Audit API Gaps → PLAN.md

Generate PLAN.md-ready checkboxes for missing C_* methods, LE_* constants, and Enum namespaces:

```bash
wow-cli audit-api --gaps --format plan         # Paste-ready markdown checkboxes
```

## Textures

### Texture Sources (in order of priority)

1. Addon directories — for addon-specific textures shipped under `Interface/AddOns/`.
2. CASC — the live WoW install discovered by `asset_resolver::wow_install_path()` (default `/syncthing/World of Warcraft`, overridable via `WOW_INSTALL_PATH` or `WOW_DATA_PATH`), resolved via the `asset-resolver` crate using the community listfile. Extracted BLPs are cached under `~/.cache/wow-ui-sim/casc-extract/` keyed by listfile path.

There is no longer a curated `./textures/` mirror or `~/Projects/wow/Interface` extract — adding a new texture means making sure CASC has it (the listfile is updated upstream by `asset-resolver`).

### Texture Path Resolution

WoW paths like `Interface\\Buttons\\UI-Panel-Button-Up` are resolved by:
1. Normalizing backslashes to forward slashes and stripping any extension.
2. Looking up the path under several extensions (`blp`, `tga`, `ttf`, `otf`, plus case variants) in the listfile to get a fileDataID.
3. Asking `asset_resolver::ensure_cached(fdid, out_path)` to extract the BLP from CASC into the on-disk extract cache.
4. Falling back to addon-relative paths if the CASC tier misses.

### Verifying CASC lookups

Use the smoke test to verify a path resolves end-to-end:

```bash
cargo run --example casc_smoke
```

It probes a handful of textures and prints the fileDataID for each. Add a new probe entry there when investigating a missing asset.

### Button Textures

Standard WoW button textures in `Interface/Buttons/`:
- `UI-Panel-Button-Up` - Normal state (128x32, dark red gradient with gold border)
- `UI-Panel-Button-Down` - Pressed state
- `UI-Panel-Button-Highlight` - Hover state
- `UI-Panel-Button-Disabled` - Disabled state
