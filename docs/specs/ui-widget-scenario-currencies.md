# Scenario header currency widget

UIWidgetManager exposes supplied scenario header and currency records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:293-304,1649-1669`; Mists, Era and Anniversary declare them at lines 276 and 1611. Retail declares visualization type 11 in `UIWidgetManagerSharedDocumentation.lua:146`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records with ordered, fresh currency rows for supplied IDs.
- [ ] Host type 11 updates publish the supplied record before notification, and other visualization types do not return it.
- [ ] The original scenario header Setup consumes the header and currencies.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_scenario_currencies.rs`

## Out of scope

- Wrath publication without a pinned declaration.
