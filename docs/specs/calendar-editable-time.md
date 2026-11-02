# Calendar editable event time

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:476-484` declares `EventSetTime(hour, minute)`. The original Calendar time control passes its selected hour and minute.

WM30 describes the local setter: required numeric inputs are converted to unsigned 32-bit values, truncating fractions. Values whose signed 32-bit interpretation is at least 24 hours or 60 minutes are rejected. Argument errors leave state unchanged. An allowed changed candidate marks the editable record dirty before range validation. A local rejection preserves the time, emits `CALENDAR_UPDATE_ERROR` with `CALENDAR_ERROR_INVALID_TIME`, clears action pending, then emits `CALENDAR_ACTION_PENDING(false)`. Missing records and denied edits take the same local error path. Announcement permission remains supplied state.

The modeled time starts as an unsupplied value. WM30 does not establish general return arity, listener delivery or server effects. The original time control test covers two supplied times and an invalid time.
