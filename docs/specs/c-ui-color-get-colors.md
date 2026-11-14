# Retail `C_UIColor.GetColors`

## Versioned Evidence

- **VERIFIED (Retail 12.1.5.70688, R88o.3):** native `C_UIColor.GetColors` uses registration flag 3, ignores arguments, returns one fresh table, copies `ColorMixin` members, and exports packed `GlobalColor` values through a tag-keyed tree.
- **VERIFIED (local CASC Retail 12.1.5.70783):** FileDataID 4328468 contains 362 physical rows and 5 copy rows; the CSV has 367 rows and no unreadable records.
- **INFERRED:** the 70688 native cache rules apply to the 70783 `GlobalColor` snapshot. These are separate client builds; do not report the mapping as verified.

## Requirements

### Publication and arguments

- Publish `C_UIColor.GetColors` to `_G` and `__secureenv` with the same function value.
- Ignore all supplied arguments without conversion or rejection.
- Return exactly one fresh dense array table. An empty export returns one empty table.

### Catalog feed and export

- Visit main row-ID entries by ascending ID, then auxiliary copy entries by ascending ID. Do not merge both phases into one ID sort.
- Compare `LuaConstantName` keys case-sensitively as unsigned bytes. Compare the shared byte range first, then string length.
- Keep the first inserted row for each equal key; discard later duplicates.
- Do not filter empty tags or row flags. Skip only source entries that cannot resolve to a valid row.
- Export the unique rows in ascending tag-key order, not feed order.
- The pinned CSV has 367 decoded rows. Applying the verified feed and duplicate rules gives 361 exported tags. The six duplicate tags and their winning rows are preserved in the source evidence and tests.

### Records and colors

- Create one record per exported tag with `baseTag` and a fresh `color` table.
- Copy the members of `ColorMixin` into each color table, then set numeric `r`, `g`, `b`, and `a` fields.
- Read the packed value as ARGB from the high byte to the low byte. Use its alpha byte; do not force alpha to 1.
- Convert each byte to `f32`, multiply by the `f32` reciprocal of 255, then publish the result as a Lua number. Do not replace this with double-precision division.
- For a non-empty export, raise `unable to find mixin or metatable (ColorMixin)` when `ColorMixin` is absent or is not a table. An empty export does not need the mixin lookup.

### Original consumer and profile scope

- After normal Retail startup, `Blizzard_SharedXMLBase/Color.lua` consumes `GetColors()` and publishes catalog-backed globals and `_CODE` values in the current environment. The original-consumer test checks both `_G` and `__secureenv`.
- Preserve hand-set Retail color globals until load order is proved before every consumer. Do not remove the values in `colors.rs` based only on `Color.lua` assigning the same names later.
- Use this generated catalog only on Retail. Keep non-Retail fallback behavior unchanged until that profile's own GlobalColor rows are wired.

## Implementation Inventory

- `data/db2/retail-12.1.5.70783/GlobalColor.csv` — pinned source rows.
- `data/db2/retail-12.1.5.70783/GlobalColor.provenance.json` — CASC, DB2, schema, copy-pair, and CSV provenance.
- `src/bin/wow_cli/gen_global_colors.rs` — `wow-cli generate global-colors` generator and feed/dedup/order rules.
- `crates/sim-data-profile/data/global_colors_retail.rs` — generated 361-row Retail export.
- `src/c_api/c_ui_color.rs` — Retail dual-published C API implementation.
- `src/lua_api/workarounds/temporary/color_defaults.rs` — unchanged non-Retail five-row fallback.

## Tests

- `tests/c_ui_color_globalcolor.rs` — optional API test for dual publication, ignored arguments, fresh tables, mixin error/copy, 361 rows, sample DB2 winners, ARGB channels, and tag order.
- The same file uses `prefork_full_ui_case!` to test the original `Color.lua` consumer after Retail startup in both environments.
- `src/bin/wow_cli/gen_global_colors.rs` — unit test for main-before-auxiliary duplicate selection, empty-tag preservation, and unsigned-byte ordering.

## Runtime Unknowns

- **UNKNOWN (Retail 12.1.5.70688):** the actual loaded GlobalColor catalog and its external update schedule are runtime state. The pinned 70783 snapshot is not proof of the current live cache.
- Runtime DB2 decoding remains out of scope; the simulator uses a committed generated snapshot.
