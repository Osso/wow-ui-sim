# Patch 3.4.0 source accounting

Account only the frozen Wrath Classic [page](../../data/patch-api/sources/3.4.0-api-changes.wikitext), preserving publication claims separately from model/native proof. [Audit](../wiki/investigations/patch-3-4-0-api-audit.md) explains the boundaries.

## What it must do

- [ ] Verify exact manifest/response/content identity and hashes before ingestion; replay independently without Git.
- [ ] Account every nonblank raw row, inventory occurrence, direction and table count, including the command and bare CVar removal.
- [ ] Preserve literal CVar metadata, four summary contracts and the partial UnitAura return contract without inventing a full signature.
- [ ] Reject fabricated capability/runtime/native credit, omitted rows, changed identities, metadata, partial contracts and foreign supersession.
- [ ] Separate immutable compact historical evidence from dynamic validator; reject serialized-source/log tampering, restore bytes, replay from relocated Git-free archive under 5MB.

## How it works

- [Source/model-contract audit](../wiki/investigations/patch-3-4-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/3.4.0-page-coverage.json` — literal source and signature ledger.
- `data/patch-api/evidence/3.4.0-session-2026-10-09/` — exact source pins, static model/profile observations and owned replay fixtures.

## Tests asserting this spec

- Owned `test_source_accounting.py`, `validate.py`, `replay_controls.py` under the evidence directory (pending).

## Known gaps (current cycle)

- [ ] Targeted source fixtures and portable negative controls pending.

## Out of scope

Runtime/factory/publication measurement, native TOC30400 parity, shared tooling changes, linked-page reconstruction, retail/Cata supersession, runtime shims/retirements, caches/vendor/Wowless writes and final gates. Configured Wrath38001 is not a native Classic30400 target. `-WOTLKC` is a static candidate gap for parent integration; UnitAura consolidation state and exact tuple position are not established.
