# Quest log item rewards (Retail)

## What it must do

- Retail `GetNumQuestLogRewards(questID)` returns the number of item rewards supplied for that quest's log entry, and zero for an unknown or omitted quest.
- Retail `GetQuestLogRewardInfo(index, questID)` returns `name, texture, count, quality, isUsable, itemID` for a supplied reward. `itemID` is nil when the scenario supplies no item ID. An out-of-range index or unknown quest returns zero values.
- Other profiles keep their existing paths: Wrath uses its selected-quest legacy reads, and the remaining profiles that share this registration keep reporting zero rewards and five values.

## Best-guess basis

These legacy globals have no pinned API documentation. Retail consumers read `itemID` as the sixth value: `Blizzard_UIPanels_Game/Mainline/WorldMapFrame.lua:52`, `Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua:1011` and `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:2562`. `GetNumQuestLogRewards(questID)` is read with a quest argument at `WorldMapFrame.lua:50` and `QuestUtils.lua:1009`.

## Known gaps

- No pinned consumer reads a seventh value; an item level is not published.
- Reward items are supplied scenario state, not server quest data.
