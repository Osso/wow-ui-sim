# Retail garrison landing-page type

`C_Garrison.GetLandingPageGarrisonType` supplies the garrison type read by the original Blizzard Minimap refresh path. **VERIFIED (local Retail UI cache; exact client build UNKNOWN):** its generated API documentation has no declaration for this member. **VERIFIED (Retail 12.1.5 build 70688):** native code registers the member under `C_Garrison`. The simulator exposes a neutral host input; it does not select a garrison from character or expansion state. See the [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#garrison-landing-page-type-r88a-2026-11-13).

## What it must do

- [x] Publish `C_Garrison.GetLandingPageGarrisonType` for Retail. **VERIFIED (Retail 12.1.5 build 70688):** the native member takes no arguments and returns exactly one number on normal completion.
- [ ] The original `Blizzard_Minimap` refresh path must complete without a Lua error once its required `C_Garrison` members are modeled. The current test stops after the native type query, before the separately blocked visibility predicate.
- [ ] **VERIFIED (Retail 12.1.5 build 70688):** the empty or unmatched native helper path returns one number, `0`. Do not infer that every no-page state uses this path.
- [ ] **INFERRED (simulator policy):** the neutral host input starts at `0`. This does not prove that `0` is the native no-active-page sentinel in every state.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#garrison-landing-page-type-r88a-2026-11-13)

## Implementation inventory

- `src/c_api/c_garrison_landing_page.rs`: Retail query and neutral host-input read.
- `src/c_api/mod.rs`: Retail-gated namespace module registration.
- `src/lua_api/state/sim_state.rs`: per-environment host input and inferred zero default.
- `tests/retail_garrison_minimap_consumer.rs`: original Blizzard Minimap consumer test.

## Tests asserting this spec

- `tests/retail_garrison_minimap_consumer.rs::retail_minimap_refresh_stops_at_unmodeled_visibility_query`

## Known gaps (current cycle)

- [ ] **UNKNOWN (Retail 12.1.5 build 70688):** The state that selects each nonzero landing-page type and the functions that write that state are not established. Keep selection unmodeled.
- [ ] **VERIFIED (20-addon Retail run at production-code SHA `bc01ba60cdb7b844bf88adba7ff2cb6e809b665f`):** after `GetLandingPageGarrisonType` returns, the original Minimap call fails on the missing `C_Garrison.IsLandingPageMinimapButtonVisible` member. **VERIFIED (local Retail UI cache; exact build UNKNOWN):** its generated declaration takes a number and returns a boolean. **UNKNOWN:** its native predicate and result. This is a separate blocker; do not register a production stub.
- [ ] The consumer test verifies the numeric type query and stops at the visibility member. It does not claim that the full `RefreshButton` path passes or assert the inferred zero host default.

## Out of scope

Non-Retail publication, nonzero selection rules, garrison lifecycle state, the separate `IsLandingPageMinimapButtonVisible` predicate, and any mapping from the native garrison database table to active selection.
