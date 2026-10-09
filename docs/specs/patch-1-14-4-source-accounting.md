# Patch 1.14.4 frozen SOURCE accounting

Account frozen Warcraft Wiki page267043/revision2581777/timestamp `2023-07-19T19:32:56Z` offline. [Audit](../wiki/investigations/patch-1-14-4-api-audit.md).

## What it must do

- [x] Validate manifest/registry/response/returned raw identity before derivation.
- [x] Preserve all literal rows, headers, navigation, TOC, summary/prose and link occurrences; explicitly account for local inventory/signature absence.
- [x] Preserve the all-API Wrath3.4.0/1/2 inclusion claim separately from the unspecified Dragonflight10.0.0 through10.1.5 subset; expand neither and invent no state contracts.
- [x] Separate configured Era/Anniversary11507 from source11404/native equivalence and same-Era1.15.0 in-flight,1.15.1/2 queued,1.15.3..9 integrated-canonical inputs not applied here.
- [x] Retain own SOURCE/portable RED/GREEN, reject serialized ledger/log tampering, restore exact bytes, preserve original seals and separate later receipts.

## How it works

Own evidence adapter and copied frozen/configured/historical inputs under `data/patch-api/evidence/1.14.4-session-2026-10-09/`; shared tools unchanged.

## Tests asserting this spec

Own `test_source_accounting.py` and `test_portable.py`; targeted only. SOURCE RED11 failures and portable RED3 failures retained; SOURCE GREEN11/11 at `183437dc5`; default generator flags[] produces empty inventory, not linked API absence. Original67 seals/732649 bytes unchanged;68-member/114575-byte archive. Portable GREEN3/3 at `fb6e6ba53`: copied SOURCE11/11, validator/default-byte replay, serialized ledger/log rejection and exact restoration; four archive/current receipt seals separate. [Exact receipts](../../data/patch-api/evidence/1.14.4-session-2026-10-09/portable-proof.json).

## Known gaps

- [ ] Linked Wrath contracts, Dragonflight subset membership, signatures/state/security/native correspondence remain UNPROVEN. No concrete local API/state contract grounds a meaningful model change.
- [ ] Main owns actual successor application, native and final gates.

## Out of scope

Runtime/shared-tool/vendor/cache/Wowless changes, linked/transclusion expansion, foreign supersession, invented APIs/defaults/aliases, broad/check/lint/type/coverage/acceptance gates, delegation/model CLIs, push/merge/deploy.
