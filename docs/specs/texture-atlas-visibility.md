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

- `src/lua_api/frame/methods/widgets/texture/atlas.rs`: applies atlas and updates recognized button slots.

## Tests asserting this spec

- `tests/xml_animation_group_onload.rs`: actual Forever template, animation lifecycle, and custom button key.
- `tests/frame_creation_checkbutton.rs`: standard CheckButton NormalTexture atlas propagation.

## Known gaps (current cycle)

- [ ] Confirm the full XP/reputation bar startup visuals after this fix.

## Out of scope

Animation timing changes, Blizzard Lua changes, and pixel-level rendering.
