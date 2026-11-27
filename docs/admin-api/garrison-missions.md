# Garrison missions

### `A_Admin.SetGarrisonCombatAllyMission(followerTypeID, missionID)`

Selects the combat-ally mission returned by `C_Garrison.GetCombatAllyMission(followerTypeID)` on Retail and WoW Forever.

- `followerTypeID` must be an unsigned integer.
- `missionID` must identify a mission already supplied with `A_Admin.SetGarrisonMissions` for that follower type. Pass `nil` to clear the selection.
- The default is no selected combat-ally mission. An unknown or mismatched mission is rejected without changing state.

The pinned Garrison API documentation does not declare `GetCombatAllyMission`. INFERRED (R88g9, native evidence pending): the API returns the supplied mission record or nil, based on the selected consumer at `Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonMissionUI.lua:867`, which stores the result for the Order Hall mission UI.
