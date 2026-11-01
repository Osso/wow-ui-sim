# Mainline season, housing-market, account-quest and profession queries

Registered on Retail, PTR and Forever, whose Blizzard UI calls them.
`C_Housing.IsHousingMarketEnabled` is registered wherever `C_Housing` market
methods are registered.

## Behavior

- `C_SeasonInfo.GetCurrentDisplaySeasonID()` returns
  `mythic_plus.display_season_id`; default `0` (no active display season).
- `C_Housing.IsHousingMarketEnabled()` returns `housing.market_enabled`;
  default `false`, like `IsHousingMarketShopEnabled`.
- `C_QuestLog.IsQuestFlaggedCompletedOnAccount(questID)` returns whether
  `questID` is in the completed-quest set.
- `C_CraftingOrders.HasFavoriteCustomerOptions()` returns `false`.
- `C_TradeSkillUI.IsAnyRecipeFromSource(sourceType)` returns `false`.

## Best-guess basis

best-guess, native UNKNOWN.

- The display season is server-supplied; no offline source gives the live ID,
  so it stays unset until a host configures it.
- The simulator models one character per account, so account completion equals
  character completion.
- No favourite customer-order searches are stored.
- The recipe catalogue records no acquisition source, so no recipe matches a
  source filter and the Professions source submenu stays empty.

## Known gaps

- Account-wide quest completion across alts.
- Recipe acquisition sources and source-type filters.
- Favourite customer-order searches.

Test: `tests/mainline_season_profession_queries.rs`.
