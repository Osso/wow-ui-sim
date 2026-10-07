# Patch 10.2.5 publication sweep

Account for Warcraft Wiki page 564286, revision 5993852 (2024-03-24T16:20:32Z), against current retail 12.1.0, not reconstructed 10.2.5. [Audit](../wiki/investigations/patch-10-2-5-api-audit.md) owns findings and proof limits.

## What it must do

- [ ] Probe all 59 inventory occurrences in isolated cached Game UI; failures exactly equal the reviewed gap fixture.
- [ ] Apply sixteen later registers (10.2.7 through 12.1.0) chronologically. Adding independent 10.2.6 requires one include before 10.2.7.
- [ ] Construct VertexColor as a texture animation, not an invented frame kind; missing endpoint methods remain gaps.
- [ ] Retain all 356 extract occurrences, including verbatim Lua/XML examples; account for all 415 source IDs exactly once. Documentary backfill gives no runtime credit.
- [ ] Negative control changes one published, unsuperseded API to removed and produces exactly one additional gap.

## How it works

- [Publication accounting and boundaries](../wiki/investigations/patch-10-2-5-api-audit.md).
- [Client profiles](../wiki/systems/client-profiles.md).

## Implementation inventory

- `tests/common/publication_sweep.rs`: shared publication/absence classifier, real animation factory and cached-deprecation attribution.
- `tests/patch_10_2_5_publication_sweep.rs`: current chronological register list and exact fixture.
- `tools/extract_patch_non_inventory.py`: opt-in verbatim code blocks; default legacy extraction unchanged.
- `data/patch-api/sources/10.2.5-page-coverage.json`: source-ID accounting and explicit proof limits.

## Tests asserting this spec

- `tests/patch_10_2_5_publication_sweep.rs`.
- `tests/patch_10_2_5_probe_factory.rs`: animation constructor and real missing endpoint result.
- `tests/patch_10_2_5_cached_surfaces.rs`: cached Game color-picker callback/hex input consumer; no historical optional-parameter parity claim.
- `tools/test_extract_patch_non_inventory.py`: XML/Lua literal retention and legacy-output fixtures.
- `data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-validate.py`: exhaustive IDs, source hashes, chronological expectations and retained inputs.

## Known gaps (current cycle)

- [ ] Fourteen exact publication gaps: three item queries, two ping queries, transcription activity, role assignment, three legacy aura retirements and four VertexColor endpoints. [Per-ID boundaries](../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-gap-review.json).
- [ ] Eighty-two substantive extract occurrences require distinct behavior/security/DTO/enum or historical proof. [Scout](../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-extract-scout.json).

## Out of scope

Historical epoch reconstruction, native parity, expanded linked pages, fabricated host data, broad role/ping/animation systems, vendor/cache edits and classic changes. No runtime behavior replaced merely to improve a sweep count. Full suites, agents/models, push and merge prohibited.

## Local proof

Pending targeted gates. Commands/revisions/results recorded in `data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-proof.json`.
