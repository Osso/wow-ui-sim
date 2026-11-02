# Calendar editable-event reads

Retail calendar invite and edit controls read the separately supplied being-edited event, not the opened or context selection descriptors. Pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:246,290,280,356,742` respectively for `EventGetInvite`, `EventGetSelectedInvite`, `EventGetInviteSortCriterion`, `EventHaveSettingsChanged`, and `GetNumInvites`.

## What it must do

- [ ] `EventHaveSettingsChanged` returns one boolean from the editable record's dirty state, or false without a record; the original edit button follows it.
- [ ] `EventGetSelectedInvite` rematches the selected invite identity against the current list. It returns one-based position or 0 when unmatched, and one nil without an editable record.
- [ ] `EventGetInvite` publishes one fresh invite-info table for a valid supplied index and zero values for a missing record or index. The original invite button displays the supplied name.
- [ ] `EventGetInviteSortCriterion` returns the supplied criterion and direction, or empty string and false without an editable event; the original sort controls follow the criterion.
- [ ] `GetNumInvites` reflects the supplied invite list; zero without an editable event is a simulator best guess.

The result connections are based on Retail 70688 local decoding and inferred/emulated publication, not live-client proof.

## How it works

- [Calendar context permissions](calendar-context-permissions.md)

## Implementation inventory

- `src/c_api/c_calendar/editable_reads.rs`: editable-event and invite projections.
- `src/c_api/c_calendar.rs`: supplied event access and invite records.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: original edit button, invite button, selection and sort controls.

## Known gaps (current cycle)

- [ ] Other WM22 p1b editable reads, including event type, club, textures, response time, status options and pending invite, remain unimplemented.
- [ ] Native name/class lookup and null slots in the invite list are not modeled.
- [ ] `GetNumInvites` absent-event default, complete argument coercion, and secret restrictions are not verified.

## Out of scope

- Invite mutations, server delivery and editable-event creation.
