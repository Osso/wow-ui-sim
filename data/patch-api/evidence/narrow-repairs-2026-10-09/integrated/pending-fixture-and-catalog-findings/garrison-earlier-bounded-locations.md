# Local data discovery: Garrison auto-combat damage-class catalog

Scope: bounded, read-only inspection of the requested local repo/cache/data/dependency locations. The two prior reports and `docs/wiki/index.md` were read first. No network, builds, tests, runtime, CASC extraction/writes, operations, delegation, or project edits. Frozen/vendor files were not modified. The known missing-member fallback producer was not re-investigated.

## Result

No local literal native-catalog rows or matching DB2 table/schema were found in the inspected data. The only concrete local values are generated API/documentation shape and `Enum.Damageclass` metadata; neither establishes `C_Garrison.GetAutoCombatDamageClassValues()` output rows or localized labels. No `(damageClassValue, locString)` row is established by this inspection.

## Bounded evidence

- `data/db2/` has CSV data for `QuestPOIBlob`, `UiMapArt`, `UiMapArtStyleLayer`, `UiMapArtTile`, `UiMapFogOfWar`, `UiMapFogOfWarVisualization`, `UiMapXMapArt`, `UiTextureAtlasElementSliceData`, `WorldMapOverlay`, and `WorldMapOverlayTile`; no garrison/auto-combat/damage-class catalog file is present in that directory listing.
- The only versioned subdirectory, `data/db2/wowforever-1.60.1.69913/`, contains `GlobalStrings.csv`, atlas CSVs, and listfile/provenance artifacts. Its provenance identifies source as exact-build Wago DB2CSV exports supplied in `/tmp/wowforever-db2`; its recorded hashes cover only those listed files. This is a different client/build dataset and contains no matching catalog table among the listed exports.
- `Cargo.toml` declares `asset-resolver` pinned to git revision `462097bba2aa72d1afed8e0ca5b7c7315096427d`, optional, alongside CASC-related dependencies. The expected checkout `/home/osso/Repos/asset-resolver` is absent. This project configuration and dependency identity do not expose a local DB2 schema, table, or rows. No asset-resolver docs/source were available at the checked location; no dependency fetch was attempted.
- `/home/osso/.cache/wow-ui-sim/` contains caches including `blizzard-ui/` and `casc-extract/`. Bounded discovery found no standalone DB2/CSV file by those suffixes under this cache (the file-location search helper was unavailable, so no blanket filesystem traversal was attempted). Existing cached UI Lua was already covered by prior reports; it contains consumers, not a complete localized catalog.
- `docs/wow-client-diff/WowDiscovery.lua` includes generated `Enum.Damageclass` metadata: base values Physical=0, Holy=1, Fire=2, Nature=3, Frost=4, Shadow=5, Arcane=6, plus mask constants. These are enum declarations, not evidence that the catalog method returns these rows, nor locale-resolved `locString` values.

## Available source shape vs missing data

Generated API documentation metadata in the cached report declares a required table result with row fields `damageClassValue` (number) and `locString` (string). That is shape only. Enum metadata supplies candidate numeric constants but no method-to-row mapping or localized strings. No exact table/schema name, record IDs, localization join, or row set is available from inspected local data.

## Gaps / limit

The project’s currently inspected local DB2 exports do not include a matching catalog. The asset-resolver source/docs checkout was absent, and this inspection did not establish any DB2 parser/schema in a dependency cache. Accordingly, hashes for hypothetical target rows are unavailable: there are no target rows to hash. Exact source provenance, included damage classes, locale-specific labels, and completeness remain unestablished. This is not evidence that no such table exists in all local/installed WoW data; the requested bounded locations and available source did not reveal one.
