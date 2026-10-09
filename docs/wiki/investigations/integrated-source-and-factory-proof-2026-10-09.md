# Integrated source and factory proof — 2026-10-09

## Integration scope

Main-provided integration identifiers: p303 `4470912f5`, p302 `16b9658da`, p242 `0b3144090`, p240 `923b0c986`, p230 `48691e5a8`, p220 `c2481c4c5`, p210 `8877566d2`, p201 `88217d328`, p1601 `861e7ae5e`, p1159 `60524271c`, p1158 `dd710c6fc`. Integration alone gives no execution or native credit. Existing per-page source ledgers, immutable source/factory seals and counts remain authoritative; new source slices remain source-only unless explicit bounded proof says otherwise.

## Fixture corrections — bounded PASS at `9252c6cc9`

Two source-only chat/scenario placement tests were removed: broad module-name matches falsely rejected legitimate state-backed C API modules, while their remaining check read a deleted legacy shim file and could pass vacuously. Existing return-value and provider-preservation tests in `c_chat_info_defaults.rs` and `scenario_defaults.rs` remain unchanged. No runtime module moved or behavior changed; removal follows the prohibition on substituting source shape for behavior. This does not repair unrelated structural tests or establish a passing overall suite.

The housing compatibility probe now expects its separately modeled catalog searcher to start with count zero, rather than requiring invented variants. Type and count failures have separate sentinels; other compatibility assertions remain unchanged. Catalog registration is unconditional across profiles, and the existing catalog spec defines empty host-backed inputs. This is a test-expectation correction, not a runtime seed/fallback or native owned-instance-count claim.

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fixture-corrections/report.md) records default-Retail library tests **2/2 chat + 2/2 scenario + 1/1 housing**, workspace formatting and changed-Rust readability at `9252c6cc9`. Six existing manifest deprecation warnings remain. Integration-test deletion was source-inspected, not freshly compiled/listed; no full-suite or alternate-profile acceptance. [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fixture-corrections/main-retention.json) preserves 30 top-level artifacts byte-for-byte; no commands rerun.

## Era standalone target gate — restoration

The 1.13.2 target insertion accidentally left `patch_1_13_3_npc_health` without its former `client-era` required-feature gate. The NPC target gate is restored; the new CVar target keeps its own gate. Test bodies and runtime APIs remain unchanged. Original source/proof epochs retain the defect rather than being rewritten. [Direct target-selection receipts](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era-target-gates/main-retention.json) at `306cf7c43` show metadata exit0 with both gates and each default-profile request rejected before compilation (exit101). Six inherited manifest warnings remain. Metadata start/end times were not captured; negative-request receipts have timestamps, and revision comes from the same evaluation's commit/command ordering. This is configuration proof, not runtime execution; existing Era execution still concerns explicitly enabled `client-era`.

## Historical TransmogSituation fixture — exact epoch, partial proof preserved

`src/loader/tests/wow_api_globals/transmog_situation.rs` asserts the explicitly named 12.0.0 register: 22 members and metadata through 21. Its former cumulative feature gate also ran it under current 12.1.0, where later sourced additions extend metadata through 31. The gate now selects 12.0.0 without 12.0.5; every historical assertion is unchanged, and existing current weather/time-category publication coverage remains untouched. No runtime enum values or snapshots changed. An independent memo discussed a different integration additions test instead of this exact failing library identity; that target attribution was rejected. [Independent epoch report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/transmog-epoch/report.md) records formatting/default compilation and exact default omission, plus hash-verified existing current integration **1/1**. Historical compilation failed on three GUI-only references in an unrelated hittable-order test; no historical runtime pass. This partial epoch remains separate from the headless repair below.

The separate BigDefensiveIconSize proposal is not accepted: explicit 12.0.0 initialization removes that member, so merely moving its existing value-21 assertion into 12.0.0 would be wrong. No correction was made without proper source-epoch attribution.

## Headless / GUI hittable-order fixture — bounded execution proof

