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

## Major faction display observations (Retail and Forever)

- `A_Admin.SetMajorFactionHidden(factionID, hidden)` supplies a boolean for an existing faction. The empty hidden set makes `C_MajorFactions.IsMajorFactionHiddenFromExpansionPage` false; setting false removes the row from the hidden set.
- `A_Admin.SetRenownNPCFactionID(factionID)` supplies the currently relevant faction or `0` for no faction; other IDs must match an existing faction. The initial value is `0`.
- `A_Admin.SetMajorFactionJourneyFlags(factionID, displayAsJourney, useJourneyRewardTrack)` supplies both existing journey flags for the selected faction on both profiles.

These are host observations; invalid inputs are rejected before mutation. No event is emitted. VERIFIED declaration shapes: Retail `MajorFactionsDocumentation.lua:86–92,126–139,156–184`, Forever `:101–107,157–170,187–215`. Both `Blizzard_MajorFactions.toc:6` select `Blizzard_MajorFactionsLandingTemplates.xml` and its Lua consumer at `:79–103`, which filters the displayed rows. Native policy derivation and update timing are unknown.

## SetMajorFactionRenown

`A_Admin.SetMajorFactionRenown(factionID, renownLevel, maxLevel)` supplies Renown progress for one major faction on every profile that registers `C_MajorFactions`. It updates an existing row or adds an unlocked row with no imported name or metadata. `GetRenownLevels` then lists levels `1..maxLevel`; levels above `renownLevel` are locked and the last level is the capstone (INFERRED). Validation: `factionID` positive, `maxLevel` a positive integer, `renownLevel` within `0..maxLevel`; invalid input raises `A_Admin.SetMajorFactionRenown: ...` before any change.

A level change fires the documented synchronous `MAJOR_FACTION_RENOWN_LEVEL_CHANGED(majorFactionID, newRenownLevel, oldRenownLevel)` (Forever `MajorFactionsDocumentation.lua:233-242`); a new row counts as old level 0 (INFERRED). Forever's Legacy system reads faction `Constants.LegacyConsts.LEGACY_REWARD_TRACK_FACTION_ID` (2802): `LegacyMicroButton` and the Camelot achievement toggle open Legacy Challenges only when its renown is positive.

```lua
A_Admin.SetMajorFactionRenown(Constants.LegacyConsts.LEGACY_REWARD_TRACK_FACTION_ID, 3, 10)
```
