# Unit-power bar widget

UIWidgetManager exposes supplied unit-power bar records to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:512,2180`; Mists, Era and Anniversary declare them at lines 481 and 2142. Retail declares type 23 in `UIWidgetManagerSharedDocumentation.lua:158`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records for supplied IDs.
- [ ] Host type 23 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes the supplied bar values and tooltip.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_unit_power_bar.rs`

## Out of scope

- Wrath publication without a pinned declaration.