The historical headless library compile exposed three E0433 errors in `iced_app::frame_collect::tests::hittable_order_follows_render_buckets_not_raw_child_strata`: its hit-grid block references GUI-only `strata_emit`, `hit_grid` and optional `iced::Point`. Only that existing GUI block is feature-gated now. Shared registry setup and collection-order assertion remain active under headless; all GUI assertions remain when GUI is enabled. No production rendering/layout behavior, assertion values or host services changed. [Historical/headless proof](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/headless-gui/report.md) at `4163299fa`: formatting and historical compilation exit0; exact historical Transmog **1/1**, headless collection order **1/1**. Twelve compiler warning records and six manifest deprecations remain; no warning-free claim. Earlier failed historical epoch above is unchanged.

[Separate GUI proof](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/gui-window/report.md) at `b02b9f544`: warm fresh default artifact reports `build-finished success=true`; exact collection/hit-grid test **1/1**, exit0, both GUI assertions exercised. Compile exit integer was lost to a post-command API mismatch; full combined stream survived. Do not convert success output into an invented exit. Enabling GUI was not used to conceal headless errors. Existing default omission/current weather-time proof remains separate; no redundant rebuild/listing performed.

Retention manifests preserve 61 headless and six GUI artifacts byte-identically; process observations and redundant tracked-path listing excluded with hashes. Original reports' pending-GUI wording describes their earlier epoch, superseded only by the separate GUI receipt. No broad/profile/native/parent acceptance.

## Mists prefork wrappers — typecheck PASS, execution blocked

[Retry report](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/mists-capture-retry/report.md): required Mists `cargo check --tests` exit0 across actual `f8da74e31`→`43d9ce02e` epochs, six manifest deprecations retained. Integration compile attempt returned `Agent run aborted`; exit/output/compiler result unknown. Chat click/submit, chat activation color, and cast-bar lock wrappers **0/3 executed**, not three failing tests. Later process scan found no dedicated-target child; scan stream and duplicate embedded ledger excluded for privacy, hashes retained. Ten remaining artifacts preserved byte-identically; no automatic retry or parent acceptance.

## Historical redirect target — HTML discovery only

[Capture report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/historical-redirect-target-discovery/report.md) preserves observed Historical index body, revision5580481, and links. Page ID209816 was supplied, not independently captured; precise capture time absent. No revision wikitext/API response obtained after CDP loss. Index and linked-source research remain distinct from frozen redirect SOURCE accounting; no declarations or historical behavior credit inferred from link discovery.

## Retail TOC fixtures — bounded PASS at `41f1abbeb`

Three stale fixture expectations were corrected without loader/cache/vendor changes. The pinned Retail FrameXML TOC contains 17 dependency declarations, two annotated classic-only (`Blizzard_UnitPopup`, `Blizzard_MirrorTimer`). `src/toc/mod.rs::insert_metadata` rejects incompatible annotations before storing metadata; `dependencies()` therefore returns the other 15 in order. An independent memo that examined only `dependencies()` missed this earlier filtering boundary and was rejected after inspecting the insertion path and actual TOC bytes. Retail Reforging discovery now asserts no selected TOC; its Classic source metadata and explicit-load tests remain unchanged. The changed finder assertion is Retail-gated; no other-profile execution claim. Saved `3de874658` failures remain historical RED evidence.

[Independent proof](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/retail-toc/report.md) passes formatting, one integration compile and exactly **1/1 each** corrected test. Actual proof revision `41f1abbeb`, not requested `c5124f4c9`; changed tests match byte-for-byte, with only docs/evidence between revisions. All 23,161 tracked-source and 4,044 Retail-cache hashes unchanged during proof. Six manifest deprecation warnings remain; changed-Rust manual readability found no introduced issues. [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/retail-toc/main-retention.json) preserves 29 artifacts byte-for-byte. No alternate-profile, explicit-load, native-parity or full-suite acceptance; no commands rerun for retention.

