local followerTypeID = CovenantMissionFrame.followerTypeID
A_Admin.SetGarrisonMissions({
    {
        missionID = 900001,
        name = "Supplied available mission",
        followerTypeID = followerTypeID,
        level = 80,
        durationSeconds = 3600,
        duration = "1 hour",
        timeLeft = "",
        xp = 100,
        canBeCompleted = false,
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
        duration = "2 hours",
        timeLeft = "1 hour remaining",
        xp = 200,
        canBeCompleted = false,
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
        duration = "3 hours",
        timeLeft = "",
        xp = 300,
        canBeCompleted = false,
        rewards = {},
        inProgress = false,
        completed = true,
    },
})
A_Admin.SetGarrisonFollowerQualityTable(followerTypeID, { [1] = 11, [2] = 22 })
A_Admin.SetGarrisonCombatAllyMission(followerTypeID, 900001)

-- VERIFIED: generated sim-data MapArt rows give maps 1550 and 1552 these canvas dimensions.
A_Admin.SetAdventureMapMapID(1550)
CovenantMissionFrame:Show()
local firstCanvasWidth, firstCanvasHeight = CovenantMissionFrame.MapTab:GetCanvas():GetSize()
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
local combatAllyMission = C_Garrison.GetCombatAllyMission(followerTypeID)
assert(
    combatAllyMission and combatAllyMission.missionID == 900001,
    "the supplied combat-ally mission is available to the original consumer"
)
local combatAllyMissionID = list.combatAllyMission and list.combatAllyMission.missionID or 0
assert(
    combatAllyMissionID == 900001,
    "the original mission-list consumer stores the supplied combat-ally mission"
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
CovenantMissionFrame.MapTab:SetMapID(1552)
local secondCanvasWidth, secondCanvasHeight = CovenantMissionFrame.MapTab:GetCanvas():GetSize()
return #list.combinedMissions,
    combatAllyMissionID,
    list.combinedMissions[1].missionID,
    list.combinedMissions[2].missionID,
    list.combinedMissions[3].missionID,
    firstCanvasWidth,
    firstCanvasHeight,
    secondCanvasWidth,
    secondCanvasHeight
