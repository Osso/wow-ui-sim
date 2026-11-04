# Timed area POI data

The Retail/PTR timed classification uses a generated static lookup. It does not consult the separate timer cache.

The generator selects AreaPOI records whose referenced PlayerCondition has a nonzero `WorldStateExpressionID`, and stores bit 22 of AreaPOI `Flags` as the hide-timer value. WM75b verified the CSV mapping against client 70688. The wago.tools 12.1.5.70391 enUS exports and their SHA-256 hashes are recorded in the generated file. Whether those export records equal the 70688 client records is unknown. No expression or live server state is evaluated.

## Sources

- [Classification contract](../../specs/area-poi-timed.md).
- [Generated data](../../../src/c_api/area_poi_timed_data.rs) and [generator](../../../tools/gen_area_poi_timed.py).

## See Also

- [Area POI publication](../../specs/area-poi-patch-12-0-5.md).
