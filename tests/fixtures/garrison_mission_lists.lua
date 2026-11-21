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
})

local list = CovenantMissionFrame.MissionTab.MissionList
list:UpdateMissions()
return #list.availableMissions,
    #list.inProgressMissions,
    list.availableMissions[1].missionID,
    list.inProgressMissions[1].missionID
