# Garrison follower level XP

`C_Garrison.GetFollowerXPTable(followerTypeID)` accepts a numeric follower type and returns a fresh table mapping follower levels to XP required for the next level. Rows are generated from `GarrFollowerLevelXP.csv` build 12.1.5.70391 by `python3 tools/gen_garrison_follower_xp.py`. Unknown follower types return an empty table. The query is published for Retail and PTR.

## Best-guess basis (best-guess, native UNKNOWN)

No generated API declaration was found in the pinned Retail or PTR documentation. Both profiles' `Blizzard_GarrisonUI/Mainline/Blizzard_AdventuresRewardsScreen.lua:13` call this query; line 201 indexes its result by follower level for the next-level XP. The level-to-XP mapping and empty result for an unknown type are inferred from that use. Values come from the client DB2 export.

## Known gaps

Native validation, unknown-type behavior, and other possible fields or indexing semantics are unknown. The export was not obtained separately for PTR.
