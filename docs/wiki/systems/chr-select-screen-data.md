# Character select screen data

**VERIFIED:** This module provides profile-selected `ChrClasses.SelectScreenFileDataID` and `ChrRaces.SelectScreenFileDataID`/`LowResScreenFileDataID` values. It keeps exported rows separate from the native character-select API and roster.

## Content

**VERIFIED:** `tools/gen_chr_select_screen_data.py` runs the game-engine DB2 exporter on local CASC files and writes selected CSV rows, Rust lookup tables, and provenance. The generated data directories name the exact source build; each provenance file records the product, build key, DB2 layout and hash, WoWDBDefs revision and hash, row count, and unreadable-row count. The Rust module uses sorted rows, returns `None` for an absent row, and retains present zero values.

**VERIFIED:** Retail data uses installed build 12.1.5.70783, not the 12.1.5.70575 simulator pin. Forever data uses installed build 1.60.1.70888. **UNKNOWN:** Forever 1.60.1.70707 values and PTR values; this data set contains no PTR rows. **VERIFIED per R52 lane brief:** `lt-a667` owns the native fallback cascade and character roster consumer. **VERIFIED on 2026-11-11:** Retail and Forever each pass all three targeted unit tests; Retail and Forever test binaries built with zero compiler warnings; PTR, Mists, Era, Anniversary, and Wrath all-target checks pass with zero warnings and compile `wow_ui_sim` with `fresh=false`. The focused tests ran on code source commit `38e54010cda306d15ef10118256a620b0d648dcc`; profile checks ran on `7e91a97b17175750bf0b8df6d3855dec1bf68cf2`.

## Sources

- [Feature contract](../../specs/chr-select-screen-data.md)
- [Generated data and provenance](../../../data/chr-select-screen/)
- [WoWDBDefs schemas](../../../data/chr-select-screen/definitions/)

## See Also

- [[client-profiles]] — profile-specific data selection.

**VERIFIED:** The PTR profile's no-data test passes for both accessors. **UNKNOWN:** PTR DB2 rows are not included.
