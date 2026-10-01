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

- [x] Independent315 bounded stored-index/DTO verification accepted; current-VM five tests PASS, applicable independent310 fmt/check reused, no readability issues. Exact scope below; no separate profile build requested.
- [ ] Native Unicode matching: create names `Été`, `Straße`, `İ`, and decomposed `e\u0301`; query upper/lowercase and normalized equivalents, recording exact matches and locale.
- [ ] Native sparse/index semantics: create three outfits, delete the middle, reorder if supported, inspect all published indices, and compare lookup by those indices versus positions and IDs.
- [ ] Native invalid inputs: call each lookup with missing/nil, booleans, tables, numeric strings, fractions, zero, negative numbers, NaN and infinity; record errors and exact return counts.
- [ ] Native empty enumeration: query with no outfits and record exact return count/type, distinguishing no return from an empty table.

Inferred policies: Rust Unicode lowercase comparison without normalization; first catalog-order duplicate match; enumeration in stored order; exact numeric equality with published integer IDs/indices (fractional/nonfinite values miss); strict number/string types reject nil or coercible strings. None claims native equivalence beyond documented name case-insensitivity.

## Independent stored-index/DTO acceptance — 2026-10-01

Parent read and accepts `/tmp/patch-12.0.5-outfit-index-independent-proof.md`. Current pinned VM run: **5/5 PASS**, exit0; `/tmp/patch-12.0.5-outfit-index-independent-test-result.json` and `-test.log` bind integration SHA256 `398ef0d1f5a49f8e9415021f833e37de99cba63a2a2b05819898db7b61bbb5e9` to batch37 saved build. Applicable independent310 default fmt/check0 at `6755f0e6c` reused for unchanged catalog source; no readability issues. No new runtime runs/builds for accounting.

| Accepted slice | Proof limit |
|---|---|
| Stored index7 → ID91, index42 → ID305; index91/index1 and ID7 miss | Host-seeded records, not runtime catalog population or native index lifecycle |
| Seven-field first-record DTO through ID/name/index/enumeration; fresh nested results | Concrete first-record assertions and mutation fixtures, not exhaustive pairwise field coverage |
| Second record: index42 result fields, ID305 publishes index42, name/enumeration identify ID305 | Index42 result does not separately assert its published index; name/enumeration second-record assertions are ID-only. Remaining full schema is shared-builder source inspection, not exhaustive per-path runtime assertions |
| Invalid inputs and current-VM secret caller fixtures | Inferred policies, not exhaustive numeric/secret/native behavior |

Only `structures-TransmogOutfitEntryInfo-673` promotes **audit-pending → bounded-coverage**, capability `outfit-stored-index-dto`: **258/90/14 → 257 pending / 91 bounded / 14 partial = 362**. All IDs, unrelated rows and original capabilities preserved. No native, catalog population/allocation/create/delete/reorder/persistence/events/UI closure, all-profile or full-page parity. No test expansion required for this bounded acceptance.

`/tmp/patch-12.0.5-outfit-index-accounting-before-after.json` stores exact before/after row, totals and identity comparisons. Register/source SHA256 `eaea58ae8adf215587cea6de12349b3586fcb2520a4c8aefd4d7cee5406046ed`; plaintext SHA256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`. Local ledger and ignored PLAN updated, never staged. Earlier aura acceptance's unaccepted315 checkpoint is historical and superseded here.

## Out of scope

Active/pending outfit lifecycle, catalog mutation APIs, fabricated runtime fixtures, persistence, events, UI behavior, full startup validation and non-default-profile verification. Existing lifecycle state remains untouched; catalog ownership does not reinterpret active or pending IDs.