## Era1.13.2 / three historical Retail redirects — bounded PASS

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1132-retail-redirects/report.md) covers 1.13.2, 1.12.0, 1.11.0 and 1.10.2: current/copy SOURCE **23/23 each**, portable **12/12**, separate Era successor **3/3**, receipt-validator **3/3**, and Era CVar storage **1/1** with six exact host transitions. **162 original +62 separate seals** remain intact; eight serialized tamper rejections and exact restorations pass. Deleted source worktrees did not supply replay inputs. All 5,011 Era contracts and each redirect's target contract remain UNPROVEN; no historical default/effect/native closure.

Python proof ran at `f8da74e31`; Cargo ran from `43d9ce02` through `f2eebd359` in a mutable checkout. Later docs/SOURCE and excluded historical-test-gate changes are recorded with scoped applicability, not immutable-SHA execution. Thirteen inherited diagnostics remain. The report's NPC target-gate caveat is retained; the later restoration above does not rewrite that proof epoch. Its minor Vec/push readability suggestion is rejected: this test's loop explicitly performs ordered state mutations and records each observation; a collection closure would hide those effects without a behavior benefit.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1132-retail-redirects/main-retention.json) preserves 191 root/command artifacts byte-for-byte; build targets, copied inputs, scratch and TMPDIR excluded. No proof commands rerun for retention. This does not cover later 1.10.1/1.10.0/1.9/1.8 audits or close the parent.

## Current CVar lifecycle PASS / duration consumer RED — separate modeled proof

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/current-cvar-duration-red/report.md) reused the hash-matching `41f1abbeb` integration artifact, with 3,882/3,885 relevant source/build hashes equal and three unrelated deltas explicitly excluded. Exactly selected current CVar registration lifecycle **1/1**, exit0: global and namespace registration preserve override `1.25` and first default `0` after re-registration `6`. Getters are global only. This is genuine bounded current modeled behavior, not historical 1.1/native/persistence/namespace-getter parity. No rebuild or CVar rerun.

Exact numeric duration consumer **0/1**, exit101, reproduces `expected LuaDurationObject at argument 1`; scalar fixture input fails before any visible-string assertion. Subsequent fixture correction uses real duration objects, zero-time manual clock and callback `GetRemainingDuration()`, preserving all expected strings. Production API and dirty-phase behavior unchanged. Corrected execution remains pending; original RED epoch is not rewritten.

[Retention](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/current-cvar-duration-red/main-retention.json) preserves 22 artifacts byte-identically, including full streams/provenance/exclusions. No full-suite or parent closure.

## Final literal Retail pages — integrated SOURCE PASS only

[Independent summary](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/final-literal-pages/SUMMARY.md): 1.4/1.3/1.1/1.0 copied SOURCE **30/30**, portable **12/12**, separate 1.4 current-category **3/3**; 13 commands exit0, four validators. **182 original +69 receipt entries +3 extension seals** intact; eight serialized tamper rejections/restorations. Actual `e20c472ca` covers 1.4/1.3; actual `1b505c7c1` covers 1.1/1.0, without rerunning earlier scopes. Full merged streams retained; separate child-channel attribution remains unproven.

These pages are not redirects: literal inventory **27/31/11/854**, substantive historical contracts **32/76/13/859**, all UNPROVEN. 1.4 category correction remains a separately sealed extension, not rewritten original credit. 1.1 current CVar candidate is unexecuted in this SOURCE gate; level-based ranged approximation receives no meaningful-model credit. 1.0 names, including literal misspellings, imply no signatures/defaults/aliases. Frozen registry endpoint1.0.0 is not meaningful-behavior or parent closure.

[Retention](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/final-literal-pages/main-retention.json) preserves 54 artifacts byte-identically, excludes copied archive inputs, and retains actual epochs/channel limits. No Cargo/runtime/native proof or broad rerun.

## Seven remaining Retail redirects — integrated SOURCE PASS only

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/remaining-retail-redirects/report.md) at actual `e20c472ca`: 1.10.1/1.10.0/1.9.0/1.8.0/1.7.0/1.6.0/1.5.0 copied SOURCE **38/38**, portable **21/21**, seven validators exit0. **231 original +70 separate seals**, 238 archive members intact; 14 serialized tamper rejections and exact restorations. All 21 proof commands ran once; evidence-scope hashes unchanged, requested-to-actual evidence diff empty. Each exact redirect keeps one UNPROVEN target contract and zero local declarations/model/runtime/native credit.

