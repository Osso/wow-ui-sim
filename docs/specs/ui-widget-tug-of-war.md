# Tug-of-war widget

UIWidgetManager exposes supplied tug-of-war records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:497,2012`; Mists, Era and Anniversary declare them at lines 467 and 1974. Retail declares type 28 in `UIWidgetManagerSharedDocumentation.lua:163`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records with fresh icon records for supplied IDs.
- [ ] Host type 28 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied range, values and tooltip.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_tug_of_war.rs`

## Out of scope

- Wrath publication without a pinned declaration.
