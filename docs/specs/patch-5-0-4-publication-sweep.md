# Retail Patch 5.0.4 publication sweep

## What it must do

- [x] Probe every one of the 626 pinned main/diff occurrences against current retail.
- [x] Apply real retail successors from 5.1.0 onward; exclude Classic 5.5.x.
- [x] Classify each occurrence, main prose statement, and retained signature without equating publication with behavior.
- [x] Preserve current consumers, later readditions, and Classic behavior.
- [x] Retain portable, revision-pinned evidence and exact negative control.

## Current boundary

626 current observations: 469 bounded publication/absence rows, 157 exact mismatches. Merged 5.1.0 pet-ID retirements resolve two historical gaps (159 → 157), without new retirements or gap silencing. All 73 extract rows and five separate signatures are classified; 704 total IDs. Mismatches include probe-category limitations, not just missing APIs. One new [pet-type state read](pet-battle-pet-type.md) has concrete RED/GREEN proof. No retirements; current consumers and four later readditions preserved. Source TOC is 50001, not 50004.

## Tests

- `tests/patch_5_0_4_publication_sweep.rs`
- `tests/patch_5_0_4_behavior.rs`
- [Occurrence ledger](../../data/patch-api/sources/5.0.4-page-coverage.json)
- [Audit](../wiki/investigations/patch-5-0-4-api-audit.md)

## Out of scope

Native 2012 gameplay parity, vendor edits, shims, full suite, push and merge. Coordinator owns final integration and CI.