[Retention](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/remaining-retail-redirects/main-retention.json) preserves 249 artifacts byte-identically, excluding archive copies/scratch. Actual setup capture losses disclosed; no proof-command exit/stream loss. Target research, native/meaningful behavior, newer literal-page independent gates, full suites and parent acceptance remain separate. No Cargo or broad rerun.

## Era1.13.4 / 1.13.3 integration — bounded PASS

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1134-1133/report.md) at `9252c6cc9`: SOURCE **8/8 + 9/9**, separate successors **3/3**, portable **3/3 each**, original/separate seals **112+9 / 84+12**, and 1,873 scoped hashes unchanged. One combined offline/locked Cargo invocation passed each bare-Era target **1/1**. Totem replacement changes the existing slot name; expiry uses an already-expired slot. NPC absolute-health observations cover three explicit host mutations with percentage controls. No production behavior changed; all 30/38 historical contracts remain UNPROVEN.

Four literal 1.13.4 overlaps with 1.13.3 are retained separately; frozen queue/source ledgers remain untouched. Historical 1.13.3 extractor rejection is preserved, not repaired. Source worktrees remained present; empty-PATH copied-input proof is not an OS sandbox or deletion-portability proof. Existing manifest, dead-code and unused-import warnings remain. [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1134-1133/main-retention.json) preserves 66 top-level artifacts byte-for-byte, excluding scratch/copy subdirectories. No broad or full-suite rerun, native parity or parent-completion credit.

## Bare Retail factory: bounded PASS, mixed checkout

[Independent report](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-report.md) records ten tests passing: p302 7/7, p303 3/3. Exact observations: p302 **373 rows / 185 matches / 188 gaps**; p303 **3 CVars / 1 match / 2 gaps**. Every observation equals retained GREEN JSON. `syncronizeConfig` and `synchronizeMacros` are nil/nil; `synchronizeBindings` is `"1"`/`"1"`. These are present factory observations, not historical defaults or synchronization semantics.

Requested revision `16b9658da`; observed start HEAD `2d83b9b8`, end HEAD `c2481c4c`, with staged integration and unrelated unresolved files. Relevant source tree, classifier, tests and Cargo.lock matched the requested revision; Cargo.toml additionally registered p242. Hash equality supports relevant-input scope, **not an immutable-commit execution claim**. [Epoch scope](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-epoch-scope.json) and [comparison](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-integrated-independent-comparison.json) retain that distinction.

Original p302 36+1+2 seals, p303 16 seals, factory p302 112 seals and p303 16 seals validated; 197 protected files unchanged. The retained fabricated-global negative receipt has 373 rows / 189 gaps, exit 101, exactly AutoLootMailItem changed. Its older source tree differs from this run: sealed classifier/receipt evidence only, **not fresh current-runtime mutation rejection**. No loaded-UI, native, synchronization, model, global gate or final acceptance credit.

[Byte-identical retention manifest](../../../data/patch-api/evidence/3.0.2-factory-2026-10-09/integrated/p30x-retention.json) links original command, full streams, observations, comparisons and preservation receipts. Original seals were not rewritten.

## TBC SOURCE: retained bounded PASS, historical global FAIL

[Single retained independent report](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/p251-256-independent-report.md) is the count/status SSOT for 2.5.1–2.5.6: source 50/50, historical 6/6, relocated archive 6/6 at `fcdf1d151`. All source/native contracts remain UNPROVEN. Its global gate failed on three archived template snapshots; this is historical failure, not a fresh current gate. Later discovery repair proof is separately scoped in [narrow repairs](narrow-validator-discovery-and-forever-cfg-repairs.md).

[Aggregate raw receipts](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/p251-256-independent-raw-receipts.json) and [retention manifest](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/tbc-independent-retention.json) preserve original bytes and per-page receipts without duplicating facts across pages.

## Browser readiness handoff

