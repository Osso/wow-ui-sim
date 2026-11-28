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
