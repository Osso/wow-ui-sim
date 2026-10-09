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

Acceptance pending; receipts and the portable validator will record exact revisions, scope, derived counts, failures and immutable log seals. Required gates: all publication sweeps, own bare/prefork and journal/namespace caller controls, library comparison to pinned master, matching addons-enabled startup errors, Python fixtures, every saved reproduction, warning-clean non-vendor Mists check and format. No full integration suite, push, merge or delegation.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/5.1.0-api-changes.provenance.json).
- [Session evidence](../../../data/patch-api/evidence/5.1.0-session-2026-10-08/).
- [Spec](../../specs/patch-5-1-0-publication-sweep.md).

## See Also

- [[patch-5-4-2-api-audit]] — retained 2013 table parser and proof template.
- [[patch-audit-validator-portability]] — historical register and clean/later gates.