[Main-thread readiness report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/browser-vnc-independent-report.md) retains loopback-only CDP 9222/VNC 5901 and a readable current GetSessionTime page with no visible challenge at observation. No remote access/authentication test, frozen-revision validation or native API proof. Earlier direct HTTP403 and current browser readability are distinct observations.

[Pre-auth VNC observation](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/private-vnc-security-observation.json), verified 2026-10-09T16:45:06Z: loopback endpoint responds with RFB 3.8 and advertises security type 2 only. No authentication attempted; remote login remains unverified. SSH local forwarding provides the private client access path; no public listener or noVNC deployment claim.

## 2.4.x shared-tool integration: source/historical PASS

[Independent report](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/integrated/p24x-integrated-independent-report.md) covers selected content at `dd710c6fc`: **18/18** targeted fixtures; **170/170** default outcomes equal archived baseline. One whole replay, with a scoped harness correction for the CLI-only `--text-only` flag, not a repeated whole gate. Own 2.4.0/2.4.2/3.0.2/2.1.0 opt-ins match; both parser function returns retained.

Recorded saved outputs: **83/87 registers**, **78/87 extracts**. The report enumerates inherited differences, absent current provenance flags and the exact 12.1.0 template error; baseline equality is not universal saved-byte reproduction. Fresh copied no-Git validators pass; 23+5 and 432+3 seals unchanged. [Full retention](../../../data/patch-api/evidence/2.4.0-source-2026-10-09/integrated/main-retention.json) preserves receipts/output/hash scopes. No native/model acceptance.

## Older source integration: bounded PASS

[Independent report](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/integrated/older-source/report.md) at `dd710c6fc` records current fixtures: 2.3.0 **6/6**, 2.2.0 **5/5**, 2.1.0 **4/4**, 2.0.1 **6/6**, WowForever1.60.1 **8/8**, Era1.15.9 **8/8**, 1.15.8 **8/8**. Seven copied no-Git historical validators pass; 137 original seals preserved, 14 serialized tamper rejections and 14 exact restorations. One verifier-only wrong path retained and corrected. [Receipts](../../../data/patch-api/evidence/2.0.1-session-2026-10-09/integrated/older-source/main-retention.json) preserve selected hash scopes. Source-only proof; the separate Forever configured-name test is linked below, not credited to these fixtures.

## Era1.15.7–1.15.5 integration: metadata/link-only PASS

[Independent report](../../../data/patch-api/evidence/1.15.7-session-2026-10-09/integrated/era1157-1155/REPORT.md) at `65b94c050`: each current/copy SOURCE **8/8**, portable **3/3**. Original/separate seals 28+8, 34+6, 38+6 unchanged; disk ledger/log rejections and exact restorations pass. Each page has four nonblank rows, two UNPROVEN links and zero local inventory. Default empty-register byte equality and nine successor identity comparisons are not supersession, runtime/model or native proof. [Receipts](../../../data/patch-api/evidence/1.15.7-session-2026-10-09/integrated/era1157-1155/main-retention.json) retain exact commands, streams and environments. Literal TOCs, absent client names and configured Era11507 remain distinct.

## Era1.15.4–1.15.0 integration: bounded SOURCE PASS

Verified: 2026-10-09. Counts/status below are the current SSOT; original queued/in-flight ledger notes remain historical, not current integration status. Actual canonical successor identity/context reconciliation does not expand linked contracts or establish semantic supersession.

| Page | SOURCE / copied SOURCE | Portable | Original + separate receipt seals | Retained proof epoch |
|---|---|---|---|---|
| 1.15.4 | 9/9 | 3/3 | 44 + 6 | `0b64e636c` |
| 1.15.3 | 10/10 | 3/3 | 50 + 6 | `0b64e636c` |
| 1.15.2 | 9/9 | 3/3 | 54 + 6 | `bf0e83720` |
| 1.15.1 | 8/8 | 3/3 | 57 + 9 | `17cc9b0f2` |
| 1.15.0 | 10/10 | 3/3 | 64 + 5 | `17cc9b0f2` |

