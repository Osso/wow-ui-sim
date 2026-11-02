# Calendar event creation

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:177,181,185,189` declares `C_Calendar.CreateCommunitySignUpEvent`, `CreateGuildAnnouncementEvent`, `CreateGuildSignUpEvent` and `CreatePlayerEvent`. The original day context menu (`Blizzard_Calendar/Mainline/Blizzard_Calendar.lua:1988-2018`) calls them before it shows the create-event frame.

Evidence: WM40, Retail 12.1.5 build 70688, reviewed static decoding (PARTIAL for all four members). It is not live-client proof.

## What it must do

- [ ] Each call returns zero values and parses no arguments.
- [ ] A successful call replaces the editable event record and its invites, and clears the opened-event selection. The context-menu selection is kept.
- [ ] The new record has event ID 0, event type 0, empty title and description, texture ID 4294967295, and locked and auto-approve false. Its creator is the local-player GUID.
- [ ] Month, day and relative year are -1 (unset). Hour and minute are -1, stored as all-ones unsigned 32-bit words. They are not the selected day or the current time.
- [ ] Category flags are 0x1 (player), 0xc00 (guild signup), 0x40 (guild announcement) and 0x400 (community signup). `EventGetCalendarType` then reports `PLAYER`, `GUILD_EVENT`, `GUILD_ANNOUNCEMENT` and `COMMUNITY_EVENT`.
- [ ] Player and both signup calls add one self-invite when a local-player GUID is present: invite ID 0, status 3 (confirmed), moderator status `CREATOR`, empty notes and the player's level byte. The invite type is 0 for player events and 1 for signups. Announcements add no invite.
- [ ] Both guild calls need a supplied kind-2 club record; the first one supplies the record's club ID. Without one, the call emits `CALENDAR_UPDATE_ERROR("ERR_GUILD_PLAYER_NOT_IN_GUILD")`, clears action-pending and emits `CALENDAR_ACTION_PENDING(false)`. It does not replace the record or clear the selection.
- [ ] Only `CreatePlayerEvent` emits an event on success: `CALENDAR_UPDATE_INVITE_LIST` with exactly one `true` argument. WM40 corrects WM22's "two true flags": those encode one optional boolean's value and presence.

## Simulator mappings

- INFERRED: native self-invites need nonzero GUID type bits. Every valid GUID kind has a nonzero type field and an empty GUID is zero, so the simulator uses "local player present (`player.present`) with a GUID".
- A kind-2 record key that is not a decimal or `0x` hexadecimal club ID (such as the legacy roster's simulator key) supplies no association.
- The initial club ID is native-UNKNOWN for every kind. The simulator uses 0. Guild creation marks the record dirty only when the assigned ID differs from that placeholder.

## How it works

- [Calendar editable-event reads](calendar-editable-event-reads.md)
- [Calendar editable club](calendar-editable-club.md)

## Implementation inventory

- `src/c_api/c_calendar/create_event.rs`: the four constructors.
- `src/c_api/c_calendar/club.rs`: club ID parsing for supplied club records.
- `src/c_api/c_calendar/editable_reads.rs`: the `PLAYER` calendar type.

## Tests asserting this spec

- `tests/blizzard_calendar_loads.rs`: the original `CalendarDayContextMenu_CreateEvent` consumer, notification order and arguments, two supplied guild club IDs, and present or absent local players.

## Known gaps (current cycle)

- [ ] `GetEventInfo` is not implemented. The signup default of status 7 (not signed up) and invite type 1 without a self-invite cannot be observed yet.
- [ ] Unmodeled: repeat option (0), maximum size (100), weekday (-1) and lockout time (zeros). They have no simulator field or reader.
- [ ] UNKNOWN: the initial club ID, texture-list contents, club cache population, `CALENDAR_NEW_EVENT` and other indirect notifications, the policy guards before the return, and server outcomes.
- [ ] There is no player object or zero-type GUID input beyond `player.present`. The level byte is 0 without a player object, but that state creates no invite in this model.
- [ ] The `PLAYER` flag's priority relative to other category flags is unknown; it is checked last.
