# Mythic+ request response events

This contract covers response events for `C_MythicPlus.RequestMapInfo`, `RequestCurrentAffixes`, and `RequestRewards`. Their native request and response mapping is **INFERRED R88ev (native evidence pending)**; tests define the simulator behavior.

## What it must do

- [ ] **INFERRED R88ev:** `RequestMapInfo` requires an existing player, a simulator-clock interval strictly greater than 60 seconds since the last admitted request, and a connected player. It saves the request timestamp before checking connection.
- [ ] When connected, `RequestMapInfo` queues `CHALLENGE_MODE_MAPS_UPDATE` for the next timer-processing tick; it does not dispatch synchronously or read a local cache as a response.
- [ ] A request with no prior timestamp is eligible in the simulator. Later calls at or below 60 seconds are suppressed; calls after more than 60 seconds can produce another response.
- [ ] **INFERRED R88ev:** `RequestCurrentAffixes` has no cooldown, returns zero Lua values, and queues `MYTHIC_PLUS_CURRENT_AFFIX_UPDATE` for the next timer-processing tick when connected.
- [ ] **INFERRED R88ev:** `RequestRewards` returns zero Lua values. The recovered handler has no send; the simulator retains `WEEKLY_REWARDS_UPDATE` as a deferred compatibility response for existing consumers.
- [ ] Response consumers can read explicitly supplied map data through `C_ChallengeMode.GetMapTable` and `GetMapUIInfo`.

## How it works

- [Mythic+ request reentrancy investigation](../wiki/investigations/mythic-plus-map-request-reentrancy.md)

## Implementation inventory

- `src/c_api/c_mythic_plus.rs` — request admission and deferred response events.
- `src/lua_api/state_types/mythic_plus_scenario.rs` — last admitted map-info request time.
- `src/lua_api/globals/group_queries_relationships.rs` — shared unit connection predicate.
- `src/lua_api/timer_layout.rs` — deferred Rust-event timer.

## Tests asserting this spec

- `tests/c_mythic_plus_probes.rs::mythic_plus_request_events_are_deferred`
- `tests/c_mythic_plus_probes.rs::request_map_info_coalesces_and_delivers_supplied_data_after_handler`

## Known gaps (current cycle)

- [ ] R88ev does not establish the initial native request timestamp, Forever's clock unit, or the server response packet handler. The simulator allows the first request when no timestamp exists and uses seconds on both profiles.
- [ ] The test host always reports `UnitExists("player")` and `UnitIsConnected("player")` as true; absent-player and disconnected-player paths lack a normal test input.
- [ ] The simulator does not send or receive native server packets. `RequestRewards` response-event delivery remains a compatibility inference.

## Out of scope

- Native server map-data acquisition. Tests use explicitly supplied map records.
- Introducing a network transport or server packet model.
