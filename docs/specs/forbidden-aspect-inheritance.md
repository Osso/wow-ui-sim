# Forbidden-aspect inheritance

Frames and regions retain owned forbidden aspects and separate hierarchy/layout propagation masks. The authoritative names and bits come from `Enum.ForbiddenAspect`; native propagation paths are `ScriptObjectPropagationPath.Hierarchy = 0` and `Layout = 1`.

## What it must do

- [x] Publish `Enum.ScriptObjectPropagationPath` (`Hierarchy=0`, `Layout=1`) and its metadata before frame methods are called.
- [x] Resolve every XML aspect name through the active enum, report unknown names, and apply the same declarations to literal XML and runtime template instances.
- [x] Merge declarations with inherited state. A nonempty mask implies `SetToDefaults`.
- [x] Propagate `UntrustedScriptExecution`, `UntrustedLayoutScriptExecution`, and `AlwaysPropagateInput` through parent ownership; only `UntrustedLayoutScriptExecution` propagates through layout dependencies by default.
- [x] Give newly registered children their parent's hierarchy-inheritable aspects before Lua observes the child, including textures, font strings, lines, mask textures, and native default children.
- [x] Preserve denial of later `SetParent`/`SetPoint` operations that would acquire unowned propagating restrictions from a foreign object.

## How it works

- [Widget system](../widget-system.md)
- [XML template system](../xml-template-system.md)
- [Script-object environments](script-object-environments.md)

## Implementation inventory

- `src/widget/{frame,frame_defaults}.rs`, `registry/mod.rs` — masks and new-widget inheritance.
- `src/lua_api/frame/methods/forbidden_aspects.rs` — native defaults, active-enum resolution, merging, and ownership checks.
- `src/loader/xml_frame/setup.rs`, `src/lua_api/globals/create_frame/template_chain{,/runtime}.rs` — literal and runtime XML application.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs` — public enum-path conversion and bounded argument handling.

## Tests asserting this spec

`tests/forbidden_aspect_creation.rs` reproduces BetterBlizzFrames' icon/cooldown/overlay/border creation order, validates masks and native child paths, and rejects foreign anchor/parent gains. Existing loader tests use the native enum and propagation paths rather than obsolete aliases. Focused proof at `d12cfead4` / `ee05355ae`: four forbidden-aspect cases passed in `/tmp/pi-aura-followup-green.*`.

## Known gaps

- This slice does not introduce a policy for dynamically propagating subsequently added restrictions over existing parent/anchor graphs.
- Actual addon/SavedVariables startup returned `[]`, exit 0 after separate dispel-filter, controlled-player token, and curve-userdata fixes (`/tmp/pi-accepted-final-startup.*`). That integration result does not extend this slice to dynamic propagation.

## Native sources

Cached retail `ForbiddenAspectConstantsDocumentation.lua` establishes all eleven bits and default propagation. `SimpleFrameScriptObjectConstantsDocumentation.lua` establishes public path values. `Blizzard_SharedXML/UI.xsd` declares only the `aspect` attribute on `ForbiddenAspectType`; the simulator's former `inheritance` attribute was unsupported by that schema.
