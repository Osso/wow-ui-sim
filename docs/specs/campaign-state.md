# Supplied campaign state

## Best-guess basis — best-guess, native UNKNOWN

Retail and PTR expose campaign IDs associated with supplied quest state. Unknown campaign IDs return nil info and Invalid state. A campaign whose known quests are all complete is Complete; one with any active or completed quest, but not all complete, is InProgress; otherwise it is Invalid. GetCampaignID maps a supplied quest ID to its campaign ID, or zero when no association is supplied. The active quest-log projection carries that ID.

GetChapterIDs projects explicitly supplied ordered chapter IDs; absent chapter data returns nil. This is a best guess, not measured native behavior. No chapters are fabricated.

The pinned Retail `WarCampaignDocumentation.lua:35-122` declares the ID, nullable info, nullable chapter list, and non-nil state signatures; its `CampaignState` enum appears at lines 163-174. [Warcraft Wiki: GetCampaignID](https://warcraft.wiki.gg/wiki/API:C_CampaignInfo.GetCampaignID) confirms the quest ID input. [Warcraft Wiki: GetCampaignInfo](https://warcraft.wiki.gg/wiki/API:C_CampaignInfo.GetCampaignInfo) lists the nullable info and its fields. Neither source establishes the above progression rules.

Retail `Blizzard_ObjectAPI_Mainline.toc:15` selects Campaign.lua. `Blizzard_UIPanels_Game_Mainline.toc:67-76` selects WarCampaignTemplates.lua, CampaignOverview.lua, and QuestMapFrame.lua; QuestMapFrame.lua:1881-1885 selects a campaign header and WarCampaignTemplates.lua:166-187 reads its state. CampaignOverview.lua:45-47 uses that header.

## Known gaps

- Native unknown-ID, state transitions, and Stalled rules are unknown; Stalled is not modeled.
- No campaign metadata source exists. A known campaign has no fabricated name, description, texture kit, or flags; this is an incomplete CampaignInfo shape.
- Quest-to-campaign and chapter associations require supplied state; current seeded quests do not supply them.
- Chapter completion, chapter metadata, ordering rules, and live-client parity are not modeled.
