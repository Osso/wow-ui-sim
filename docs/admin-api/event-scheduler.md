# Event scheduler

### `A_Admin.SetActiveEventContinentName(name)`

Supplies the active continent name returned by Retail and WoW Forever `C_EventScheduler.GetActiveContinentName()`.

- `name` must be a string or `nil`. `nil` clears the supplied value.
- The default state is `nil` (no active continent supplied).
- The query may return no values when unset, matching the pinned API's `MayReturnNothing` contract.

The query is declared in the pinned `Blizzard_APIDocumentationGenerated/EventSchedulerUIDocumentation.lua`. The host-supplied state is a simulator input; no current-event schedule is inferred from the name.
