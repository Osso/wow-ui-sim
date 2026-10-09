# Patch 1.14.3 frozen Era accounting

Account exact legacy page480026/revision4615755/timestamp2023-07-10T10:27:33Z without expanding links. [Audit](../wiki/investigations/patch-1-14-3-api-audit.md).

## What it must do

- [ ] Verify manifest/response/body/101-page registry before derivation.
- [ ] Preserve all463 nonblank raw rows,412 inventory occurrences,188 unspecified signatures,14 headers,14 prose limits,27 references and six default-extracted rows.
- [ ] Preserve caption1.14.2/build42214 →1.14.3/build43639, source11403 versus configuredEra11507, qualified TBC2.5.4 comparison and eleven unapplied same-Era successors.
- [ ] Reject every omitted occurrence, invented behavior/signature/native credit and foreign supersession.
- [ ] Replay copied historical inputs without Git/target/current tools; reject serialized ledger/log tampering and restore exact original bytes. Preserve immutable original seals and separate current receipts.
- [ ] Bound any runtime measurement to standalone current Era bare factory/shared classifier and exact own inventory. Distinguish raw registration from generic fallback; publication is not model/native proof.

## How it works

- [Literal proof matrix](../wiki/investigations/patch-1-14-3-api-audit.md).

## Implementation inventory

`data/patch-api/evidence/1.14.3-session-2026-10-09/audit.py`: own occurrence accounting and portable historical replay; pinned tools/configuration/successors retained alongside it. Shared parser/runtime files unchanged.

## Tests asserting this spec

Own `test_source.py`: eight SOURCE fixtures, omission/credit/history controls. Own `test_portable.py`: three copied-source and serialized tamper fixtures. Initial RED retained; GREEN pending.

## Known gaps (current cycle)

- [ ] Source inventory gives identities, not callable signatures/event payloads. CVar effects lack concrete bounded simulator state contracts; no runtime implementation/defaults/aliases invented.
- [ ] Main owns successor integration, loaded-UI/native and final gates.

## Out of scope

Linked-page expansion, foreign-history supersession, runtime changes without grounded proposal to main, vendor/cache/Wowless edits, broad/check/lint/type/coverage/final gates, delegation/model CLI, push/merge/deploy.
