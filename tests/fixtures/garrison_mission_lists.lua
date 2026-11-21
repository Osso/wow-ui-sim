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
list:UpdateMissions()
return #list.combinedMissions,
    list.combinedMissions[1].missionID,
    list.combinedMissions[2].missionID,
    list.combinedMissions[3].missionID
