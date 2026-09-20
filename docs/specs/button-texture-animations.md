# Button texture animations

XML button texture slots must retain their declared animation groups on the texture returned by the slot getter. Source: Forever `Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.xml` overflow button and `FCFDockOverflowButton_UpdatePulseState`.

## What it must do

- [x] Instantiate the overflow highlight's `FlashAnim` on its actual `GetHighlightTexture()` object, once.
- [x] Preserve the texture identity while the unchanged overflow consumer stops its animation and shows the highlight.

## How it works

- [XML template system](../xml-template-system.md)

## Implementation inventory

- `src/loader/button.rs`: XML button-slot properties and animation construction.

## Tests asserting this spec

- `tests/xml_animation_group_onload.rs`: `forever_dock_overflow_highlight_keeps_its_flash_animation` loads real dock XML and unchanged consumer functions.

## Known gaps (current cycle)

- [ ] Full-startup and cross-profile verification remain parent-owned. Targeted animation group module: 4/4 passing.

## Out of scope

Full chat startup acceptance, unrelated gamepad initialization, and vendor modifications.
