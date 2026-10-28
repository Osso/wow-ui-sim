# Shared Retail atlas data

## Contract

The shared atlas data uses Wago DB2CSV exports for Retail build 12.1.5.70448. The older local 70575 export is retained as historical evidence, not used for generation. Retail `UiTextureAtlas` contains `ID,FileDataID,AtlasWidth,AtlasHeight,UiCanvasID`; it has no atlas-set column. `--no-atlas-sets` skips set filtering and does not require that column. Without the option the column remains mandatory. `UiCanvasID` is present and remains required.

Element 35032, member 38450, atlas 3956 provide `common-sidetab`: 45×60, atlas 128×128, FDID 8033669 (`interface/common/commonsidetab.blp`), bounds 48,93,1,61. PTR 12.1.7.70493 has identical element, member and common atlas fields. PTR adds `UiTextureAtlasSetID=0` to atlas 3956; Retail has no set column.

## Reproduction

Source exports, listfile subset, URLs and hashes are in `data/db2/retail-12.1.5.70448/`. Run:

```text
wow-cli generate atlas --csv-dir data/db2/retail-12.1.5.70448 --listfile data/db2/retail-12.1.5.70448/listfile.csv --no-atlas-sets --output crates/sim-data-profile/data/atlas.rs --elements-output crates/sim-data-profile/data/atlas_elements.rs
```

The generator reports 17,652 atlas entries, 261 skipped rows, and 19,680 element mappings. Six skipped rows lack file mappings for four distinct FDIDs; the remaining 255 are duplicate names. Compared with the previous output, 1,641 names were added and 27 removed (counts based on parsed atlas keys); the generation includes other output changes as well.
