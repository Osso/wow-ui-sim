# Calendar editable event type

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:497-504` declares `EventSetType(typeIndex)`. The original Calendar texture picker sets the event type.

WM39 describes local mutation: required inputs convert to signed 32-bit numbers, truncate and retain the low byte; only 0–5 are accepted. Failed non-nil coercions produce zero. With edit authority, stored type 5 compares as 1; a different requested type is stored unchanged and marks the record dirty. No other field is reset. Announcement authority remains supplied state.

The wrapper's complete return arity, security transitions and indirect effects remain unknown. `tests/blizzard_calendar_loads.rs` exercises the original picker.
