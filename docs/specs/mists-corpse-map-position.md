# Mists corpse map position

Mists exposes the pinned `C_DeathInfo.GetCorpseMapPosition(uiMapID: number)` query, returning a nilable vector2 (`Vector2DMixin`). The declaration is `Blizzard_APIDocumentationGenerated/DeathInfoDocumentation.lua:5,11–22` in the profile cache. [Consumer and state notes](../wiki/systems/mists-corpse-map-position.md) describe selection and data flow.

## What it must do

- [x] Publish the query on Mists through the active-profile gate; do not add other DeathInfo APIs.
- [x] Return one nil result when alive, without a corpse, or without supplied coordinates for the requested map.
- [x] Project supplied coordinates without replacing them with the player's current map position.
- [x] Let the original `DeathMapDataProvider` acquire, position, show and remove its selected `CorpsePinTemplate` across state and map changes.
- [x] Observe the existing `RetrieveCorpse` transition without changing resurrection behavior.

## How it works

- [Consumer and state notes](../wiki/systems/mists-corpse-map-position.md).

Retail, PTR and WoW Forever publish the same query (their `DeathInfoDocumentation.lua:10-22` blocks are byte-identical). There the coordinates come from `A_Admin.SetCorpseMapPosition`; see [health and power inputs](../admin-api/health-power.md). Test: `tests/corpse_map_position.rs`.

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — explicit corpse map observations alongside existing corpse availability.
- `src/lua_api/state.rs` — empty observation default.
- `src/c_api/c_death_info.rs` — Mists-only state-backed query.
- `src/c_api/mod.rs` — namespace registration.

## Tests asserting this spec

- `tests/mists_corpse_position.rs` — original selected WorldMap provider, supplied pin coordinates and removal.

## Known gaps (current cycle)

None. Original-consumer RED/GREEN, producer-only RED, raw stock delta and warning-free Mists/Retail checks are recorded in the [proof notes](../wiki/systems/mists-corpse-map-position.md#executed-proof).

## Out of scope

Native corpse discovery, automatic coordinate capture, map-space transforms, other DeathInfo APIs, and native death/ghost timing are not inferred from the signature. Coordinates are explicit supplied observations.
