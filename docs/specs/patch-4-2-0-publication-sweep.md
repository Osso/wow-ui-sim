# Patch 4.2.0 publication accounting

Account for the pinned historical retail page without inventing missing signatures or behavior. [Audit](../wiki/investigations/patch-4-2-0-api-audit.md).

## What it must do

- [ ] Reproduce all 65 inventory IDs and both source counts using recorded existing flags.
- [ ] Account for the sole navigation metadata row and zero prose/signatures.
- [ ] Probe every inventory occurrence; require the exact retained gap set.
- [ ] Apply actual later retail registers, with explicit pending 4.3.0/4.3.4 placeholders; exclude Classic histories.

## How it works

[Source boundary and publication scope](../wiki/investigations/patch-4-2-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.2.0-*` — pinned inventory, provenance, extract and per-ID ledger.
- `tests/patch_4_2_0_publication_sweep.rs` — own retail prefork case.
- `tests/data/patch_4_2_0_sweep_known_gaps.json` — exact development-observed mismatch set.
- `data/patch-api/evidence/4.2.0-session-2026-10-09/` — compact source and development receipts.

## Tests asserting this spec

Own prefork filter `patch_4_2_0_publication_sweep`; source accounting development fixture.

## Known gaps (current cycle)

- [ ] Pending runtime publication observations and 4.3.0/4.3.4 successor reconciliation.

## Out of scope

Native behavior parity, invented linked-page signatures, Classic changes, new shims/fallbacks, vendor changes, broad/final acceptance, push/merge/deploy/delegation.
