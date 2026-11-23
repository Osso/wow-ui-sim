# Black Market state

`C_BlackMarket` exposes supplied auction rows to the Retail and WoW Forever Blizzard Black Market UI. Its model is in [`src/c_api/c_black_market.rs`](../../src/c_api/c_black_market.rs); see the [Lua API architecture](../lua-api.md) for the API boundary.

## What it must do

- [x] Show two distinct supplied listings and the supplied hot deal in the original Blizzard panel.
- [x] Allow `hotItem.marketID` to match a listing while keeping listing IDs unique; bids update both copies.
- [x] Reject bids below `max(minBid, currBid + minIncrement)` without changing the listing state.
- [x] Dispatch `BLACK_MARKET_BID_RESULT(marketID, resultCode)` for each bid. **INFERRED (R88em; native evidence pending):** `0` means success, `1` means below minimum, and `2` means the market ID is unknown.
- [x] Respect the supplied view-only flag by hiding the Blizzard bid controls.
- [x] Treat index zero as invalid; it must not return the first listing.

## How it works

- The Rust API registration and supplied-state model live in [`src/c_api/c_black_market.rs`](../../src/c_api/c_black_market.rs).
- The Lua frame and row consumers live in Blizzard's `Blizzard_BlackMarketUI` addon.

## Implementation inventory

- `src/c_api/c_black_market.rs` — Retail/Forever listing model, `C_BlackMarket` methods, and `A_Admin.SetBlackMarketState`.
- `src/c_api/mod.rs` — profile-gated module declaration.
- `src/lua_api/globals/missing_surface.rs` — profile-gated namespace registration.
- `src/lua_api/globals/admin.rs` — profile-gated setter registration.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — per-environment state and default.

## Tests asserting this spec

- `tests/black_market_model.rs` — loads the Blizzard UI, tests a shared hot-item/listing ID, places rejected and accepted bids through the row and confirmation controls, and checks refreshed listing and hot-deal state, view-only controls, and index-zero behavior on Retail and Forever.

## Known gaps (current cycle)

- [ ] **INFERRED (R88em; native evidence pending):** confirm native bid-result, close, and getter return semantics.

## Out of scope

- Live auction-server requests, bid settlement, and persistent auction data are not modeled; the host supplies scenario state through `A_Admin`.
