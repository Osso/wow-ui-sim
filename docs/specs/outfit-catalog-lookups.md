# Outfit catalog lookups

`C_TransmogOutfitInfo` queries one empty-default per-environment outfit catalog. The cached retail `Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua` lines 294–373 and 886–898 define the four signatures and seven entry fields. Name lookup explicitly promises case-insensitivity; duplicate-name selection is unspecified. No native probe has established the remaining policies below.

## What it must do

- [x] `GetOutfitInfo(outfitID)`, `GetOutfitInfoByName(name)`, `GetOutfitInfoByPlayerFacingIndex(playerFacingOutfitIndex)` and `GetOutfitsInfo()` query the same catalog under cumulative `retail-12-0-5`.
- [x] Start empty, without fabricated records. Singular misses return zero values; enumeration returns an empty table (inferred empty-list policy).
- [x] Return `outfitID`, `name`, `situationCategories`, `icon`, `isEventOutfit`, `isDisabled`, and `playerFacingOutfitIndex` with documented types. Each call owns fresh tables, including nested categories.
- [x] Resolve mixed-case names and explicit player-facing indices independently of outfit IDs and list offsets.
- [x] Singular lookups use the existing VM secret-argument helper: untainted callers can resolve secret arguments; tainted callers cannot. Ordinary arguments remain allowed for tainted callers.

Focused default-retail GREEN: 5/5 tests, exit 0, on shared build revision `72220958b` containing implementation `9d89c7021`; `/tmp/patch-12.0.5-outfit-green-test.log` and `/tmp/patch-12.0.5-outfit-ledger.md` retain proof. These are simulator behavioral claims, not native-verified semantics.

## How it works

- [C API boundary and Lua environment](../lua-api.md)

## Implementation inventory

- `src/c_api/c_transmog_outfit_info.rs`: existing namespace registration and public catalog type exports; existing situations-setting behavior preserved.
- `src/c_api/c_transmog_outfit_info/catalog.rs`: catalog entries, queries, secret validation, fresh entry serialization.
- `src/c_api/mod.rs`: public module visibility for state fixtures.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: patch-gated empty catalog state.
- `src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs`: obsolete `GetOutfitInfo` nil fallback removed, not retained as an alternate path.

## Tests asserting this spec

`tests/patch_12_0_5_outfit_catalog.rs` is discovered by the existing grouped `integration` target. Fixtures seed only `env.state()` with IDs 91/305 and explicit indices 7/42. Tests cover empty/missing results, all fields, case-insensitive lookup, sparse indices, nonalias returns, invalid inputs, and secret caller validation.

## Known gaps (current cycle)

- [ ] Independent final verification; focused default-retail GREEN is complete. No separate profile build requested.
- [ ] Native Unicode matching: create names `Été`, `Straße`, `İ`, and decomposed `e\u0301`; query upper/lowercase and normalized equivalents, recording exact matches and locale.
- [ ] Native sparse/index semantics: create three outfits, delete the middle, reorder if supported, inspect all published indices, and compare lookup by those indices versus positions and IDs.
- [ ] Native invalid inputs: call each lookup with missing/nil, booleans, tables, numeric strings, fractions, zero, negative numbers, NaN and infinity; record errors and exact return counts.
- [ ] Native empty enumeration: query with no outfits and record exact return count/type, distinguishing no return from an empty table.

Inferred policies: Rust Unicode lowercase comparison without normalization; first catalog-order duplicate match; enumeration in stored order; exact numeric equality with published integer IDs/indices (fractional/nonfinite values miss); strict number/string types reject nil or coercible strings. None claims native equivalence beyond documented name case-insensitivity.

## Out of scope

Active/pending outfit lifecycle, catalog mutation APIs, fabricated runtime fixtures, persistence, events, UI behavior, full startup validation and non-default-profile verification. Existing lifecycle state remains untouched; catalog ownership does not reinterpret active or pending IDs.
