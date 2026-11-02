# Calendar editable event club

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:409-416` declares `C_Calendar.EventSetClubId`. The original community dropdown passes the selected club ID.

WM22 p2 verifies the optional WM17 club-ID parser but only infers its connection to the ordinary-record setter. This model applies that candidate connection: nil selects zero, an allowed change marks the supplied editable record dirty, and announcement permission remains supplied state. Permission failures report an error and clear pending state. No server delivery is implied.

`tests/blizzard_calendar_loads.rs` selects two supplied communities through the original dropdown and checks the resulting record, nil input and denied edit.
