# Calendar context-menu permissions

Retail `C_Calendar.ContextMenuEventCanEdit`, `ContextMenuEventCanRemove`, and `ContextMenuEventCanComplain` use supplied day-event identity, category and role markers, and local guild, editable-event and complaint state. Pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:47,64,81`.

## What it must do

- [ ] Resolve supplied day coordinates, reject missing entries and return one boolean for each predicate.
- [ ] For ordinary categories, edit and removal require either role marker 0x2 or 0x4. Special categories use the separate editable event, associated guild identity and respective guild rights; the matching-guild removal branch accepts 0x4 but not 0x2 alone.
- [ ] Complaint rejects pending state, excluded category flags, local-player targets and, when the legacy-friend CVar is enabled, matching legacy-friend identities. Otherwise it follows the supplied complaint-policy conjunction or integer.
- [ ] The original Calendar context-menu consumer reflects supplied allowed and rejected states.

These decisions follow Retail 70688 recovered local code; complete wrapper execution and runtime transitions are unknown. Category bits and role markers are independent of displayed calendar-type text.

## How it works

- [Calendar event selection](calendar-event-selection.md)

## Implementation inventory

- `src/c_api/c_calendar/context_policy.rs`: indexed lookup and policy decisions.
- `src/c_api/c_calendar/day_event_record.rs`: supplied role, category and target identity inputs.
- `src/c_api/c_calendar.rs`: policy state.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: original context-menu decisions with supplied ordinary and guild states.

## Known gaps (current cycle)

- [ ] Native production of the local-edit credential and runtime complaint-policy changes are unknown.
- [ ] Native listener delivery and complete argument coercion/secret policy remain unverified.

## Out of scope

- Server complaint processing and permission changes.
