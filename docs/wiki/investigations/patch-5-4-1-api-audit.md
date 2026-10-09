# Patch 5.4.1 API audit

Pinned pageid 221650, revision 2150406 (2018-11-26T23:10:07Z), fetched 2026-10-08. This is retail Mists: parent revision 6428967 states release October 29, 2013, build 17538, TOC 50400; API captions compare 5.4.0.17359 with 5.4.1.17538. Neither redirect nor stub. No 5.5.x Classic source/register participates.

## Coverage matrix

| Contract | Proof / limit |
|---|---|
| 9 functions and 6 events | All 15 inventory occurrences probed; 13 publication/absence observations pass after later retail supersession, 2 reviewed gaps |
| `PRODUCT_CHOICE_UPDATE` | Unknown strict-catalog event; no retained product-choice selection producer after 8.3.0 namespace removal. Registering a historical name alone would be a shim, not event behavior |
| `SelectedRealmName` | Missing global; selected realm navigation/session state is not modeled. Existing player realm constant/account realm fields are not this state |
| `realmName` CVar migration prose | Existing CVar model reads return nil in default Game state; focused prefork assertion. Player realm replacement is a temporary constant; registration/rejection and real player realm identity remain pending |
| Protected `C_StorePublic.IsDisabledByParentalControls` bug | Publication absent by 12.0.0 supersession; no historical parental-control secure-service/taint policy proof |
| Existing `C_RecruitAFriend.GetRecruitInfo` | Function publication only; disabled-by-default probe implementation is not historical service behavior |
| Editorial context | Seven context rows; reference-list marker separately metadata-only, not an API contract |

No runtime code changed. No new modeled service API, shim or retirement. Default CVar absence is bounded proof of existing state reads, not a new realm model. Every saved inventory and extract ID is accounted exactly once in the [ledger](../../../data/patch-api/sources/5.4.1-page-coverage.json).

## Ordering and retirements

Integrated sweep uses merged retail 5.4.2, 5.4.7, 5.4.8 registers before 6.0.1, 6.0.2 and remaining later retail registers. Classic registers never participate in retail supersession; separate Classic sweeps belong to the cross-page regression proof.

Page contains no removed member. Whole-word `/usr/bin/grep` qualified and bare scans nevertheless cover all 15 inventory symbols plus `realmName`, `GetRealmName`, `GetCVar`: 72 untruncated saved outputs in the own session, excluding Documentation from cached retail sources. Recursive src/tests scans include indirect `pcall(Name, ...)` and conditional name uses without filtering them out. Master and queued p548/p547/p542 register trees are recorded at exact Git revisions, using `git ls-tree` and `git show`, not live worktree contents.

Current cached `Blizzard_MicroMenu/Classic/MainMenuBarMicroButtons.lua` consumes `C_StorePublic.IsDisabledByParentalControls`; this is a Classic-subfolder consumer under the retail cache and forbids a new retirement. Existing later-patch removal remains unchanged; this audit does not endorse it or claim historical secure semantics. No removal is introduced here.

## Tooling and verification

After rebase onto `279a38f3d`, generator is byte-identical to master: `--mists-automated-diff` comes from merged 5.4.2, not the rewritten 5.4.1 tool commit. Extractor is master plus 5.4.1's opt-in `--lowercase-reflist`; original copied-parser provenance remains historical. Separate `--lowercase-reflist` normalization retains this page's lowercase reference marker without changing default extraction. Behavioral fixtures prove opt-in handling; reproduction covers all historical registers and extract modes. Three inherited extract failures (12.0.5, 12.0.7, 12.1.0) must remain exactly unchanged.

The initial discovery launch preceded its commit and is explicitly invalidated as acceptance evidence. Committed-source proof replaces it. A recursive grep accidentally scanned its own redirected output; terminated, verified OS process exit, deleted incomplete output and rescanned only explicit source inputs. [Incident record](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/scan-incident.txt).

Final targeted receipts and clean/later-audit gate results are retained in the [session](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/). No full integration suite. Runtime caller/lib/startup comparisons are conditional on src changes; none are introduced by this audit. Validator seals only own session bytes, proves shared files at pinned revisions and derives historical sets/counts from Git/files.

## Historical targeted results

At committed source revision `8462a4c89`: 2/2 own prefork cases, 56/56 publication/factory cases, 86/86 Python fixtures and format pass. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with zero non-vendor warnings (six unchanged vendor manifest deprecations). All 55 registers and 52 extracts reproduce; three inherited extract failures remain unchanged. Negative control changes publication gaps from 2 to 3 and fails as required. No src changes trigger caller/lib/startup comparison gates. Sealed validator passes locally and rejects own-log tampering. `tools/check_patch_validators.py` at `e9135e73d` passes 37/37 clean validators and 38/38 after an unrelated synthetic later audit (including the synthetic validator). [Gate summary](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/gate-summary.json).

## Integrated refresh (2026-10-08)

[Integrated receipts](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/) pin master `279a38f3d`; [rebase mapping](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/rebase-mapping.json) preserves seven branch identities and three external pins using stable patch IDs, tree/blob hashes and archived original bytes. All 205 historical files remain unchanged except the entry validator, whose original bytes are retained and replayed against mapped pins without weakening its invariants.

All 61 registers and 58 extracts reproduce; three inherited extract failures (12.0.5, 12.0.7, 12.1.0) match master. Recorded 5.4.1 flags are generator `--mists-automated-diff`, extractor `--mists-automated-diff --lowercase-reflist`. All pre-audit source bytes and extract modes remain unchanged. Fresh sweep, negative, cross-page, warning and portability results pending.

## Sources

- [API source](../../../data/patch-api/sources/5.4.1-api-changes.wikitext) and [provenance](../../../data/patch-api/sources/5.4.1-api-changes.provenance.json).
- [Parent revision](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/parent.wiki).
- [Publication spec](../../specs/patch-5-4-1-publication-sweep.md).
- [Bounded gap review](../../../data/patch-api/evidence/5.4.1-session-2026-10-08/gap-review.json).

## See Also

- [[patch-6-0-2-api-audit]] — later retail inventory template.
- [[patch-audit-validator-portability]] — historical, relocated and synthetic-later-audit proof requirements.
