# Active timerunning season

Retail exposes the active event season separately from the player's timerunning season. The pinned retail `Blizzard_APIDocumentationGenerated/TimerunningUIDocumentation.lua:11-20` declares `C_TimerunningUI.GetActiveTimerunningSeasonID`.

## What it must do

- [ ] Return the supplied signed active-season ID as one Lua number, including `0` for the no-season scenario.
- [ ] Keep the active season independent of the player's timerunning season.
- [ ] Let the pinned `Blizzard_TimerunningUtil` select its no-season and Pandaria configurations from the supplied value.

## Best-guess basis

Native evidence for retail 12.1.5 build 70688 identifies a dedicated cached signed integer and numeric output. The connection between the cached read and output is emulated, not verified. Initial `0` is the simulator's no-season scenario, not an observed live default.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_timerunning_ui.rs`: retail namespace and query.
- `src/lua_api/state/sim_state.rs`: supplied active-season state.
- `src/lua_api/state.rs`: initial no-season scenario.

## Tests asserting this spec

- `tests/blizzard_timerunning_util_loads.rs`: pinned consumer with no season and Pandaria season.

## Known gaps (current cycle)

- [ ] Native cache refresh, events, and runtime transitions are unknown.
- [ ] Other client profiles are not published without profile-specific declaration evidence.

## Out of scope

- Simulating server-side season scheduling.
