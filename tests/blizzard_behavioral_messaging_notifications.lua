local function find_notification(notification_type)
    for notification in BehavioralMessagingTray.pool:EnumerateActive() do
        if notification.notificationType == notification_type then
            return notification
        end
    end
end

local function acknowledge_notification(notification_type, db_id)
    local notification = find_notification(notification_type)
    assert(notification and notification:IsShown(), "notification button was not shown")
    local label = notification.TitleText:GetText()
    notification:Click()
    local details = BehavioralMessagingDetails.Body.BodyText:GetText()
    assert(details and details ~= "", "notification details were not rendered")
    BehavioralMessagingDetails.CloseButton:Click()
    local receipts = A_Admin.GetBehavioralNotificationReceipts()
    local receipt = receipts[#receipts]
    assert(receipt and receipt.dbId == db_id, "consumer receipt was not recorded")
    assert(type(receipt.openTimeSeconds) == "number", "receipt open time was not recorded")
    assert(type(receipt.readTimeSeconds) == "number", "receipt read time was not recorded")
    return label, details
end

A_Admin.QueueBehavioralNotification("ComplaintWarning_Social", 87001)
local warning_label, warning_details = acknowledge_notification("ComplaintWarning_Social", 87001)
A_Admin.QueueBehavioralNotification("ComplaintThankYou_Social", 87002)
local thanks_label, thanks_details = acknowledge_notification("ComplaintThankYou_Social", 87002)
assert(warning_label ~= thanks_label, "notification labels should differ")
assert(warning_details ~= thanks_details, "notification details should differ")
local receipts = A_Admin.GetBehavioralNotificationReceipts()
assert(#receipts == 2, "both consumer receipts should be recorded")
assert(receipts[1].dbId == 87001 and receipts[2].dbId == 87002, "receipts should preserve notification IDs")
