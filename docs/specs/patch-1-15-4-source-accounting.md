# Patch 1.15.4 literal SOURCE accounting

Account frozen Warcraft Wiki page600355/revision6172581/timestamp `2024-11-13T21:59:45Z` offline. Preserve literal scope without importing linked retail contracts. [Audit](../wiki/investigations/patch-1-15-4-api-audit.md).

## What it must do

- [ ] Validate exact response/raw identity and hashes against frozen legacy manifest and 101-page registry ending1.0.0.
- [ ] Preserve six nonblank rows, two headers, TOC11504 and unexpanded navigation.
- [ ] Account the unspecified War Within subset summary, two linked retail pages and two unexpanded diffs; invent no APIs, signatures, defaults, aliases or subset membership.
- [ ] Separate configured Era/Anniversary11507 and same-Era successor inputs: queued1.15.5/1.15.6, canonical-integrated1.15.7/1.15.8/1.15.9 not applied here.
- [ ] Reject omissions/fabricated credit/foreign-history supersession and serialized tampering; preserve original seals/logs and keep current replay receipts separate.

## How it works

- [Literal coverage matrix](../wiki/investigations/patch-1-15-4-api-audit.md#literal-coverage-matrix).

## Implementation inventory

`data/patch-api/evidence/1.15.4-session-2026-10-09/`: own adapter, exact frozen source/configuration/successors, SOURCE and copied portable fixtures. Shared tools unchanged.

## Tests asserting this spec

Own `test_source_accounting.py`: retained RED nine assertion failures against empty-accounting scaffold. Own `test_portable.py`: RED three assertion failures for absent archive. GREEN pending implementation commit; no final gates.

## Known gaps (current cycle)

- [ ] Subset membership, linked contracts, API/state/security/signatures/native correspondence UNPROVEN; no literal client name.
- [ ] Main owns actual successor integration, native and final gates.

## Out of scope

Runtime/model/shim/default/alias edits; linked-page/transclusion expansion; vendor/cache/Wowless/shared-tool changes; foreign-history supersession; network; broad/check/lint/type/readability/coverage/acceptance gates; delegation/model CLIs; push/merge/deploy. No local literal contract identifies a modeled behavior to implement.
