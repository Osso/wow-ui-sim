# Patch 10.0.5 publication sweep

Account for Warcraft Wiki page 75033, revision 742761 (2023-02-01T00:52:10Z), against current retail 12.1.0. [Page audit](../wiki/investigations/patch-10-0-5-api-audit.md) owns source accounting and proof boundaries.

## What it must do

- [x] Probe all 93 inventory occurrences with all 23 chronological later registers, 10.0.7 through 12.1.0, and require exactly 27 reviewed publication gaps.
- [x] Keep ContinueRecast, GetRecipeRepeatCount and HasRecipesTracked absent under repeated C_TradeSkillUI lookup in bare and cached Game environments; preserve tracked-recipe successor publication.
- [x] Preserve existing Mists legacy namespace lookup. The existing retail epoch module gate excludes Wrath, Mists, Era and Anniversary; this audit does not reconstruct historical retail epochs.
- [x] Detect a one-row negative control: C_Mail.SetOpeningAll added → removed produces only that new failure, 27 → 28.
- [ ] Prove signatures, populated outputs, event payloads, security and historical/native parity; publication/absence is insufficient.

## How it works

- [Page audit](../wiki/investigations/patch-10-0-5-api-audit.md) — accounting, retirement evidence and proof boundaries.
- [Client profiles](../wiki/systems/client-profiles.md) — retail/classic gates.

## Implementation inventory

- `data/patch-api/sources/10.0.5-*` — pinned source, provenance, inventory and 160-ID coverage ledger.
- `src/c_api/patch_retired_members.rs` — separate 10.0.5 retirement constant under existing retail-only gate.
- `tests/common/publication_sweep.rs` — shared current-publication classifier and later supersession.
- `tests/data/patch_10_0_5_sweep_known_gaps.json` — exact reviewed gap IDs.
- `data/patch-api/evidence/10.0.5-session-2026-10-07/` — per-ID gap/extract review, caller scans and hashed proof logs.

## Tests asserting this spec

- `tests/patch_10_0_5_publication_sweep.rs` — all inventory occurrences and exact fixture.
- `tests/patch_10_0_5_publication_fixes.rs` — repeated bare lookup and retained successors.
- `tests/patch_10_0_5_cached_surfaces.rs` — same assertions after unmodified cached Game startup.
- `tests/patch_10_0_5_classic_surfaces.rs` — Mists lookup preservation.

## Known gaps (current cycle)

- [ ] Twenty-seven publication gaps require concrete producers; [per-ID review](../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-gap-review.json) owns precise reasons.
- [ ] Sixty substantive extract occurrences require occurrence-specific behavioral proof; [scout](../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-extract-scout.json) retains every literal and proof boundary.

## Out of scope

Historical epoch reconstruction, new placeholders, linked-page expansion, vendor/cache edits, native parity, full suites, agents/models, push and merge. Complete accounting does not mean complete API compatibility.
