# Item reward classifiers (Retail)

## What it must do

- `C_Item.IsAnimaItemByID(itemInfo)` returns true when the ItemCurrencyValue rows for the converted item, counting each row whose four currency slots contain Reservoir Anima (1813) and whose value is positive once, sum to a positive signed 32-bit wrapping total. Otherwise it returns false. There is no item-catalog prerequisite and no zero-ID shortcut. VERIFIED (WM45).
- `C_Item.IsArtifactPowerItem(itemInfo)` returns true when the item is in the catalog and one of its ItemEffect associations, excluding trigger 8 and triggers above 10, names a spell whose effect index 0 is SpellEffect type 240. Otherwise it returns false. VERIFIED (WM34-A).
- Both convert itemInfo through the shared item reader. A missing or unsupported argument raises an error. A string that converts to item ID zero returns false. VERIFIED (WM45).
- Both publish exactly one Boolean. EMULATED (WM34-A, WM45).

## Data

`wow-cli generate items` writes `crates/sim-data/data/item_classifiers.rs` from `ItemCurrencyValue.csv` and `SpellEffect.csv` (build 70391). In that build, spells 196499 and 196500 have type 240 at effect index 0, and no ItemEffect names them, so `IsArtifactPowerItem` is false for every catalog item. Example: 181544 (Confessions of Misdeed) is an anima item; 6948 (Hearthstone) is not.

## Known gaps

- Item-description flag 256, which can suppress effect collection natively, is not modelled. No 70391 item can reach a type-240 spell, so the flag cannot change a result in this data.
- Native per-difficulty selection of the effect-index-0 row is UNKNOWN; the generator reads DifficultyID 0 rows only.
- Packed-currency palette modes are not modelled; the CSV export supplies plain currency IDs.
- The publication secrecy and taint state of the returned Boolean are UNKNOWN; no declassification is modelled.
- Runtime ItemSparse loading and server replies are not modelled.
