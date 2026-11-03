# Pet information by item

Retail 12.1.5 `C_PetJournal.GetPetInfoByItemID` identifies a pet learned by an item and supplies its species data to the selected PlayerChoice reward consumer.

## What it must do

- [ ] Accept a number or numeric string, ignore extra arguments, truncate to a signed 32-bit integer, and report the WM78 usage or overflow error for invalid input.
- [ ] Return no values for zero, unknown, non-pet, and otherwise unmatched item IDs.
- [ ] Return the 13-position WM78 species tuple for a matched item, including nil icon, name, or display positions where the source data requires them.
- [ ] Derive matches from ItemSparse, ItemXItemEffect, ItemEffect, SpellEffect, SummonProperties, and BattlePetSpecies using WM78 and WM78b. Do not select a winner for competing difficulty rows.
- [x] Allow the unmodified PlayerChoice item reward consumer to distinguish a pet item from a non-pet item.

## How it works

- [C API and Lua boundary](../lua-api.md)

## Implementation inventory

- `tools/generate_pet_item_catalog.py` — derives the pinned item-to-species data.
- `data/pet-item-catalog.json` and `.provenance.json` — generated catalog, input hashes, and unresolved display IDs.
- `src/c_api/c_pet_item.rs` — Retail namespace registration, argument parsing, and tuple projection.

## Tests asserting this spec

- `tests/mount_item_player_choice.rs` — original consumer, non-pet control, tuple, and argument checks.

## Known gaps (current cycle)

- [ ] Equal lowest `OrderIndex` rows with distinct display IDs have an unknown winner. The 70391 export has no affected pet items. The generator records any future cases; the simulator returns no values for them rather than choosing an invented display.
- [ ] EMULATED: SpellName record presence backs the native spell-presence check; their equivalence is unverified. A failed check does not resume at a later item-effect slot.
- [ ] Live item-cache request completion and event timing are unknown.
- [ ] Secret-value and restriction behavior is unknown.
- [ ] Native evidence is build 70688; the generated source data is 70391 and is not claimed to be identical.

## Out of scope

No simulated cache-request lifecycle, native event timing, or cross-profile publication is claimed.
