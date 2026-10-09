# Historical retail Patch 2.4.2 audit

Frozen page 81145/revision 803933 (2021-12-28T02:03:05Z revision timestamp) documents 2008 retail, not TBC Classic. [Audit](../wiki/investigations/patch-2-4-2-api-audit.md).

## What it must do
- [ ] Literally account APIs, widgets, constants, signatures, prose, navigation, headers and references; no linked-page expansion.
- [ ] Opt-in parsing preserves default bytes and recorded outputs.
- [ ] Separate supported current-retail publication, real model behavior and historical native acceptance.
- [ ] Seal original ledger/gaps/tools/code/logs for fresh Git-free replay; future closures separate.
- [ ] Serialized ledger/log tampering rejects and restored bytes replay.

## How it works
- [Audit](../wiki/investigations/patch-2-4-2-api-audit.md)

## Implementation inventory
- `data/patch-api/evidence/2.4.2-session-2026-10-09/`: pinned source and historical proof.

## Tests asserting this spec
- Pending bounded development fixtures.

## Known gaps (current cycle)
- [ ] Publication/source accounting pending measurement.
- [ ] Queued retail 3.0.2/3.0.3/3.0.8/3.1.0 overlaps need main integration; use only actual retail 3.2.0/3.3.0/3.3.3/3.3.5/4.0.1 successors.

## Out of scope
Classic supersession; guessed aliases/shims/fallbacks; vendor/cache/Wowless edits; native historical acceptance; broad/check/lint/readability/profile/startup/full-suite/final gates; push/merge/deploy/delegation.
