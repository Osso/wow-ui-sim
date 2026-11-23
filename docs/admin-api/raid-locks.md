# Raid locks

INFERRED R88fb (native evidence pending): supply saved raid-lock completions with `A_Admin.SetCompletedEncounters(entries)`. Each entry has `mapID`, `encounterID`, and `difficultyID`; calling the setter replaces the current supplied completion list.

VERIFIED from the pinned `RaidLocksDocumentation.lua:27-40`: `C_RaidLocks.IsEncounterComplete(mapID, encounterID, difficultyID?)` returns a non-nil boolean, and `difficultyID` is optional.

INFERRED R88fb (native evidence pending): the query returns true only for a supplied matching map and encounter, and also matches difficulty when one is given. Without a difficulty, any matching lock is complete. Unmatched IDs return false.

INFERRED R88fb (native evidence pending): `RequestRaidInfo()` queues `UPDATE_INSTANCE_INFO` for a later simulator timer tick. It does not dispatch the event during the function call.
