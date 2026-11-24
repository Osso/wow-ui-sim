# Raid locks

`A_Admin.SetCompletedEncounters(rows)` replaces the supplied raid-lock rows. Each row has `mapID`, `difficultyID`, and an ordered `completedEncounterIDs` array. Example:

```lua
A_Admin.SetCompletedEncounters({
    { mapID=100, difficultyID=14, completedEncounterIDs={501, 502} },
    { mapID=100, difficultyID=15, completedEncounterIDs={501} },
})
```

VERIFIED (Forever 1.60.1.70707) R88fb: `C_RaidLocks.IsEncounterComplete(mapID, encounterID, difficultyID?)` scans valid matching-map rows. When `difficultyID` is supplied, it tests the matching map-and-difficulty row. When omitted, it tests only the first valid row for that map, then stops; it does not combine completions across difficulties.

INFERRED (Retail 12.1.5.70688) R88fb: the native loop bypasses the difficulty comparison when the optional argument is omitted. The matched-result continuation is unresolved, so the full Retail result behavior is not proved.

VERIFIED from the pinned `RaidLocksDocumentation.lua:27-40`: `C_RaidLocks.IsEncounterComplete(mapID, encounterID, difficultyID?)` returns a non-nil boolean, and `difficultyID` is optional. The simulator returns `true` only when the selected row contains `encounterID`; unmatched rows or IDs return `false`.

INFERRED (simulator behavior): `RequestRaidInfo()` queues `UPDATE_INSTANCE_INFO` for a later simulator timer tick. It does not dispatch the event during the function call.
