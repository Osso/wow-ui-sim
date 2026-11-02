# Double-state icon row widget

UIWidgetManager exposes supplied double-state icon rows to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:125-136,1404-1424`; Mists, Era and Anniversary declare them at lines 118 and 1366. Retail declares visualization type 14 in `UIWidgetManagerSharedDocumentation.lua:149`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records with ordered, fresh left and right icon rows for supplied IDs.
- [ ] Host type 14 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original double-state icon Setup consumes both sets of icons.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_double_state_icon.rs`

## Out of scope

- Wrath publication without a pinned declaration.
