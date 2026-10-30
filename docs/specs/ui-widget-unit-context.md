# Widget unit context

This is a **user-approved simulator default**, not evidence of native behavior. The pinned `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua` declares these four APIs at Retail, PTR, and Forever lines 557–600, and Mists, Era, and Anniversary lines 523–561. Wrath has no pinned declaration.

## What it must do

- [ ] The six declaring profiles expose `RegisterUnitForWidgetUpdates` and `UnregisterUnitForWidgetUpdates`: a required string and an optional boolean defaulting to false. Registration is an idempotent set of `(string, isGuid)`; removing an absent key does nothing.
- [ ] `SetProcessingUnit` and `SetProcessingUnitGuid` accept a nilable string. Each selects its own kind of processing context; nil clears the selection.
- [ ] All four functions return no values, validate arguments, and authenticate secret arguments using the VM's AllowedWhenUntainted pattern. They fire no events and do not alter visualization queries.
- [ ] Context belongs to each simulator environment. The existing cached Blizzard manager and spell-display polling paths continue to work.

## How it works

- [Widget manager state](../wiki/systems/ui-widget-manager-state.md)

## Implementation inventory

- `src/c_api/ui_widget_manager/unit_context.rs` — context state and API handlers.
- `src/c_api/ui_widget_manager/queries.rs` — profile-gated publication.
- `src/c_api/ui_widget_manager/inputs.rs` — environment-local context.

## Tests asserting this spec

- `tests/widget_spell_display/unit_apis.rs` — direct calls and state transitions.
- `tests/widget_spell_display/consumer.rs` — cached manager and polling consumers.

## Known gaps (current cycle)

- [ ] Native registration, event, and processing-unit semantics are not established.

## Out of scope

- Changing visualization queries or modeling native unit-specific feed filtering.
