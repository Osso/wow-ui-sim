# Forever addon comparison audit

## Scope and evidence

Audit all projects in the captured CurseForge Forever / 1.60.1 catalog, not a popularity sample. Compare distinct Forever and non-Forever packages where available; for shared packages, inspect pre/post-support releases and the actual introduction commits. Addon adaptations identify hypotheses, not native API contracts. Simulator changes require a reproduced producer discrepancy backed by the pinned Forever UI/API documentation or stronger evidence.

The [catalog](../data/forever-addon-audit/catalog.json) records browser-cli public-page provenance, UTC capture times, project identities, and page coverage. Its 44 pages contain 875 unique projects without duplicate rows. Each page reported 875; an end-of-run first-page check retained the count and ordering. This is a live-catalog observation, not an atomic snapshot or a claim that every tagged addon works.

## Coverage

| Work | Coverage | Evidence level |
| --- | --- | --- |
| Catalog enumeration | 875 projects, 44/44 pages | Public rendered catalog; archived provenance |
| File acquisition and pairing | In progress across full catalog | File IDs, versions, flavors, and timestamps required |
| Package/source differences | In progress | Exact archives/hashes or parent/commit pairs required |
| Simulator corrections | In progress | RED/GREEN behavioral proof required |
| Full addon compatibility | Not established | Tags, static diffs, and startup alone are insufficient |

## Initial findings

- **BetterBags:** introduction commit `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` replaces hard-coded bank-tab lists with contiguous `Enum.BagIndex` enumeration. Pinned Forever `BagIndexConstantsDocumentation.lua` confirms character tab IDs 6–14 and account tab IDs 15–23. Commit `bb83a4c0a` corrects the Forever-only publication and metadata. Two focused behavioral tests went RED then GREEN; independent final verification remains pending. The addon's private `hasWarbank` flag and its event-unregistration workaround are not simulator API requirements.
- **DBM:** Forever introduction `6ffd4a1136e806177e1496cc5649b085b6be0c32` and subsequent fixes distinguish restricted Mainline behavior from content-era decisions. Its TOC conditionals are already supported. Avoid inferring API removal from optional calls: pinned Forever documentation still includes the ChallengeMode and Encounter Timeline surfaces that DBM elects not to use in some paths.
- **Auctionator:** file `8925257` carries Forever-aware copper pricing. Current simulator pricing already preserves copper values; no discrepancy established. Source-repository acquisition failed, so a current-package inspection is not credited as a historical diff.

## Acquisition rules

Use public browser pages for catalog and file metadata. The installed `curseforge` CLI currently exposes packaging/upload operations, not addon search or download; no publishing operations belong to this audit. Obtain archives through the public download link exposed by the file/download page and record their hashes. Do not execute package installers or change the user's installed addon directories.

Keep missing historical counterparts, unpaired forks, failed downloads, and insufficient runtime evidence explicit. Do not turn any of them into a completed comparison. Keep earlier 12.x audit/probe preparation separate.

## Sources

- CurseForge catalog: `https://www.curseforge.com/wow/search?class=addons&page=1&pageSize=20&sortBy=a-z&version=1.60.1&gameVersionTypeId=88568`
- BetterBags: `https://github.com/Cidan/BetterBags/commit/411a6f6ee1ea40eca8ac96927ccdd49a6aab3941`
- DBM: `https://github.com/DeadlyBossMods/DeadlyBossMods/commit/6ffd4a1136e806177e1496cc5649b085b6be0c32`
- Auctionator: `https://www.curseforge.com/wow/addons/auctionator/files/8925257`
- [Forever runtime report](wowforever-1.60.1.md)
