function QuestHelperSupplyInputs()
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
end
