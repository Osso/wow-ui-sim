# Patch 10.1.0 publication sweep

Account for page 230704, revision 2236681 (2023-06-15T22:35:50Z) against current retail 12.1.0.

## Requirements

- [x] Retain all 129 inventory occurrences and 287 non-inventory occurrences, including the level-two Structures tail and verbatim Lua/XML examples.
- [x] Apply all twenty-one chronological later registers, 10.1.5 through 12.1.0; require exact reviewed gaps.
- [x] Retire ten unused namespace autostubs without modifying classic registrations or cached/vendor Lua. Keep successors published.
- [x] Account for every source ID in patch-page-coverage/v1, distinguishing publication/absence/event registration from signatures, outputs, security, historical and native parity.
- [x] Preserve all existing registers and capture bytes; keep prior extractor reproduction results unchanged under both modes.

## Tests and proof

`tests/patch_10_1_0_publication_sweep.rs`, `tests/patch_10_1_0_publication_fixes.rs`, `tests/patch_10_1_0_cached_surfaces.rs`, existing loot-history successor shape and extractor/parser fixtures. Twenty-two isolated sweeps, exact one-row negative control, raw/repeated retirement RED/GREEN, cached Game prefork, reproduction, formatting, default/Mists checks and startup `[]` pass. No full suite or native acceptance claim.

## Out of scope

Historical epochs, speculative model producers, placeholders, 3D behavior, vendor/cache edits, linked-page expansion, full suites, push, merge and agents/models.

## Evidence

[Page audit](../wiki/investigations/patch-10-1-0-api-audit.md) owns per-ID boundaries and complete sweep table. [Ledger](../../data/patch-api/sources/10.1.0-page-coverage.json): 416 IDs, 36 publication gaps, 238 pending substantive extracts and 49 metadata occurrences.
