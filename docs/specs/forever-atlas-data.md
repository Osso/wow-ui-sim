# Forever atlas data

## Contract

- `client-wowforever` uses atlas and element data from build `1.60.1.69913`, never the retail atlas database.
- Explicit `wow-cli generate atlas --csv-dir DIR --listfile FILE --output FILE --elements-output FILE` reads slice data only from DIR. Without options, existing input/output paths and retail slice override remain unchanged.
- Atlas CSV requires named `ID`, `FileDataID`, `UiTextureAtlasSetID`, `AtlasWidth`, `AtlasHeight`, and `UiCanvasID` columns in any order. Missing columns and rows whose field count differs from the header fail explicitly; numeric errors identify the row and column. No positional or set/canvas defaults are accepted.
- Named CSV atlas dimensions account for set/canvas columns. Public element names resolve to their default-set (`1`), 1×-canvas (`1`) members; concrete member names remain addressable. This is the selected simulator canvas policy, not native canvas-selection conformance.
- Element 35224 (`UI-HUD-Minimap-Frame-Cycle`) exposes 42×42 geometry with UV bounds 256/512 to 298/512 from atlas 3944, FDID 8026705.

## Reproduction

Source exports and a relevant community-listfile subset live in `data/db2/wowforever-1.60.1.69913/`; `provenance.json` records hashes and unresolved IDs.

```text
wow-cli generate atlas --csv-dir data/db2/wowforever-1.60.1.69913 --listfile data/db2/wowforever-1.60.1.69913/listfile.csv --output data/atlas_wowforever.rs --elements-output data/atlas_elements_wowforever.rs
```

Generation emits 19,203 atlas entries and 20,684 element mappings. The 2,306 skipped rows comprise 1,560 missing-file-mapping rows (55 distinct FDIDs) and 746 duplicate names. No geometry is fabricated for unresolved files; this is not complete asset coverage.

## Proof

- `tests/wowforever_atlas.rs`: Lua lookup geometry and element identity.
- `src/bin/wow_cli/gen_atlas_tests.rs`: concrete CSV fixture, canvas choice, generated geometry and local slice output.
- Full startup and rendering acceptance are separate integration gates.
