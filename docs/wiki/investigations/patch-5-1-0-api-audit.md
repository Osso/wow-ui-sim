# Retail Patch 5.1.0 API audit

Verified source on 2026-10-08: API pageid 282785 revision 2736212, transcluded diff pageid 472925 revision 4546247. Parent pageid 30231 revision 6544843 states `toc = 50100`, build 16309, release November 27, 2012. This is retail Mists of Pandaria, not Mists Classic 5.5.x. The API page is neither redirect nor stub; its inventory lives on a separately fetched diff.

## Source and accounting

63 inventory occurrences include 12 explicit summary references and 51 diff members; 21 extract rows retain prose and the transclusion boundary. Every source ID is classified in the [ledger](../../../data/patch-api/sources/5.1.0-page-coverage.json). Summary `added` labels identify references in New features, not newly introduced native helpers: restricted-environment availability and cooldown visibility remain separate pending contracts.

The summary parser is copied byte-identically from pinned p540-page; owner-specific Widget Handlers parser from pinned p520-page. [Provenance](../../../data/patch-api/evidence/5.1.0-session-2026-10-08/p510-parser-provenance.json) records commits. All new behavior is opt-in. The main extract deliberately does not inline the diff; the register does include its pinned inventory.

## Coverage boundary

- Current publication/absence: 46 bounded API occurrences, 17 pending occurrences.
- Existing meaningful behavior: concrete action-to-spell loss-control interval read, replacement, clear via cached legacy wrapper. No native CC producer or visible notification credit.
- Prose: six metadata rows, fifteen pending statements. Pet GUID ownership/name search/random favorites, instance/home chat routing, restricted execution security and automatic Cooldown show/hide are not proven.
- Precise missing backing systems and current consumers: [gap review](../../../data/patch-api/evidence/5.1.0-session-2026-10-08/p510-gap-review.json). No placeholder-return shim added.

## Retirements

Two consumer-free ID spellings, `C_PetJournal.GetSummonedPetID` and `SummonPetByID`, were fabricated by namespace autostub lookup despite being raw-absent. Marking them in the existing retail C API retirement registry prevents synthesis; GUID successors remain published. Full-prefork RED reproduces the real loaded lookup boundary.

Four removed globals/events and two Animation/AnimationGroup OnEvent handlers are already absent, requiring no runtime change. Bare OnEvent hits are retained, not treated as permission to retire unrelated frame scripts. Preserve `C_LossOfControl.GetEventInfo`: cached consumers remain. Preserve `InActiveBattlefield`: source/test caller remains. `GetNumEvents` is an existing later-page gap, not a retirement in this source.

Whole-word `/usr/bin/grep` scans retain qualified/bare retail results (excluding Documentation) and untruncated src/tests results, including conditional/function-value calls. Pinned master and all four queued retail branch register trees were checked for re-additions. No new Classic retirement or supersession.

## Verification

Targeted runtime/tool proof completed at pinned revisions. Portable gate PASS at `460f599815b9c6104d22743b6dc48bc6a6e08093`: clean 47/47 and synthetic later-audit 48/48, zero failures. Own validator seals 304 session inputs and proves 329 shared inputs at recorded revisions; own-log tampering is rejected and original bytes restored. Publication sweeps: branch 59/59, master 58/58; all 9,817 observations on 57 other retail pages identical. Own prefork 3/3 and bare retirement 1/1. Journal/namespace integration controls 19/13/2 pass on both revisions; prefork pet controls 22 branch/21 master (one new absence case). Namespace library controls 24/24 on both. The touched retirement module's library selector has zero unit cases on both: no coverage credited; bare/full-LoD behavior supplies its positive proof. All 92 Python fixtures, cargo format and Mists tests check pass with zero non-vendor warnings. 61 registers and 58 extracts reproduce; inherited failures remain exactly 12.0.5, 12.0.7 and 12.1.0. The committed negative input creates exactly one extra gap, 17 → 18.

Addons-enabled startup is **not clean**: both pinned master and branch report the same 505 unique errors / 519 occurrences, exit 1. A same-source repeat is identical. Acceptance uses the complete frozen 6,658-file addon tree, bytecode caching disabled, and both binaries compiled in the same worktree path. No addon enable-state filtering or source patching. Earlier live/archive-root comparisons were not equivalent and are retained as failed probes, not accepted proof; input drift was suspected, not established. The frozen manifest is unchanged. Existing third-party failures remain outside this audit.

All runtime changes are the two retirement markers. No new gameplay model, vendor change, full integration suite, push, merge or delegation. The existing action/spell interval model supplies meaningful bounded read/update/clear behavior, not native 2012 parity.

## Integration refresh

Rebased onto master `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a`. Both merged parser behaviors remain available; `--mists-code-removals` stays opt-in. Real retail 5.2.0, 5.3.0, 5.4.0 and 5.4.1 registers replace queued placeholders. Classic 5.5.x remains outside the retail successor chain. Eleven own/four external original/rebased identities and original artifacts are preserved in [integrated evidence](../../../data/patch-api/evidence/5.1.0-session-2026-10-08/integrated/). The unchanged historical validator is archived and replayed through mapped pins; exact source-directory tree hashes derive from existing historical inventories, never a new per-file mapping. Original-pin denial passes with all fifteen old identities blocked.

Every 5.1.0 observation remains identical: **17 → 17 gaps**, no replacements or new gaps; known-gap fixture and occurrence ledger unchanged. Refreshed qualified/bare whole-word `rg` scans find zero cached consumers and only retirement/test literals in all `src/` and `tests/`. Retail-only registration remains intact. Worktree-local retail sweeps 63/63, own prefork 3/3, own integration 1/1, journal/namespace integration 19/13/2, prefork pet 22/22 and library namespace 24/24 pass. Mists sweeps 6/6, all 101 Python fixtures, format and Mists check pass; zero non-vendor warnings. All 67 registers/64 extracts and the separate 5.4.0 diff reproduce; three inherited extract failures unchanged. Negative rejects exactly 17 → 18. Same-worktree master comparisons and final portable/tamper gates remain pending; no canonical/sibling/vendor writes, push, merge or delegation.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/5.1.0-api-changes.provenance.json).
- [Session evidence](../../../data/patch-api/evidence/5.1.0-session-2026-10-08/).
- [Spec](../../specs/patch-5-1-0-publication-sweep.md).

## See Also

- [[patch-5-4-2-api-audit]] — retained 2013 table parser and proof template.
- [[patch-audit-validator-portability]] — historical register and clean/later gates.
