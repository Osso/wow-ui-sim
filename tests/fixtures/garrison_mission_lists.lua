local followerTypeID = CovenantMissionFrame.followerTypeID
A_Admin.SetGarrisonMissions({
    {
        missionID = 900001,
        name = "Supplied available mission",
        followerTypeID = followerTypeID,
        level = 80,
        durationSeconds = 3600,
        rewards = {},
        inProgress = false,
        completed = false,
    },
    {
        missionID = 900002,
        name = "Supplied in-progress mission",
        followerTypeID = followerTypeID,
        level = 81,
        durationSeconds = 7200,
        rewards = {},
        inProgress = true,
        completed = false,
    },
    {
        missionID = 900003,
        name = "Supplied completed mission",
        followerTypeID = followerTypeID,
        level = 82,
        durationSeconds = 10800,
        rewards = {},
        inProgress = false,
        completed = true,
    },
})

local list = CovenantMissionFrame.MissionTab.MissionList
if not list.ScrollBox:GetView() then
    local view = CreateScrollBoxListLinearView()
    view:SetElementInitializer("CovenantMissionListButtonTemplate", function(button, elementData)
        CovenantMissionButton_InitButton(button, elementData)
    end)
    view:SetPadding(8, 0, 13, 0, -4)
    ScrollUtil.InitScrollBoxListWithScrollBar(list.ScrollBox, list.ScrollBar, view)
end
list:UpdateMissions()
return #list.combinedMissions,
    list.combinedMissions[1].missionID,
    list.combinedMissions[2].missionID,
    list.combinedMissions[3].missionID
