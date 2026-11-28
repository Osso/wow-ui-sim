# Open the world map

VERIFIED declarations: the pinned Retail, PTR, and Forever `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:578-585` declare `C_Map.OpenWorldMap(uiMapID?)`, with no return values and `HasRestrictions = true`. Lines 641-648 declare the synchronous `WORLD_MAP_OPEN` event with the same payload. The original Retail `Blizzard_WorldMap/Blizzard_WorldMap.lua:150,175-177` and Forever `Blizzard_WorldMap/Blizzard_WorldMap.lua:159,186-188` register and handle that event.

## What it must do

- [ ] Publish `C_Map.OpenWorldMap` on Retail, PTR, and Forever. Preserve the existing combat-only protected-call policy: an insecure request in combat leaves the map selection unchanged and reports `ADDON_ACTION_BLOCKED`.
- [ ] INFERRED: synchronously signal `WORLD_MAP_OPEN` with the supplied map ID or nil; return no values.
- [ ] INFERRED: the original world map opens on the requested map and processes a second distinct map request while open.

## How it works

- [Synchronous event dispatch](synchronous-event-dispatch.md)
- [Protected global calls](protected-globals.md)

## Implementation inventory

- `src/c_api/c_map.rs`: profile-gated publication, existing combat-only protection, and synchronous event request.
- `tests/world_map_interaction_close.rs`: original map frame and event callback consumer.

## Tests asserting this spec

- `open_world_map_selects_requested_pages` (Retail prefork) and `forever_open_world_map_selects_requested_pages`: two distinct explicit map IDs, visible map page, event payload and omission, zero return values, and loader-tainted request refusal in combat.

## Known gaps (current cycle)

- [ ] Native invocation and restrictions details are unverified; existing simulator protection policy is unchanged.

## Out of scope

Map assignment and art catalog regeneration; other profiles do not declare this API in the pinned generated documentation.