[1.15.4/3 report](../../../data/patch-api/evidence/1.15.3-session-2026-10-09/integrated/era1154-1153/report.md), [1.15.2 report](../../../data/patch-api/evidence/1.15.2-session-2026-10-09/integrated/report.json), and [1.15.1/0 report](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/REPORT.md) retain exact scopes and commands. Canonical-cwd copied controls reproduce default bytes, reject serialized ledger/log changes and restore exact bytes; original seals remain unchanged. For 1.15.4/3 the scratch wrapper changes only historical harness cwd; initial SyntaxError attempts earn no credit. For 1.15.0 independent canonical-cwd controls are separate from the original worktree-bound harness. No runtime/model/native credit follows from SOURCE counts.

Byte-identical retention manifests: [1.15.4/3](../../../data/patch-api/evidence/1.15.3-session-2026-10-09/integrated/era1154-1153/main-retention.json), [1.15.2](../../../data/patch-api/evidence/1.15.2-session-2026-10-09/integrated/main-retention.json), [1.15.1/0](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/main-retention.json). Main retains the 144 selected files separately; this documentation update does not alter or reseal them.

## Official Lua alias: retained 1/1 REUSED

The [1.15.1/0 report](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/REPORT.md#retained-headless-runtime-proof-reuse) supports reuse of **1/1 at `714277147`**, not a fresh Cargo run at current HEAD. Its original relevant-input inventory is linked in [runtime hash evidence](../../../data/patch-api/evidence/1.15.1-session-2026-10-09/integrated/era1151-1150/runtime-reuse-input-hashes.json); the independent verifier confirms current relevant-input hashes unchanged. No Cargo rerun was performed for this docs update.

Real WowLuaEnv, existing `IsPublicBuild=true`, raw SeasonOfDiscovery numeric2 and initially absent Placeholder; direct unchanged pinned official Lua yields both raw names numeric2/equal. No mocks, flag injection or pre-test enum write. Native numeric correspondence, pre-vendor timing, full Blizzard UI loading, C_Seasons state and non-public branch remain UNPROVEN. 1.15.0 receives no alias/runtime credit.

## Era1.14.4 and 1.13.7/6/5: retained independent SOURCE epoch

[Exact independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/older-era-source/report.md) requested `3de874658`; actual commands and final HEAD were `bf8598e1da52895b707db0bf7bd9fb350c448628`. The intervening delta was docs/proof retention only. Recorded 335 scope hashes match before/after/final, not every runtime dependency. SOURCE canonical **34/34** and copied **34/34**, portable **12/12** pass: 1.13.7 **8/8**, 1.13.6 **7/7**, 1.13.5 **8/8**, 1.14.4 **11/11**, each portable3/3. **280 original +42 separate receipt seals** unchanged; eight serialized tamper rejections and eight exact restorations. Original audit/development epochs remain unchanged; this later independent epoch supersedes prior integration-pending status only.

Bare Era factory **2/2** at this epoch publishes `specular` and `textureErrorColors` as string1/default1; `nameplateCommentatorMaxDistance` and unknown control remain nil/nil. Historical 1.14.0 literal source removes `textureErrorColors`, while current factory still publishes it: no successor-application or native parity credit. All57 retained successor raw bodies match actual canonical sources, not semantic supersession. Source accounting earns zero native/model/runtime credit; factory observations earn no modeled rendering/nameplate, historical-default, loaded-UI, security or parent acceptance. Thirteen warning diagnostics remain unsuppressed.

[Separate byte-retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/older-era-source/byte-retention-manifest.json) binds **70 original files /765,499 bytes**, all source/retained hashes equal, including immutable report, command/environment ledgers, scopes, preservation, successor comparisons, receipts and full streams. Environment key names inspected before copying; no actual secret fields identified, ordinary environment bytes unchanged. Disposable scratch trees/TMPDIR excluded; no fresh execution, broad suite or parent completion.

## Era narrow-helper integration: bounded PASS

[Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era-helper/report.md) records Era **2/2 + 1/1** and Retail **78/78** at report HEAD `5b12dac256abc0ffaf541df2bc445e8381612d2d`; relevant source hash scopes unchanged across concurrent docs/data commits. All **412** factory observations equal retained inventory, including IDs/order/details/values/defaults/classifications: **286** direction matches / **126** reviewed mismatches unchanged. The original **18** target warnings are absent; baseline **7 simulator** (6 library + 1 binary) and **6 vendor manifest** warnings remain. No suppressions or visibility expansion.

Copied SOURCE **8/8 each**, corrected portable **3/3 each**; original/current seals **54+21** (1.14.3) and **72+17** (1.14.2) unchanged. Initial portable harness setup failures remain retained, not passing proof. First combined Era streams were transcribed verbatim from the tool result, not independently captured raw streams; exact start/end timestamps unavailable. Later receipts retain direct captures and timestamps. No native/model/event-producer credit: all observation native/model credit fields false; accepting a nonsense event name proves no producer.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era-helper/main-retention.json) preserves 32 original files / 1,155,904 bytes byte-identically, including report, receipts, both hash scopes, comparison, observations, successful and failed streams, and reused consumer logs. Original seals untouched; scratch fixture trees and recording helper excluded. These are bounded proof epochs, not latest-HEAD blanket acceptance.

