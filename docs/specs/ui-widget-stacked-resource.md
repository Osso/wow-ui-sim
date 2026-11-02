# Stacked resource widget

The UIWidgetManager exposes supplied StackedResourceTracker visualizations to the original Blizzard widget template. Retail, PTR and Forever declare the getter and structure in `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:368-379,1767-1786`; Mists, Era and Anniversary declare them at lines 346-357 and 1729 onward. The Retail visualization type is declared in `UIWidgetManagerSharedDocumentation.lua:141`.

## What it must do

- [ ] The getter returns nil for an absent ID and a fresh, complete record for a supplied ID, including ordered fresh resource rows.
- [ ] The host feed accepts type 6 and publishes supplied records before notification. Different visualization types do not return this record.
- [ ] The original StackedResourceTracker Setup consumes the supplied resources.
- [ ] No output secrecy policy is inferred from the declarations.

## How it works

- [UIWidgetManager state](ui-widget-manager-state.md)

## Tests asserting this spec

- `tests/ui_widget_stacked_resource.rs`

## Out of scope

- Wrath publication without a pinned declaration.
