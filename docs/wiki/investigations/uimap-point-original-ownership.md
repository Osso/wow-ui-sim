# UiMapPoint original constructor ownership

Retail's original ObjectAPI owns `UiMapPoint.CreateFromCoordinates` and
`CreateFromVector2D`. Temporary constructor removal is a PRESERVATION change,
not a fix for the currently unmodeled waypoint callback.

## Ownership and scope

`Blizzard_ObjectAPI_Mainline.toc` selects `Mainline/UiMapPoint.lua` for Game,
mainline. The original file replaces the `UiMapPoint` table and supplies the
constructors before WorldMap loading. Coordinates produce a real Vector2D;
the vector constructor preserves the supplied object's identity. Both retain
optional z. The old temporary constructors produced flat x/y tables instead.

The bootstrap now omits only these two temporary constructors on Retail.
Other profiles retain their existing fallback pending profile-specific
original ownership evidence. In particular, local PTR runtime ObjectAPI and
Mists cache inputs were unavailable; this is not evidence of absent owners.
`__wow_map_point_from_table` remains because exploration independently uses it.
No original Blizzard source is changed.

## Proof boundary

`tests/stub_f_uimap_point_ownership.rs` follows ordinary selected game startup
and WorldMap addon loading, checks original source bindings, and verifies
multiple coordinate/map/z inputs and shared-vector identity/mutation. It
asserts unfiltered startup/load errors are empty. The library boundary test
checks absence before original loading on Retail and retained behavior on
other profiles. These are preservation selectors, not a RED waypoint test.

The earlier runtime diagnostic found the actual WaypointLocation provider,
but `C_Map.SetUserWaypoint`, `GetUserWaypoint`,
`GetUserWaypointPositionForMap`, and `CanSetUserWaypointOnMap` were absent.
Original refresh and click callbacks therefore fail before the relevant
constructor use. Native API modeling is separate work; no fake provider,
manual native state or guessed waypoint implementation belongs in this slice.

## Sources

- [Temporary map state](../../../src/lua_api/workarounds/temporary/map_runtime_state.rs)
- [Original ownership selector](../../../tests/stub_f_uimap_point_ownership.rs)
- Runtime cache: `Blizzard_ObjectAPI/Blizzard_ObjectAPI_Mainline.toc` and
  `Blizzard_ObjectAPI/Mainline/UiMapPoint.lua` (Retail).
- Tracker `lt-6c8e`, coordinator dispositions #884 and #895: Retail-only
  preservation permitted; waypoint modeling is a separate prerequisite.

## See Also

- [[consumer-selection-and-red-proof]] — runtime selection and preservation labels.
