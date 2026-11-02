# Fill-up frames widget

UIWidgetManager exposes supplied fill-up frames to the original Blizzard template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:155,1460`; Mists, Era and Anniversary declare them at lines 146 and 1422. Retail declares type 24 in `UIWidgetManagerSharedDocumentation.lua:159`.

## What it must do

- [ ] The getter returns nil for an absent ID and fresh complete records for supplied IDs.
- [ ] Host type 24 updates publish supplied records before notification; other visualization types do not return them.
- [ ] The original Setup consumes supplied fill and frame values.
- [ ] No output secrecy policy is inferred.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_fill_up_frames.rs`

## Out of scope

- Wrath publication without a pinned declaration.
