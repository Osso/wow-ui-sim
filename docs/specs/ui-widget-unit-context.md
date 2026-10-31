# Widget unit context

## Best-guess basis

**Best-guess, native UNKNOWN.** No native evidence establishes registration, event, query-context, duration, or invalid-token behavior. The following are simulator behavior inferred from the pinned consumers, not a native claim:

- Retail, PTR and Forever `Blizzard_APIDocumentationGenerated/UIWidgetManagerDocumentation.lua:557–600` and Mists, Era and Anniversary `:523–561` declare registration and processing-unit APIs; Wrath does not declare them. Retail/PTR/Forever `:604–621` declare synchronous `UPDATE_ALL_UI_WIDGETS` and `UPDATE_UI_WIDGET`.
- Retail/PTR/Forever `Blizzard_UIWidgets/Blizzard_UIWidgetManager.lua:46–52, 266–282, 307–318, 479–481, 648–655` register attached units, filter widget updates by attached unit, unregister them, and set processing unit before visualization queries.
- Retail `Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:1565–1571` uses `UnitWidgetSet(frame.unit)` and passes the unit to the original widget container. Forever `Blizzard_PlayerChoice/Blizzard_PlayerChoiceOptionBase.lua:268–272` passes a GUID-attached unit.
- Retail `Blizzard_UIWidgets/Blizzard_UIWidgetTemplateSpellDisplay.lua:99–106` sets processing unit before querying the spell-display visualization.

## Contract

- The six declaring profiles expose registration and processing APIs with the declared argument validation, secret-argument handling, nil clearing, and no return values. Registration is an idempotent set of token/GUID keys local to each simulator environment.
- Host updates with an attached token or GUID store a visualization for that unit and widget ID. Unit-specific removal removes that unit's record, retains membership while another unit has a record, and notifies only if the removed unit is registered. The registered unit receives synchronous `UPDATE_UI_WIDGET` after the state change; an unregistered unit receives no event. Global widget updates retain their existing behavior.
- Global set replacement and global membership removal use `UPDATE_ALL_UI_WIDGETS`; the documented event has no payload and the pinned container processes it without filtering.
- Visualization queries use the selected processing token or GUID for unit-specific records. A unit-specific widget with no record for that context returns nil; clearing context leaves no unit-specific record selected. Widgets with no unit-specific records retain their supplied global record.
- Unit-specific state is supplied, not inferred from widget IDs. Native registration lifetime and unknown-token behavior are not claimed.

## Tests

- `tests/widget_spell_display/consumer.rs` exercises two attached original Blizzard containers, unit-specific queries, replacement, removal, and registration-controlled event delivery.
- `tests/widget_spell_display/unit_apis.rs` covers argument validation and context state.

## Out of scope

- Native feed discovery, automatic unit/GUID identity mapping, and unrelated widget APIs.
