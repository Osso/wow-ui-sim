# Map art data

`C_Map.GetMapArtID`, `GetMapArtLayers`, and `GetMapArtLayerTextures` use the shared generated `UiMapXMapArt` relation. Their simulator implementation lives in `src/c_api/c_map.rs`; see the [texture and atlas system](../texture-atlas-system.md) for rendering details.

## What it must do

- [x] Return the art ID from the generated map-art record for a known map with art data.
- [x] Return layer dimensions and tile IDs from the same generated map-art record.
- [x] Return `0` for a known map without generated art data, and no values for an unknown map.
- [x] Keep the ID, layer dimensions, and tile count consistent for maps 947 and 2248 on Retail and WoW Forever.

## How it works

- [Texture and atlas system](../texture-atlas-system.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_map.rs` — map-art API lookups.
- `src/lua_api/state/defaults/maps.rs` — seeded map identity and hierarchy; art IDs are not stored here.
- `src/lua_api/state_types/collections.rs` — `MapData` fields other than generated art data.
- `crates/sim-data/src/lib.rs` — shared generated map-art module.
- `crates/sim-data/data/map_art.rs` — generated map-to-art, layer, and tile records.
- `tests/c_map_probes.rs` — API consistency tests.
- `tests/blizzard_map_canvas_loads.rs` — two-map world-map loading proof.

## Tests asserting this spec

- `tests/c_map_probes.rs::get_map_art_apis_share_generated_relation`
- `tests/blizzard_map_canvas_loads.rs::world_map_canvas_uses_model_for_two_maps`

## Known gaps (current cycle)

- No profile-specific Forever map-art catalog is present. The generated table is shared by all client builds. The listed Retail and Forever tests verify simulator behavior for maps 947 and 2248, not full native catalog parity.

## Out of scope

- Native map-art parity outside the supplied map IDs.
