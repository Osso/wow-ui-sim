# Pet information by item

Retail 12.1.5 and PTR 12.1.7 expose `C_PetJournal.GetPetInfoByItemID`, which identifies a pet
learned by an item and supplies its species data to the selected PlayerChoice reward consumer.

Neither pinned `Blizzard_APIDocumentationGenerated` cache declares it:
`PetJournalInfoDocumentation.lua` lists `GetPetInfoTableBySpeciesID` (`:166`) immediately followed
by `GetPetLoadOutInfo` (`:182`) in both profiles. The published contract therefore comes from the
pinned consumers:

- `Blizzard_PlayerChoice/Blizzard_PlayerChoiceOptionBase.lua:691` — calls it unguarded from
  `PlayerChoiceBaseOptionItemRewardMixin:IsDressupReward` and uses the truthiness of the result.
- `Blizzard_UIPanels_Game/Mainline/DressUpFrames.lua:178` — reads 13 positions, using creatureID
  (4), displayID (12) and speciesID (13).
- `Blizzard_SharedXML/LinkUtil.lua:127` (PTR only) — reads displayID from position 12.

## What it must do

- [ ] Accept a number or numeric string, ignore extra arguments, truncate to a signed 32-bit integer, and report the WM78 usage or overflow error for invalid input.
- [ ] Return no values for zero, unknown, non-pet, and otherwise unmatched item IDs.
- [ ] Return the 13-position WM78 species tuple for a matched item, including nil icon, name, or display positions where the source data requires them.
- [ ] Derive matches from ItemSparse, ItemXItemEffect, ItemEffect, SpellEffect, SummonProperties, and BattlePetSpecies using WM78 and WM78b. Do not select a winner for competing difficulty rows.
- [x] Expose the same contract on Retail and PTR, where the pinned sources call the same namespace member.
- [x] Allow the unmodified PlayerChoice item reward consumer to distinguish a pet item from a non-pet item.

## How it works

- [C API and Lua boundary](../lua-api.md)

## Implementation inventory

- `tools/generate_pet_item_catalog.py` — derives the pinned item-to-species data.
- `data/pet-item-catalog.json` and `.provenance.json` — generated catalog, input hashes, and unresolved display IDs.
- `src/c_api/c_pet_item.rs` — Retail and PTR namespace registration, argument parsing, and tuple projection.

## Tests asserting this spec

- `tests/mount_item_player_choice.rs` — original consumer, non-pet control, tuple, and argument checks.

## Known gaps (current cycle)

- [ ] Equal lowest `OrderIndex` rows with distinct display IDs have an unknown winner. The 70391 export has no affected pet items. The generator records any future cases; the simulator returns no values for them rather than choosing an invented display.
- [ ] EMULATED: SpellName record presence backs the native spell-presence check; their equivalence is unverified. A failed check does not resume at a later item-effect slot.
- [ ] PTR uses the Retail catalog. `data/pet-item-catalog.json` derives from build 12.1.5.70391, but the pinned PTR UI is 12.1.7.70493. Running the same generator over the 70493 tables yields 1645 records instead of 1636: 9 added item IDs, 3 changed records (item 277270 renames `Kirin Tor Kitty` to `Bewitching Familiar`, items 274923 and 275020 change description text). Until a PTR catalog is generated, PTR answers for those 12 items reflect the older build.
- [ ] Live item-cache request completion and event timing are unknown.
- [ ] Secret-value and restriction behavior is unknown.
- [ ] Native evidence is build 70688; the generated source data is 70391 and is not claimed to be identical.

## Out of scope

No simulated cache-request lifecycle, native event timing, or cross-profile publication beyond the
Retail and PTR caches whose consumers call this member.
