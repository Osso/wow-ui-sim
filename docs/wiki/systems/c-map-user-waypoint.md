# C_Map user waypoint

The Retail `WaypointLocationDataProvider` calls `GetUserWaypointPositionForMap` when it refreshes, `CanSetUserWaypointOnMap` and `SetUserWaypoint` when it handles a placement click, and `ClearUserWaypoint` when the existing pin is clicked. **VERIFIED:** the simulator stores one `UserWaypoint` on `SimState` and registers six `C_Map` methods for Retail, PTR and WoW Forever. **VERIFIED:** the pinned Mists `MapDocumentation.lua` does not contain these six names, so the simulator does not publish them on Mists.

## Content

### State and queries

**VERIFIED from code:** `SimState.user_waypoint` stores one point with a signed map ID, normalized x/y and optional z. It is session-only in the simulator. **INFERRED:** this scope avoids claiming account/character ownership or persistence, which R88h.1 leaves UNKNOWN.

**VERIFIED (Retail 12.1.5.70688):** the setter checks secret access before parsing, truncates an in-range signed map ID toward zero, requires finite normalized coordinates in [0, 1], and leaves old state unchanged on a rejected point. A missing or rejected z follows the no-z path. The returned `UiMapPoint` is built with the loaded `UiMapPoint.CreateFromCoordinates` constructor.

**VERIFIED (Retail 12.1.5.70688):** getters validate the stored map ID and coordinates before publishing. The position query returns the point only for the same map. **INFERRED:** the simulator does not project coordinates to a different map.

**INFERRED:** `CanSetUserWaypointOnMap` accepts every nonzero signed map ID because the native map categories and allowed-ID list are UNKNOWN in R88h.6. The optional change-callback hook dispatches nothing because its identity/event is UNKNOWN; no event is synthesized.

### Original consumer boundary

**VERIFIED:** the Retail manifest selects `Blizzard_SharedMapDataProviders_Mainline.toc`; that TOC permits Game/mainline loading and includes `WaypointLocationDataProvider.xml`, which loads the original Lua provider. The provider calls `GetUserWaypointPositionForMap` at line 57 and `CanSetUserWaypointOnMap`/`SetUserWaypoint` at lines 91/100. The prefork test loads this addon and invokes its original `RefreshAllData` and `HandleClick` methods.

**INFERRED:** PTR shares the Retail implementation path; the tests make no PTR-specific behavioral assertions. **UNKNOWN:** whether the Retail handler behavior is identical on PTR or Forever.

## Sources

- [C_Map user-waypoint contract](../../specs/c-map-user-waypoint.md) — required API behavior and explicit UNKNOWN/INFERRED boundaries.
- [C_Map registration](../../../src/c_api/c_map.rs) and [waypoint handlers](../../../src/c_api/c_map/user_waypoint.rs) — current Rust model.
- [SimState](../../../src/lua_api/state/sim_state.rs) and [state initialization](../../../src/lua_api/state.rs) — session storage and empty default.
- [Original-consumer test](../../../tests/test_c_map_user_waypoint.rs) — Retail provider calls, replacement and refresh behavior.
- [Retail Blizzard UI manifest](../../../data/blizzard-ui-files/retail.txt) — selected profile file list.
- [Lua API](lua-api.md) — broader C_Map surface.
- [Mists corpse map position](mists-corpse-map-position.md) — separate map-position query and supplied-state boundary.

## See Also

- [[lua-api]] — C_Map API surface.
- [[mists-corpse-map-position]] — a separate original WorldMap position consumer.
- [[client-profiles]] — profile selection and cache layout.
