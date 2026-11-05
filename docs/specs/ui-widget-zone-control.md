# Zone control widget

UIWidgetManager exposes supplied zone-control records to the original Blizzard template. Retail, PTR and Forever declare the getter, visualization and zone entry in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:543,2212,2237`; Mists, Era and Anniversary declare them at lines 510, 2174 and 2199. Retail declares type 16 in `UIWidgetManagerSharedDocumentation.lua:151`.

## What it must do

- [x] The getter returns nil for an absent ID and fresh complete records with ordered, fresh zone entries for supplied IDs.
- [x] Host type 16 updates publish supplied records before notification; other visualization types do not return them.
- [x] The original zone control Setup consumes supplied zone entries.
- [x] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_zone_control.rs`
- `tests/ui_widget_visualization_feed.rs`

## Out of scope

- Wrath publication without a pinned declaration.
