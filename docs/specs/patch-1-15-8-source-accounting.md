# Patch 1.15.8 literal SOURCE accounting

Account frozen page 686952 / revision 6778071 / timestamp `2026-07-22T05:34:51Z` offline. Preserve exact literal source, not expanded wiki/API/native behavior. [Audit](../wiki/investigations/patch-1-15-8-api-audit.md).

## What it must do

- [x] Validate exact response/raw hashes, manifest and 101-page registry through 1.0.0; preserve all four nonblank rows and their occurrences.
- [x] Derive TOC 11508 and navigation from literal text; leave unnamed client line UNPROVEN rather than guessing from numbers.
- [x] Account one Resources header, two unexpanded diff contracts and one unexpanded navigation template; no invented API/event/CVar/widget/command/signature/prose contracts.
- [x] Separate configured Era/Anniversary 11507 from native measurement; retain 1.15.9 only as queued placeholder awaiting main integration, never foreign supersession.
- [x] Reject omitted rows/contracts/headers, fabricated credit and source/revision tampering.
- [ ] Preserve compact historical originals independently of later receipts; copied Git/target/current-tool-free replay and serialized ledger/log tamper rejection with exact restoration.

## How it works

- [Coverage matrix and proof boundary](../wiki/investigations/patch-1-15-8-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.15.8-session-2026-10-09/audit.py` — own literal derivation/validation, not shared parser or runtime code.
- Same directory: frozen source/response/manifest/registry/configuration/historical tools, queued successor inputs, ledger and SOURCE fixtures.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/1.15.8-session-2026-10-09/test_source_accounting.py`: eight SOURCE-only fixtures; own RED eight assertion failures retained. GREEN **8/8** at `384699f11`; default historical generator flags `[]` yields empty inventory (not parity). Original 23 seals retain 660,202 bytes, largest 219,034 bytes; copied historical controls pending.

## Known gaps (current cycle)

- [ ] Client name is absent from this literal page; template/link contents and precise API/signature/state/security/native contracts remain UNPROVEN.
- [ ] Main owns actual same-history successors, native observations, integration and final acceptance.

## Out of scope

Runtime/default/profile/shim/alias/fallback edits, shared parser/extractor changes, foreign supersession, model/native measurements, canonical/vendor/cache/Wowless writes, network, check/lint/readability/coverage/broad/startup/final gates, delegation/model CLI and operations. PLAN.md remains excluded under explicit plan skill policy.
