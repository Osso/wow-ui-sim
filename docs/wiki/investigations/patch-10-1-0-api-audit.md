# Patch 10.1.0 API page audit

Page 230704, revision 2236681 (June 15, 2023, 22:35:50 UTC), retrieved October 7, 2026 UTC. Branch `p1010-page` starts from master `7745f013e`. Default retail carries 12.1.0, not reconstructed 10.1.0.

## Source accounting

129 inventory occurrences: 93 added, 28 removed, eight changed. All published header counts match actual inventories. Twenty-one chronological later registers, 10.1.5 through 12.1.0, govern current publication.

416 unique source IDs: 129 inventory + 287 extract occurrences. Ledger statuses: 41 bounded-coverage, 52 partial-development-green, 274 audit-pending, 49 metadata-only. Pending rows comprise 36 exact publication gaps and 238 substantive extract occurrences. Metadata includes headings, source context, example fences, filenames and table scaffolding; each remains accounted without runtime credit. One CVar string-default mismatch (`nameplateGameObjectMaxDistance`: page `30`, current `30.000000`) has equal numeric values and grants no historical default-parity claim. Signatures, populated DTOs, security and event payloads are not proved by publication/registration.

Extractor initially dropped all 103 nonblank occurrences in the level-two Structures tail. A behavioral RED fixture reproduces the omission. Extending the existing level-two inventory-exit rule to Structures retains the enum/constant/DTO tail without new rendering rules. Lua/XML examples remain verbatim. All prior saved extract success/failure results stay identical with and without `--preserve-examples`. The three pre-existing 12.0.5/12.0.7/12.1.0 capture failures remain untouched; examples on other pages require their existing capture mode.

## Bounded closures

Discovery: 83 OK / 46 gaps. Final: 93 OK / 36 gaps. Ten unused namespace autostubs are retired through the existing retail-only module gate:

- C_CharacterServices: AssignPFCDistribution.
- C_LootHistory: CanMasterLoot, GetExpiration, GetItem, GetNumItems, GetPlayerInfo, GiveMasterLoot, SetExpiration.
- C_TooltipInfo: GetQuestLogRewardSpell, GetQuestRewardSpell.

[Qualified and bare cached searches](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-retirement-consumers.json) retain all matches. No qualified retired-member consumers exist. Bare-name matches refer to unrelated APIs: for example global GiveMasterLoot at `Blizzard_UIPanels_Game/Mainline/GroupLootFrame.lua:781`, CommunitiesTicketManagerDialogMixin:SetExpirationTime at `Blizzard_Communities/CommunitiesTicketManagerDialog.lua:412`, and current C_QuestInfoSystem reward queries at `Blizzard_UIPanels_Game/Mainline/QuestInfo.lua:618,655`. These are not retired namespace calls.

[Whole src/tests caller scan](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-final-whole-caller-scan.txt) keeps untruncated bare-name results, including embedded Lua, function references and guards. No pre-existing retail callers of the ten qualified members require migration. Mists' CanMasterLoot shim remains classic-only and unchanged. Existing loot-history successor shapes, repeated namespace lookup and one cached Game prefork pass. No Blizzard deprecation wrappers, cache files or vendor behavior changed.

## Retained gaps and proof boundaries

[Per-ID review](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-gap-review.json) assigns every one of the 46 discovery failures exactly once: ten closed, 36 retained. Calendar transformations, roster-name matching, account service transitions, remote talent viewing, chat colors, map/vignette projection, quest reward producers, upgrade-track/high-watermark/binding state, recraft/enchant validation, generic-widget/party-pose/trial/spectating lifecycle and item-display DTOs require real modeled contracts rather than fabricated publication. Sound-entry count needs catalog data; pcallwithenv needs protected VM call-environment semantics. Two front-end ModelSceneActor methods remain at the permanent unsupported 3D boundary.

GetAddOnMetadata retains a modeled legacy registration and explicit retail test consumers (`tests/addon_api.rs:422,427`, `tests/patch_10_2_0_publication_fixes.rs:41`). Coordinated profile gating/caller migration is separate from unused autostub retirement; this bounded audit preserves it as an exact gap. Cached uses are C_AddOns successor calls, not evidence of old-global use. GetSpellLinkFromSpellID is superseded to absent by a later register, but existing SpellBook transition-alias retirement policy deliberately preserves that family. Ordinary fallback still fabricates the member; no absence credit granted.

[Extract scout](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-extract-scout.json) assigns all 287 extract occurrences once with literal text, line, section and proof boundary. Private aura secrecy/display/sound contracts, addon compartment callbacks/icon precedence, local-only chat link dispatch, Lua/XML scroll examples, deprecated-template/renamed-handler behavior, numeric enum/constant identities and populated DTO field changes remain unproved. Later subsystem tests are not credited as occurrence-specific proof.

## Verification

Runtime/test revision `338075b03`; extractor revision `c47c60c45`. Twenty-two isolated sweeps pass exact fixtures; table below. Bare retirement RED/GREEN, retained loot-history successor behavior and one cached Game prefork pass. Whole-tree caller scan identifies no other affected direct callers/prefork cases.

Negative control flips only AnimationGroup:GetElapsed from added to removed: exactly one new failure, no resolved failures, 36 → 37, expected exit 101. Twenty extractor and six parser fixtures pass after observed extraction RED. All 22 registers regenerate byte-identically; 134 pre-existing source/register/ledger/gap-fixture inputs remain byte-identical. Before/after extract checks preserve every prior result under both modes.

Cargo fmt and fmt --check, default cargo check, Mists `--tests` check and separate retail build pass. Mists has zero non-vendor warnings; six iced vendor manifest deprecations and their summary remain unsuppressed. Bounded startup exits zero and returns `[]`. Changed Rust manually audited for readability: flat retirement data, one registration call and bounded assertions; no changed-line violations.

[Proof ledger](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-proof.json) records exact revisions/scopes, commands, logs, hashes and expected exits. Saved outputs inspected rather than rerun. Documentation/data-only follow-up commits preserve compiled proof scopes. Local targeted evidence is not native or independent acceptance. No agents or model CLIs, full suite, push, merge, canonical working-file changes, sibling target reuse or vendor/cache edits. Every command has explicit cwd `p1010-page` and uses its own target; worktree creation used only prescribed canonical Git metadata with cwd in the empty destination.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.1.0 | 129 | 93 | 36 | 0 |
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

## Sources

- [Provenance](../../../data/patch-api/sources/10.1.0-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.1.0-page-coverage.json)
- [Publication contract](../../specs/patch-10-1-0-publication-sweep.md)
- [Preservation](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-preservation.json)
- [Register reproduction](../../../data/patch-api/evidence/10.1.0-session-2026-10-07/p1010-register-reproduction.json)

## See Also

- [[patch-10-1-5-api-audit]] — publication/accounting template.
- [[patch-10-1-7-api-audit]] — cached UI and extract proof boundaries.
- [[client-profiles]] — retail/classic boundaries.
