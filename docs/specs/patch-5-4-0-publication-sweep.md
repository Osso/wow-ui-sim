# Patch 5.4.0 publication sweep

Audit the pinned 2013 retail Mists API page and separately pinned automated diff. [Audit](../wiki/investigations/patch-5-4-0-api-audit.md) describes evidence and limits.

## What it must do

- [x] Probe every register occurrence under the retail prefork fixture and compare the exact retained gap set.
- [x] Apply later retail registers in chronological order; use merged 5.4.1, 5.4.2 and 5.4.7 registers before 5.4.8.
- [x] Reproduce pinned sources without changing older extracts/registers.
- [x] Preserve all removal consumer scans and account for every prose/enum occurrence without claiming publication proves semantics.

- [x] Observe current GetInstanceInfo return #9 changing from 17 to 22 independently of capacity 25 in prefork and standalone fixtures.
- [x] Observe current Frame:IsForbidden flag transitions in both fixtures without claiming historical security enforcement.

## How it works

- [Shared sweep](../../tests/common/publication_sweep.rs).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_5_4_0_publication_sweep.rs`: occurrence probes and later retail registers.
- `tests/patch_5_4_0_behavior.rs`: shared assertions executed by prefork markers and standalone test wrappers.
- `tests/data/patch_5_4_0_sweep_known_gaps.json`: exact known gap identities.
- `tools/gen_patch_wikitext_register.py`: opt-in reused Mists inventory parser and summary/diff flags.
- `tools/extract_patch_non_inventory.py`: opt-in source markup and master-owned Mists inventory stripping.

## Tests asserting this spec

- `tests/patch_5_4_0_publication_sweep.rs`, `tests/patch_5_4_0_behavior.rs`.
- `tools/test_patch_mists_register.py`, `tools/test_patch_mists_extract.py`.

## Known gaps (current cycle)

- [ ] 21 publication gaps and 27 substantive extract contracts retained with precise reasons in the audit ledger.
- [x] Integrated 5.4.1/5.4.2/5.4.7; 5.4.2 removes securerandom, replacing exactly one historical gap with current absence.

## Out of scope

Classic 5.5.x; inferred historical behavior; full integration suite; vendor changes; shims to manufacture publication.
