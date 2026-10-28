# Classic reagent quality

`C_TradeSkillUI.GetItemReagentQualityInfo` exposes host-supplied reagent quality records to the selected Blizzard consumers.

## What it must do

- [x] WoW Forever accepts item IDs or item links and returns supplied records to the Professions consumer. Its declared `CraftingQualityInfo` includes `iconQuestObjective`.
- [x] Mists accepts a numeric item ID and returns supplied records to the Auction House consumer. Its declared `CraftingQualityInfo` omits `iconQuestObjective`; its other fields match the supplied model.
- [ ] Unsupplied records return nil. Mists rejects nonnumeric arguments.
- [ ] Retail and PTR retain their existing publication and return shape. The Retail item-button consumer is covered; PTR preservation is not yet tested.

The pinned declarations are `Blizzard_APIDocumentationGenerated/TradeSkillUIDocumentation.lua:11` and `TradeSkillUITypesDocumentation.lua:40` for Mists, and `TradeSkillUIDocumentation.lua:379` and `TradeSkillUITypesDocumentation.lua:194` for Forever. Selected consumers are Mists `Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseTableBuilder.lua:705` and Forever `Blizzard_ProfessionsTemplates/Blizzard_Professions.lua:54`.

## Implementation inventory

- `src/c_api/c_trade_skill_quality.rs` — supplied records, profile publication and projection.
- `src/lua_api/state/sim_state.rs` — per-environment supplied state.

## Tests asserting this spec

- `tests/forever_reagent_quality_consumer.rs`
- `tests/mists_reagent_quality_consumer.rs`

## Out of scope

Era, Anniversary and Wrath have no selected consumer that reaches this API. Native quality values are not inferred.
