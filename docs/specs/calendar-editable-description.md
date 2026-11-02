# Calendar editable event description

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:431-438` declares `C_Calendar.EventSetDescription`. The original Calendar description control calls it on user changes.

The setter compares the NUL-terminated input to the supplied editable record before filtering. A changed value removes vertical bars, truncates at 1,024 bytes or 256 non-continuation-byte counts, and marks the record dirty. Newlines are retained. Missing records and denied edits do not change the record; announcement authority remains supplied state.

WM22 p2 is bounded static evidence, not live-client proof. Filter ordering relative to limits, numeric coercion and complete secret restrictions remain unknown. No server delivery is implied.

`tests/blizzard_calendar_loads.rs` exercises the original control with allowed and denied edits.
