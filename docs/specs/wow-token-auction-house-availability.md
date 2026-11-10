# PTR WoW Token auction-house availability

VERIFIED: PTR 12.1.7 declares `C_WowTokenUI.ShouldDisplayInAuctionHouse()` with one non-nil boolean result in `Blizzard_APIDocumentationGenerated/WowTokenUIDocumentation.lua:11–17`; `Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseCategoriesList.lua:19` calls it. INFERRED from R22: the PTR 12.1.7.70668 query returns true before and after server feature-status updates, so this simulator contract models it as true.

## What it must do

- [ ] On PTR, the original auction-house category-list consumer can call `C_WowTokenUI.ShouldDisplayInAuctionHouse()` without an argument.
- [ ] The query returns a non-nil true boolean, so the WoW Token category remains available to the consumer.

## How it works

- [`C_WowTokenUI` system model](../wiki/systems/c-wow-token-ui.md).

## Implementation inventory

- `src/c_api/c_wow_token_ui.rs` — registers the PTR query and returns the modeled display result.
- `src/c_api/mod.rs` — declares and registers the C API module.
- `tests/blizzard_auction_house_categories_list.rs` — exercises the original consumer and query.

## Tests asserting this spec

- `tests/blizzard_auction_house_categories_list.rs::auction_house_categories_list_refresh_count_matches_categories`

## Known gaps (current cycle)

- [ ] The targeted PTR integration test must pass after the producer change.

## Out of scope

- `C_WowTokenUI.StartTokenSell`: R22 confirms its declaration and argument type, but this cycle does not model its behavior.
- Changes to token APIs or auction-house behavior on other client profiles.
