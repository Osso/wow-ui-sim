# Calendar pending-invite lookup

Retail `C_Calendar.GetFirstPendingInvite` projects a host-supplied pending flag on calendar day-event records. Its pinned declaration is `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:609`.

## What it must do

- [ ] Search the supplied events for the specified day in order and return the one-based index of the first pending event, or one nil if none matches or the date is outside the previous/current/next month or invalid for that month.
- [ ] Keep pending status independent of opened, context-menu and editable selections. The lookup emits no events.
- [ ] The original Calendar month-view update consumes this result.

The return path is based on end-to-end emulation of Retail build 70688, not live-client observation.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_calendar/day_event_record.rs`: supplied pending flag.
- `src/c_api/c_calendar/day_event_queries.rs`: date validation and ordered lookup.
- `src/c_api/c_calendar.rs`: Retail publication.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: pending event positions and original Calendar OnShow.

## Known gaps (current cycle)

- [ ] Complete native numeric conversion and argument-error details await WM22 review.

## Out of scope

- Server delivery and pending-invite mutations.
