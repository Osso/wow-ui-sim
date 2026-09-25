# Texture atlas visibility

Changing a texture's atlas must preserve its XML visibility except for standard Button/CheckButton texture slots. See [XML templates](../wiki/systems/xml-template-system.md) for template creation.

## What it must do

- [x] A `GradualAnimatedStatusBarTemplate` instance starts with both XML-hidden animation textures hidden; assigning their atlases does not show them.
- [x] Playing gain-flare or level-up animation shows its texture; natural completion hides it again.
- [x] Changing the atlas of an XML-hidden custom-key texture on a Button does not show it.
- [x] Standard Button/CheckButton texture slots still synchronize atlas and visibility with button state.

## How it works

- [XML template system](../wiki/systems/xml-template-system.md)
- [Button texture rendering](../button-text-rendering.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/texture/atlas.rs`: only `Button`/`CheckButton` children with recognized state-slot keys (`NormalTexture`, `PushedTexture`, `HighlightTexture`, `DisabledTexture`, `CheckedTexture`, `DisabledCheckedTexture`) synchronize atlas and visibility with their parent. Other children retain their own visibility.

## Tests asserting this spec

- `tests/xml_animation_group_onload.rs`: copied vendor `GradualAnimatedStatusBar` template starts with its XML-hidden gain-flare and level-up textures hidden; `SetAnimationTextures()` keeps them hidden; play/tick completion shows then hides each target. It also covers an XML-hidden custom Button child.
- `tests/frame_creation_checkbutton.rs`: standard CheckButton NormalTexture atlas propagation.

## Evidence

`/tmp/wow-xml-atlas-green.log` records all six targeted `xml_animation_group_onload` tests passing for `83a43ca61`, including the actual-template lifecycle and custom-key regression. This is simulator and cached-vendor-source evidence; it does not establish native client behavior.

## Known gaps (current cycle)

- [ ] Confirm full XP/reputation bar startup visuals in the GUI.
- [ ] Run final checks after the GUI result.

## Out of scope

Animation timing changes, Blizzard Lua changes, pixel-level rendering, and native-client conformance.
