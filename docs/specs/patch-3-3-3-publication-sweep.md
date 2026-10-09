# Historical retail Patch 3.3.3 source accounting

Account for the frozen 2010 retail [page](../../data/patch-api/sources/3.3.3-api-changes.wikitext), page 261776 / revision 2531935. Publication, signatures, modeled behavior and native parity are separate. [Audit](../wiki/investigations/patch-3-3-3-api-audit.md).

## What it must do

- [ ] Opt-in heading parsing retains all 36 named occurrences and original lines; without the flag previous output bytes remain unchanged.
- [ ] Full source, rendered extract, signatures, prose and editorial headers receive explicit dispositions.
- [ ] Only actual later retail registers supersede, with 3.3.5 then 4.0.1 placeholders preceding actual 4.1.0 onward; never Wrath Classic 3.4.x or other Classic histories.
- [ ] Portable historical replay derives counts and validates sealed source/log receipts without Git, target or mutable current accounting inputs.

## How it works

- [Audit](../wiki/investigations/patch-3-3-3-api-audit.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in `--historical-api-headings`.
- `data/patch-api/sources/3.3.3-*`: frozen input, provenance and publication register.

## Tests asserting this spec

- `tools/test_patch_3_3_3_source.py`: full pinned page and unrelated-section exclusion.

## Known gaps (current cycle)

- [ ] Historical signatures and producers require established contracts and behavior evidence; no native 2010 receipt supplied.
- [ ] Runtime publication development test not yet executed.

## Out of scope

Classic supersession, guessed models, shims, fallbacks, vendor/cache changes, broad gates, deploy/push/merge and coordinator integration.
