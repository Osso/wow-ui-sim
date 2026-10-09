# Patch 1.15.5 literal SOURCE accounting

Account frozen Warcraft Wiki page 610284 / revision 6235411 / timestamp `2025-02-09T17:32:34Z` offline. Preserve literal source without expanding navigation or links. [Audit](../wiki/investigations/patch-1-15-5-api-audit.md).

## What it must do

- [x] Validate exact response/raw identity and hashes against the frozen legacy manifest and 101-page registry ending 1.0.0.
- [x] Preserve four nonblank rows, Resources header, TOC11505 and navigation; infer no client name.
- [x] Account two unexpanded diff links; invent no APIs, signatures, prose contracts, defaults or aliases.
- [x] Separate configured Era/Anniversary11507 and same-Era successor inputs: queued1.15.6/1.15.7, canonical-integrated1.15.8/1.15.9 not applied here.
- [x] Reject omissions, fabricated coverage, foreign-history credit and tampering; keep original seals/logs immutable and later replay receipts separate.

## How it works

- [Literal coverage matrix](../wiki/investigations/patch-1-15-5-api-audit.md#literal-coverage-matrix).

## Implementation inventory

`data/patch-api/evidence/1.15.5-session-2026-10-09/`: own adapter, exact frozen sources/configuration/successors, fixtures and historical replay. Shared tooling unchanged.

## Tests asserting this spec

Own `test_source_accounting.py`: retained RED eight assertion failures; SOURCE GREEN8/8 at `3ea25422f`. `test_portable.py`: GREEN3/3 at `38fa9fba4`, fresh copied SOURCE8/8, default byte replay, serialized ledger/log seal rejection and exact restoration. Original38 seals unchanged; later receipts separate. [Exact commands and results](../../data/patch-api/evidence/1.15.5-session-2026-10-09/portable-proof.json). No final gates.

## Known gaps (current cycle)

- [ ] Client name absent; linked/transcluded content, API/state/security/signatures/native correspondence UNPROVEN.
- [ ] Main owns actual successor integration, native and final gates.

## Out of scope

Runtime/model/shim/default/alias edits; transclusion expansion; vendor/cache/Wowless modification; foreign-history supersession; network; broad/check/lint/type/readability/coverage/acceptance gates; delegation/model CLIs; push/merge/deploy. Literal metadata and links specify no concrete modeled behavior.
