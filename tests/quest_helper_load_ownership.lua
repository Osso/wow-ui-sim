QuestHelperProbe = { events = 0, dirty = 0, containerUpdates = 0, moduleUpdates = 0, watchBuilds = 0 }
hooksecurefunc(QuestObjectiveTracker, "OnEvent", function(_, event)
    if event == "QUEST_WATCH_LIST_CHANGED" or event == "QUEST_LOG_UPDATE" then QuestHelperProbe.events = QuestHelperProbe.events + 1 end
end)
hooksecurefunc(QuestObjectiveTracker, "MarkDirty", function() QuestHelperProbe.dirty = QuestHelperProbe.dirty + 1 end)
hooksecurefunc(ObjectiveTrackerFrame, "Update", function() QuestHelperProbe.containerUpdates = QuestHelperProbe.containerUpdates + 1 end)
hooksecurefunc(QuestObjectiveTracker, "Update", function() QuestHelperProbe.moduleUpdates = QuestHelperProbe.moduleUpdates + 1 end)
hooksecurefunc(QuestObjectiveTracker, "BuildQuestWatchInfos", function() QuestHelperProbe.watchBuilds = QuestHelperProbe.watchBuilds + 1 end)
C_Timer.After(0, function() QuestHelperProbe.nextFrame = true end)
local firstID = 990001
local secondID = 990002
A_Admin.SetQuestLogEntry(firstID, "Group and item", false)
A_Admin.SetQuestLogEntry(secondID, "No group or item", false)
A_Admin.SetQuestGroupCreatable(firstID, true)
A_Admin.SetQuestGroupCreatable(secondID, false)
local firstIndex = C_QuestLog.GetLogIndexForQuestID(firstID)
local secondIndex = C_QuestLog.GetLogIndexForQuestID(secondID)
assert(firstIndex and secondIndex and firstIndex ~= secondIndex)
A_Admin.SetQuestLogSpecialItem(firstIndex, 19001)
A_Admin.SetQuestLogSpecialItem(secondIndex, nil)
C_QuestLog.AddQuestWatch(firstID)
C_QuestLog.AddQuestWatch(secondID)
A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", firstID, true)
A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", secondID, true)
