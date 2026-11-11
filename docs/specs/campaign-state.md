# Campaign metadata and state

`C_CampaignInfo` reads campaign, chapter, quest-association and progression data. The simulator currently supplies quest-to-campaign and chapter-ID associations, but it does not model Campaign metadata rows or campaign condition state. See [Campaign data model](../wiki/systems/campaign-data.md) for implementation details.

## What it must do

- [ ] `GetCampaignID(questID)` returns the explicitly supplied campaign association, or 0 when none exists.
- [x] `GetChapterIDs(campaignID)` returns explicitly supplied chapter IDs in stored order; an absent association returns nil. This input mapping is a simulator scenario, not proof of native row contents.
- [x] Without a Campaign metadata row, `GetCampaignInfo(campaignID)` returns nil. Quest associations alone must not create an empty metadata table.
- [x] Without Campaign rows and condition inputs, `GetState(campaignID)` returns `Enum.CampaignState.Invalid`. Quest completion alone must not imply Complete or InProgress.

## How it works

- [C API boundary](../wiki/index.md)
- [Campaign data model](../wiki/systems/campaign-data.md)

## Implementation inventory

- `src/c_api/c_campaign_info.rs` — projects supplied quest and chapter associations; returns the verified missing-row result for metadata and state queries.
- `src/lua_api/state_types/quest_log.rs` — stores simulator-supplied campaign associations and chapter IDs.

## Tests asserting this spec

- `tests/campaign_info.rs::campaign_metadata_and_state_are_not_inferred_from_quest_progress`
- `tests/campaign_info.rs::campaign_chapters_project_supplied_ids`
- `tests/campaign_state_consumer.rs::campaign_header_does_not_infer_progress_from_quest_associations`

## Known gaps (current cycle)

- [ ] No Campaign metadata catalog exists. Populated metadata and all eight fields remain unmodeled.
- [ ] No current-chapter selector, completion-condition evaluator, or stalled-condition evaluator exists. State for present Campaign rows is unmodeled.
- [ ] The chapter-ID input does not encode whether its Campaign row exists separately from its chapter relation list.


## Out of scope

- Native condition writers, chapter completion rules, events, protected-return edge cases, and live-client catalog parity.
