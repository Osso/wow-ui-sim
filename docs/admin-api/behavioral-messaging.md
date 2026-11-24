# Behavioral messaging

**VERIFIED (Retail/Forever `BehavioralMessagingDocumentation.lua:27-34`):** `BEHAVIORAL_NOTIFICATION` carries a non-nil `notificationType` string and `dbId` value. **INFERRED (R88fm (native evidence pending)):** `A_Admin.QueueBehavioralNotification(notificationType, dbId)` supplies this event payload synchronously.

**VERIFIED (Retail/Forever `Blizzard_BehavioralMessaging.lua:17-21`):** the original consumer maps `ComplaintWarning_Social` and `ComplaintThankYou_Social` to different locale labels and detail text. The values come from the active client locale.

**VERIFIED (Retail/Forever `BehavioralMessagingDocumentation.lua:11-19`):** `C_BehavioralMessaging.SendNotificationReceipt(dbId, openTimeSeconds, readTimeSeconds)` takes these three arguments; the declaration has no `receiptType` argument. **INFERRED (R88fm (native evidence pending)):** the simulator stores the arguments sent by the Blizzard consumer.

**INFERRED (R88fm (native evidence pending)):** `A_Admin.GetBehavioralNotificationReceipts()` returns a fresh dense array of stored records. Each record has `dbId`, `openTimeSeconds`, and `readTimeSeconds`.

```lua
A_Admin.QueueBehavioralNotification("ComplaintWarning_Social", 87001)
-- Open the notification and use its close button to acknowledge it.
local receipts = A_Admin.GetBehavioralNotificationReceipts()
```

The original consumer calls `SendNotificationReceipt` when it acknowledges a notification (`Blizzard_BehavioralMessaging.lua:162-166`).
