# Forever supplied spellbook outfit observations

Forever 1.60.1.70532 uses the shared transmog availability observation and a separately supplied outfit-list observation. See [shared availability](transmog-availability.md) and [model and proof](../wiki/systems/forever-camelot-spellbook.md).

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
- `src/c_api/forever_outfits.rs`: Forever-only supplied query.
- `src/c_api/outfit_entry.rs`: complete shared record and fresh snapshot conversion.
- `src/lua_api/state/sim_state.rs` and `state.rs`: shared outfit catalog.
- `tests/forever_camelot_spellbook.rs`: original selected consumers.

## Tests asserting this spec

Three exact selectors in `tests/forever_camelot_spellbook.rs` cover enabled-first availability, disabled-first availability, and independent sparse catalog projection.

## Known gaps (current cycle)

None within the approved bounded slice. Three exact RED/GREEN pairs, paired raw stock and seven zero-warning all-target checks pass; the compile gate is recorded in the [wiki](../wiki/systems/forever-camelot-spellbook.md#proof-status).

## Out of scope

Native availability/default values, automatic supply, events, persistence, wardrobe mutations and other outfit queries. The default no-outfit result is INFERRED (R88ex; native evidence pending) from the pinned `MayReturnNothing` declaration and guarded Blizzard consumers. Pinned generated documentation establishes return shape, not policy: `TransmogOutfitInfoDocumentation.lua:383–392` declares GetOutfitsInfo with MayReturnNothing; entries at 896–907 declare seven non-nil fields. IsTransmogEnabled at 631–638 declares one non-nil bool.
