# Character select screen data

This module provides profile-selected `ChrClasses.SelectScreenFileDataID` and `ChrRaces.SelectScreenFileDataID`/`LowResScreenFileDataID` values. It keeps exported rows separate from the native character-select API and roster.

## Content

`tools/gen_chr_select_screen_data.py` runs the game-engine DB2 exporter on local CASC files and writes selected CSV rows, Rust lookup tables, and provenance. The generated data directories name the exact source build; each provenance file records the product, build key, DB2 layout and hash, WoWDBDefs revision and hash, row count, and unreadable-row count. The Rust module uses sorted rows, returns `None` for an absent row, and retains present zero values.

The Retail data uses installed build 12.1.5.70783, not the 12.1.5.70575 simulator pin. The Forever data uses installed build 1.60.1.70888; its 1.60.1.70707 pin/decomp values remain UNKNOWN. PTR data is UNKNOWN and is not included. The native fallback cascade and character roster remain with the separate `lt-a667` API lane.

## Sources

- [Feature contract](../../specs/chr-select-screen-data.md)
- [Generated data and provenance](../../../data/chr-select-screen/)
- [WoWDBDefs schemas](../../../data/chr-select-screen/definitions/)

## See Also

- [[client-profiles]] — profile-specific data selection.
