# Calendar copy, paste and removal

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:107,120,131,842` declares `C_Calendar.ContextMenuEventCopy`, `ContextMenuEventPaste(offsetMonths, monthDay)`, `ContextMenuEventRemove` and `RemoveEvent`. The original consumers are the day context menu copy and paste buttons (`Blizzard_Calendar/Mainline/Blizzard_Calendar.lua:2021,2237`), the `CALENDAR_DELETE_EVENT` confirmation (`:8`) and the view-event remove button (`:2939`).

Evidence: WM41-A, Retail 12.1.5 build 70688, reviewed static decoding (PARTIAL). Server responses, delivery and pending completion are UNKNOWN. The simulator models only local state, pending state and events.

## What it must do

- [ ] Copy resolves the context-menu selection, not the opened or editable event. Without a context row it changes nothing. Edit permission (the same inputs as `ContextMenuEventCanEdit`) is required. A denial reports `CALENDAR_ERROR_PERMISSIONS` and keeps the clipboard. Success stores the complete day-row snapshot as the clipboard and emits no event.
- [ ] Paste requires an idle action state and a clipboard. The target is the displayed month plus `offsetMonths`, clamped to November 2004 through December 2030. The day must fit that month. Busy state, an empty clipboard, an invalid day and out-of-window dates stop silently.
- [ ] The creation window uses the supplied calendar date (`GetCurrentCalendarTime`), not the displayed month or the host clock. The supplied clock time does not decide acceptance.
- [ ] Paste shares AddEvent's throttle: a nonzero previous timestamp fewer than 5000 ticks old reports `CALENDAR_ERROR_EVENT_THROTTLED`. Success refreshes the timestamp, sets pending and emits `CALENDAR_ACTION_PENDING(true)`. It creates no local event and keeps the clipboard and day rows.
- [ ] Context removal checks removal permission before pending state. A denial reports `CALENDAR_ERROR_PERMISSIONS`. An authorized busy request does nothing. Success sets pending and emits `CALENDAR_ACTION_PENDING(true)`, without deleting the row or changing selections.
- [ ] `RemoveEvent` needs an existing, non-new editable record and an idle action state. It finds the local player's own invitation; with none it does nothing. A `CREATOR` invitation reports `CALENDAR_ERROR_DELETE_CREATOR_FAILED`. Otherwise it sets pending and emits `CALENDAR_ACTION_PENDING(true)`, without changing the invite list.
- [ ] Every error emits `CALENDAR_UPDATE_ERROR` with one key string, clears pending and emits `CALENDAR_ACTION_PENDING(false)`. All four calls return zero values.

## Simulator mappings

- INFERRED window: the supplied calendar date through the last day of its month next year, intersected with 2004-11-24 through 2030-12-31. Dates strictly before or strictly after are rejected. Components compare in year, month, day order and skip any component that is negative on either side.
- INFERRED ticks: the native cached tick counter maps to the simulator clock (`GetTime`) in milliseconds, compared as a signed, wrapping 32-bit difference.
- Copy's second permission evaluation, which would clear the clipboard, reads the same supplied inputs as the first, so it cannot deny alone.
- The local player's own invitation matches the present local-player GUID ([Calendar event creation](calendar-event-creation.md)).

## How it works

- [Calendar context permissions](calendar-context-permissions.md)
- [Calendar event selection](calendar-event-selection.md)

## Implementation inventory

- `src/c_api/c_calendar/context_actions.rs`: copy, paste and both removal paths.
- `src/c_api/c_calendar/submission_policy.rs`: the shared throttle and creation window.
- `src/c_api/c_calendar/selection.rs`: the context-row lookup.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: the original copy button, including a stale menu after revoked permission; paste windows for two supplied calendar dates; the idle, clipboard, day and throttle rules; the delete confirmation; and the view remove button.

## Known gaps (current cycle)

- [ ] Paste's throttle position relative to target construction is UNKNOWN; it precedes the date policy as in AddEvent.
- [ ] Malformed dates, timezones, server messages and the copied auxiliary pointer are not modeled.
- [ ] Outgoing request contents are not recorded.
- [ ] `C_Calendar.IsActionPending` remains the temporary false default and does not read the modeled pending state.
