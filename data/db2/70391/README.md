# Retail scaling tables — 12.1.5.70391

Verified: 2026-11-04. This is an offline data snapshot, not a runtime loader change.

| CSV | Readable rows | Omitted rows |
| --- | ---: | ---: |
| ItemScalingConfig | 418 | 0 |
| ItemOffsetCurve | 70 | 0 |
| ItemSquishEra | 2 | 0 |
| ContentTuning | 2,950 | 0 |
| ItemBonusListLevelDelta | 1,801 | 0 |
| Curve | 62,817 | 2 |
| CurvePoint | 172,321 | 6 |

## Identity and limits

[manifest.json](manifest.json) pins the Blizzard build config, root and encoding content keys, each FileDataID and layout, CSV hashes, definition hashes, exporter files and decoder binary. WoWDBDefs revision `d6f50d4f351193e84226aa62a36faa0f58528afa` supplies the exact layouts found in the 70391 WDC5 headers; selection is by layout hash, not a guessed column order or an explicit 70391 BUILD annotation.

The five complete payloads match their root content keys. The two partial payloads do **not** match their full content keys: the existing decoder zeroed encrypted sections whose key is unknown. Their encoded BLTE headers match their encoding keys. The CSV exporter omits those sections. No replacement rows are supplied.

Missing key `DAC5AA105D0C1B67` blocks Curve IDs `121405`, `121406` and CurvePoint IDs `355127`–`355132`. These IDs come from the clear WDC5 encrypted-record metadata. Do not treat an absent row in these CSVs as proof that the native table lacks it.

The current installed Retail build was 12.1.5.70783. It was not selected as the export identity. Reads addressed immutable 70391 archive entries through the authenticated 70391 root/encoding metadata. No current-build table data was substituted.

## Export and verification

The existing game-engine exporter was used without changes:

```text
PYTHONDONTWRITEBYTECODE=1 python3 <game-engine>/scripts/export_db2_csv.py <TABLE> <pinned-raw>/<FDID>.db2 <output>/<TABLE>.csv --definitions <WoWDBDefs>/definitions
```

Curve and ItemBonusListLevelDelta BLTE decoding used the existing `asset-resolver` `casc-local` executable, a private pinned-build configuration/cache, and read-only local archives. Layout `52DF470F` places `ItemLevelDelta` before `ID`; the CSV keeps that exact order. No new decoder, dependency, build, network test, or shared data write was needed.

[verification.json](verification.json) records offline hash, header, row-count, unique-ID and reference checks. ItemBonus input is separately hash-pinned; it is not copied into this snapshot. Native dispatch evidence is from build 70688, not native proof for 70391. Only VERIFIED selectors in `WM97-item-descriptor.md`, DQ3, are checked. The manifest lists selectors whose exact target remains unverified; they are not inferred from ID ranges.

Every checked nonzero reference resolves. All 1,801 ItemBonusListLevelDelta IDs occur as `ItemBonus.ParentItemBonusListID` values. Every curve used by ItemOffsetCurve, ItemSquishEra and ContentTuning has readable points. This does not establish that those curves have every native point, or that simulator interpolation/item-level behavior is correct.
