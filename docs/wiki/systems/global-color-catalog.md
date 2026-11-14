# Retail GlobalColor catalog

Retail `C_UIColor.GetColors` publishes the color rows that `Blizzard_SharedXMLBase/Color.lua` converts into named globals and `_CODE` strings. The simulator uses a generated static snapshot of Retail `GlobalColor` DB2 data.

## Content

### Source data

**VERIFIED (authenticated local Retail 12.1.5.70783 CASC):** FileDataID 4328468 is WDC5, 14,262 bytes, table hash `B8BA5FB3`, layout `760E0516`, SHA-256 `aa23f7f48bf490e24f6dcb3c11b525279c0575f050ccc04682cb4f1af1159ef8`, and its MD5 matches the active local root. **VERIFIED (existing WDC5 reader):** the DB2 has 362 physical records and 5 copy records; all records decode, and the CSV contains 367 rows. The raw DB2 payload is not committed. Its copy pairs are preserved in [`GlobalColor.provenance.json`](../../../data/db2/retail-12.1.5.70783/GlobalColor.provenance.json).

**VERIFIED (existing CSV exporter):** the export sorts the merged physical and copy rows by ID, so the CSV alone does not preserve the native feed phases. **VERIFIED (raw WDC5 section and copy table):** the physical main-index IDs are already ascending; copy pairs are `(173,56)`, `(174,57)`, `(175,58)`, `(148,145)`, `(355,328)`. The generator separates them, sorts each phase by ID, visits main rows first, and visits auxiliary copies second.

### Native cache and `GetColors`

**VERIFIED (native Retail 12.1.5.70688, R88o.3):** `GlobalColor` rows enter a tree keyed by `LuaConstantName`. The comparison is case-sensitive and compares unsigned bytes across the shared prefix, then uses string length. The first inserted row wins an equal key. Native visits main-index rows by ascending ID, then auxiliary entries by ascending ID; it skips invalid/unresolved source entries but applies no empty-tag or flag filter. Export walks the tree in ascending key order.

**VERIFIED (native Retail 12.1.5.70688):** `C_UIColor.GetColors` is flag 3 and ignores arguments. It returns one fresh dense array table, with one record per exported entry. Each record has the tag and a fresh color table copied from `ColorMixin`; the builder writes `r`, `g`, `b`, and `a` from ARGB bytes. Each byte becomes f32 and is multiplied by the f32 reciprocal of 255 before publication as a Lua number. Alpha comes from the packed value.

**VERIFIED (DB2 data and native feed rules):** 367 source rows produce 361 unique tags. Six duplicate keys keep these first rows: `AREA_DESCRIPTION_FONT_COLOR` ID 56, `HOUSING_REWARD_TOAST_LABEL_FONT_COLOR` ID 328, `INVASION_DESCRIPTION_FONT_COLOR` ID 58, `INVASION_FONT_COLOR` ID 57, `PURE_RED_COLOR` ID 30, and `REFUND_TOOLTIP_COLOR` ID 145.

**INFERRED:** applying native behavior evidenced on Retail 12.1.5.70688 to the GlobalColor DB2 extracted from Retail 12.1.5.70783 is valid. The version mapping is not directly verified. Runtime loaded catalog contents and external update schedule remain UNKNOWN.

### Original Lua consumer

**VERIFIED (active Retail UI cache):** `Blizzard_SharedXMLBase.toc` has `AllowLoad: Both` and lists `Color.lua`. That file calls `C_UIColor.GetColors()`, creates a `Color` object for each entry, then writes `baseTag` and `baseTag .. "_CODE"` into `GetCurrentEnvironment()`.

The simulator registers the Retail C API after secure-environment creation in [`c_ui_color.rs`](../../../src/c_api/c_ui_color.rs). It stores the same namespace member in `_G` and `__secureenv`; the optional API test checks function identity, fresh results, copied mixin members, sample DB2 winners, channel values, and tag ordering. The original `Color.lua` test verifies a catalog-only color global and its `_CODE` value in both environments after normal startup.

### Profile boundary and preservation

Retail selects the generated 361-row catalog with `client_profile::ACTIVE`. Non-Retail profiles keep their existing temporary five-row fallback because their own GlobalColor datasets are not wired.

**PRESERVATION:** `colors.rs` keeps its 43 hand-set names that also occur in the Retail catalog, plus `TOOLTIP_DEFAULT_COLOR` and `TOOLTIP_DEFAULT_BACKGROUND_COLOR`. No load-order proof shows that `Color.lua` runs before every consumer of every name. The four classic cast-bar names are not written on Retail by `classic_castbar_colors()`.

## Sources

- [Retail `C_UIColor.GetColors` contract](../../specs/c-ui-color-get-colors.md) — versioned requirements and test scope.
- [GlobalColor data provenance](../../../data/db2/retail-12.1.5.70783/GlobalColor.provenance.json) — build, CASC identity, hashes, DB2 copy pairs, and CSV row count.
- [Blizzard UI loading pipeline](../../addon-loading-pipeline.md) — TOC and addon load-order rules.
- Local native evidence: `pr46-49-decomp-answers.md`, sections `C_UIColor.GetColors` (12414–12439) and R88o.3 (13036 onward), Retail 12.1.5.70688.

## See Also

- [[lua-api]] — C API placement and Lua runtime behavior.
- [[client-profiles]] — profile selection and runtime data boundaries.
