# Patch 4.3.0 publication sweep

Audit the pinned historical retail pageid 167555, revision 1639407. [Audit](../wiki/investigations/patch-4-3-0-api-audit.md).

## What it must do

- [ ] Account every inventory identity with existing generator; retain non-inventory context separately.
- [ ] Probe current cached retail publication after actual retail successors, excluding Classic. Queued 4.3.4 placeholder is coordinator-owned.
- [ ] Require exact reviewed gap IDs and derive counts from inputs/results, not frozen receipts.
- [ ] Distinguish publication from behavior, signature, output and native parity.

## How it works

[Audit](../wiki/investigations/patch-4-3-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/4.3.0-*` — pinned source accounting.
- `tests/patch_4_3_0_publication_sweep.rs` — own retail prefork discovery.
- `tests/data/patch_4_3_0_sweep_known_gaps.json` — reviewed mismatches.

## Tests asserting this spec

Own prefork filter `patch_4_3_0`; targeted source-accounting tests.

## Known gaps (current cycle)

- [ ] Discovery and coordinator acceptance pending.

## Out of scope

Linked API-page reconstruction, speculative models, shims, vendor changes, native parity without proof, broad/final gates, push/merge/deploy.
