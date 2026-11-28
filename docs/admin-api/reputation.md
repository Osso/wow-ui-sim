# Reputation

`A_Admin.SetReputationFactions(rows[, guildRow])` supplies the Retail and WoW Forever reputation list that backs `C_Reputation` (see [modern reputation host state](../specs/modern-reputation-host-state.md)). It replaces the whole row list and the guild row, then fires `UPDATE_FACTION`. It does not infer faction acquisition, server state or rewards. Other profiles keep their seeded legacy fixture and do not register this function.

Each row uses the `FactionData` field names declared in both pins' `Blizzard_APIDocumentationGenerated/ReputationInfoDocumentation.lua`:

- Required integers: `factionID`, `reaction`, `currentReactionThreshold`, `nextReactionThreshold`, `currentStanding`.
- Optional strings: `name`, `description`.
- Optional booleans (absent means false): `atWarWith`, `canToggleAtWar`, `isChild`, `isHeader`, `isHeaderWithRep`, `isCollapsed`, `hasBonusRepGain`, `canSetInactive`, `isAccountWide`.
- Optional `isWatched`: at most one row with a positive `factionID` may set it; it becomes the faction that `C_Reputation.GetWatchedFactionData` returns.
- Simulator activity inputs: optional boolean `isInactive` and optional integer `reputationIndex` (0–65535) feed `C_Reputation.IsFactionActive` and `SetFactionActive`.

Rows keep the supplied order. Header and child flags group following rows. The selected faction is kept only when its ID is still supplied. An invalid row raises `A_Admin.SetReputationFactions: invalid <field>`.

`UPDATE_FACTION` after the replacement is INFERRED: a supplied list stands for a server reputation update, which the original `StatusTrackingManagerMixin` (Mainline `StatusTrackingManagerOverrides.lua:3`) and `ReputationFrame` observe through that event.

```lua
A_Admin.SetReputationFactions({
    { factionID = 2590, name = "Council of Dornogal", reaction = 5,
      currentReactionThreshold = 3000, nextReactionThreshold = 9000,
      currentStanding = 4500, isWatched = true },
})
```

## SetMajorFactionRenown

`A_Admin.SetMajorFactionRenown(factionID, renownLevel, maxLevel)` supplies Renown progress for one major faction on every profile that registers `C_MajorFactions`. It updates an existing row or adds an unlocked row with no imported name or metadata. `GetRenownLevels` then lists levels `1..maxLevel`; levels above `renownLevel` are locked and the last level is the capstone (INFERRED). Validation: `factionID` positive, `maxLevel` a positive integer, `renownLevel` within `0..maxLevel`; invalid input raises `A_Admin.SetMajorFactionRenown: ...` before any change.

A level change fires the documented synchronous `MAJOR_FACTION_RENOWN_LEVEL_CHANGED(majorFactionID, newRenownLevel, oldRenownLevel)` (Forever `MajorFactionsDocumentation.lua:233-242`); a new row counts as old level 0 (INFERRED). Forever's Legacy system reads faction `Constants.LegacyConsts.LEGACY_REWARD_TRACK_FACTION_ID` (2802): `LegacyMicroButton` and the Camelot achievement toggle open Legacy Challenges only when its renown is positive.

```lua
A_Admin.SetMajorFactionRenown(Constants.LegacyConsts.LEGACY_REWARD_TRACK_FACTION_ID, 3, 10)
```
