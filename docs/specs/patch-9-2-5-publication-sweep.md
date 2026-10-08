# Patch 9.2.5 publication sweep

Account for Warcraft Wiki page 237255 revision 2301036 against current retail 12.1.0. Sources live under `data/patch-api/sources/9.2.5-*`; [audit](../wiki/investigations/patch-9-2-5-api-audit.md) describes implementation and proof boundaries. Publication accounting is not complete compatibility.

## What it must do

- [x] Retain source/provenance, all 84 inventory occurrences, exact annotations/defaults and verbatim examples.
- [x] Recover multiline security warning and inline structure boundaries with fixtures; preserve prior inputs and both extraction-mode outcomes.
- [x] Probe all occurrences using all 27 chronological later registers and an exact reviewed gap fixture (currently 33 IDs after the [8.2.5 ReportPosting closure](../wiki/investigations/patch-8-2-5-api-audit.md)).
- [x] Prevent raw/repeated lookup fabrication of nine unused retail removals without deleting live cached reporting/deprecation APIs or changing Mists legacy lookup.
- [x] Account for all 220 inventory/extract/context IDs, assigning literal source lines and explicit proof boundaries.
- [x] Detect a single GAME_PAD_POWER_CHANGED direction mutation: exactly 34 → 35 gaps, unchanged IDs.
- [x] Retain all 28 sweep receipts, behavioral/cached/Mists/report regressions, 38 fixtures, formatting/default/Mists checks and exit-0 startup [].

## How it works

- [Page audit](../wiki/investigations/patch-9-2-5-api-audit.md).
- [Proof ledger](../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-proof.json).

## Implementation inventory

- `data/patch-api/sources/9.2.5-*` — pinned source, register, extract, provenance and coverage ledger.
- `src/c_api/patch_retired_members.rs` — separate nine-member retail retirement const.
- `tools/{extract_patch_non_inventory,gen_patch_wikitext_register}.py` — fixture-backed source/default retention.
- `tests/patch_9_2_5_*.rs` — publication, bare/cached retirement and classic preservation proofs.
- `tests/data/patch_9_2_5_sweep_known_gaps.json` — exact remaining failure IDs.

## Tests asserting this spec

- `tests/patch_9_2_5_publication_sweep.rs`.
- `tests/patch_9_2_5_publication_fixes.rs`, `tests/patch_9_2_5_cached_surfaces.rs`, `tests/patch_9_2_5_classic_surfaces.rs`.
- `tools/test_extract_patch_non_inventory.py`, `tools/test_gen_patch_wikitext_register.py`.
- `data/patch-api/evidence/9.2.5-session-2026-10-07/validate.py`.

## Known gaps (current cycle)

- [ ] 33 exact publication failures retain per-ID model/source reasons; ReportPosting absence is now bounded by the 8.2.5 audit.
- [ ] 120 substantive extract statements and successful inventory annotations/payloads lack occurrence-specific behavior/native parity proof.
- [ ] GetSourceLocation creating script/line and two historical CVar default comparisons remain explicit mismatches.

## Out of scope

Historical reconstruction, broad suites, speculative models, placeholders, vendor/cache edits, agents/models, push and merge. Retirements require qualified/bare cached-consumer evidence and retail gating. The restored temporary canonical artifact-write violation is documented, not presented as compliant isolation.
