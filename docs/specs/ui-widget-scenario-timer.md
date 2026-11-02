# Scenario header timer widget

UIWidgetManager exposes supplied scenario timer records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:323,1699`; Mists, Era and Anniversary declare them at lines 304 and 1661. Retail declares type 20 in `UIWidgetManagerSharedDocumentation.lua:155`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records for supplied IDs.
- [ ] Host type 20 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied header and timer values.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_scenario_timer.rs`

## Out of scope

- Wrath publication without a pinned declaration.
