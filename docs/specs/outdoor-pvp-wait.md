# Outdoor PvP wait time

The original WorldMap zone timer reads supplied outdoor PvP timing state. Retail and PTR declare `C_PvP.GetOutdoorPvPWaitTime` in `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:455`.

## What it must do

- [ ] Map 123 reads gate 3801 and deadline 4354; map 244 reads gate 5387 and deadline 5332.
- [ ] Unknown maps and absent or zero gates return one nil. Otherwise return the supplied deadline minus current Unix seconds only when positive; a missing deadline acts as zero.
- [ ] Accept signed 32-bit numeric-convertible map IDs, truncating finite fractions after range checking. Reject missing or invalid IDs.
- [ ] The original WorldMap zone timer shows a supplied positive wait and hides when the gate clears.

## How it works

- [UI and event architecture](../event-system.md)

## Tests asserting this spec

- `tests/blizzard_world_map_loads.rs::world_map_zone_timer_consumes_supplied_pvp_world_states`

## Out of scope

- Native clock adjustment ownership and server-driven world-state updates are unknown.
