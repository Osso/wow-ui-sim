# Map display info

`C_Map.GetMapDisplayInfo` exposes supplied per-map `hideIcons` input from `SimState`. This preparation adds input and test source only; the getter in `src/c_api/c_map.rs` remains unimplemented. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Current Retail/PTR API accepts required non-nil numeric `uiMapID`; a supplied result is exactly one non-nil boolean `hideIcons`, never a DTO.
- [ ] Explicit inputs for maps 84/85 return supplied true/false, including false as one result rather than no result.
- [ ] Queries reflect input updates independently across maps and simulator environments.
- [ ] Simulator-input policy: empty input or removal returns zero values, not one nil or false. Map catalog presence alone supplies no display input.
- [ ] Canonical input starts empty; no guessed defaults, derivation from map flags, or new Lua setter.

## How it works

- [Lua API and simulator state](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: explicit `map_display_hide_icons: HashMap<i32, bool>` input.
- `src/lua_api/state.rs`: empty canonical initialization.
- `src/c_api/c_map.rs`: pending getter/registration; unchanged in preparation.

## Tests asserting this spec

`tests/c_map_probes.rs`, gated by `retail-12-1-0`:

- `get_map_display_info_returns_one_supplied_bool`
- `get_map_display_info_tracks_updates_with_map_and_environment_isolation`
- `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy`

Test source only: no admitted RED or GREEN execution yet. Map 85 is explicitly seeded as a test-only catalog fixture; fixture metadata and supplied booleans are not native map facts.

## Known gaps (current cycle)

- [ ] Implement/register getter after main commits preparation and observes admitted behavioral RED; then establish GREEN and applicable controls/checks.

## Out of scope

Native unknown-map-ID behavior is unestablished. Cached Retail/PTR `MapDocumentation.lua` declarations pin `MayReturnNothing = true` and one `bool hideIcons`, but do not identify which IDs yield zero results. Missing simulator input is not equivalent to an unknown native map. Current cached consumer truthiness does not establish that boundary.

Earlier historical/profile applicability and publication reconciliation remain separate. No native catalog-completeness prerequisite is imposed on this supplied-input slice.
