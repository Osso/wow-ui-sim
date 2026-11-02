# Calendar editable event title

Retail's title edit control calls `C_Calendar.EventSetTitle` for user changes. The pinned declaration is `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:487`.

## Behavior

- [ ] The editable record is distinct from the opened and context selections. With ordinary local edit authority, the setter compares the NUL-terminated input bytes to the stored title before filtering and marks a difference dirty.
- [ ] The title filter removes vertical bars, truncates at carriage return, line feed or literal backslash followed by lowercase `n`, and limits the output to 128 bytes and 32 non-continuation-byte counts.
- [ ] Missing records and denied edits do not change the title. Announcement edit privilege is a separately supplied policy decision; its native derivation is not established.

## Evidence and gaps

WM22 p2 is bounded static evidence, not live-client proof. The filter's ordering relative to limits is a simulator interpretation. Numeric argument coercion, complete secret restrictions and announcement privilege derivation remain unknown. This slice models local title edits only; no server delivery is implied.

## Tests

- `tests/blizzard_calendar_loads.rs`: original title edit control with supplied edit authority and denied edits.
