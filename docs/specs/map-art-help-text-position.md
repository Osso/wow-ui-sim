# Map art help-text position

Retail and Forever `C_Map.GetMapArtHelpTextPosition(uiMapID)` read `HelpTextPosition` from the matching `UiMap` record (**VERIFIED — Retail 12.1.5.70688 and Forever 1.60.1.70707**). An unknown numeric map ID returns `0` (**INFERRED — Retail 12.1.5.70688 and Forever 1.60.1.70707, R88g9**; the full entry-to-publisher path is unresolved). Retail data comes from the pinned local-CASC export in `data/db2/retail-12.1.5.70961/UiMap.csv`; Forever data comes from the exact-build Wago export in `data/db2/wowforever-1.60.1.70707/UiMap.csv`. VERIFIED (CSV byte comparison): the 70959 export in `data/db2/wowforever-1.60.1.70959/UiMap.csv` is byte-identical, so the existing generated table remains valid for these exported rows. Native semantics remain 70707-evidenced.

## What it must do

- [x] Retail returns each known map's stored `HelpTextPosition` value, including distinct positions for maps 14, 994, and 1532.
- [x] **INFERRED — Retail 12.1.5.70688:** Retail returns `0` for an unknown numeric map ID; the API test asserts this simulator output.
- [x] Retail's original `FlightMapFrame` consumer uses the returned value to place its ClickToZoom labels after the normal taxi-map open event.
- [x] Forever returns the exported value for a present map ID and the Retail-equivalent miss result for an absent map ID.
- [x] Forever's original `FlightMapFrame` consumer reads the provider for present map 947 and absent map 14 without applying the retired BottomRight fallback style.
- [x] PTR keeps its existing temporary value; this change does not register the producer on PTR.

**Native evidence:** **VERIFIED — Retail 12.1.5.70688 and Forever 1.60.1.70707:** the handler reads `UiMap.HelpTextPosition` for a matching `UiMap` row and sets the result byte to zero when the numeric map ID has no row. **INFERRED — R88g9:** the complete API returns `0` for an unknown numeric map ID because the documented result is non-nil and the separately recovered publisher returns the byte; the full entry-to-publisher path is unresolved. See `pr46-49-decomp-answers.md`, `R88fu`.

**Forever data evidence:** **VERIFIED — exact-build Wago export 1.60.1.70707:** `UiMap.csv` contains 60 unique rows and every `HelpTextPosition` is `0`; SHA-256 `1f4aac70eaac015b1d2d0ea0faf6e3d7fb2cf7afdf45e5bb6772d9b80a72346b`. **VERIFIED — branch behavior:** present map 947 changes from the old fallback `2` to exported `0`; absent map 14 changes from `2` to `0` (**INFERRED — R88g9** for the complete miss return).

**Supporting FlightMap input:** **VERIFIED (Retail 12.1.5.70688 and Forever 1.60.1.70707):** `GetTaxiMapID()` checks the stored value, returns no Lua values when it is zero, and returns one integer when it is nonzero. **INFERRED (R88fx; native evidence pending):** the simulator supplies the map ID before it dispatches `TAXIMAP_OPENED`; native event ordering is UNKNOWN. **UNKNOWN (R88fx):** native close-time clearing is unknown; the simulator retains its last supplied map ID. **INFERRED (R88g0; native evidence pending):** Retail `CloseTaxiMap()` ends the active `TaxiNode` interaction through `C_PlayerInteractionManager`, which emits `TAXIMAP_CLOSED` and the manager hide event; it does nothing when no taxi interaction is active. See `pr46-49-decomp-answers.md`, `R88fx` and `R88g0`.

## How it works

- [C_Map and frame APIs](../lua-api.md)
- [Addon loading and runtime dispatch](../addon-loading-pipeline.md)

## Implementation inventory

| File | Role |
|---|---|
| `src/c_api/c_map.rs` | Retail and Forever `C_Map.GetMapArtHelpTextPosition` registration and lookup. |
| `src/lua_api/globals/real/taxi_map.rs` | Retail `GetTaxiMapID` projection of the stored taxi map ID. |
| `src/c_api/c_taxi_map.rs` | Taxi map ID input and taxi-node model. |
| `src/bin/wow_cli/gen_ui_maps.rs` | Generates the Retail and Forever help-position tables from the pinned UiMap CSVs. |
| `crates/sim-data/data/ui_map_help_text_positions.rs` | Generated Retail map-ID-to-position table. |
| `crates/sim-data/data/ui_map_help_text_positions_forever.rs` | Generated Forever map-ID-to-position table. |
| `data/db2/retail-12.1.5.70961/UiMap.csv` | Local-CASC Retail source export. |
| `data/db2/retail-12.1.5.70961/UiMap.provenance.json` | Build, source, schema, and hash evidence for the export. |
| `data/db2/wowforever-1.60.1.70707/UiMap.csv` | Exact-build Forever source export. |
| `data/db2/wowforever-1.60.1.70707/UiMap.provenance.json` | Forever export source, hash, row count, and value counts. |
| `src/lua_api/workarounds/temporary/map_art_help_text_defaults.rs` | Retained for PTR, Mists, Wrath, Era, and Anniversary. |

## Tests asserting this spec

- `tests/c_map_probes.rs::get_map_art_help_text_position_reads_retail_uimap_records`
- `tests/c_map_probes.rs::ptr_map_art_help_text_position_keeps_existing_temporary_default`
- `tests/blizzard_flight_map_loads.rs::flight_map_open_path_places_click_to_zoom_labels_from_uimap_positions`
- `tests/blizzard_flight_map_loads.rs::flight_map_hide_closes_taxi_interaction_without_clearing_map_id`
- `tests/forever_shim_retirement.rs::forever_map_help_position_consumer_reads_present_and_missing_rows`

## Known gaps (current cycle)

- [ ] Native timing for the taxi map ID write relative to `TAXIMAP_OPENED`, and whether the value is cleared on close, remain UNKNOWN (R88fx).

## Out of scope

- Changing PTR behavior.
- Replacing unrelated generated `UiMapInfo` fields or map-art data.
- Treating the Retail 12.1.5.70961 export as a Forever data source.
