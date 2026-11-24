# Contribution Collector

`A_Admin.SetContributionCollectors(collectors)` supplies contribution-collector data for Retail and WoW Forever. It replaces the full collector snapshot. It does not infer server state or grant rewards.

Each collector row uses `mapID`, `areaPoiID`, `collectorCreatureID`, `name`, `atlasName`, `x`, `y`, and optional `contributions`. Each contribution row uses `contributionID`, `name`, `state`, `amount`, `startTime`, `description`, `result`, `orderIndex`, `active`, and `rewardQuestID`. `nextStateTime` and `appearance` are optional. `requiredCurrency` is optional and has `id` and `amount`; `requiredItem` is optional and has `id` and `count`. `atlases` is an optional ordered list of atlas names, and `buffs` is an optional ordered list of spell IDs.

`active=true` includes the contribution in `C_ContributionCollector.GetActive()`. The remaining supplied fields feed the matching `C_ContributionCollector` getters. The original `Blizzard_Contribution.lua` consumer uses the result and cost to update its Contribute button, and uses description, order, reward quest, atlases, and buffs while building contribution entries.

The Retail and Forever pinned `ContributionCollectorDocumentation.lua` files declare `GetActive` (lines 25–31), `GetAtlases` (34–45), `GetBuffs` (49–60), `GetContributionResult` (95–105), `GetDescription` (110–120), `GetOrderIndex` (155–165), `GetRequiredContributionCurrency` (170–182), `GetRequiredContributionItem` (187–199), and `GetRewardQuestID` (204–214). The supplied-state behavior is INFERRED (R88fz; native evidence pending).

```lua
A_Admin.SetContributionCollectors({
    {
        mapID = 2248, areaPoiID = 901, collectorCreatureID = 301,
        name = "Collector", atlasName = "", x = 0.2, y = 0.3,
        contributions = {
            {
                contributionID = 501, name = "Ready", state = 1, amount = 0.5,
                startTime = 100, description = "Ready description",
                result = Enum.ContributionResult.Success, orderIndex = 1,
                active = true, rewardQuestID = 0,
                requiredCurrency = { id = 2032, amount = 5 },
            },
        },
    },
})
```

The C API declarations are in `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ContributionCollectorDocumentation.lua` and the matching `wowforever` profile file. The consumer is `Blizzard_Contribution/Blizzard_Contribution.lua`.
