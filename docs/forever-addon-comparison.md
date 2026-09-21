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
| Simulator corrections | Independently verified at `d40397025` | 34 focused tests, format/readability, default/Forever checks, and Forever startup pass |
| Full addon compatibility | Not established | Tags, static diffs, and startup alone are insufficient |

## Initial findings

- **BetterBags:** introduction commit `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` replaces hard-coded bank-tab lists with contiguous `Enum.BagIndex` enumeration. Pinned Forever `BagIndexConstantsDocumentation.lua` confirms character tab IDs 6–14 and account tab IDs 15–23. Commit `bb83a4c0a` corrects the Forever-only publication and metadata. Two focused behavioral tests went RED then GREEN; the final eight-test Forever constants module passes independently. The addon's private `hasWarbank` flag and its event-unregistration workaround are not simulator API requirements.
- **DBM:** Forever introduction `6ffd4a1136e806177e1496cc5649b085b6be0c32` and subsequent fixes distinguish restricted Mainline behavior from content-era decisions. Its TOC conditionals are already supported. Avoid inferring API removal from optional calls: pinned Forever documentation still includes the ChallengeMode and Encounter Timeline surfaces that DBM elects not to use in some paths.
- **Auctionator:** file `8925257` carries Forever-aware copper pricing. Current simulator pricing already preserves copper values; no discrepancy established. Source-repository acquisition failed, so a current-package inspection is not credited as a historical diff.

## Candidate dispositions

Static triage classified 58 packaging-only, 8 data-only, 33 mixed packaging/data-only, 30 with no new gap established by source inspection, 46 confounded pairs, 25 needing deeper review, and 8 runtime candidates. These are triage labels, not runtime acceptance.

| Candidate | Disposition | Reason |
| --- | --- | --- |
| BetterBags enum shape | Implemented and verified | Exact authored IDs and contiguous enumeration prove the publication mismatch |
| BetterBags/Camelot bank capacities | Deferred | Nine enum members do not prove purchased tabs. Current purchased count is zero; inventing slot capacities would not fix the missing bank-state model |
| EasyFishing cursor transfers | Implemented and verified | Exact cached three-call sequence reproduced 0/4, then 25/25 focused inventory tests passed. Shared transfer operations replace the namespace no-op and preserve displaced item IDs/counts; [spec and existing model limits](specs/cursor-item-transfer.md) |
| BeastAndBow ammo counts | Deferred | Addon claim lacks independent aggregate-count evidence. Its blanket event-registration prohibition is not adopted as simulator policy; authored Camelot consumers register those events |
| AvoidanceStats | Deferred | UI rewrite does not isolate a producer failure |
| Ackis cooking/core | No fix justified | Changed API references alone do not demonstrate failure in existing surfaces |
| ChatBarBlocks/BugSack restrictions | Deferred | Documentation and defensive addon guards do not establish the missing lockdown/secret state transitions; no nil fallbacks or inferred API removals added |

## Independent verification

Final Rust source `d40397025` passed independent offline verification:

- 25 inventory-transfer tests, 8 Forever finite-constant tests, and 1 existing container-shape regression: **34/34**.
- `cargo fmt --check`, default and Forever `cargo check --offline`, and changed-code readability: pass, no compiler warnings or readability findings.
- Separately built Forever `wow-sim --no-addons --no-saved-vars lua-errors`: exit 0, stdout `[]`.
- Existing unchanged WorldMap/SpellBook sustained-lifecycle proof reused with its original scope; no fresh full-addon or GUI-runtime claim.
- Previously verified catalog, 477 indexed archive hashes, and 208 diff hashes reused unchanged. Full ledger: `/tmp/verify-forever-addon-final-ledger.json`; prior artifact ledger: `/tmp/verify-forever-addon-cache-ledger.json`.

This verifies the initial two-correction checkpoint, not complete Forever compatibility. Further cached-consumer investigation is recorded below. No implementation push or deployment was performed.

## Deeper cached-consumer follow-up

The 71 initially confounded/needs-deeper rows now have [source-branch dispositions](../data/forever-addon-audit/deep-dispositions.json). These identify changed call sites, existing counterparts, addon-policy/data changes, or specific missing evidence; they are not blanket compatibility claims.

Five actual cached packages were also loaded in isolated temporary addon roots using the already-built Forever binary, with SavedVariables disabled:

