# C_Map user waypoint

This contract defines the session-scoped user waypoint consumed by Blizzard's Retail `WaypointLocationDataProvider`. The state and API implementation live in `src/c_api/c_map.rs` and `src/c_api/c_map/user_waypoint.rs`; see the [system page](../wiki/systems/c-map-user-waypoint.md) for the current model and evidence limits.

## What it must do

### State and validation

- [ ] **INFERRED:** Keep one shared waypoint in `SimState` for the simulator session. Account-versus-character ownership and persistence are UNKNOWN.
- [x] **VERIFIED (Retail 12.1.5.70688):** Parse `uiMapID` as signed 32-bit with truncation toward zero; require a nonzero result.
- [x] **VERIFIED (Retail 12.1.5.70688):** Require finite normalized `x` and `y` values in the inclusive range 0 to 1; numeric-convertible coordinate strings are accepted.
- [x] **VERIFIED (Retail 12.1.5.70688):** A missing or rejected `z` uses the no-z path. Serialization includes `z` only when present.
- [x] **VERIFIED (Retail 12.1.5.70688):** A valid point replaces the shared point. A rejected point leaves the stored point unchanged.

### API surface

- [ ] `SetUserWaypoint` performs its `AllowedWhenUntainted` secret check before point conversion. **VERIFIED:** the Retail, PTR and Forever API documentation declares this mode; R88h.1 supplies the tainted-denial template. The other secret-mode templates are unreachable for this API.
- [x] `GetUserWaypoint` returns a valid stored point with `uiMapID` and `position`, and includes `z` only when present.
- [x] **VERIFIED (Retail 12.1.5.70688):** `GetUserWaypointPositionForMap` returns the stored position for the matching map.
- [ ] **INFERRED:** cross-map projection is not modeled; other maps return no position.
- [x] `HasUserWaypoint` reports whether the stored point is valid. `ClearUserWaypoint` removes the stored point.
- [ ] **INFERRED:** `CanSetUserWaypointOnMap` returns true for a nonzero signed map ID and false for zero. Native map categories and allowed-ID rules are UNKNOWN.
- [ ] **INFERRED:** `SetUserWaypoint` returns one Boolean named `wasSet`; exact native return publication is UNKNOWN in R88h.1.
- [ ] **UNKNOWN:** The exact `z` conversion predicate and waypoint-change callback identity/event are not established. The simulator accepts finite numeric z values and dispatches no event.

### Profile boundary

- [ ] Publish the six methods only for Retail, PTR and WoW Forever, where their pinned `MapDocumentation.lua` files declare them. PTR behavior is **INFERRED** from the Retail path; do not make PTR-specific behavioral assertions.
- [ ] Do not publish these methods on Mists; the pinned Mists `MapDocumentation.lua` does not contain these names. Wrath is compile-only for this change.

## How it works

- [C_Map user waypoint system](../wiki/systems/c-map-user-waypoint.md)
- [Client profile selection](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_map.rs` — C_Map registration and the shared waypoint record.
- `src/c_api/c_map/user_waypoint.rs` — six handlers, validation, serialization and profile gate.
- `src/lua_api/state/sim_state.rs` — session waypoint field.
- `src/lua_api/state.rs` — empty-state initialization.

## Tests asserting this spec

- `tests/test_c_map_user_waypoint.rs` — original provider refresh/click/clear flow, state replacement, rejected coordinates and point shape.
- `tests/prefork_full_ui.rs` — optional Retail full-UI case registration.

## Known gaps (current cycle)

- [ ] Native account/character ownership, persistence and waypoint-change callback/event remain UNKNOWN.
- [ ] Cross-map projection and native map-category eligibility remain UNKNOWN; the simulator uses the labeled nonzero-ID policy.
- [ ] PTR and Forever behavior is not separately asserted. Mists and Wrath receive compile-only receipts.
- [ ] **VERIFIED (Retail prefork test):** the original `HandleClick` stores the point, then errors at `WaypointLocationDataProvider.lua:101` because `C_SuperTrack.SetSuperTrackedUserWaypoint` is nil; that separate API is out of scope.

## Out of scope

- Cross-map coordinate projection, persisted waypoint state, `USER_WAYPOINT_UPDATED`, `C_SuperTrack.SetSuperTrackedUserWaypoint`, and other waypoint APIs such as hyperlink parsing.
- Mists publication, because the pinned Mists API documentation does not declare these six methods.
