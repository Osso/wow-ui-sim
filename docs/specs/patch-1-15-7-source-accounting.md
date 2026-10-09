# Patch 1.15.7 literal SOURCE accounting

Account frozen page 626071 / revision 6778069 / timestamp `2026-07-22T05:34:20Z` offline. Preserve exact literal source, not expanded wiki/API/native behavior. [Audit](../wiki/investigations/patch-1-15-7-api-audit.md).

## What it must do

- [ ] Validate response/raw hashes, manifest and 101-page registry through 1.0.0; preserve all four nonblank source rows and occurrences.
- [ ] Derive TOC 11507 and navigation from literal text; leave unnamed client line UNPROVEN, without inference from numbers.
- [ ] Account Resources header, two unexpanded diff contracts and unexpanded navigation; invent no API/event/CVar/widget/command/signature/prose declarations.
- [ ] Separate configured Era/Anniversary 11507 from native measurement; retain same-Era 1.15.8/1.15.9 pending main integration only, with no applied or foreign supersession.
- [ ] Reject omitted rows/contracts/headers, fabricated credit, source/revision tampering; retain immutable logs/seals and copied Git/target/current-tool-free replay with disk ledger/log tamper rejection and exact restoration.

## How it works

- [Coverage matrix and proof boundary](../wiki/investigations/patch-1-15-7-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.15.7-session-2026-10-09/audit.py` — own literal adapter; no shared parser or runtime code.
- Same directory: exact source/response/manifest/registry/configuration, historical tools, queued successor inputs, ledger and SOURCE fixtures.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/1.15.7-session-2026-10-09/test_source_accounting.py`: eight own SOURCE-only fixtures; RED eight assertion failures against empty accounting retained. GREEN and portable receipts pending.

## Known gaps (current cycle)

- [ ] Literal page does not name its client; linked/template contents and precise API/signature/state/security/native contracts remain UNPROVEN.
- [ ] Main owns actual same-history successor integration, native observations and final acceptance.

## Out of scope

Runtime/default/profile/shim/alias/fallback edits, shared parser changes, foreign supersession, model/native measurements, canonical/vendor/cache/Wowless writes, network, broad/final/check/lint/type gates, delegation/model CLIs, push/merge/deploy. Source-only targeted development proof does not complete parent gates.
