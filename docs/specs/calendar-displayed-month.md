# Calendar displayed month and club selection

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:693,708,846,867` declares month and next-club queries and setters. `DateAndTimeDocumentation.lua:90` declares the current calendar-time query.

## What it must do

- [ ] Project one host-supplied civil date, initially 2026-04-14 12:00, as current calendar time.
- [ ] `SetAbsMonth` requires month and year, applies the current-year ±5 clamp followed by the 2004–2030 clamp, and selects the normalized month. It publishes `CALENDAR_UPDATE_EVENT_LIST`.
- [ ] `GetMonthInfo` projects the displayed Gregorian month and supplied offset, with day count and first weekday.
- [ ] `SetMonth` advances the displayed month by the supplied relative offset and bounds it to November 2004 through December 2030; the original month-navigation consumer reflects the new month.
- [ ] The original calendar OnShow selects the current month for two supplied dates.
- [ ] `SetNextClubId` retains an optional selection, including numeric zero; nil and omission clear it. The original calendar OnHide clears it.
- [ ] Club numeric range, conversion, valid large hexadecimal inputs, and failure behavior follow WM17.

## Best-guess basis

Retail 12.1.5 build 70688 evidence verifies local month calculation and club state decisions. Month-set completion and club-set completion were emulated, not verified end-to-end. The month event has a static association; synchronous delivery is simulator policy. Getter representation of club IDs above 2^53-1 is a best guess. Other profiles' current-time publication follows their pinned declarations, not independently verified native semantics.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_calendar/civil.rs`: supplied current time and Gregorian displayed-month queries.
- `src/c_api/c_calendar/club.rs`: optional next-club selection.
- `src/c_api/c_date_and_time.rs`: current-time publication.
- `src/lua_api/workarounds/temporary/date_and_time_defaults.rs`: remaining temporary date helpers.
- `src/lua_api/state/sim_state.rs`: supplied civil and calendar state.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: original OnShow and OnHide consumers, date normalization, and club selection.

## Known gaps (current cycle)

- [ ] Native event timing, downstream calendar refresh effects, and exact secret-context behavior are unknown. `SetMonth` does not model the native shared refresh operation yet.
- [ ] Club selection persistence across creation, closing, reload, or login is unknown; large-ID getter representation is unverified.
- [ ] Existing temporary date-adjustment and epoch helpers do not model a complete server clock.

## Out of scope

- Server-side calendar data, membership validation, and event creation.
