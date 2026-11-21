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
        encounterIconInfo = {
            portraitFileDataID = 12345,
            missionScalar = 1.25,
            isElite = true,
            isRare = false,
        },
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
A_Admin.SetGarrisonFollowerQualityTable(followerTypeID, { [1] = 11, [2] = 22 })

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
local encounterInfo = C_Garrison.GetMissionEncounterIconInfo(900001)
assert(
    encounterInfo.portraitFileDataID == 12345
        and encounterInfo.missionScalar == 1.25
        and encounterInfo.isElite
        and not encounterInfo.isRare,
    "mission encounter icon info comes from the supplied record"
)
local emptyEncounterInfo = C_Garrison.GetMissionEncounterIconInfo(999999)
assert(
    emptyEncounterInfo.portraitFileDataID == 0
        and emptyEncounterInfo.missionScalar == 0
        and not emptyEncounterInfo.isElite
        and not emptyEncounterInfo.isRare,
    "unsupplied mission encounter icon info has the documented empty shape"
)
local qualityTable = C_Garrison.GetFollowerQualityTable(followerTypeID)
assert(
    qualityTable[1] == 11
        and qualityTable[2] == 22
        and next(C_Garrison.GetFollowerQualityTable(followerTypeID + 1)) == nil,
    "follower quality table uses supplied entries and an inferred empty default"
)
return #list.combinedMissions,
    list.combinedMissions[1].missionID,
    list.combinedMissions[2].missionID,
    list.combinedMissions[3].missionID
