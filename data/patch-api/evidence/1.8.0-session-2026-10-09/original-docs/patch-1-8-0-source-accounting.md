# Patch 1.8.0 frozen source accounting

Bounded accounting of page360905/revision3476931/timestamp2020-04-05T21:06:17Z. [Audit](../wiki/investigations/patch-1-8-0-api-audit.md) records source identity and proof epochs; revision timestamp is not release date.

## What it must do

- [x] Validate exact frozen response/body and manifest-linked registry101 before deriving occurrences.
- [x] Account every literal row/link/header/template/prose/signature/default; reject omissions, mutated counts, invented declarations and model/runtime/native credit.
- [x] Keep the redirect target unexpanded and contracts UNPROVEN; zero meaningful modeled subset.
- [x] Separate original historical Retail from Era/Forever; retain task-start newer1.9.0 queue separately/unapplied.
- [ ] Preserve own SOURCE RED/GREEN and copied own-base default bytes/error behavior. Replay an immutable archive without Git/target/current tools/network; reject serialized ledger/log tampering and restore exact bytes without resealing.
- [ ] Retain immutable original seals/archive separately from actual later revision/cwd/argv/times/full-stream/scoped-hash receipts. Inspect credential patterns/environment key names, not values.

## How it works

- [Audit and limits](../wiki/investigations/patch-1-8-0-api-audit.md)
- [Replay](../../data/patch-api/evidence/1.8.0-session-2026-10-09/REPLAY.md)

## Implementation inventory

- `data/patch-api/evidence/1.8.0-session-2026-10-09/audit.py`: isolated source accounting and historical default replay.
- Evidence-local `historical-tools/`: unchanged own-base extractor/generator copies; no shared-tool edits.

## Tests asserting this spec

- Evidence-local `test_source_accounting.py`: exact identity, occurrences/counts/inventions/source mutation, history and defaults/errors.
- Evidence-local `test_portable.py`: copied sealed replay, serialized ledger/log rejection and restoration.

## Known gaps (current cycle)

- [ ] Redirect target revision/content and patch-specific delta absent. Arguments, returns, defaults, events, state transitions, security and native equivalence UNPROVEN.

## Out of scope

Runtime/shared-tool/vendor changes, guessed aliases/defaults/shims, native/model closure, builds, checks/broad/final gates and operations. Main owns ordered integration, independent proof and parent acceptance.
