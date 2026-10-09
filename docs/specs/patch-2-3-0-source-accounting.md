# Historical retail Patch 2.3.0 SOURCE accounting

Account only frozen page 44655/revision 6055877, timestamp 2024-06-04T05:03:50Z. [Literal source](../../data/patch-api/sources/2.3.0-api-changes.wikitext), [audit and proof boundaries](../wiki/investigations/patch-2-3-0-api-audit.md).

## What it must do

- [x] Validate response/content identity and hashes against the frozen manifest and 101-page registry through 1.0.0; preserve cache bytes.
- [x] Retain every nonblank source row, heading, explicit labeled occurrence, rename endpoint, typo, NOTE, consolidated identity and literal call/command syntax without expanding links or inferring additions from consolidated membership.
- [x] Retain precise prose/signature limits, duplicates and unknowns; reject omitted rows, invented runtime/native/model credit and foreign or unapplied successor registers.
- [x] Leave shared generator/extractor/default bytes unchanged; use page-owned source accounting and the retained canonical-navigation text extractor.
- [ ] Replay exact serialized register/ledger/text and sealed logs in a relocated copied root without Git/target; reject serialized ledger/log tampering and restore original bytes.

## How it works

- [Source accounting and immutable evidence](../wiki/investigations/patch-2-3-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/2.3.0-*`: complete raw/text mirror, occurrence register and literal SOURCE ledger.
- `data/patch-api/evidence/2.3.0-session-2026-10-09/accounting.py`: page-owned source parser/builder and frozen provenance validation.
- Owned `test_source_accounting.py`, `validate.py`: source omission/credit fixtures and byte-exact historical replay.
- Owned frozen manifest/registry/response/pin and historical generator/extractor: immutable inputs, not runtime observations.

## Tests asserting this spec

- Owned `test_source_accounting.py`: targeted RED/GREEN for mixed labels, return prefixes, optional arguments, full raw/header/prose/link accounting, omission and invented-credit controls; references are not publication changes.
- Owned `validate.py`: frozen manifest/registry/identity/hash and serialized-byte assertions; replay receipt pending.

## Known gaps (current cycle)

- [ ] Every historical API/event/widget/command/prose contract remains runtime/native/model UNPROVEN; SOURCE completeness never proves loaded client parity.
- [ ] Main owns actual pinned retail 2.4.0/2.4.2 and 3.0.2/3.0.3/3.0.8 successor integration and native/final gates; no register is applied here.

## Out of scope

Network, linked-page/transclusion reconstruction, Classic 2.5/Wrath 3.4/Era supersession, runtime/model edits or proposals inferred from mere presence, retirements, caches/vendor/Wowless/canonical writes, rebase/push/merge/deploy/delegation/model CLIs, broad checks/lint/type/readability/coverage/startup/final gates. No runtime change is authorized without a separate proposal and explicit backing-state contract.
