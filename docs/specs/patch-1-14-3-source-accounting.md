# Patch 1.14.3 frozen Era accounting

Account exact legacy page480026/revision4615755/timestamp2023-07-10T10:27:33Z without expanding links. [Audit](../wiki/investigations/patch-1-14-3-api-audit.md).

## What it must do

- [x] Verify manifest/response/body/101-page registry before derivation.
- [x] Preserve all463 nonblank raw rows,412 inventory occurrences,188 unspecified signatures,14 headers,14 prose limits,27 references and six default-extracted rows.
- [x] Preserve caption1.14.2/build42214 →1.14.3/build43639, source11403 versus configuredEra11507, qualified TBC2.5.4 comparison and eleven unapplied same-Era successors.
- [x] Reject every omitted occurrence, invented behavior/signature/native credit and foreign supersession.
- [x] Replay copied historical inputs without Git/target/current tools; reject serialized ledger/log tampering and restore exact original bytes. Preserve immutable original seals and separate current receipts.
- [x] Bound any runtime measurement to standalone current Era bare factory/shared classifier and exact own inventory. Distinguish raw registration from generic fallback; publication is not model/native proof.

## How it works

- [Literal proof matrix](../wiki/investigations/patch-1-14-3-api-audit.md).

## Implementation inventory

`data/patch-api/evidence/1.14.3-session-2026-10-09/audit.py`: own occurrence accounting and portable historical replay; pinned tools/configuration/successors retained alongside it. `patch-tests/patch_1_14_3_factory.rs` and explicit Era Cargo target measure exact current inventory/shared classifier. Current observations and reviewed126-ID mismatch fixture are separate from immutable SOURCE. Shared parser/runtime files unchanged.

## Tests asserting this spec

Own `test_source.py`: RED8 retained; SOURCE GREEN8/8 at1ce4359e9,1,135 omission controls. Own `test_portable.py`: RED3 retained; GREEN3/3 at7c2a251e8, copied SOURCE8/default-byte replay and both serialized tamper reject/exact restore controls.54 original seals unchanged. Factory GREEN2/2 at3b0989667;126-ID strict mismatch fixture RED at1f82e93a5 against empty list, reviewed fixture GREEN1/1 ata6a10d654, unchanged generic-fallback negative control retains GREEN1/1 at3b0989667. All412 observations match across initial/RED/GREEN receipts. Main explicitly authorized ordinary offline/locked Cargo bookkeeping; optional CoW failure is not a blocker. No Blizzard/CASC/vendor cache access.18 imported shared-test-helper dead-code warnings and seven inherited simulator/six vendor-manifest warnings retained, not suppressed.

## Known gaps (current cycle)

- [ ] Source inventory gives identities, not callable signatures/event payloads. CVar effects lack concrete bounded simulator state contracts; no runtime implementation/defaults/aliases invented.
- [ ] Main owns successor integration, loaded-UI/native, warning cleanup if in scope, and final gates.126 strict current publication mismatches and all substantive model/native contracts remain unproven; not126 established absent native APIs.

## Out of scope

Linked-page expansion, foreign-history supersession, runtime changes without grounded proposal to main, vendor/cache/Wowless edits, broad/check/lint/type/coverage/final gates, delegation/model CLI, push/merge/deploy.
