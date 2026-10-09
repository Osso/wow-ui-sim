# Retail Patch 5.2.0 publication sweep

Audit the pinned [API page](../../data/patch-api/sources/5.2.0-api-changes.wikitext) and its [transcluded inventory](../../data/patch-api/sources/5.2.0-api-changes-diff.wikitext), not the Mists Classic 5.5.x line. [Audit](../wiki/investigations/patch-5-2-0-api-audit.md) records evidence and limits.

## What it must do

- [x] Preserve bare removed widget handler owners, directions and source lines; match the transclusion's numerical inventory headers.
- [x] Probe every inventory occurrence against the retail prefork SharedXML surface, separating exact known gaps from publication/absence credit.
- [x] Apply only later retail registers, oldest first; keep queued 5.3.0/5.4.0/5.4.1/5.4.2 placeholders ahead of 5.4.7.
- [x] Account separately for every retained prose statement and transclusion build caption.
- [x] Keep behavioral credit bounded: raid difficulty reads distinct state values, cooldown duration follows timing updates/clear, and school mapping remains linked to its cached deprecated alias.
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
- `tests/patch_5_2_0_publication_sweep.rs`: prefork publication/absence discovery and exact negative-control rejection.
- `tests/patch_5_2_0_backing_behavior.rs`: integration and prefork raid-difficulty state reads.
- `tests/cooldown_widget.rs`, `tests/spell_api.rs`, `tests/blizzard_deprecated_spell_script_loads.rs`: existing duration, school mapping and alias-identity behavior.

## Known gaps (current cycle)

- [ ] Pass historical validator portability and own-evidence tamper controls.
- [ ] 57 publication gaps and 16 historical prose contracts remain documented, not replaced with shims. Queued retail register integration may supersede some later.

## Out of scope

Publication alone does not prove 2013 signatures, outputs, security, services or rendering parity. Model/PlayerModel 3D rendering remains intentionally unsupported. No vendor behavior modifications, compatibility shims or Classic supersession.
