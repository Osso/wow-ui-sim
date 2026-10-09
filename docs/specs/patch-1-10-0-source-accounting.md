# Patch 1.10.0 frozen source accounting

Bounded accounting of page 385987/revision 3714298/timestamp 2020-04-05T21:06:58Z from the retained legacy manifest. [Audit](../wiki/investigations/patch-1-10-0-api-audit.md) describes evidence and proof epochs; revision timestamp is not release date.

## What it must do

- [x] Validate exact response/body and manifest-linked registry identity before ledger derivation.
- [x] Account every literal row, link, header, template, prose, signature and default; reject occurrence omissions, count mutations and invented inventory/model/native credit.
- [x] Retain the redirect target unexpanded and its missing historical contracts UNPROVEN. No local declarations means zero meaningful modeled subset.
- [x] Keep original Retail, Classic Era and Forever histories distinct; frozen task-start 1.10.1 queue is separate and unapplied. Later integration context must not rewrite that queue. Integrated 1.10.2 supplies methodology, not missing target content.
- [x] Preserve own SOURCE RED/GREEN and copied own-base tool default bytes/error behavior. Replay an immutable copied archive without Git, target, current tools, addons or network; reject serialized ledger/log tampering and restore exact bytes without resealing.
- [ ] Keep immutable original seals/archive separate from later actual revision/cwd/argv/timestamps/full-stream/scoped-hash receipts. Retain environment keys only after credential-pattern inspection.

## How it works

- [Audit and proof limits](../wiki/investigations/patch-1-10-0-api-audit.md)
- [Portable replay instructions](../../data/patch-api/evidence/1.10.0-session-2026-10-09/REPLAY.md)

## Implementation inventory

- `data/patch-api/evidence/1.10.0-session-2026-10-09/audit.py`: isolated frozen-source derivation and historical replay.
- Evidence-local `historical-tools/`: unchanged base extractor/generator copies; shared tools remain untouched.

## Tests asserting this spec

- Evidence-local `test_source_accounting.py`: literal identity, omission/count/invention/source mutation, history and default bytes/errors.
- Evidence-local `test_portable.py`: copied sealed replay, serialized ledger/log rejection and exact restoration.

## Known gaps (current cycle)

- [ ] Redirect target revision/content and patch-specific historical API delta are absent; arguments, returns, defaults, events, state transitions, security and native equivalence remain UNPROVEN.

## Out of scope

Runtime/shared-tool/vendor changes, guessed aliases/defaults/shims, builds, broad/check/final gates, native acceptance and operations. Main owns ordered integration, independent proof and parent acceptance.
