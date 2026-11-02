# Scenario header delves widget

UIWidgetManager exposes supplied scenario delves records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:308,1671`; Mists, Era and Anniversary declare them at lines 290 and 1633. Retail declares type 29 in `UIWidgetManagerSharedDocumentation.lua:164`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records with fresh nested rows and reward data for supplied IDs.
- [ ] Host type 29 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied header, tier, currencies and reward.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_scenario_delves.rs`

## Out of scope

- Wrath publication without a pinned declaration.
