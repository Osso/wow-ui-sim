# Quest and guild-bank item tooltips

Retail and WoW Forever `C_TooltipInfo` item getters must publish tooltips from supplied quest and guild-bank state. The interface is declared in each profile's `TooltipInfoDocumentation.lua`; API internals are described in the [Lua API overview](../lua-api.md), and quest reward inputs are specified in [quest log item rewards](quest-log-reward-items.md).

## What it must do

- [x] `GetQuestItem(type, itemIndex, allowCollectionText?)` selects the indexed reward or choice item from the active NPC quest and publishes its shared item tooltip. This mapping is inferred from the FrameXML consumer.
- [x] `GetQuestLogItem(type, itemIndex, questID?, allowCollectionText?)` selects the indexed reward or choice item from the requested quest, or the selected quest when `questID` is nil, and publishes its shared item tooltip. This mapping is inferred from the FrameXML consumer.
- [x] `GetGuildBankItem(tab, slot)` publishes the shared item tooltip for a supplied item at that guild-bank tab and slot. This mapping is inferred from the FrameXML consumer.
- [x] Missing source rows do not produce item data. No item ID or item text may be invented.
  Behavioral proof: `tests/tooltip_quest_item_missing_sources.rs::quest_log_item_uses_supplied_rows_and_returns_no_data_for_misses` supplies two distinct reward items through `A_Admin.SetQuestLogRewardItems`, verifies both public tooltip outputs, then checks an out-of-range index and a quest with no rows. It also checks invalid-input atomicity.
- [ ] Other client profiles keep their current getter behavior.
  Classification: (c) preservation only, outside the Retail and Forever runtime scope. No other-profile behavior is claimed.

## How it works

- [Tooltip data and item context](tooltip-item-context.md)
- [Quest log item reward inputs](quest-log-reward-items.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_item_sources.rs` — reads supplied quest and guild-bank item rows and publishes item tooltips.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs` — routes Retail and Forever registrations to the state-backed getters.
- `src/lua_api/globals/admin_quests.rs` — provides the Retail/Forever test input for existing quest-log reward rows.
- `src/lua_api/globals/bank_storage_verbs.rs` and `src/lua_api/globals/register.rs` — expose the existing selected-tab state reader to the Retail and Forever guild-bank consumer.

## Tests asserting this spec

- `tests/tooltip_quest_items.rs` — drives the original QuestInfo reward-item and GuildBank item `OnEnter` consumers with distinct supplied items.
- `tests/tooltip_quest_item_missing_sources.rs` — drives the public quest-log tooltip API with distinct supplied items, an out-of-range index and an empty quest.

## Known gaps (current cycle)

- [ ] `type = "required"` has no supplied item-ID source in `QuestLogEntry` or quest objective state; both getters return no item data for it.
  Classification: (c) not modelable from current state. `QuestObjective` carries text, type, progress and criteria ID, but no item ID; do not derive one from text or invent a mapping.
- [ ] `allowCollectionText` is accepted but collection text has no supplied input model.
  Classification: (c) not modelable from current state or pinned data in this lane. No collection-text input or native text source is present.

## Out of scope

- Quest item requirements and collection text are not derived from quest objectives or item catalog data.