## Saved full suite: FAIL, broader goal open

[Full-suite report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-dd/fullsuite-dd-independent-report.md) and [exact comparison](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-dd/fullsuite-dd-independent-comparison.json) retain **FAIL at `dd710c6fc`**: 23 integration + one Garrison prefork + six library failures, same identities/boundaries as `614402d56`. 29/30 raw assertion payloads equal; the remaining payload differs only a random missing-directory suffix. No newly failing identity against that baseline; no root-cause repair inferred. This is saved-run proof, not execution at later HEAD.

Twelve new integration cases and two new prefork cases PASS. Exactly 75 executed publication sweeps plus three helpers PASS; selected-name inventories are not extra sweep credit. Excluded Era/Forever profile targets receive no credit. The runner's `new_failures` flag is not the direct `614` comparison; its precise baseline selection remains unproven.

At this saved-suite epoch, main's four partial-startup cases remained in ordinary integration; the later bounded exact-fixture proof below supersedes placement, not this suite's result. The new Cargo/shared-helper refactor was in flight and not proven by this retained suite; the later bounded Era-helper proof above is separate, not full-suite acceptance; ignored, untracked PLAN is not acceptance evidence. No fresh tests or broad gates run here. **Broader goal remains open**, including native/full-UI and final integration gaps.

## Era1.14.1/1.14.0: independent bounded epoch

[Exact report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/report.md) requested `ce40cfb899bcc342457fc1cd03c2b23b26969406`; checkout later advanced to `045e396b0`. Copied SOURCE **8/8** (1.14.1) and **10/10** (1.14.0), portable **3/3 each**, and standalone 1.14.1 getter **1/1** pass. First FontString changes 1.5→2.25; second remains 0.75. Original/separate receipt seals **160+10 / 92+5** match; original seal-map hashes and all protected evidence bytes remain unchanged. The 1.14.0 default generator's expected `: Scripts` rejection remains a negative control, not a failure or new default inference.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/main-retention.json) binds 95 byte-identical files, including original receipts, streams, archives, preservation and runtime scope. [Concurrent scope note](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/concurrent-scope-note.json) records Cargo drift; [literal successor comparison](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/era1141-1140/successor-literal-comparison.json) is identity/input inspection only, not semantic supersession. Frozen queued/in-flight statements retain their original epoch. No native/signature/default/alias, security, loaded-UI or broad acceptance credit.

## Forever standalone contracts: independent bounded epoch

