# Historical retail Patch 3.3.3 source accounting

Account for the frozen 2010 retail [page](../../data/patch-api/sources/3.3.3-api-changes.wikitext), page 261776 / revision 2531935. Publication, signatures, modeled behavior and native parity are separate. [Audit](../wiki/investigations/patch-3-3-3-api-audit.md).

## What it must do

- [x] Opt-in heading parsing retains all 36 named occurrences and original lines; without the flag previous output bytes remain unchanged.
- [x] Full source, rendered extract, signatures, prose and editorial headers receive explicit dispositions.
- [x] Only actual later retail registers supersede, with actual 3.3.5 then 4.0.1 preceding 4.1.0 onward; both added registers have no section/symbol overlap with the 36 own occurrences; never Wrath Classic 3.4.x or other Classic histories.
- [x] Existing pet scalar has one return and tracks two explicit seeded values after full cached-retail UI loading; no new model/native credit.
- [x] Portable historical replay derives counts and validates sealed source/log receipts without Git, target or mutable current accounting inputs.

## How it works

- [Audit](../wiki/investigations/patch-3-3-3-api-audit.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in `--historical-api-headings`.
- `data/patch-api/sources/3.3.3-*`: frozen input, provenance, register, signatures and complete ledger.
- `data/patch-api/evidence/3.3.3-session-2026-10-09/validate.py`: self-contained frozen historical evidence validator; not a current-head/native gate.

## Tests asserting this spec

- `tools/test_patch_3_3_3_source.py`: full pinned page and unrelated-section exclusion, GREEN 2/2.
- `tests/patch_3_3_3_publication_sweep.rs`: RED 11 exact gaps; reviewed GREEN 1/1; fabricated-global negative 11 → 12 rejected.
- `tests/patch_3_3_3_behavior.rs`: existing pet scalar at full cached-UI load boundary, GREEN 1/1.
- `tools/test_patch_3_3_3_validator.py`: detached fresh-process replay, synthetic later drift, eight serialized tamper/restoration controls and missing-gap rejection; RED then GREEN 1/1 at `e8905b8f3`.

## Known gaps (current cycle)

- [ ] Historical signatures and producers require established contracts and behavior evidence; no native 2010 receipt supplied.
- [ ] Eleven current publication mismatches, three changed-prose contracts and 35 signature rows remain pending; existing scalar read does not establish historical native/event behavior.

## Out of scope

Classic supersession, guessed models, shims, fallbacks, vendor/cache changes, broad gates, deploy/push/merge and coordinator integration.
