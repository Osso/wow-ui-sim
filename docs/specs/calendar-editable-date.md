# Calendar editable event date

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:419-428` declares `EventSetDate(month, monthDay, year)`. The original Calendar create-event control passes the selected day and displayed month.

WM30 describes the local setter: arguments are required numeric values in the unsigned 32-bit range. Month and day are reduced by one before truncation; year is truncated and reduced by 2,000, then by another 2,000 if the signed result is at least 2,000. Converted month 12 or more, day 31 or more, and year above 31 are rejected. There is no lower or month-length check. For overflowing signed month/day conversion, the model uses the inferred masked-floating-exception result; live exception behavior is unknown.

An allowed changed candidate marks the supplied record dirty before validation. Rejection preserves the date, emits `CALENDAR_UPDATE_ERROR` with `CALENDAR_ERROR_INVALID_DATE`, clears action pending, then emits `CALENDAR_ACTION_PENDING(false)`. Missing records and denied edits use the same local error path. Announcement authority remains supplied state. Wrapper return arity, listener delivery and server outcomes remain unknown.
