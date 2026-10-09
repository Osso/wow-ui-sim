# Patch 4.3.0 publication sweep

Audit the pinned historical retail pageid 167555, revision 1639407. [Audit](../wiki/investigations/patch-4-3-0-api-audit.md).

## What it must do

- [x] Account every inventory identity with existing generator; retain non-inventory context separately.
- [x] Probe current cached retail publication after actual retail successors, excluding Classic. Queued 4.3.4 placeholder is coordinator-owned.
- [x] Require exact reviewed gap IDs and derive counts from inputs/results, not frozen receipts.
- [x] Distinguish publication from behavior, signature, output and native parity.

## How it works

[Audit](../wiki/investigations/patch-4-3-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.3.0-*` — pinned source accounting.
- `tests/patch_4_3_0_publication_sweep.rs` — own retail prefork discovery.
- `tests/data/patch_4_3_0_sweep_known_gaps.json` — reviewed mismatches.

## Tests asserting this spec

Own prefork filter `patch_4_3_0` GREEN and scratch missing-publication negative; `tools/test_patch_4_3_0_accounting.py` source/accounting/seal fixtures.

## Known gaps (current cycle)

- [ ] Retain 25 individually reasoned publication gaps; no proved cheap backing-model closure.
- [ ] Coordinator replaces queued 4.3.4 placeholder and owns integrated final acceptance.

## Out of scope

Linked API-page reconstruction, speculative models, shims, vendor changes, native parity without proof, broad/final gates, push/merge/deploy.
