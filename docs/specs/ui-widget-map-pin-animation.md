# Map-pin animation widget

UIWidgetManager exposes supplied map-pin animation records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:245,1605`; Mists, Era and Anniversary declare them at lines 230 and 1567. Retail declares type 26 in `UIWidgetManagerSharedDocumentation.lua:161`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records for supplied IDs.
- [ ] Host type 26 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied animation and padding settings.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_map_pin_animation.rs`

## Out of scope

- Wrath publication without a pinned declaration.
