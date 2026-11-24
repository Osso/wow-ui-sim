# Behavioral messaging

`A_Admin.QueueBehavioralNotification(notificationType, dbId)` supplies one scenario notification and synchronously fires `BEHAVIORAL_NOTIFICATION` with `notificationType` and `dbId`. The original Blizzard consumer supports `ComplaintWarning_Social` and `ComplaintThankYou_Social`; their labels and detail text come from the active client locale. This supplied-input model is **INFERRED (R88fm (native evidence pending))**.

`C_BehavioralMessaging.SendNotificationReceipt(dbId, openTimeSeconds, readTimeSeconds)` stores the values sent by the Blizzard consumer. Retail and Forever API documentation declares these three arguments; it does not declare a `receiptType` argument.

`A_Admin.GetBehavioralNotificationReceipts()` returns a fresh dense array of records. Each record has `dbId`, `openTimeSeconds`, and `readTimeSeconds`. The getter exposes acknowledgements made by the loaded behavioral messaging consumer. Receipt storage and the scenario notification producer are **INFERRED (R88fm (native evidence pending))**.

```lua
A_Admin.QueueBehavioralNotification("ComplaintWarning_Social", 87001)
-- Open the notification and use its close button to acknowledge it.
local receipts = A_Admin.GetBehavioralNotificationReceipts()
```

API declarations: Retail and Forever `Blizzard_APIDocumentationGenerated/BehavioralMessagingDocumentation.lua` (`SendNotificationReceipt`, `BEHAVIORAL_NOTIFICATION`). Consumer: `Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging.lua`.
