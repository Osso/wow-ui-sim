# Patch 10.0.7 API page audit

Page 108565, revision 1063344 (May 9, 2023, 20:31:43 UTC), retrieved October 7, 2026 UTC. Branch `p1007-page` starts from master `7745f013e`. Default retail carries 12.1.0, not reconstructed 10.0.7.

## Source accounting

70 inventory occurrences: 45 added, 24 removed, one changed. Global API 28/18, events 6/4 and CVars 11/2 match every published header; Widgets says None. Twenty-one chronological later registers from master (10.1.5 through 12.1.0) govern current publication. A one-line placeholder reserves 10.1.0 at the list start; sibling work is not a dependency.

2,284 unique source IDs: 70 inventory + 2,214 extract occurrences. Ledger statuses: 28 bounded-coverage, 16 partial-development-green, 2,229 audit-pending, 11 metadata-only. Pending comprises 26 exact publication gaps and all 2,203 substantive extract occurrences. Publication/absence/event registration does not prove signatures, populated DTOs, security, event payloads or historical/native parity. Seven missing VALAR CVars also lack page-default matches; none is credited as a supported current graphics setting.

Source loss was upstream of runtime probing. The inventory parser replaced its event bucket when encountering the later `===Events===` heading under Type Changes, dropping ten publication rows. The extractor failed to stop inventory mode at level-two Structures, dropping 1,661 nonblank statements. Two behavioral RED fixtures reproduce these boundaries. Parser stops before Type Changes; extractor reuses the identical Structures exit present in the read-only 10.1.0 worktree. No new rendering syntax or vendor mutation. Final extract covers 42 initial Structures occurrences, 1,092 type-function occurrences, 526 type-event occurrences, 545 type-structure occurrences, summary/resources and headings.

## Bounded closures

Discovery 26 OK / 44 gaps. Final 44 OK / 26 gaps. Eighteen unused namespace autostubs are retired through the existing retail-only module gate:

- C_QuestOffer.GetHideRequiredItemsOnTurnIn.
- Seventeen C_Social historical Twitter/screenshot/last-item members, enumerated in [per-ID review](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-gap-review.json).

[Qualified and bare cached Lua searches](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-removal-consumers.json) find no matches for any retired member. [Whole src/tests caller scan](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-whole-caller-scan.json) finds only three existing 11.0.0 gap-ID references, not callers; embedded Lua, function references and guards are included. No existing callers require migration and no existing prefork cases call these names. C_Social.GetFriends/GetFriendInfo remain published. Three historical 11.0.0 changed-member rows remain exact gaps; their changed-row expectations do not assert absence.

Repeated bare-environment lookup and one unmodified cached Game prefork prove absence. No Blizzard deprecation wrapper was deleted or rewritten. Classic exclusion remains at the existing module gate; Mists preservation assertions await execution.

## Retained gaps

[Per-ID review](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-gap-review.json) assigns all 44 discovery failures once: 18 closed, 26 retained. Indexed chat identity/censor state and deferred whisper permission, console-script catalogs, gossip option selection, per-creature pet caps/counts, loose-target unit interaction, queue-assigned specialization, quest required-item visibility, title-icon lifecycle, trait commit readiness and actual graphics-backend reporting require modeled producers, not inert publication. Seven VALAR tuning CVars lack supported current registry/host backing.

GetNumPetsInJournal is not a GetNumPets alias: cached `Blizzard_APIDocumentationGenerated/PetJournalInfoDocumentation.lua:91-103` declares creatureID and maxAllowed/numPets. Existing GetNumPets counts all world.pets and collected entries (`pet_journal.rs:35-40`); a per-creature cap/identity contract is missing. [Native declaration evidence](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-pet-journal-native-contract.txt).

[Extract scout](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-extract-scout.json) accounts for every literal nonblank occurrence with line, parent, section and proof boundary. Restricted advflyable macro behavior, PNG asset loading, enum identities/renames and populated DTO/type-label acceptance remain unproved. A type-label annotation is not proof that runtime gained a separate Rust type. Existing subsystem tests are not credited as source-specific coverage.

[Read-only 10.1.0 comparison](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-possible-1010-supersessions.json) finds no symbol/section intersection with the 26 retained gaps. Integration must still add that register and recompute fixtures; sibling runtime changes are not imported.

## Verification

Runtime revision `484b78646`. Twenty-two isolated publication sweeps, repeated retirement RED/GREEN and one cached Game prefork pass. Parser/extractor: 27 GREEN after two observed RED failures. Remaining requested checks and artifact verification are pending; [proof ledger](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-proof.json) tracks exact commands, revisions/scopes and saved outputs.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.0.7 | 70 | 44 | 26 | 0 |
| 10.1.5 | 101 | 69 | 32 | 0 |
| 10.1.7 | 48 | 34 | 14 | 0 |
| 10.2.0 | 150 | 120 | 30 | 0 |
| 10.2.5 | 59 | 45 | 14 | 0 |
| 10.2.6 | 220 | 200 | 20 | 0 |
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |

Changed Rust manually audited: flat retirement data, one registration call and bounded lookup assertions; no changed-line readability violations. No full suite, agents/model CLIs, push, merge, sibling target reuse or canonical working-file/vendor/cache changes. Every command uses explicit cwd `p1007-page` and its own target; worktree creation performed only the prescribed canonical Git metadata operation with cwd in the empty destination.

## Sources

- [Provenance](../../../data/patch-api/sources/10.0.7-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.0.7-page-coverage.json)
- [Publication contract](../../specs/patch-10-0-7-publication-sweep.md)
- [Before extraction checks](../../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-extract-before.json)

## See Also

- [[patch-10-1-5-api-audit]] — publication and accounting template.
- [[patch-10-1-7-api-audit]] — cached runtime and extract proof boundaries.
- [[client-profiles]] — retail/classic module gates.
