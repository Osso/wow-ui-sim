# Garrison mission lists

Until the garrison mission model exists, `C_Garrison.GetInProgressMissions` and `C_Garrison.GetAvailableMissions` return empty lists on Retail, PTR and WoW Forever. With `(table, followerTypeID)`, they clear and return the supplied table. With `(followerTypeID)` or `(nil, followerTypeID)`, they return a new empty table. The follower type must be numeric; the first argument must be a table, number or nil.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonMissionUI.lua:871–872` pass output tables and a follower type. `Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonShipyardUI.lua:1173–1174` and `Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonLandingPage.lua:542` use a numeric first argument; `Blizzard_AdventureMap/AM_ZoneSummaryDataProvider.lua:87,94` uses nil-plus-number and number-only forms. No declaration was found in the pinned Retail generated API documentation. Empty results reflect the missing simulator mission model.

## Known gaps

Native clearing behavior and the return value for the supplied-table form are unknown. Replace this temporary workaround when the garrison mission model is implemented.
