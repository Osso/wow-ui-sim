# TBC Classic 2.5.1 SOURCE accounting

Account for frozen page 71215/revision 701879 (`2022-02-06T21:22:09Z`), separate from retail and other Classic histories. [Audit](../wiki/investigations/patch-2-5-1-api-audit.md); [frozen input](../../data/patch-api/source-cache/legacy-2026-10-09/2.5.1-wikitext.txt).

## What it must do

- [x] Validate frozen identity, response/content hashes and full registry through 1.0.0 before source accounting.
- [x] Preserve every nonblank literal row, API occurrence, script, command, header and linked summary; retain unspecified signatures/state/security/native contracts as UNPROVEN.
- [x] Separate literal TOC 20501 and navigation from configured profile interfaces; Anniversary 11507 is not presumed TBC 205xx. Profile absence is not an unsupported-API diagnosis.
- [x] Keep 2.5.2–2.5.6 as pending same-history main-integration references only; no foreign-history supersession or empty-inventory parity credit.
- [x] Replay sealed original source/config/tool/ledger/log bytes in a fresh copied process without Git, target, current runtime or current mutable tools; reject serialized ledger/log tampering and restore exact bytes.

## How it works

- [Coverage and proof boundaries](../wiki/investigations/patch-2-5-1-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/2.5.1-*` — owned literal source, plaintext and contract ledger.
- `data/patch-api/evidence/2.5.1-session-2026-10-09/` — immutable historical inputs, local source validator, tests, seals and replay receipts.

## Tests asserting this spec

`data/patch-api/evidence/2.5.1-session-2026-10-09/test_source_accounting.py` asserts serialized source behavior and negative controls: 9/9 GREEN at `985fd2c95`. Owned `validate.py` reproduces historical accounting at `eb48b0fc6`; `replay_controls.py` passes fresh copied replay and both serialized seal rejection/restoration controls at `c1391db3f`. [Proof ledger](../../data/patch-api/evidence/2.5.1-session-2026-10-09/proof-ledger.json). These are SOURCE-only proofs, not native/runtime/final acceptance.

## Known gaps (current cycle)

- [ ] Linked diffs, deprecation file and community notes not fetched/expanded; members, migration semantics, signatures, state/security and native contracts remain UNPROVEN.
- [ ] Inventory publication, removed-member absence, state behavior, event/script payloads, CVar defaults and command effects not measured on native TBC.

## Out of scope

Runtime/shared classifier/profile changes, native probes, shims/fallbacks, vendor/cache/Wowless edits, linked-source reconstruction, foreign-history retirements, broad/check/lint/startup/full-suite/final gates and operational actions. Main owns integration and any future native/profile decision; this task provides SOURCE proof only.