| Package | Observed boundary at `d40397025` | Disposition |
| --- | --- | --- |
| EpicDamageMeter `8930362` | Startup `[]`, but direct consumer assertion shows the modern meter branch was not selected | Correction `9862dc7b3` prevents public getter synthesis; Internal/Secure remain available. Actual current-source replay selects the modern meter path and completes 60 consumer updates with two visible named rows; live combat/secrets remain unproven |
| Carbonite `8926975` | Core provider absent, so three dependent addons fail | Selection `31ac46b3a` now loads the provider/dependents (44 focused selector/profile tests pass). Full package advances to a separate UIMap 2521 map-art boundary described below; it is not a full Carbonite pass |
| Baganator | Missing Syndicator, then missing LibStub supplied by that dependency | Dependency-blocked, not a simulator API defect. The repository's Syndicator directory is only a partial Search fixture without a TOC; no new download is authorized |
| CooldownMaster | Startup `[]` with the actual package loaded | Startup-only evidence; no full cooldown interaction claim |
| DragonGildMaster | Startup `[]` with the actual package loaded | Startup-only evidence; no guild-service or chat-restriction claim |

The combat-log finding also corrects an evidence trap: `Deprecated_CombatLog.lua` assigning a public member to a legacy global cannot prove that the member exists. `loadDeprecationFallbacks=1` does not make an absent source member callable. See the [client-oriented namespace explanation](wowforever-1.60.1-ui-api-deltas.md#2-combat-log-getter-ownership-do-not-reconstruct-a-public-superset).

Logs, exact commands, archive identities and isolated enable-state files are retained under `/tmp/forever-addon-runtime/<slug>/`. No vendor or addon source was edited for these probes.

### Carbonite's remaining map-art boundary

The live diagnostic identifies Zephras Isle, UIMap `2521`, as the third continent. Camelot `MapEngine.lua` first requests modern art for roots above 1000; both `C_Map.GetMapArtLayers(2521)` and `GetMapArtLayerTextures(2521, 1)` return nil in the current data set. Only then does the addon fall back to a missing legacy `FileName`. An earlier static trace blaming the filename alone was incomplete: the preceding modern-API path matters.

`data/db2/UiMapXMapArt.csv` has no 2521 row, and the acquired exact-build Forever CSVs cover atlases/global strings, not the four map-art tables. The installed Beta build is 69913, but there is no configured exact-build DB2-to-CSV/schema chain in this checkout: the map generator consumes pre-exported CSVs, while wowless's offline dumper needs extracted files and generated runtime definitions that are absent here. A dedicated map-data ingestion step or supplied exact-build exports is needed before deciding whether native 69913 provides this art and what IDs, dimensions and phases apply.

No legacy filename, tile IDs, layer dimensions, synthetic capacity, player-map change, or vendor nil guard is fabricated. Full Carbonite compatibility remains blocked at this concrete evidence/data boundary, independently of the corrected TOC selection. The map also exposes a separate Retail-seeded player-map policy (2248); changing that does not supply continent 2521's missing art.

### Follow-up verification

Independent verification at `572f1c23c` passes the bounded selector and namespace claims. Valid development proof was reused: 44 selector/profile tests and 7 combat-namespace/shared-state tests. The previously unexecuted Forever combat-navigation unit also passes, for 52 focused cases across these scopes. Fresh formatting, default/Forever offline checks, changed-function readability, and Forever startup (`[]`) pass. Eight unrelated pre-existing lib-test warnings remain; production `cargo check` emits none.

The verifier inspected and reused the actual Epic package startup/60-update consumer output and all 71 deeper dispositions. Carbonite's map-art boundary and Baganator's missing dependency remain explicitly unresolved, not hidden by fallback data. Ledger: `/tmp/verify-forever-followup-ledger.json`. Later documentation-only changes do not invalidate these source proofs.

## Acquisition rules

Acquisition is stopped for this pass. Existing archives remain local; no risk/flagging status is known. The 477 count denotes indexed CurseForge file versions, including comparison versions—not unique addons or HTTP requests. The browser download probe also produced a duplicate DBM archive outside the batch cache.

The completed acquisition used public browser pages for catalog and file metadata. The installed `curseforge` CLI currently exposes packaging/upload operations, not addon search or download; no publishing operations belong to this audit. Obtain archives through the public download link exposed by the file/download page and record their hashes. Do not execute package installers or change the user's installed addon directories.

Keep missing historical counterparts, unpaired forks, failed downloads, and insufficient runtime evidence explicit. Do not turn any of them into a completed comparison. Keep earlier 12.x audit/probe preparation separate.

## Sources

- CurseForge catalog: `https://www.curseforge.com/wow/search?class=addons&page=1&pageSize=20&sortBy=a-z&version=1.60.1&gameVersionTypeId=88568`
- BetterBags: `https://github.com/Cidan/BetterBags/commit/411a6f6ee1ea40eca8ac96927ccdd49a6aab3941`
- DBM: `https://github.com/DeadlyBossMods/DeadlyBossMods/commit/6ffd4a1136e806177e1496cc5649b085b6be0c32`
- Auctionator: `https://www.curseforge.com/wow/addons/auctionator/files/8925257`
- [Forever runtime report](wowforever-1.60.1.md)
