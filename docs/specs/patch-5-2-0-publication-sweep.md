# Retail Patch 5.2.0 publication sweep

Audit the pinned [API page](../../data/patch-api/sources/5.2.0-api-changes.wikitext) and its [transcluded inventory](../../data/patch-api/sources/5.2.0-api-changes-diff.wikitext), not the Mists Classic 5.5.x line. [Audit](../wiki/investigations/patch-5-2-0-api-audit.md) records evidence and limits.

## What it must do

- [x] Preserve bare removed widget handler owners, directions and source lines; match the transclusion's numerical inventory headers.
- [ ] Probe every inventory occurrence against the retail prefork SharedXML surface, separating exact known gaps from publication/absence credit.
- [ ] Apply only later retail registers, oldest first; keep queued 5.3.0/5.4.0/5.4.1/5.4.2 placeholders ahead of 5.4.7.
- [ ] Account separately for every retained prose statement and transclusion build caption.
- [ ] Preserve historical proof in fresh checkouts and after unrelated later audits; reject own evidence tampering.

## How it works

- [Publication audit](../wiki/investigations/patch-5-2-0-api-audit.md).
- [Historical validator gate](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in 2013 table/transclusion parsers copied from p530-page; separate bare-handler normalization.
- `tests/patch_5_2_0_publication_sweep.rs`: retail-only inventory discovery and known-gap fixture.
- `data/patch-api/sources/5.2.0-*`: pinned source, provenance, inventory and coverage.
- `data/patch-api/evidence/5.2.0-session-2026-10-08/`: retained execution and retirement proof.

## Tests asserting this spec

- `tools/test_patch_mists_520_register.py`: concrete owner/direction/source-line fixture and complete pinned table counts.
- `tests/patch_5_2_0_publication_sweep.rs`: prefork publication/absence discovery.

## Known gaps (current cycle)

- [ ] Finish discovery accounting and historical validator proof.

## Out of scope

Publication alone does not prove 2013 signatures, outputs, security, services or rendering parity. Model/PlayerModel 3D rendering remains intentionally unsupported. No vendor behavior modifications, compatibility shims or Classic supersession.
