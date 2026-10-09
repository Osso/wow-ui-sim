# Patch 1.3.0 frozen SOURCE accounting

Bounded original historical Retail page350208/revision3376287/timestamp2021-04-23T02:34:08Z accounting. Frozen source lives in `data/patch-api/evidence/1.3.0-session-2026-10-09/`; [audit](../wiki/investigations/patch-1-3-0-api-audit.md) records implementation and proof epochs. Revision timestamp is not release date.

## What it must do

- [x] Verify exact frozen body/response/manifest-linked registry101 before derivation; preserve every physical row, API declaration, signature spelling, prose, header, link and template parameter.
- [x] Reject every populated-boundary omission, derived-count mutation, invented alias/default/return/expansion/native credit and source/identity mutation.
- [x] Keep all missing contracts precisely UNPROVEN; retain old/new TabardModel rename identities and no inferred current aliases/defaults/retirements.
- [ ] Replay unchanged own-base generator/extractor default bytes and errors from a fresh copied immutable archive without Git/target/current tools/network.
- [ ] Reject both serialized ledger/log mutations, restore exact bytes/hashes/original seal map without resealing; separate later actual execution receipts from originals.

## How it works

- [Literal SOURCE audit](../wiki/investigations/patch-1-3-0-api-audit.md)

## Implementation inventory

- Evidence `audit.py`: exact source validation, literal accounting and historical default replay.
- Evidence `capture_defaults.py`: one-time own-base default-byte/error capture.
- Evidence `historical-tools/`: unchanged own-base source tools, not shared-tool edits.

## Tests asserting this spec

- Evidence `test_source_accounting.py`: eight own targeted SOURCE tests.
- Evidence `test_portable.py`: three copied replay/serialized tamper/restoration tests.

## Known gaps (current cycle)

- [ ] Historical returns/defaults/types/validation/lifecycle/security/native traces absent. Candidate identities alone do not prove meaningful behavior.

## Out of scope

Runtime/model/native implementation or acceptance, current retirements, link/template expansion, broad/check/final gates and operational changes. Main owns separate target research, ordered integration and independent acceptance. Newer1.4.0 active `p140-page` then1.5.0 `p150-page` queued separately/unapplied; Retail/Era/Forever histories remain distinct.
