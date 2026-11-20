# Quest and guild-bank item tooltips

Retail and WoW Forever `C_TooltipInfo` item getters must publish tooltips from supplied quest and guild-bank state. The interface is declared in each profile's `TooltipInfoDocumentation.lua`; API internals are described in the [Lua API overview](../lua-api.md), and quest reward inputs are specified in [quest log item rewards](quest-log-reward-items.md).

## What it must do

- [ ] `GetQuestItem(type, itemIndex, allowCollectionText?)` selects the indexed reward or choice item from the active NPC quest and publishes its shared item tooltip. This mapping is inferred from the FrameXML consumer.
- [ ] `GetQuestLogItem(type, itemIndex, questID?, allowCollectionText?)` selects the indexed reward or choice item from the requested quest, or the selected quest when `questID` is nil, and publishes its shared item tooltip. This mapping is inferred from the FrameXML consumer.
- [ ] `GetGuildBankItem(tab, slot)` publishes the shared item tooltip for a supplied item at that guild-bank tab and slot. This mapping is inferred from the FrameXML consumer.
- [ ] Missing source rows do not produce item data. No item ID or item text may be invented.
- [ ] Other client profiles keep their current getter behavior.

## How it works

- [Tooltip data and item context](tooltip-item-context.md)
- [Quest log item reward inputs](quest-log-reward-items.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_item_sources.rs` — reads supplied quest and guild-bank item rows and publishes item tooltips.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs` — routes Retail and Forever registrations to the state-backed getters.

## Tests asserting this spec

- `tests/tooltip_quest_items.rs` — drives the original QuestInfo reward-item and GuildBank item `OnEnter` consumers with distinct supplied items.

## Known gaps (current cycle)

- [ ] `type = "required"` has no supplied item-ID source in `QuestLogEntry` or quest objective state; both getters return no item data for it.
- [ ] `allowCollectionText` is accepted but collection text has no supplied input model.

## Out of scope

- Quest item requirements and collection text are not derived from quest objectives or item catalog data.
