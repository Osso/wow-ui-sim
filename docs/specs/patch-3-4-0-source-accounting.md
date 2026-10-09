# Patch 3.4.0 source accounting

Account only the frozen Wrath Classic [page](../../data/patch-api/sources/3.4.0-api-changes.wikitext), preserving publication claims separately from model/native proof. [Audit](../wiki/investigations/patch-3-4-0-api-audit.md) explains the boundaries.

## What it must do

- [x] Verify exact manifest/response/content identity and hashes before ingestion; replay independently without Git.
- [x] Account every nonblank raw row, inventory occurrence, direction and table count, including the command and bare CVar removal.
- [x] Preserve literal CVar metadata, four summary contracts and the partial UnitAura return contract without inventing a full signature.
- [x] Reject fabricated capability/runtime/native credit, omitted rows, changed identities, metadata, partial contracts and foreign supersession.
- [x] Separate immutable compact historical evidence from dynamic validator; reject serialized-source/log tampering, restore bytes, replay from relocated Git-free archive under 5MB.

## How it works

- [Source/model-contract audit](../wiki/investigations/patch-3-4-0-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/3.4.0-page-coverage.json` — literal source and signature ledger.
- `data/patch-api/evidence/3.4.0-session-2026-10-09/` — exact source pins, static model/profile observations and owned replay fixtures.

## Tests asserting this spec

- Owned `test_source_accounting.py`: 8/8 GREEN and 987 individual omission controls at `2497ec7a8`; `validate.py` replays source-only summaries. `replay_controls.py`: two seal rejections with restoration, relocated no-Git source replay exit 0 at `30071d654`.

## Known gaps (current cycle)

- [ ] Runtime/native contracts remain UNPROVEN and parent-owned; source fixtures/portable controls satisfy this bounded slice.

## Out of scope

Runtime/factory/publication measurement, native TOC30400 parity, shared tooling changes, linked-page reconstruction, retail/Cata supersession, runtime shims/retirements, caches/vendor/Wowless writes and final gates. Configured Wrath38001 is not a native Classic30400 target. `-WOTLKC` is a static candidate gap for parent integration; UnitAura consolidation state and exact tuple position are not established.
