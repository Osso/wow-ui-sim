# Forever supplied spellbook outfit observations

Forever 1.60.1.70532 uses the shared transmog availability observation and the shared outfit catalog for the Player Spells panel. See [shared availability](transmog-availability.md) and [model and proof](../wiki/systems/forever-camelot-spellbook.md).

## What it must do

- [x] Extend IsTransmogEnabled publication and provide the Forever GetOutfitInfo/GetOutfitsInfo queries via the Forever profile gate, while storing rows in the shared outfit catalog.
- [x] Use the shared outfit catalog as the sole list state: empty returns zero values; populated returns one array of records.
- [x] Create supplied Forever outfits through `C_TransmogOutfitInfo.AddNewOutfit` after an A_Admin capacity input; do not raise an unset-list error.
- [x] Return fresh rows and nested situationCategories arrays on each snapshot, preserving supplied record order and all seven declared fields.
- [x] Original selected Camelot construction, availability and sparse range consumers discriminate transitions, replacements and independent environments.

## How it works

- [Model and proof boundary](../wiki/systems/forever-camelot-spellbook.md)

## Implementation inventory

- `src/c_api/transmog_availability.rs`: shared availability registrar and observation.
- `src/c_api/forever_outfits.rs`: Forever-only empty-default queries.
- `src/c_api/outfit_entry.rs`: complete shared record and fresh snapshot conversion.
- `src/lua_api/state/sim_state.rs` and `state.rs`: shared outfit catalog.
- `src/c_api/c_transmog_outfit_info/lifecycle.rs`: Forever A_Admin capacity input and public AddNewOutfit writer.
- `tests/forever_camelot_spellbook.rs`: original selected consumers and two-outfit panel fixture.

## Tests asserting this spec

Four exact selectors in `tests/forever_camelot_spellbook.rs` cover enabled-first availability, disabled-first availability, independent sparse catalog projection, and Player Spells opening with no outfits and then two created outfits. `src/c_api/forever_lib_baseline_tests.rs` also checks the catalog query against supplied entries.

## Known gaps (current cycle)

- [ ] Native default return arity for an empty outfit catalog remains unverified; current zero-value behavior is INFERRED (R88ex) from the pinned `MayReturnNothing` declaration and guarded Blizzard consumers.

The selected original Player Spells toggle and supplied two-outfit projection pass in this lane. This does not establish clean full-addon startup or native behavior.

## Out of scope

Native availability/default values, automatic supply, events, persistence, wardrobe mutations and other outfit queries. The default no-outfit result is INFERRED (R88ex; native evidence pending) from the pinned `MayReturnNothing` declaration and guarded Blizzard consumers. Pinned generated documentation establishes return shape, not policy: `TransmogOutfitInfoDocumentation.lua:383–392` declares GetOutfitsInfo with MayReturnNothing; entries at 896–907 declare seven non-nil fields. IsTransmogEnabled at 631–638 declares one non-nil bool.
