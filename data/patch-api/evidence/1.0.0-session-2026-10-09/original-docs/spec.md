# Patch 1.0.0 frozen source accounting

Bounded literal accounting of frozen page67688/revision5580410/timestamp2023-10-16T00:50:33Z; revision timestamp is not release date. [Audit](../wiki/investigations/patch-1-0-0-api-audit.md) records methodology and limits.

## What it must do

- [x] Verify exact body/response and manifest-linked registry101 before deriving occurrences.
- [x] Account all literal rows, bare names, unspecified signatures, prose/comment, headers, links and navigation without fixing spelling or inventing aliases/defaults/model credit; reject omissions/count mutations/inventions.
- [x] Preserve frozen task-start queue unapplied and separate original Retail/Era/Forever histories; endpoint does not close parent goal.
- [x] Replay own-base default tool bytes/errors independently of literal ledger; retain own SOURCE RED/GREEN.
- [ ] Copy immutable original sealed evidence without Git/target/current tools; portable3 controls must reject serialized ledger/log tampering and restore exact bytes/hashes/map without resealing.
- [ ] Retain original seals/archive independently from later actual revision/cwd/argv/times/full-stream/scoped-hash receipts; environment key names only. Independent metadata acceptance belongs to main.

## How it works

- [Literal audit and proof epochs](../wiki/investigations/patch-1-0-0-api-audit.md)
- [Replay instructions](../../data/patch-api/evidence/1.0.0-session-2026-10-09/REPLAY.md)

## Implementation inventory

- `data/patch-api/evidence/1.0.0-session-2026-10-09/audit.py`: isolated accounting and historical replay.
- Evidence-local `historical-tools/`: unchanged own-base default tool copies; shared tools untouched.

## Tests asserting this spec

- Evidence-local `test_source_accounting.py`: exact identity/occurrences, unspecified contracts, all omissions/counts/inventions, separate histories and default bytes/errors.
- Evidence-local `test_portable.py`: copied sealed replay and serialized ledger/log rejection/exact restoration.

## Known gaps (current cycle)

- [ ] 854 names have no local argument/return/default/event/state/security/native contracts. Archive1.1.2.4115, oldid4864/string dump and linked API/template bodies unexpanded. All859 substantive contracts UNPROVEN.

## Out of scope

Runtime/model changes, inferred aliases/defaults/shims, target research, operations, builds/clients/checks/broad/final gates. Main owns ordered integration, independent full acceptance and parent completion; registry endpoint alone is not closure.
