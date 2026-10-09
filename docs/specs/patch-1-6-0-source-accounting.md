# Patch 1.6.0 frozen source accounting

Bounded accounting of frozen page310843/revision2998524/timestamp2020-04-05T21:05:34Z from the retained legacy manifest. Revision timestamp is not release date. [Audit](../wiki/investigations/patch-1-6-0-api-audit.md) records methodology and proof epochs.

## What it must do

- [x] Verify exact frozen response/body and manifest-linked registry101 before deriving the ledger.
- [x] Account every literal row/link/prose/header/template/signature/default; reject omission, count mutation, invention and unsupported proof credit.
- [x] Retain redirect target unexpanded and missing contracts UNPROVEN; zero local declarations imply zero grounded model/native subset, not historical absence of changes.
- [x] Separate original Retail, Era and Forever histories; retain 1.7.0 active and 1.8.0 pending behind 1.9.0 as unapplied references.
- [x] Preserve own SOURCE RED/GREEN and own-base default bytes/error replay; copied portable three-test controls must reject serialized ledger/log tampering and restore exact bytes/map without resealing.
- [ ] Independently accept outer receipt metadata and retention; original seals/archive immutability and separate later actual revision/cwd/argv/timestamps/full-stream/scoped-hash receipts are recorded, environment keys only after credential-pattern inspection. Child does not run this main-owned gate.

## How it works

- [Literal audit and proof limits](../wiki/investigations/patch-1-6-0-api-audit.md)
- [Portable replay](../../data/patch-api/evidence/1.6.0-session-2026-10-09/REPLAY.md)

## Implementation inventory

- `data/patch-api/evidence/1.6.0-session-2026-10-09/audit.py`: isolated frozen-source derivation and replay.
- Evidence-local `historical-tools/`: unchanged own-base extractor/generator; shared tools unchanged.

## Tests asserting this spec

- Evidence-local `test_source_accounting.py`: literal identity, omissions/counts/inventions, source mutations, separate history and default bytes/errors.
- Evidence-local `test_portable.py`: copied historical replay, serialized ledger/log rejection and exact restoration.

## Known gaps (current cycle)

- [ ] Target revision/content, patch-specific delta, callable identities, signatures, arguments/returns/defaults, events, transitions, security and native equivalence remain UNPROVEN.

## Out of scope

Redirect expansion belongs to main's separate primary-target research. No invented API/state/alias/default/shim, runtime/shared-tool/vendor changes, builds, broad/check/final gates or operations. Main owns ordered integration and independent acceptance; empty local inventory cannot close parent meaningful-behavior goal.
