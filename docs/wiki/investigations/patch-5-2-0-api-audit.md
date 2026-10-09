# Retail Patch 5.2.0 API audit

Verified source: 2026-10-08. API pageid 422312 revision 4057299; transcluded diff pageid 471131 revision 4529204. Parent states TOC 50200, build 16650 and March 5, 2013 release. This is the retail Mists line, not Mists Classic 5.5.x.

## Coverage and limits

The transclusion contains 163 inventory occurrences, including four widget handlers. All numerical table headers match parsed counts. Main prose remains separate: new features, global API changes and FrameXML changes are not silently replaced by inventory publication credit.

Parser hunks came byte-identically from the pinned p530-page generator; the separate opt-in `--mists-bare-widget-handlers` normalizes `Model OnAnimFinished` / `Model OnUpdateModel` removals without changing source lines. Main prose extraction uses existing `--retain-patch-diff-reference`; no extractor changes.

Retail 5.3.0, 5.4.0, 5.4.1 and 5.4.2 remain queued placeholders, in that order. The merged chain begins 5.4.7 then 5.4.8 and 6.0.1. No 5.5.x Classic register is included. Prefork publication observes SharedXML/bootstrap, not every load-on-demand Game addon or 2013 client behavior.

## Retirement safety

Every removed occurrence has a retained whole-word `/usr/bin/grep -R -n -w -F` cache and src/tests scan, including indirect calls (`pcall(Name, ...)`) and guards (`and Name then`) because the bare identifier is searched independently. Cache scans exclude `*Documentation*`. Output is untruncated. Pinned master and all four queued branch register sets are checked for re-additions; no matches found.

No simulator retirement is implemented. Existing cache/caller consumers are retained; Model/PlayerModel 3D methods remain intentionally unsupported rather than being removed from the common frame-method surface. Historical absence assertions do not authorize breaking current callers.

## Coverage matrix

| Scope | Accounted | Proof / limit |
|---|---:|---|
| Inventory publication/absence | 106 of 163 | Retail prefork sweep; no domain-parity credit |
| Inventory gaps | 57 | 20 API/FrameXML additions, 35 retained Model methods, 2 PlayerModel handlers |
| Existing meaningful backing | 3 occurrences | Raid difficulty state, cooldown duration transitions, school-name mapping plus cached alias identity |
| Substantive prose | 16 pending | Precise per-line reasons; historical bugs and uncertain universal claims not imposed on current runtime |
| Source metadata | 12 | Seven main-page context rows and five pinned build captions; total 191 unique accounted IDs |

## Existing backing behavior

`GetRaidDifficultyID` reads `SimState.world.instance_difficulty`. New integration and prefork cases exercise distinct values 14 and 16. Existing cooldown tests assert stateful millisecond duration reads and clear transitions. School-name mapping belongs to `C_Spell`, with the cached legacy global alias handled by Blizzard's unmodified deprecated wrapper.

`UnitGetTotalAbsorbs` still returns a fixed zero, `UnitStagger` is a temporary default and pet cooldown queries are defaults. These are publication only, not modeled behavior credit. The 2013 absorption/stagger/cooldown producer contracts remain pending.

## Verification

Publication/factory cases pass 58/58; own prefork 2/2, raid integration 1/1, cooldown integration 13/13, school integration 1/1 and cached school-alias prefork 1/1 pass. Negative control changes 57 → 58 gaps and fails as required. All 89 Python fixtures pass. All 59 registers reproduce byte-identically; 56 saved extracts reproduce, with inherited 12.0.5/12.0.7/12.1.0 failures unchanged. No `src/` runtime edits: master-runtime/lib/startup comparison is therefore not triggered. Scoped register lib cases pass 3/3; format and Mists `--no-default-features --features sound,gui,casc,client-mists --tests` checks pass. Mists has zero non-vendor warnings (six inherited iced_wgpu manifest deprecations remain untouched). Historical portability gate remains in progress.

[Accounting](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/accounting-summary.json), [gap reasons](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/gap-review.json) and [retirement decisions](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/retirement-decisions.json) retain the limits. No full integration suite, new runtime shim or vendor change.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/5.2.0-api-changes.provenance.json).
- [Evidence](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/context.json) — base and parser-origin revisions.
- [Spec](../../specs/patch-5-2-0-publication-sweep.md).
- [Sweep](../../../tests/patch_5_2_0_publication_sweep.rs).

## See Also

- [[patch-5-4-7-api-audit]] — next merged retail register.
- [[patch-audit-validator-portability]] — historical-scope and clean/later-audit gates.
