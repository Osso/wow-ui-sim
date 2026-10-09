# Patch 1.13.7 literal SOURCE accounting

Account frozen page46829/revision458410/timestamp `2021-09-04T08:55:38Z` offline. [Audit](../wiki/investigations/patch-1-13-7-api-audit.md).

## What it must do

- [x] Validate exact manifest/registry/response/body identity, hashes and 1,179 source bytes; registry101 ends at1.0.0.
- [x] Preserve all25 nonblank raw rows, five default-extract rows, five inventory occurrences, one unspecified callable signature and four unspecified CVar defaults/descriptions.
- [x] Retain three sections/two nonnumeric Added headers, two unexpanded diff links and six unexpanded templates; TOC11307 and navigation1.13.6→1.13.7→1.14.0 remain literal.
- [x] Separate configured Era/Anniversary11507 from historical11307. Freeze unapplied same-history successors:1.14.0/1 completed-queued,1.14.2–1.15.9 integrated-not-applied.
- [x] Reject omissions, invented signatures/defaults/client names, foreign supersession and fabricated model/native credit.
- [ ] Preserve immutable originals with independent copied no-Git/no-target replay; reject serialized ledger/log tampering and restore exact bytes.

## How it works

[Coverage and proof boundaries](../wiki/investigations/patch-1-13-7-api-audit.md).

## Implementation inventory

`data/patch-api/evidence/1.13.7-session-2026-10-09/`: own `audit.py`, serialized ledger, copied historical default tools, frozen source/manifest/registry/configuration and same-Era successor inputs. No shared/runtime changes.

## Tests asserting this spec

Own `test_source_accounting.py`: eight SOURCE fixtures, omission and fabrication controls. Original RED eight assertion failures retained; current GREEN pending. Portable proof pending. Main owns integration/native/final gates.

## Known gaps (current cycle)

- [ ] GetDefaultScale signature/output/scale state/security unspecified. Existing return1 temporary shim does not establish a display model.
- [ ] Four CVar names have no source defaults/descriptions/state transition contracts. Availability is not modeled behavior.
- [ ] Two diff targets and API/navigation templates unexpanded; no linked expansion or native proof.

## Out of scope

Production/runtime/shared-parser changes, invented defaults/arguments/aliases/stubs, TBC/retail supersession, network/cache/vendor/CASC/Wowless edits, delegation/model CLI, broad/check/lint/type/coverage/final gates and push/merge/deploy. PLAN remains untracked.