[Exact report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/report.md) records **3/3** at actual `045e396b0c6f717c2d962b97cdf46b736a3328ce`, with [compiled-scope equivalence](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/equivalence.json) to requested `f611a6752a14b4d660902417c59e14073de9ac31`. Later SOURCE merges do not promote that epoch to latest prefork-runtime acceptance. [Function-gate SSOT](forever-cfg-failure-and-function-gates.md#independent-standalone-proof-epoch) details ordered records, environment isolation, two invalid-prefix results with one shared final snapshot, and exact getter absence. Formatting and manual changed-Rust readability pass; **14 warnings retained**, not warning-free proof.

[Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/main-retention.json) binds 14 byte-identical report/receipt/scope/comparison/stream files; original seals unchanged. No native/security/transport/inbound-delivery or fresh Retail restriction credit. Prefork migration remains outside this retention task; no migration-pass claim.

## Exact startup fixtures: retained RED/GREEN epoch

[Immutable GREEN report](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/report.md) is the bounded runtime count/status SSOT at `3de87465828db7cc7f6f900d4b63e24f1b399825`: **4/4 original case bodies actually execute after fork**, runner conformance **21/21**, prefork listing **2,325** with all four stable names once, Retail integration listing **10,681** with all four absent. These are new-epoch counts; no historical sealed numbers were replaced.

[Immutable initial RED report](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/report-red.md) retains denied absolute `$crate::` macro invocation before runtime execution. Repair `3de874658` uses imported unqualified macro scope without fixture/body/gate changes. `cargo fmt --check` passed before repair; repair layout was inspected afterward, not command-reverified. Six existing vendor manifest deprecations remain; no zero-warnings claim.

Three fresh single-fixture parent-process groups preserve the process-global one-shot bytecode seal. Original chat manual startup/SAY/white assertions, cast-bar parent/anchor transitions and profile-specific spellbook S dispatch remain unchanged; no normalization, reset or callback framework. Non-Retail gates and preservation inspected only: execution **UNVERIFIED**. Runner conformance is not exhaustive per-group failure/selection proof or measured speed improvement.

[Separate byte-retention manifest](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/byte-retention-manifest.json) binds **71 original files / 2,758,400 bytes**, each source/retained SHA-256 equal: command ledger, complete streams, both hash scopes, listings, preservation/body ranges, conformance and reports. Inherited environment JSON key names were inspected before copying for secret/token/password/APIkey/cookie fields; no actual secret fields identified, so ordinary environment bytes retained unchanged. File mode is not safety evidence; no secret-bearing raw file or sanitized derivative was required.

[Mists capture-failure report](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/mists-capture-failure/report.md) retains source-level preservation of all four original bodies/constructors and three Mists integration wrappers; the Mainline spellbook is intentionally excluded there. The required check was attempted once but its spawned process handle, exit and streams were lost. Integration compilation/runtime tests were not run: no Mists typecheck/execution or zero-warning credit. No old process remained before one terminal-capture retry. [Selected retention](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/mists-capture-failure/main-retention.json) excludes process-list streams and source copies; original failure remains separate.

Full suite at `3de874658`, `unitfull-suite-1791563703.service`, completed **FAIL** on 2026-10-09 at 11:49:01 -0500. [Independent artifact comparison](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/fullsuite-comparison/fullsuite-3de-dd-independent-comparison.md) confirms the same **23 integration + 1 prefork + 6 lib** failure identities as `dd710c6fc`, with no additions/removals. Integration totals decrease by four. The 2,321 prefork summary covers the base registry only: saved log lines 14561–14572 show all four migrated cases passing, with separate chat 2/2, cast-bar 1/1 and spellbook 1/1 summaries. [Group coverage adjudication](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/fullsuite-comparison/prefork-full-suite-group-coverage.md) falsifies missing execution. Failure extraction remains correct; no parser defect or reporting redesign is inferred. The immutable comparison's first-summary count is not a combined-case total. [Byte retention](../../../data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/fullsuite-comparison/main-retention.json) adds no fresh test execution. Later fixture corrections are separate epochs, not retroactive repairs of this failure. Broader integration/native/full-UI/final gates remain open; no parent completion.

## Sources

- Retained independent reports, comparisons, hash evidence and retention manifests linked above — bounded execution epochs, not latest-HEAD blanket proof.

## See also

- [3.0.2 audit](patch-3-0-2-api-audit.md), [3.0.3 audit](patch-3-0-3-api-audit.md).
- [Forever failure/functions gates](forever-cfg-failure-and-function-gates.md).
