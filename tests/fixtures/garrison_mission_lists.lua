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

CovenantMissionFrame:Show()
local list = CovenantMissionFrame.MissionTab.MissionList
assert(list.ScrollBox:GetView(), "Covenant mission frame initializes its original ScrollBox view")
list:UpdateMissions()
local providerMissions = list.ScrollBox:GetDataProvider():GetCollection()
local providerIDs = {}
for _, mission in ipairs(providerMissions) do
    providerIDs[mission.missionID] = true
end
assert(
    #providerMissions == 3 and providerIDs[900001] and providerIDs[900002] and providerIDs[900003],
    "Covenant mission scrollbox receives every supplied mission"
)
return #list.combinedMissions,
    list.combinedMissions[1].missionID,
    list.combinedMissions[2].missionID,
    list.combinedMissions[3].missionID
