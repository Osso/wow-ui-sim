# Perks activities

Retail and WoW Forever expose the Traveler's Log snapshot through `C_PerksActivities`. **VERIFIED (pinned Retail and Forever `PerksActivitiesDocumentation.lua`):** the public activity, condition, threshold, tag, and tracking result fields are listed in that API document. This simulator-only input supplies the snapshot; it does not claim to fetch live account data.

### `A_Admin.SetPerksActivities(snapshotJson)`

Pass one JSON string. The object must contain `activePerksMonth`, `displayMonthName`, `secondsRemaining`, `activities`, `thresholds`, `tags`, `pendingIDs`, `themePrefix`, and `trackedIDs`. The setter rejects malformed JSON, unknown fields, missing required fields, non-positive IDs, and duplicate activity IDs. It parses the full snapshot before it changes state. A rejected snapshot leaves the prior state unchanged. It emits no event.

Each activity row uses the public `PerksActivityInfo` field names: `ID`, `activityName`, `description`, `thresholdContributionAmount`, `completed`, `inProgress`, `supersedes`, `uiPriority`, `areAllConditionsMet`, `conditions`, `eventName`, `eventStartTime`, `eventEndTime`, `requirementsList`, `criteriaList`, `tagNames`, and `chatLink`. `eventName`, `eventStartTime`, and `eventEndTime` may be `null` or omitted. `tracked` is not accepted in a row; `trackedIDs` supplies that state. Each condition has `isMet`, `text`, and `uiPriority`; each requirement has `completed` and `requirementText`; each criterion has `criteriaID` and `requiredValue`. Each threshold has `thresholdOrderIndex`, `currencyAwardAmount`, `requiredContributionAmount`, `pendingReward`, and optional `itemReward`.

Activity, pending, and tracked IDs must be positive integers. Activity IDs must be unique. The setter supplies data only; calls to `C_PerksActivities.AddTrackedPerksActivity` and `RemoveTrackedPerksActivity` retain their own event behavior.

```lua
A_Admin.SetPerksActivities([==[
{
  "activePerksMonth": 11,
  "displayMonthName": "November",
  "secondsRemaining": 3600,
  "activities": [{
    "ID": 901,
    "activityName": "Copper tasks",
    "description": "",
    "thresholdContributionAmount": 0,
    "completed": false,
    "inProgress": false,
    "supersedes": 0,
    "uiPriority": 0,
    "areAllConditionsMet": false,
    "conditions": [],
    "requirementsList": [],
    "criteriaList": [],
    "tagNames": [],
    "chatLink": ""
  }],
  "thresholds": [],
  "tags": [],
  "pendingIDs": [],
  "themePrefix": "",
  "trackedIDs": []
}
]==])
```

Unset native Traveler's Log data, native getter defaults, and snapshot event timing remain unknown. The model uses explicit host input and does not invent a live rotation.
