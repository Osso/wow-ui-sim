# C_MythicPlus.RequestMapInfo response event

This contract describes the response event for `C_MythicPlus.RequestMapInfo`. Its server-response timing is **INFERRED R88ev (native evidence pending)**; simulator tests define the current behavior.

## What it must do

- [ ] **INFERRED R88ev (native evidence pending):** queue `CHALLENGE_MODE_MAPS_UPDATE` for the next timer-processing tick, after the calling handler returns.
- [ ] Coalesce repeated requests made before that response into one event.
- [ ] Let event consumers read the current supplied map data through `C_ChallengeMode.GetMapTable` and `GetMapUIInfo`.
- [x] Keep `RequestCurrentAffixes` and `RequestRewards` event publication unchanged.

## How it works

- [ ] [Mythic+ map request investigation](../wiki/investigations/mythic-plus-map-request-reentrancy.md)

## Implementation inventory

- `src/c_api/c_mythic_plus.rs` — request coalescing and response-event dispatch.
- `src/lua_api/state_types/mythic_plus_scenario.rs` — pending response marker.
- `src/lua_api/timer_layout.rs` — existing deferred Rust-event timer.

## Tests asserting this spec

- `tests/c_mythic_plus_probes.rs::mythic_plus_requests_publish_update_events`
- `tests/c_mythic_plus_probes.rs::request_map_info_coalesces_and_delivers_supplied_data_after_handler`

## Known gaps (current cycle)

- [ ] R88ev native evidence for response timing and request coalescing is pending.

## Out of scope

- Native server map-data acquisition. Tests use explicitly supplied map records.
- Asynchronous behavior for `RequestCurrentAffixes` or `RequestRewards`.
