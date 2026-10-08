# Patch 8.2.5 publication sweep

Audit pinned Warcraft Wiki page 83383, revision 6471405, against current retail—not historical reconstruction. Sources live under `data/patch-api/sources/8.2.5-*`; [audit](../wiki/investigations/patch-8-2-5-api-audit.md) describes implementation and proof boundaries.

## What it must do

- [x] Enumerate all numeric Patch X/API changes titles older than 8.3.0 with revision IDs and exhausted pagination evidence; retain pinned source, opt-in reproduction recipes, numerical headers and every inventory/extract/caption occurrence.
- [x] Apply every later master register, including merged 8.3.0, and enforce the exact reviewed gap identities rather than zero-gap assertions.
- [x] Prevent raw/repeated lookup fabrication of unused retail namespace removals; preserve current cached consumers, Blizzard deprecation aliases and Mists lookup.
- [x] Preserve earlier inputs except the explicitly proven ReportPosting ledger closure; reproduce all registers and all formerly reproducible saved extracts with recorded or labeled inferred flags.
- [x] Prove all publication sweeps, exact one-ID negative control, scoped bare/cached/classic regressions, parser fixtures, formatting, default/Mists checks and exit-0 startup `[]`; dynamically validate the receipts.

## How it works

- [Page audit](../wiki/investigations/patch-8-2-5-api-audit.md).
- [Proof index](../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-proof.json).
- [Per-ID gap review](../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-gap-review.json).

## Implementation inventory

- `data/patch-api/sources/8.2.5-*` — pinned raw/extract/provenance, register and occurrence ledger.
- `data/patch-api/sources/api-change-pages-remaining.json` — enumerated older page series.
- `src/c_api/patch_retired_members.rs` — separate retail-only 8.2.5 retirement list.
- `tools/gen_patch_wikitext_register.py` — opt-in legacy mixed-header/category parsing; extractor CLI keeps a single reference-note option.
- `tests/patch_8_2_5_*.rs` and `tests/data/patch_8_2_5_sweep_known_gaps.json` — publication, absence, cached and classic proof.

## Tests asserting this spec

- `tests/patch_8_2_5_publication_sweep.rs`.
- `tests/patch_8_2_5_publication_fixes.rs`, `tests/patch_8_2_5_cached_surfaces.rs`.
- `tools/test_extract_patch_non_inventory.py`, `tools/test_gen_patch_wikitext_register.py`.
- `data/patch-api/evidence/8.2.5-session-2026-10-08/validate.py`.

## Known gaps (current cycle)

- [ ] 61 exact publication failures retain per-ID producer/lifecycle/migration/unsupported-domain reasons.
- [ ] Four substantive extract statements lack hardware-event/native-security or Party Sync/RAF overhaul proof.
- [ ] Successful publication observations do not prove signatures, populated outputs, payloads, security or historical behavior.
- [ ] Inherited 12.0.5/12.0.7/12.1.0 saved extracts remain non-reproducible; unchanged and explicitly recorded.

## Out of scope

No historical reconstruction, placeholder APIs, vendor/cache/Wowless/WowlessData writes, Blizzard Lua overrides, other-page implementations, cwd changes, agents/models, push, merge or full integration suite. Use explicit worktree cwd and dedicated Cargo target; disabled host services remain untouched. Native 3D actor behavior is intentionally unsupported.
