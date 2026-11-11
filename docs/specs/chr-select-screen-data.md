# Character select screen data

This contract defines profile-selected class and race FileDataID lookups in `src/c_api/chr_select_screen_data.rs`. Generated rows come from local `ChrClasses` and `ChrRaces` DB2 files; source builds and hashes are recorded beside each data set. Lane `lt-a667` owns the native `C_CharacterSelect.GetSelectBackgroundModel` API and supplies its class/race maps; these accessors feed those maps after that PR merges.

## What it must do

- [ ] Return the selected class FileDataID for a present class row. Preserve a present zero as `Some(0)` and return `None` for an absent row.
- [ ] Return the race select and low-resolution FileDataIDs for a present race row. Preserve zero fields and return `None` for an absent row.
- [ ] Return `None` when the active profile has no select-screen data.

## How it works

- [Character select screen data](../wiki/systems/chr-select-screen-data.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `tools/gen_chr_select_screen_data.py`: exports selected fields from local DB2 files and writes profile data plus provenance.
- `data/chr-select-screen/definitions/`: the two pinned WoWDBDefs schemas used by the exporter.
- `data/chr-select-screen/retail-12.1.5.70783/`: generated Retail rows and provenance.
- `data/chr-select-screen/forever-1.60.1.70888/`: generated Forever rows and provenance.
- `src/c_api/chr_select_screen_data.rs`: profile-selected lookups and row tests.
- `src/c_api/mod.rs`: registers the data module.

## Tests asserting this spec

- `src/c_api/chr_select_screen_data.rs`: exact Retail and Forever rows, present-zero values, absent IDs, and unsupported-profile behavior.

## Known gaps (current cycle)

- [ ] Retail 12.1.5.70575 rows are UNKNOWN. The exported Retail data is from installed build 12.1.5.70783 and is labeled with that build.
- [ ] Forever 1.60.1.70707 select-screen rows are UNKNOWN. The exported Forever data is from installed build 1.60.1.70888 and is labeled with that build.
- [ ] PTR select-screen rows are UNKNOWN; no PTR data is provided.

## Out of scope

- `C_CharacterSelect.GetSelectBackgroundModel`, `C_Login`, character roster state, per-character overrides, and the native cascade. The separate API owner will consume these accessors through host-supplied class and race maps after merge.
