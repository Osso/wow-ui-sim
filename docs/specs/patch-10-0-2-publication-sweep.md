# Patch 10.0.2 publication sweep

Account for Warcraft Wiki page 303146, revision 2926754 (2023-01-27T02:58:55Z), against current retail 12.1.0. [Page audit](../wiki/investigations/patch-10-0-2-api-audit.md) owns occurrence accounting and proof boundaries.

## What it must do

- [x] Historically probe all416 inventory occurrences with all24 chronological later registers, 10.0.5 through12.1.0; match the then-reviewed149-item gap set.
- [x] Replay against the current148-item expected set after TextWithSubtext row309 reconciliation; see the owning-after checkpoint below. Historical pre-reconciliation exact-set FAIL remains preserved.
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

## TextWithSubtext follow-up — 2026-10-14

[TextWithSubtext](ui-widget-text-with-subtext.md) owns separate supplied-record and cached Retail/PTR lifecycle proof. Historical pre-reconciliation exact-set FAIL remains:416 rows/268 OK/148 non-OK against149 expected gaps; only getter row309 was already OK. Commit `43158d03c` removes that exact expected-gap row.

### Owning-after checkpoint — 2026-10-14

Independent saved-runtime gate at `0d819ea3ded9904e5a69757b66e67742a78c7635`:416 unique rows,268 OK/148 non-OK exactly equal the current148-ID fixture; row309 published. One primary PASS, exit0; three zero-case auxiliary summaries excluded. All416 observed result objects equal the owning-before results: expected-gap reconciliation alone changes the comparison outcome.

No-run compile20.42s/exit0,741 Cargo records and six existing manifest deprecations retained. Peak4,307,513,344 bytes below8G high. Receipt reuses one producer digest with stable metadata; no independent executable-byte authentication. Complete runtime diagnostics reach FIRST_FRAME_RENDERED; no warning/error lines observed, not native/conformance/font/WTF/render proof. Sound/CASC disabled; caches/dependencies/settings unauthenticated.

[Whole-TEXT owning-after retention](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/widget-text-20261014/owning-after/README.md) preserves21 complete derivatives of this replay and its saved gates; the historical39 packet remains separate.

Publication is not signature/populated-output/security/native parity. Broader/all-profile/render/full-parent acceptance stays open; broad `4fc115b93` does not cover later type25 changes.

## Current reconciliation — 2026-11-04

Remove only `GetLootItem-236` and `GetBuybackItem-214` from the current expected gaps (142 → 140). Producers `d6ac8fb0d` and `2026091d4` register real `C_TooltipInfo` getters backed by supplied loot/buyback slots; see [tooltip contract](tooltip-loot-buyback.md). These rows test publication, not populated output or native parity. Earlier counts and captures are historical.

## Known gaps

- [ ] 140 current publication gaps in `tests/data/patch_10_0_2_sweep_known_gaps.json`; historical reviews remain preserved;  [per-ID review](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-gap-review.json) owns reasons and source/cached references.
- [ ] 156 substantive extract occurrences; [literal scout](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-extract-scout.json) records every line and proof boundary.

## Out of scope

Historical epochs, new placeholders, linked-page expansion, vendor/cache edits, native parity, full suites, agents/models, push and merge. Complete publication accounting is not complete API compatibility.
