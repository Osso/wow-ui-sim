# Capture zone widget

UIWidgetManager exposes supplied capture-zone records to the original Blizzard template. Retail, PTR and Forever declare the getter, visualization and zone entry in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:80,1327,2237`; Mists, Era and Anniversary declare them at lines 76, 1289 and 2199. Retail declares type 17 in `UIWidgetManagerSharedDocumentation.lua:152`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records with a fresh zone entry for supplied IDs.
- [ ] Host type 17 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original capture-zone Setup consumes the supplied zone entry.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_capture_zone.rs`

## Out of scope

- Wrath publication without a pinned declaration.
