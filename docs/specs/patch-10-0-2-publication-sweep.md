# Patch 10.0.2 publication sweep

Account for Warcraft Wiki page 303146, revision 2926754 (2023-01-27T02:58:55Z), against current retail 12.1.0. [Page audit](../wiki/investigations/patch-10-0-2-api-audit.md) owns occurrence accounting and proof boundaries.

## What it must do

- [x] Probe all 416 inventory occurrences with all 24 chronological later registers, 10.0.5 through 12.1.0; require exactly 149 reviewed publication gaps.
- [x] Keep twenty unused namespace members absent under repeated lookup in bare and unmodified cached Game environments. Preserve modeled crafting-order/item-tooltip successor publication.
- [x] Preserve Mists legacy namespace lookup. The existing retail epoch module gate excludes Wrath, Mists, Era and Anniversary; no 10.x epoch is introduced.
- [x] Detect an exact one-row negative control: UnitTokenFromGUID added → removed produces only that new failure, 149 → 150.
- [x] Preserve every existing source/register/ledger/gap fixture and before/after extract outcome; retain the literal tooltip Lua examples.
- [ ] Prove signatures, populated outputs, event payloads, security or historical/native parity. Publication/absence alone is insufficient.

## How it works

- [Page audit](../wiki/investigations/patch-10-0-2-api-audit.md) — accounting, removals, retained gaps and targeted proof.
- [Client profiles](../wiki/systems/client-profiles.md) — retail/classic gates.

## Implementation inventory

- `data/patch-api/sources/10.0.2-*` — pinned source, provenance, inventory and 583-ID coverage ledger.
- `src/c_api/patch_retired_members.rs` — separate 10.0.2 retirement constant, existing retail-only gate.
- `tools/gen_patch_wikitext_register.py` — plain CVar inventory spelling, covered by a behavioral fixture.
- `tests/common/publication_sweep.rs` — unchanged shared publication classifier and later supersession.
- `tests/data/patch_10_0_2_sweep_known_gaps.json` — exact reviewed gap IDs.
- `data/patch-api/evidence/10.0.2-session-2026-10-07/` — per-ID gap/extract review, full caller scans and proof logs.

## Tests asserting this spec

- `tests/patch_10_0_2_publication_sweep.rs` — all inventory occurrences and exact fixture.
- `tests/patch_10_0_2_publication_fixes.rs` — repeated bare lookup and successor publication.
- `tests/patch_10_0_2_cached_surfaces.rs` — same assertions after cached Game startup; prefork target.
- `tests/patch_10_0_2_classic_surfaces.rs` — Mists legacy lookup preservation.
- `tools/test_gen_patch_wikitext_register.py` — unlinked removed CVar identity, line and cardinality.

## Known gaps

- [ ] 149 publication gaps; [per-ID review](../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-gap-review.json) owns reasons and source/cached references.
- [ ] 156 substantive extract occurrences; [literal scout](../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-extract-scout.json) records every line and proof boundary.

## Out of scope

Historical epochs, new placeholders, linked-page expansion, vendor/cache edits, native parity, full suites, agents/models, push and merge. Complete publication accounting is not complete API compatibility.
