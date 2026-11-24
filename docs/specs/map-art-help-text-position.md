# Map art help-text position

Retail `C_Map.GetMapArtHelpTextPosition(uiMapID)` reads `HelpTextPosition` from the matching `UiMap` record (**VERIFIED — Retail 12.1.5.70688**). An unknown numeric map ID returns `0` (**INFERRED — Retail 12.1.5.70688**; the full entry-to-publisher path is unresolved). Retail data comes from the pinned local-CASC export in `data/db2/retail-12.1.5.70961/UiMap.csv`.

## What it must do

- [x] Retail returns each known map's stored `HelpTextPosition` value, including distinct positions for maps 14, 994, and 1532.
- [x] **INFERRED — Retail 12.1.5.70688:** Retail returns `0` for an unknown numeric map ID; the API test asserts this simulator output.
- [x] Retail's original `FlightMapFrame` consumer uses the returned value to place its ClickToZoom labels after the normal taxi-map open event.
- [x] PTR keeps its existing temporary value; this change does not register the Retail producer on PTR.
- [ ] **INFERRED — Retail 12.1.5.70688 and Forever 1.60.1.70707:** a missing or non-integer argument follows the required-integer parser's error path.
- [ ] Forever remains on its temporary default until Forever UiMap data is available from a working local CASC source.

**Native evidence:** **VERIFIED — Retail 12.1.5.70688:** the handler reads `UiMap.HelpTextPosition` for a matching `UiMap` row and sets the result to zero when the numeric map ID has no row. **INFERRED — Retail 12.1.5.70688 and Forever 1.60.1.70707:** the complete API returns `0` for an unknown numeric map ID because the documented result is non-nil and the handler sets the byte to zero; the full entry-to-publisher path is unresolved. See `pr46-49-decomp-answers.md`, `R88fu`.

**Supporting FlightMap input:** **VERIFIED (R88fx — Retail 12.1.5.70688):** `GetTaxiMapID()` checks the stored value, returns no Lua values when it is zero, and returns one integer when it is nonzero. **INFERRED (R88fx; native evidence pending):** the simulator supplies the map ID before it dispatches `TAXIMAP_OPENED`; native event ordering is unknown. The simulator does not clear this value on close because native clearing behavior is UNKNOWN. See `pr46-49-decomp-answers.md`, `R88fx`.

## How it works

- [C_Map and frame APIs](../lua-api.md)
- [Addon loading and runtime dispatch](../addon-loading-pipeline.md)

## Implementation inventory

| File | Role |
|---|---|
| `src/c_api/c_map.rs` | Retail `C_Map.GetMapArtHelpTextPosition` registration and lookup. |
| `src/lua_api/globals/real/taxi_map.rs` | Retail `GetTaxiMapID` projection of the stored taxi map ID. |
| `src/c_api/c_taxi_map.rs` | Taxi map ID input and taxi-node model. |
| `src/bin/wow_cli/gen_ui_maps.rs` | Generates the separate help-position table from the pinned UiMap CSV. |
| `crates/sim-data/data/ui_map_help_text_positions.rs` | Generated map-ID-to-position table. |
| `data/db2/retail-12.1.5.70961/UiMap.csv` | Local-CASC Retail source export. |
| `data/db2/retail-12.1.5.70961/UiMap.provenance.json` | Build, source, schema, and hash evidence for the export. |
| `src/lua_api/workarounds/temporary/map_art_help_text_defaults.rs` | Retained for non-Retail profiles, including PTR and Forever. |

## Tests asserting this spec

- `tests/c_map_probes.rs::get_map_art_help_text_position_reads_retail_uimap_records`
- `tests/c_map_probes.rs::ptr_map_art_help_text_position_keeps_existing_temporary_default`
- `tests/blizzard_flight_map_loads.rs::flight_map_open_path_places_click_to_zoom_labels_from_uimap_positions`

## Known gaps (current cycle)

- [ ] Forever UiMap export and per-map producer; local Forever CASC is known to be broken (`lt-8408`), so the temporary default remains.
- [ ] Native timing for the taxi map ID write relative to `TAXIMAP_OPENED`, and whether the value is cleared on close, remain UNKNOWN (R88fx).

## Out of scope

- Changing PTR behavior.
- Replacing unrelated generated `UiMapInfo` fields or map-art data.
- Treating the Retail 12.1.5.70961 export as a Forever data source.
