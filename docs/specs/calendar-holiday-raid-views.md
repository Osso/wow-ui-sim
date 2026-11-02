# Calendar holiday and raid views

Retail `C_Calendar.GetHolidayInfo` and `GetRaidInfo` project host-supplied calendar day records. Pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:657,760`.

## What it must do

- [ ] Look up the supplied offset, one-based day and one-based event index in the current previous/current/next month cache. A missing event returns zero values.
- [ ] Holiday lookup requires the holiday marker; a supplied holiday produces name, description, texture and, for a multi-day record with available name and description, start/end time.
- [ ] Raid lookup does not require a raid-category marker; a supplied day event projects name, calendar type, raid ID, time and difficulty. The original raid view displays the result only for RAID_LOCKOUT.
- [ ] Original holiday and raid view consumers render supplied records, and do not update for absent holiday or non-lockout raid view data.

Lookup and category decisions are based on Retail 70688 recovered local behavior. Success field serializers are inferred candidates; protected connection is emulated rather than live-client verified.

## How it works

- [Calendar day events](calendar-day-event-records.md)

## Implementation inventory

- `src/c_api/c_calendar/day_event_record.rs`: supplied holiday and raid metadata.
- `src/c_api/c_calendar/view_info.rs`: indexed lookup and view projections.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: original holiday and raid views with supplied records.

## Known gaps (current cycle)

- [ ] Native holiday texture phase-zero fallback is not modeled; the host supplies the selected texture.
- [ ] Complete native string, argument, and secret-handling behavior is unknown.

## Out of scope

- Server event catalogs and live-client reachability.
