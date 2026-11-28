# Supplied transmog availability

Retail/PTR, Mists and Forever use one explicitly supplied environment-local availability observation in `src/c_api/transmog_availability.rs`. Neither the query nor the modern pending/apply model infers a native default. [Model and proof boundary](../wiki/systems/transmog-availability.md).

## What it must do

- [x] Publish `C_TransmogOutfitInfo.IsTransmogEnabled` for Retail, PTR and Mists through `client_profile::ACTIVE`.
- [x] Extend that same observation to Forever for the [selected Camelot spellbook](forever-camelot-spellbook.md), without enabling the modern catalog. Wrath, Era and Anniversary remain excluded by the ACTIVE guard; this is not a runtime absence claim.
- [x] Initialize availability to None; Some(true)/Some(false) return exactly one corresponding boolean; None raises `C_TransmogOutfitInfo.IsTransmogEnabled: transmog availability has not been supplied`.
- [x] Modern pending staging, set/custom-set imports and application consume this same observation; collected pending records reflect supplied availability, while commit checks current availability, not only the staged record.
- [x] Original selected set/custom-set mouse callbacks and popup apply callbacks discriminate both boolean values, independent environments, unset rejection and recovery; failed application preserves money/pending/saved state.
- [x] Preserve the original Mists settings branch contract and replace old modern fixture defaults with explicit observations.

## How it works

- [Shared observation and original-consumer proof](../wiki/systems/transmog-availability.md)
- [Mists display branch](../wiki/systems/mists-transmog-enabled.md)

## Implementation inventory

- `src/c_api/transmog_availability.rs`: supplied input, named error, profile-gated query and the boolean-validated `A_Admin.SetTransmogAvailability(enabled)` host input ([admin API](../admin-api/collections.md)).
- `src/c_api/c_transmog_outfit_info/pending.rs`: pending-record and current apply guards.
- `src/c_api/registration.rs`: shared registration, without ungating the outfit module.
- `src/lua_api/state/sim_state.rs` and `state.rs`: per-environment input without guessed true.
- `tests/retail_transmog_enabled.rs`: selected original interactions and direct query contract.
- `tests/mists_transmog_enabled.rs`: preserved original Mists control branches.
- `tests/forever_camelot_spellbook.rs`: selected original Forever constructor, query, refresh and sparse catalog consumers.

## Tests asserting this spec

`tests/retail_transmog_enabled.rs`: eight exact producer-only RED/GREEN pairs across Retail/PTR. `tests/mists_transmog_enabled.rs`: two original display-branch preservation passes. Seventeen targeted legacy transmog cases also pass with explicit input. Forever adds three exact original-consumer RED/GREEN pairs under its [bounded contract](forever-camelot-spellbook.md). [Execution scope and limits](../wiki/systems/transmog-availability.md#proof-status).

## Known gaps (current cycle)

None within the approved supplied-state slice. All seven full-feature all-target checks pass with zero JSON/plain compiler warnings; eight RED/GREEN pairs, seventeen legacy cases, two Mists cases and both paired stock maps are verified.

## Out of scope

Native entitlement/default policy, automatic supply, account-wide semantics, persistence, inferred events, new admin methods, publication beyond these four profiles and cached-slot invalidation redesign. Retail 70448 has no original direct query caller; PTR IsAvailable:512 exists in a selected file but its only original constructor is excluded Camelot code. Retail/PTR interaction tests do not establish a live spellbook query flow or complete wardrobe acceptance. Forever has a separately bounded [original spellbook proof](forever-camelot-spellbook.md), not complete item browsing.
