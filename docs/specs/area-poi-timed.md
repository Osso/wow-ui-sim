# Timed area POI classification

`C_AreaPoiInfo.IsAreaPOITimed` classifies static area POI data. Retail and PTR pinned `Blizzard_APIDocumentationGenerated/AreaPoiInfoDocumentation.lua:124-137` declare the API and its two results. See [data and evidence](../wiki/systems/area-poi-timed.md).

## What it must do

- [ ] Retail and PTR accept a required numeric ID, including numeric strings, truncate fractions toward zero, and reject invalid and out-of-range arguments.
- [ ] Return exactly two values: false and nil when no qualifying record exists; otherwise true and the POI's hide-timer flag.
- [ ] Classify from the pinned AreaPOI and PlayerCondition export, independent of timer cache and clock state.

## How it works

- [Area POI timed data](../wiki/systems/area-poi-timed.md).

## Implementation inventory

- `src/c_api/c_area_poi_info.rs`: API registration and classification.
- `src/c_api/area_poi_timed_data.rs`: generated classification data.
- `tools/gen_area_poi_timed.py`: reproducible CSV selection.

## Tests asserting this spec

- `tests/blizzard_frame_xml_util_loads.rs`: original consumer and argument and classification cases.

## Known gaps (current cycle)

- [ ] GREEN verification is pending.
- [ ] Equality between 70391 export records and client 70688 records is unknown.

## Out of scope

Live server timer cache, expression evaluation, and server state are separate from static classification.
