# Calendar guild and context reads

Retail guild-event navigation and context menus use the supplied guild-event selection list, context selection, and clipboard. Pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:98,111,641`.

## What it must do

- [ ] `GetGuildEventSelectionInfo` rematches the supplied indexed guild selection against the current day list and returns one index-info table, or zero values when no match exists.
- [ ] `ContextMenuEventClipboard` returns one boolean for clipboard presence, independent of context selection.
- [ ] `ContextMenuEventGetCalendarType` resolves the current context selection and returns one string for a selected event, empty for unrecognized type, or one nil if absent.
- [ ] Original Calendar guild navigation, context paste menu and delete confirmation consume these reads with supplied state.

These decisions derive from Retail 70688 recovered local behavior; protected publication is emulated, not live-client verified.

## How it works

- [Calendar event selection](calendar-event-selection.md)

## Implementation inventory

- `src/c_api/c_calendar/context_reads.rs`: guild rematch and context reads.
- `src/c_api/c_calendar/selection.rs`: shared date/identity resolution.
- `src/lua_api/state/sim_state.rs`: supplied guild and clipboard records.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: guild navigation, paste menu and delete confirmation.

## Known gaps (current cycle)

- [ ] The tests and producer are committed but await RED and GREEN runs after the suite-admission build hold.
- [ ] Secondary identities, occurrence matching, complete argument/secret behavior and runtime transition timing remain unknown or unmodeled.

## Out of scope

- Server responses and clipboard mutation commands.
