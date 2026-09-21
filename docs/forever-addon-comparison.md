# Forever addon comparison audit

## Scope and evidence

The initial request covered all projects in the CurseForge Forever / 1.60.1 catalog. Acquisition stopped after the user raised bulk-download concerns and authorized use of existing downloads. This pass now uses cached material offline; the unacquired remainder is parked. A proposed top-20 total-download ranking was not acquired and is not claimed. Compare distinct Forever and non-Forever packages where available; for shared packages, inspect pre/post-support releases and the actual introduction commits. Addon adaptations identify hypotheses, not native API contracts. Simulator changes require a reproduced producer discrepancy backed by the pinned Forever UI/API documentation or stronger evidence.

The [catalog](../data/forever-addon-audit/catalog.json) records browser-cli public-page provenance, UTC capture times, project identities, and page coverage. Its 44 pages contain 875 unique projects without duplicate rows. Each page reported 875; an end-of-run first-page check retained the count and ordering. This is a live-catalog observation, not an atomic snapshot or a claim that every tagged addon works.

## Coverage

| Work | Coverage | Evidence level |
| --- | --- | --- |
| Catalog enumeration | 875 projects, 44/44 pages | Public rendered catalog; archived provenance |
| File metadata | 875/875 catalog projects | Public file-page metadata, not downloaded packages |
| Cached acquisition | 477 successful archives across 269 projects; 3 failed attempts | [Archive identities/hashes and candidate pairs](../data/forever-addon-audit/cached-comparisons.json) |
| Package differences | 208 complete cached pairs diffed | Exact archive hashes; release pairing does not prove introduction ancestry |
| Static triage | 208/208 cached pairs classified | [Forever-hunk/API-reference triage](../data/forever-addon-audit/triage.json); unrelated large changes not fully reviewed |
| Remaining acquisition | 61 cached projects lack a complete pair; 606 catalog projects have no successful cached archive | No further acquisition; no comparison/compatibility credit |
| Simulator corrections | BagIndex committed; fishing cursor reproduction underway | Targeted behavior only; independent verification pending |
| Full addon compatibility | Not established | Tags, static diffs, and startup alone are insufficient |

## Initial findings

- **BetterBags:** introduction commit `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` replaces hard-coded bank-tab lists with contiguous `Enum.BagIndex` enumeration. Pinned Forever `BagIndexConstantsDocumentation.lua` confirms character tab IDs 6–14 and account tab IDs 15–23. Commit `bb83a4c0a` corrects the Forever-only publication and metadata. Two focused behavioral tests went RED then GREEN; independent final verification remains pending. The addon's private `hasWarbank` flag and its event-unregistration workaround are not simulator API requirements.
- **DBM:** Forever introduction `6ffd4a1136e806177e1496cc5649b085b6be0c32` and subsequent fixes distinguish restricted Mainline behavior from content-era decisions. Its TOC conditionals are already supported. Avoid inferring API removal from optional calls: pinned Forever documentation still includes the ChallengeMode and Encounter Timeline surfaces that DBM elects not to use in some paths.
- **Auctionator:** file `8925257` carries Forever-aware copper pricing. Current simulator pricing already preserves copper values; no discrepancy established. Source-repository acquisition failed, so a current-package inspection is not credited as a historical diff.

## Candidate dispositions

Static triage classified 58 packaging-only, 8 data-only, 33 mixed packaging/data-only, 30 with no new gap established by source inspection, 46 confounded pairs, 25 needing deeper review, and 8 runtime candidates. These are triage labels, not runtime acceptance.

| Candidate | Disposition | Reason |
| --- | --- | --- |
| BetterBags enum shape | Implemented; verification pending | Exact authored IDs and contiguous enumeration prove the publication mismatch |
| BetterBags/Camelot bank capacities | Deferred | Nine enum members do not prove purchased tabs. Current purchased count is zero; inventing slot capacities would not fix the missing bank-state model |
| EasyFishing cursor transfers | Reproduction/implementation underway | Exact cached consumer calls namespace pickup, inventory pickup, then namespace drop. Existing namespace is a no-op; legacy operations overwrite held items |
| BeastAndBow ammo counts | Deferred | Addon claim lacks independent aggregate-count evidence. Its separate claim that Forever disallows event registration conflicts with authored Camelot consumers |
| AvoidanceStats | Deferred | UI rewrite does not isolate a producer failure |
| Ackis cooking/core | No fix justified | Changed API references alone do not demonstrate failure in existing surfaces |
| ChatBarBlocks/BugSack restrictions | Deferred | Documentation and defensive addon guards do not establish the missing lockdown/secret state transitions; no nil fallbacks or inferred API removals added |

## Acquisition rules

Acquisition is stopped for this pass. Existing archives remain local; no risk/flagging status is known. The successful count is archives, including comparison versions, not unique addons.

The completed acquisition used public browser pages for catalog and file metadata. The installed `curseforge` CLI currently exposes packaging/upload operations, not addon search or download; no publishing operations belong to this audit. Obtain archives through the public download link exposed by the file/download page and record their hashes. Do not execute package installers or change the user's installed addon directories.

Keep missing historical counterparts, unpaired forks, failed downloads, and insufficient runtime evidence explicit. Do not turn any of them into a completed comparison. Keep earlier 12.x audit/probe preparation separate.

## Sources

- CurseForge catalog: `https://www.curseforge.com/wow/search?class=addons&page=1&pageSize=20&sortBy=a-z&version=1.60.1&gameVersionTypeId=88568`
- BetterBags: `https://github.com/Cidan/BetterBags/commit/411a6f6ee1ea40eca8ac96927ccdd49a6aab3941`
- DBM: `https://github.com/DeadlyBossMods/DeadlyBossMods/commit/6ffd4a1136e806177e1496cc5649b085b6be0c32`
- Auctionator: `https://www.curseforge.com/wow/addons/auctionator/files/8925257`
- [Forever runtime report](wowforever-1.60.1.md)
