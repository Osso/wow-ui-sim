# WoW Token Auction House Visibility

`C_WowTokenUI.ShouldDisplayInAuctionHouse` controls whether the Auction House category list shows the WoW Token category. The cached consumer is `Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseCategoriesList.lua`.

## What it must do

- [ ] Forever registers the no-argument query in Game and Glue. It returns one boolean from the server-supplied token feature status; before a status update, false is INFERRED from zero-initialized native storage.
- [ ] Forever status updates set the feature state and fire `TOKEN_STATUS_CHANGED`; the getter does not fire an event.
- [ ] The original category-list consumer hides flagged token categories when the getter returns false and includes them when it returns true.
- [ ] PTR returns true from its native override, independent of the supplied status.
- [ ] Retail does not register `ShouldDisplayInAuctionHouse`.

R27 native probe: registration in Forever Game and Glue, PTR's always-true override, and Retail's absent member are VERIFIED for builds Forever 1.60.1.70707, PTR 12.1.7.70668, and Retail 12.1.5.70688. The Forever pre-status false value is INFERRED; startup initialization was not proven. The category-list hide rule is VERIFIED in shipped Forever Lua; its cache build is not verified.

## How it works

- [Event system](../event-system.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/c_wow_token_ui.rs`: registers the query and applies host-supplied status updates.
- `src/lua_api/state_types/wow_token.rs`: stores the supplied visibility status.
- `src/c_api/mod.rs`: registers the profile-supported namespace member.

## Tests asserting this spec

- `tests/blizzard_auction_house_categories_list.rs`: tests Forever status changes through the original category-list consumer, PTR's override, event delivery, and Retail member absence.

## Known gaps (current cycle)

- [ ] Native startup initialization of the Forever status byte is UNKNOWN; the simulator uses the evidenced zero-initialized default.
- [ ] Targeted Forever and PTR tests remain pending slot 3 admission.

## Out of scope

- Live token-status transport and server-message decoding. The simulator accepts status through a host input.
