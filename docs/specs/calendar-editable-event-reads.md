# Calendar editable-event reads

Retail calendar invite and edit controls read the separately supplied being-edited event, not the opened or context selection descriptors. Pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:246,290,280,356,742` respectively for `EventGetInvite`, `EventGetSelectedInvite`, `EventGetInviteSortCriterion`, `EventHaveSettingsChanged`, and `GetNumInvites`.

## What it must do

- [ ] `EventHaveSettingsChanged` returns one boolean from the editable record's dirty state, or false without a record; the original edit button follows it.
- [ ] `EventGetSelectedInvite` rematches the selected invite identity against the current list. It returns one-based position or 0 when unmatched, and one nil without an editable record.
- [ ] `EventGetInvite` publishes one fresh invite-info table for a valid supplied index and zero values for a missing record or index. The original invite button displays the supplied name.
- [ ] `EventGetInviteSortCriterion` returns the supplied criterion and direction, or empty string and false without an editable event; the original sort controls follow the criterion.
- [ ] `GetNumInvites` reflects the supplied invite list; zero without an editable event is a simulator best guess.

- [ ] `EventGetCalendarType` projects the known guild, community, system, holiday, lockout and reset flags; without an editable event it returns nil.
- [ ] `EventGetClubId` returns the supplied club ID as an exact number or a prefixed, 16-digit uppercase hexadecimal string; without an editable event it returns nil.
- [ ] `EventGetInviteResponseTime` returns the supplied invite's time table, or zero values when the event, invite or response time is absent.
- [ ] `EventGetTextures` publishes supplied texture records: types 1 and 5 share a list, type 0 uses another, and types 2–4 return an empty list.
- [ ] `EventHasPendingInvite` matches the editable event ID against the supplied pending IDs and returns false without an event or match.

The result connections are based on Retail 70688 local decoding and inferred/emulated publication, not live-client proof.

## How it works

- [Calendar context permissions](calendar-context-permissions.md)

## Implementation inventory

- `src/c_api/c_calendar/editable_reads.rs`: editable-event and invite projections.
- `src/c_api/c_calendar.rs`: supplied event access and invite records.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: original edit button, invite button, selection and sort controls.

## Known gaps (current cycle)

- [ ] `EventGetStatusOptions` remains nil: native status strings and the complete permission predicate are unknown. The positive pending-invite menu also depends on this member.
- [ ] `EventCanEdit` remains unimplemented. Event type's PLAYER flag masks and their priority relative to other flags are unknown.
- [ ] Native name/class lookup and null slots in the invite list are not modeled.
- [ ] `GetNumInvites` absent-event default, complete argument coercion, and secret restrictions are not verified.

## Out of scope

- Invite mutations, server delivery and editable-event creation.
