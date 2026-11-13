# Retail garrison landing-page queries

`C_Garrison.GetLandingPageGarrisonType` and `C_Garrison.IsLandingPageMinimapButtonVisible` provide the Retail Minimap landing-page queries. Their simulator state lives in `src/c_api/c_garrison_landing_page.rs`. See the [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#garrison-minimap-button-visibility-r88e1-2026-11-13) for native evidence and implementation limits.

## What it must do

- [x] Publish `C_Garrison.GetLandingPageGarrisonType` for Retail. **VERIFIED (local Retail UI cache; exact build UNKNOWN):** its generated API documentation does not declare this member. **VERIFIED (Retail 12.1.5 build 70688):** it takes no arguments and returns exactly one number.
- [x] **VERIFIED (Retail 12.1.5 build 70688):** the native empty or unmatched helper path returns one number, `0`. Do not infer that every no-page state uses this path.
- [x] Publish `C_Garrison.IsLandingPageMinimapButtonVisible(garrisonType)` for Retail. **VERIFIED (Retail 12.1.5 build 70688):** it takes one number and returns one boolean.
- [x] Keep landing-page button records as host input keyed by the garrison type's low byte. The default record set is empty; a missing record returns false.
- [x] For a matched type other than 111, return the record's state flag. Do not reject values by enum membership; values 256 apart select the same record.
- [x] For a matched type 111, return true only when PlayerCondition 105386 is true and the player-state value is nonzero. This branch does not use the record state flag.
- [x] Complete the original Retail Minimap refresh with default host state and keep the button hidden. With a matched non-111 record whose state flag is true, the original refresh must show the button.
- [x] **INFERRED (simulator policy):** because no PlayerCondition evaluator is modeled, expose PlayerCondition 105386 as host input and default it to false. This default does not claim that the native condition is false.
- [ ] **INFERRED (native interpretation):** the nonzero player-state value is the covenant ID; the identity of this field is not verified.
- [x] **INFERRED (simulator policy):** the neutral `GetLandingPageGarrisonType` input starts at zero. The consumer test verifies this simulator default; it does not establish zero as the native no-page value in every state.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#garrison-minimap-button-visibility-r88e1-2026-11-13)

## Implementation inventory

- `src/c_api/c_garrison_landing_page.rs`: Retail landing-page queries and host inputs.
- `src/c_api/mod.rs`: Retail-gated module registration.
- `src/lua_api/state/sim_state.rs`: per-environment landing-page state.
- `tests/retail_garrison_minimap_consumer.rs`: original Blizzard Minimap consumer test.

## Tests asserting this spec

- `tests/retail_garrison_minimap_consumer.rs::retail_minimap_refresh_uses_landing_page_visibility_records`

## Known gaps (current cycle)

- [ ] **UNKNOWN (Retail 12.1.5 build 70688):** the state that selects nonzero landing-page types and the functions that write it are not established. Keep selection unmodeled.
- [ ] The player-state input has no modeled producer. Its covenant-ID interpretation remains inferred.
- [ ] Native coercion for non-finite, fractional, or non-number `garrisonType` arguments is not established.

## Out of scope

Non-Retail publication, nonzero landing-page selection rules, garrison lifecycle state, and mapping from the native garrison database table to active selection.
