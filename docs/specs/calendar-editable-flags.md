# Calendar editable event flags

Retail's Calendar create-event checkboxes call `EventSetAutoApprove`, `EventClearAutoApprove`, `EventSetLocked`, and `EventClearLocked`. The pinned declarations are `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:206,210,405,452`.

## Behavior

- [ ] Each operation changes its corresponding supplied editable-event flag only when local edit permission is granted. A changed flag marks the event dirty; an unchanged flag does not.
- [ ] Missing records and denied edits leave the flags unchanged. Announcement privilege is a supplied policy decision, as in [calendar title edits](calendar-editable-title.md).

WM22 p2 establishes bounded local effects, not complete protected-wrapper returns, live-client behavior, or server delivery.

## Tests

- `tests/blizzard_calendar_loads.rs`: original auto-approve and lock checkbox consumers exercise both set and clear transitions.
