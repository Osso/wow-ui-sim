# Retail Patch 5.2.0 API audit

Verified source: 2026-10-08. API pageid 422312 revision 4057299; transcluded diff pageid 471131 revision 4529204. Parent states TOC 50200, build 16650 and March 5, 2013 release. This is the retail Mists line, not Mists Classic 5.5.x.

## Coverage and limits

The transclusion contains 163 inventory occurrences, including four widget handlers. All numerical table headers match parsed counts. Main prose remains separate: new features, global API changes and FrameXML changes are not silently replaced by inventory publication credit.

Shared `--mists-automated-diff`, `--mists-diff` and `--mists-widget-handlers` parsers now come from merged master (5.4.2/5.4.0/5.3.0). The sole 5.2.0 extension, opt-in `--mists-bare-widget-handlers`, runs after widget handlers and normalizes `Model OnAnimFinished` / `Model OnUpdateModel` removals without changing source lines. Historical parser-origin receipts remain preserved; compact commit patch IDs and directory tree pins record the rebase. Main prose extraction uses existing `--retain-patch-diff-reference`; no extractor changes.

Retail 5.3.0, 5.4.0, 5.4.1 and 5.4.2 are real merged registers, in that order, followed by 5.4.7, 5.4.8 and 6.0.1. No 5.5.x Classic register is included. Prefork publication observes SharedXML/bootstrap, not every load-on-demand Game addon or 2013 client behavior.

## Retirement safety

Every removed occurrence has a retained whole-word `/usr/bin/grep -R -n -w -F` cache and src/tests scan, including indirect calls (`pcall(Name, ...)`) and guards (`and Name then`) because the bare identifier is searched independently. Cache scans exclude `*Documentation*`. Output is untruncated. Pinned master and all four queued branch register sets are checked for re-additions; no matches found.

No simulator retirement is implemented. Existing cache/caller consumers are retained; Model/PlayerModel 3D methods remain intentionally unsupported rather than being removed from the common frame-method surface. Historical absence assertions do not authorize breaking current callers.

## Coverage matrix

| Scope | Accounted | Proof / limit |
|---|---:|---|
| Inventory publication/absence | 108 of 163 | Retail prefork sweep; no domain-parity credit |
| Inventory gaps | 55 | 18 API/FrameXML additions, 35 retained Model methods, 2 PlayerModel handlers |
| Existing meaningful backing | 3 occurrences | Raid difficulty state, cooldown duration transitions, school-name mapping plus cached alias identity |
| Substantive prose | 16 pending | Precise per-line reasons; historical bugs and uncertain universal claims not imposed on current runtime |
| Source metadata | 12 | Seven main-page context rows and five pinned build captions; total 191 unique accounted IDs |

## Existing backing behavior

`GetRaidDifficultyID` reads `SimState.world.instance_difficulty`. New integration and prefork cases exercise distinct values 14 and 16. Existing cooldown tests assert stateful millisecond duration reads and clear transitions. School-name mapping belongs to `C_Spell`, with the cached legacy global alias handled by Blizzard's unmodified deprecated wrapper.

`UnitGetTotalAbsorbs` still returns a fixed zero, `UnitStagger` is a temporary default and pet cooldown queries are defaults. These are publication only, not modeled behavior credit. The 2013 absorption/stagger/cooldown producer contracts remain pending.

## Historical verification

Publication/factory cases pass 58/58; own prefork 2/2, raid integration 1/1, cooldown integration 13/13, school integration 1/1 and cached school-alias prefork 1/1 pass. Negative control changes 57 → 58 gaps and fails as required. All 89 Python fixtures pass. All 59 registers reproduce byte-identically; 56 saved extracts reproduce, with inherited 12.0.5/12.0.7/12.1.0 failures unchanged. No `src/` runtime edits: master-runtime/lib/startup comparison is therefore not triggered. Scoped register lib cases pass 3/3; format and Mists `--no-default-features --features sound,gui,casc,client-mists --tests` checks pass. Mists has zero non-vendor warnings (six inherited iced_wgpu manifest deprecations remain untouched). Historical portability gate passes at `676fb8b24`: clean 44/44 and unrelated later-audit 45/45 (including the synthetic validator), zero failures. Own-log tampering fails on the exact sealed hash; original log bytes are restored. All 285 own artifact seals remain unchanged.

[Accounting](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/accounting-summary.json), [gap reasons](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/gap-review.json) and [retirement decisions](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/retirement-decisions.json) retain the limits. No full integration suite, new runtime shim or vendor change.

## Integrated verification

Base: `5e4e82ef66b239da8ae23a8dbb6b276213e50c71`. Exact later removals resolve only `GetArenaTeamIndexBySize` (5.4.0 `diff-wt-global-api-GetArenaTeamIndexBySize-58`) and `PrepVoidStorageForTransmogrify` (5.3.0 `diff-wt-global-api-PrepVoidStorageForTransmogrify-28`): **57 → 55 publication gaps**, no new gaps. Every other 5.2.0 observation is unchanged. These rows prove current absence only, not historical producer behavior or new retirements. Original receipts, gap reasons and accounting remain historical.

All 66 registers and 63 main extracts reproduce with recorded flags, including `--mists-bare-widget-handlers`; the three inherited extract failures remain unchanged. The separate 5.4.0 diff extract also reproduces. `src/` and vendor trees equal pinned master, so no startup/runtime-diff gate is triggered. Acceptance results and branch/master comparisons are retained in [integrated receipts](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/integrated/command-ledger.md); retail sweeps pass 62/62 on branch and 61/61 on pinned master, Mists sweeps/line control 6/6 on both. All 65 other retail/Classic pages and 9,990 observations equal master. Own prefork 2/2 and integration 1/1 pass; all 98 Python fixtures, format and Mists check pass with zero non-vendor warnings. Negative control rejects exactly 55 → 56 gaps. All 294 historical artifacts remain unchanged. The 15,523-byte mapping uses commit patch IDs and directory tree pins, never per-file inventories; largest evidence file is 568,492 bytes. Final portability gate pending.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/5.2.0-api-changes.provenance.json).
- [Evidence](../../../data/patch-api/evidence/5.2.0-session-2026-10-08/context.json) — base and parser-origin revisions.
- [Spec](../../specs/patch-5-2-0-publication-sweep.md).
- [Sweep](../../../tests/patch_5_2_0_publication_sweep.rs).

## See Also

- [[patch-5-3-0-api-audit]] — next retail register and attributable transmog removal.
- [[patch-5-4-0-api-audit]] — attributable arena-team removal.
- [[patch-audit-validator-portability]] — historical-scope and clean/later-audit gates.
