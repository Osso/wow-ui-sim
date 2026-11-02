# Calendar event selection

Retail `C_Calendar.GetEventIndex` and `ContextMenuGetEventIndex` project independent selected event identities onto the current supplied day list. Pinned `CalendarDocumentation.lua` declares the getters at lines 572 and 139, `OpenEvent` at 825, `ContextMenuSelectEvent` at 165, and `CloseEvent` at 43.

## What it must do

- [ ] Resolve the opened and context-menu event separately by identity and date against the current list, returning the first matching one-based index in a fresh `{offsetMonths, monthDay, eventIndex}` table.
- [ ] Return zero values when the selection is absent, outside the previous/current/next month, or no longer matches the supplied list. This arity is a best guess based on post-lookup emulation, not whole-client observation.
- [ ] Successful `OpenEvent` and `ContextMenuSelectEvent` update their respective selections. `CloseEvent` clears the opened selection only.
- [ ] The original Calendar OnShow and OnHide consumers handle the resulting selections and supplied civil date and club state.

Retail build 70688 evidence verifies separate identity-and-date descriptors, local writers and table fields. Lookup-to-publication paths are emulated; runtime transitions remain unknown.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_calendar/selection.rs`: selection writers and current-list projection.
- `src/c_api/c_calendar.rs`: selection representation and namespace registration.
- `src/lua_api/state/sim_state.rs`: supplied event rows and selection state.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: identity reordering and original Calendar show/hide.

## Known gaps (current cycle)

- [ ] Secondary identity, occurrence matching, and the native server-request path are not modeled.
- [ ] Native event-removal notifications and their descriptor-clearing writers are not modeled.

## Out of scope

- Server event delivery and editable-event behavior.
