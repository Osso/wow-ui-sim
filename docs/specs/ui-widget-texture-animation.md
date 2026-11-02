# Texture-with-animation widget

UIWidgetManager exposes supplied texture-with-animation records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:473,1990`; Mists, Era and Anniversary declare them at lines 444 and 1952. Retail declares type 18 in `UIWidgetManagerSharedDocumentation.lua:153`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records for supplied IDs.
- [ ] Host type 18 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied tooltip and texture kit.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_texture_animation.rs`

## Out of scope

- Wrath publication without a pinned declaration.
