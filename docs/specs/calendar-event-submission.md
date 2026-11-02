# Calendar event submission

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:11,877` declares `C_Calendar.AddEvent` and `C_Calendar.UpdateEvent` (both `HasRestrictions`). The original create button (`Blizzard_Calendar/Mainline/Blizzard_Calendar.lua:4057-4063`) calls `AddEvent` in create mode and `UpdateEvent` in edit mode.

Evidence: WM41-B, Retail 12.1.5 build 70688, reviewed static decoding (PARTIAL). Server requests and responses are UNKNOWN. The simulator models only local state, pending state, the dirty flag and events.

## What it must do

- [ ] `AddEvent` checks in order. Pending state stops silently. A missing editable record reports `CALENDAR_ERROR_INTERNAL`. The shared create/paste throttle reports `CALENDAR_ERROR_EVENT_THROTTLED`. A date outside the creation window stops silently, before the title check and without refreshing the throttle. A record that is not new stops silently.
- [ ] `AddEvent` then reports the first failure: an empty title (`CALENDAR_ERROR_NEEDS_TITLE`); a missing guild or community club (`ERR_GUILD_PLAYER_NOT_IN_GUILD`, `ERR_COMMUNITY_PLAYER_NOT_IN_COMMUNITY`); an unset hour, then minute (`CALENDAR_ERROR_INVALID_TIME`); an unset month, day or year (`CALENDAR_ERROR_INVALID_DATE`); and a date and time at or before the local cutoff (`CALENDAR_ERROR_EVENT_TIME_PASSED`).
- [ ] An accepted `AddEvent` refreshes the throttle, sets pending and emits `CALENDAR_ACTION_PENDING(true)`. It keeps the record's new and dirty flags.
- [ ] `UpdateEvent` stops silently for pending state, a missing record or a new record. Failed edit permission reports `CALENDAR_ERROR_PERMISSIONS`. A clean record stops silently. Otherwise dirty is cleared before the cutoff comparison. At or before the cutoff, it reports `CALENDAR_ERROR_EVENT_PASSED`; otherwise it sets pending and emits `CALENDAR_ACTION_PENDING(true)`. It does not repeat the title, size, throttle or window checks.
- [ ] Every error emits `CALENDAR_UPDATE_ERROR` with the error key, clears pending and emits `CALENDAR_ACTION_PENDING(false)`. Both calls return zero values. Neither emits `CALENDAR_NEW_EVENT`, `CALENDAR_UPDATE_EVENT` or an invite-list event.

## Simulator mappings

- The creation window and the 5000-tick throttle are shared with paste ([Calendar copy, paste and removal](calendar-context-actions.md)). The window reads the supplied calendar date.
- INFERRED cutoff: the supplied calendar date (`calendar_now`) with the game clock's hour and minute (`GetLocalGameTime`), minus one minute with day rollover. Components compare in year, month, day, hour, minute order and skip any negative component. An unknown game clock leaves hour and minute unset, so only the date decides.
- INFERRED club checks: guild (0x800) and announcement (0x40) records need a kind-2 club record. Other signup (0x400) records need a supplied club record matching their club ID.

## How it works

- [Calendar event creation](calendar-event-creation.md)
- [Calendar context permissions](calendar-context-permissions.md)

## Implementation inventory

- `src/c_api/c_calendar/event_submission.rs`: `AddEvent` and `UpdateEvent`.
- `src/c_api/c_calendar/submission_policy.rs`: the throttle, creation window, local cutoff and request outcome.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: the original create button in create and edit mode, covering the ordered checks, two game clock times on each side of the event, the 00:00 rollover, an unknown game clock and the dirty flag.

## Known gaps (current cycle)

- [ ] WM41-B's day and minute offset writer for the local clock is not modeled. The offset is taken as zero, and the local date is assumed to equal the calendar date.
- [ ] The protected-action gate is not modeled.
- [ ] The maximum-size check (`CALENDAR_UPDATE_ERROR_WITH_COUNT`) is not modeled; creation always sets the size to 100, and no simulator path changes it.
- [ ] Request contents (bounded text, type conversion, invitations) are not recorded.
