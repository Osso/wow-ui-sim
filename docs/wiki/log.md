## [2026-09-02] port | Rebase the manifest drift branch onto master

Rebased the [[blizzard-ui-manifest-drift]] branch from `634bb762c` onto `bbd591fe4`; the page gains a section on what master had fixed in the meantime (kept master's version), what the keep-both conflict resolutions had cut, the `client-retail` 12.1.0 manifest refresh (master already reports 0 startup Lua errors under retail, 42 distinct under ptr; the branch 0 under both), why `sync-blizzard-ui` under `client-ptr` refuses a live install (CASC product `wowt`), and the one ordering regression the rebase surfaced (`Blizzard_UIParentPanelManager` before `Blizzard_UIParent`), fixed with an explicit startup dependency.

## [2026-09-02] investigation | Preserve duplicate named region bindings

Documented commit `601cde499`: duplicate sibling Texture/FontString regions with the same parent now keep the first `_G` binding while retaining both objects. Current `Blizzard_GMChatUI.xml` relies on this for its second `GMChatTabBG` texture to anchor to the first; last-writer replacement had produced a self-anchor error and aborted XML before `GMChatStatusFrame`, breaking Behavioral Messaging. Added [[duplicate-named-region-binding]], updated [[global-frame-index]], and recorded the related retail 12.1 Tiered Entrance enum, Transmog startup root, and `HasAccessConstraints` contracts in [[patch-12-1-api-audit]] and [[addon-loading]].

## [2026-09-01] audit | Document retail 12.1 housing editor enums

Audited commit `d18654fa0`. Updated [[patch-12-1-api-audit]]: generated `Enum.HousingPetBehaviorType` publishes `Stationary=0` and `Wander=1` with metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`; `Enum.HouseEditorPlayerType` publishes `None=0`, `Owner=1`, and `Visitor=2` with metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Current `Blizzard_HouseEditor` uses pet behavior for menu options; `Blizzard_HousingControls` uses player type for visibility and owner/visitor selection. Housing service behavior remains best-effort modeled. No new page, index update, spec, manifest, `PLAN.md`, vendor/cache/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Document FrameXMLUtil secure replay

Audited commit `93761fdb4`. Updated [[addon-loading]] and the maintained addon-loading pipeline: secure `Blizzard_AuraContainer` needs `AuraUtil.DefaultAuraCompare` and `AuraUtil.UnitFrameDebuffComparator`, so `Blizzard_FrameXMLUtil` is explicitly replayed into `__secureenv`. Public-only loading left secureenv stale, aborting TargetFrame aura initialization before `FocusFrame` creation. Focused coverage: `loader::tests::lua_loading::blizzard_frame_xml_util_replays_aura_comparators_into_secure_environment`. No new page, index update, spec, cache, manifest, vendor/Blizzard, `PLAN.md`, or protected-file change was warranted.

## [2026-09-01] audit | Document partition-aware XML handlers and RestrictedAuraAPI

Audited commits `391cd75ea` and `d994939bf`. Updated [[xml-template-system]], [[lua-api]], the maintained Lua API reference, and [[patch-12-1-api-audit]]: `__wow_bind_xml_method` resolves public then forbidden methods for private XML frames, while forbidden receivers forward public `FrameHandle` methods; this applies to precompiled intrinsic `OnLoad` and ordinary XML handlers. Retail 12.1 `GetBuildOption("RestrictedAuraAPI")` returns `true`; unknown options return `nil`, selecting the forbidden aura template path. No new page, index update, spec, cache, manifest, vendor/Blizzard, `PLAN.md`, or protected-file change was warranted.

## [2026-09-01] audit | Document Macro/Trainer startup roots and TOC game types

Audited commits `7f68e2f24`, `44d5ab8df`, `b5191265b`, `7b3dba413`, and `3fde8b80e`. Updated [[addon-loading]], the addon-loading pipeline, and the prefork harness spec: `Blizzard_MacroUI` and `Blizzard_TrainerUI` remain LoadOnDemand but are explicit Game-only startup roots, publishing `MacroFrame_LoadUI` and `ClassTrainerFrame_LoadUI` while staying excluded from glue screens. Inline `[AllowLoadGameType]` values now accept comma or whitespace separators, so current `vanilla tbc mainline` annotations retain retail LoadSystem files and nested template mixins. `[Bootstrap]` remains inline-only; no bootstrap pass, all-LoD load, or TOC reordering was introduced. No new page, index update, manifest, cache content, `PLAN.md`, vendor/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Load AchievementUI startup publisher

Audited commit `f81596eb2`. Updated [[addon-loading]], the addon-loading pipeline, and the prefork harness spec: standalone Game-only LoD root `Blizzard_AchievementUI` loads its complete TOC before `AlertFrame` handles `ACHIEVEMENT_EARNED`, so `AchievementFrame_LoadUI` preserves the real achievement-toast queue path. AchievementUI remains LoadOnDemand by metadata and excluded from glue screens. This adds neither a bootstrap-only pass nor eager loading of unrelated LoD addons, and does not reorder TOC entries. No new page, index update, manifest, cache content, `PLAN.md`, vendor/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Load CombatLog startup publisher

Audited commit `a5ce1b550`. Updated [[addon-loading]], the addon-loading pipeline, and the prefork harness spec: `Blizzard_CombatLog` remains `LoadOnDemand` by TOC metadata but is an explicit `Blizzard_Game` startup dependency, so its full TOC publishes `CombatLog_LoadUI` before `PLAYER_LOGIN`; declared `Blizzard_CombatLogBase` and `Blizzard_CombatLogProcessor` dependencies load first. This does not restore a bootstrap-only pass, load all LoD addons, or reorder TOC files. No new page, index update, manifest, cache content, `PLAN.md`, vendor/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Load startup publishers in dependency order

Audited commits `50943cff0` and `34712c117`. Updated [[addon-loading]], the addon-loading pipeline, and the prefork harness spec: full game discovery adds only the LoD publishers required by current startup consumers. `Blizzard_Game` depends on `Blizzard_TimeManager`, `Blizzard_CooldownBroadcaster`, and `Blizzard_BoostTutorial`; LoD root `Blizzard_RaidUI` depends on `Blizzard_RaidFrame`, so RaidFrame loads first and RaidUI is ready before startup events. LoD keys in the implicit startup-dependency map are selected as roots. `[Bootstrap]` remains an inline TOC annotation, not a global LoD pass or reorder signal; unrelated LoD roots such as `Deprecated_PaperDoll` remain excluded. No new page, index update, manifest, cache content, `PLAN.md`, vendor/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Require retail CooldownBroadcaster bootstrap cache entry

Audited commit `af410e4b7`. Updated [[addon-loading]] and the Blizzard UI patch-update runbook: current retail `12.1.0.69497` manifest and `Blizzard_CooldownBroadcaster` TOC require `Blizzard_CooldownBroadcaster_Bootstrap.lua`; stale PTR-only filtering skipped it during retail sync, allowing an incomplete completed cache. Retail and PTR now both include and require the entry. No new page, index update, manifest, cache content, spec, `PLAN.md`, vendor/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Document retail 12.1 SocialSystemType enum

Audited commit `51b9243c5`. Updated [[patch-12-1-api-audit]]: generated retail `12.1.0.69497` `Enum.SocialSystemType` now publishes `Friends=0`, `QuickJoin=1`, `RaidList=2`, `RecruitAFriend=3`, and `RecentAllies=4`, with `MinValue=0`, `MaxValue=4`, and `NumValues=5` under the retail 12.1 epoch. Current `Blizzard_SocialUI` OnLoad reads all five for tab definitions; Social service behavior remains unmodeled. No new page, index update, spec, manifest, `PLAN.md`, vendor/cache/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Document PetBattle species/model-scene preload boundary

Audited commits `eae094261` and `b08bd4cf7`. Updated [[patch-12-1-api-audit]]: current retail `12.1.0.69497` `PetBattleFrame_OnLoad` first passed a missing `C_PetBattles.GetPetSpeciesID` result into `C_PetJournal.GetPetModelSceneInfoBySpeciesID`; after the state-backed API was added, stale default IDs `1`/`2` still had no seeded Pet Journal records, so lookup returned no values and `TransitionToModelSceneID` received `nil`. Default battle-pet IDs now align with Journal seeds `39`/`87`; unknown owner/index remains `nil`. No new page, index update, spec, manifest, `PLAN.md`, vendor/cache/Blizzard, or protected-file change was warranted.

## [2026-09-01] audit | Document temporary pet-battle breed quality

Audited commit `7ecab542f`. Updated [[lua-api]] and the maintained Lua API reference: temporary Lua-owned pet-battle sample state seeds displayed pets with rare breed quality (`3`), while `C_PetBattles.GetBreedQuality(owner, petIndex)` returns that numeric seed or `0` for an absent sample pet, allowing current PetBattleFrame OnLoad rarity rendering. Pet ownership, breeding, capture, combat outcomes, and live battle data remain unmodeled. No spec, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document Blizzard dependency-root discovery

Audited commit `d62832f08`. Updated [[addon-loading]], the maintained addon-loading pipeline, and the prefork harness spec: eligible non-LoadOnDemand `Blizzard_*` cache addons are startup roots, while the filtered candidate pool contributes only transitive hard TOC dependencies. Current `Blizzard_TutorialManager` therefore loads `middleclass` before itself; unrelated non-Blizzard directories such as `Deprecated_PaperDoll` remain excluded. LoadOnDemand addons are not roots merely because they are present in cache. No new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 housing blueprint content-type enum

Audited commit `665f5a31b`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.HousingBlueprintContentType` now publishes all seven contiguous values (`None=0` through `Other=6`) with metadata under `retail-12-1-0`, allowing the final current `Blizzard_HousingData` content label table to load. Housing blueprint operations and strings remain unmodeled. No spec, changelog, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 housing blueprint requirement flags

Audited commit `432d357a1`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.HousingBlueprintUnmetRequirementFlags` now publishes eight bitmask values (`InsufficientBudget=1` through `HouseSizeLocked=128`, with no `None`) plus metadata under `retail-12-1-0`, allowing current `Blizzard_HousingData` blueprint-requirement text tables to load. Housing blueprint validation, operations, and strings remain unmodeled. No spec, changelog, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 HousingBlueprintType enum

Audited commit `045c9fa19`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.HousingBlueprintType` now publishes all five contiguous values (`None=0` through `Exterior=4`) with metadata under `retail-12-1-0`, allowing current `Blizzard_HousingData` blueprint-type label tables to load. Housing blueprint operations and strings remain unmodeled. No spec, changelog, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 HouseSettingFlags enum

Audited commit `db678a399`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: current wowt-global-dataset `Enum.HouseSettingFlags` now publishes all 16 bitmask values through `BlueprintExportParty=16384` with metadata under `retail-12-1-0` before the stale fallback. Current `Blizzard_HousingData` uses the four non-`Anyone` blueprint-export flags in `HousingAccessTypeStrings`; housing access/export behavior and strings remain unmodeled, and the pre-12.1 contract is unchanged. No spec, changelog, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 HousingResult enum

Audited commit `e3806caba`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.HousingResult` now publishes all 112 values (`0..111`) with metadata under `retail-12-1-0` before the stale missing-enums fallback, allowing current `Blizzard_HousingTemplates` to build `HousingResultToErrorText`. Housing operations and result strings remain unmodeled; the pre-12.1 contract is unchanged. No spec, changelog, index, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 external URL failure event registration

Audited commit `fc833ab34`. Updated [[patch-12-1-api-audit]] and event-system docs: retail 12.1's strict registerable-event table now accepts `EXTERNAL_EVENT_LAUNCH_URL_FAILED`, allowing current `Blizzard_GameMenu` registration. No event producer, payload, or `C_ExternalEventURL` behavior is modeled; no spec, changelog, index, or new page was warranted.

## [2026-09-01] audit | Document Recent Allies preload enums

Audited commit `ec0e21537`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: `Enum.RecentAlliesFriendTag` (`0..5`) plus metadata and sparse `Enum.RolodexType.LegacyFriend=23` with preserved `21`/`22` gaps and metadata resolve independent `Blizzard_RecentAllies` preload tables. Recent Allies service/search behavior remains unmodeled; no spec, changelog, or new page was warranted. No code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 custom AuraButton texture-style enum

Audited commit `6eae179ff`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.CustomAuraButtonDispelTypeTextureStyle` publishes five values (`0..4`) and metadata under `retail-12-1-0`, allowing `Blizzard_Deprecated/Deprecated_12_1_0.lua` to construct AuraButton border-style aliases when deprecation fallbacks load. Texture and rendering behavior remain unmodeled. No spec, changelog, or new page was warranted; no vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document raid dispel overlay enum

Audited commit `4d90beffd`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: generated `Enum.RaidDispelOverlayType` (`Disabled=0`, `UseDebuffColor=1`, `UseBlack=2`) plus metadata satisfies the current `CompactUnitFrameOptions` lookup. The later `CompactUnitFrameUtil` `pairs(nil)` was downstream of that aborted options load; overlay rendering remains unmodeled. No spec, changelog, or new page was warranted. No code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document ChatFrame combat-audio command names

Audited commit `abcfb395b`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: two retail-12.1-gated non-probe compatibility command names, `SLASH_CAA_WHEN_TARGET_DIES` and `SLASH_CAA_PLAY_SOUND`, bring the table from 68 to 70 strings. Current `TextToSpeechCommands.lua` passes them to `AddCommand`, which normalizes them with `string.lower` during preload. No command, CVar, or audio behavior is claimed; no spec, changelog, or new page was warranted. No code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document ChatFrame combat-audio format strings

Audited commit `43075580b`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: seven retail-12.1-gated English compatibility format strings bring the table from 61 to 68 entries and satisfy unconditional `:format(min, max)` calls in `Blizzard_ChatFrame/Shared/TextToSpeechCommands.lua` preload. Their values were not captured by the existing live probe. No spec, changelog, or new page was warranted; no code, vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 Cooldown Viewer sound enum

Audited commit `9555a7649`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: retail-12.1-gated generated `Enum.CooldownViewerSound` now publishes all 94 values (`0..93`) with metadata, satisfying `Blizzard_ChatFrame/Shared/TextToSpeechCommands.lua` enum reads during preload. Source: cached `CooldownViewerConstantsDocumentation.lua` for retail `12.1.0.69497`. No spec, changelog, or new page was warranted; no vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] audit | Document retail 12.1 Social UI preload surface

Audited commits `cfec8576b` and `8f070a410`. Updated [[patch-12-1-api-audit]] and [[lua-api]]: 16 retail-12.1-gated Social UI compatibility labels bring the table from 45 to 61 strings and keep `Blizzard_SocialUIShared` tag/presence tables populated; their English values were not captured by the existing 49-candidate live probe. Documented `Enum.SocialUIPresenceType`/metadata and `Enum.SocialUIBlockType`/metadata. No spec, changelog, or new page was warranted; Social UI service/state semantics remain unmodeled. No vendor, cache, Blizzard, manifest, `PLAN.md`, or protected-file changes.

## [2026-09-01] investigation | Retire TransmogShared inventory-slot loader scope

Refreshed the retail manifest from Gethe `live` build `12.1.0.69497` and found the prior scope was based on stale 12.0.7 source: current `Blizzard_TransmogShared` calls `C_PaperDollInfo.GetInventorySlotInfo` directly. Removed the target-scoped legacy-global loader mechanism and its stale restoration test while preserving retail public removal and TransmogUtil behavior coverage. Updated [[transmog-inventory-slot-scope]], [[addon-loading]], and [[lua-api]]; cache provenance refresh prevents the stale-source condition from recurring.

## [2026-09-01] audit | Document timeout fixture capacity reservation

Audited commit `d5e017f31`. Updated [[prefork-test-harness]] and the prefork spec: outer timeout conformance fixture subprocesses reserve the shared workload gate exclusively, while an inherited marker bypasses gate acquisition for descendants. Nested timeout children therefore retain the same global two-slot permit cap without deadlocking against their fixture reservation. No new page, index update, or changelog was warranted; protected `src/c_api/c_string_util.rs`, code, tests, `PLAN.md`, and vendor/cache/Blizzard paths were untouched.

## [2026-08-31] system | Document bounded Linux timeout re-execution

Updated [[prefork-test-harness]] and the prefork spec/system docs for ordinary Linux timeout re-execution: exact one-test children use a separate two-slot cross-process `flock` permit, acquired before spawn and held through timeout cleanup, output draining, handshake validation, and failure aggregation. Queueing remains outside the post-spawn 120-second budget; default Cargo parallelism and prefork accounting are unchanged. Updated timeout optimization evidence to remove a brittle focused-test count. No final acceptance or manifest refresh was claimed.

## [2026-09-01] system | Refresh Blizzard UI profile caches by provenance

Updated [[addon-loading]] for cache provenance schema 1: `wow-cli casc sync-blizzard-ui` compares active profile, CASC product, active `.build.info` version/build key, optional install key, and compiled manifest hash before retaining an existing profile cache. Mismatches remove only that profile's `AddOns` root before extraction; matching identity retains incremental repair behavior. Patch refreshes no longer require manual cache deletion.

## [2026-08-31] audit | Document retail 12.1 housing Settings strings

Audited commit `cc02aa287`. Updated [[patch-12-1-api-audit]], [[lua-api]], and the maintained Lua API reference with the 13 exact enUS housing Settings labels/tooltips pinned from retail `12.1.0.69497`; the versioned table now contains 45 strings. Recorded the root cause: missing values let the canonical Interface registrant assign category ID 7, then abort before registration. No spec, changelog, or new page was warranted; housing service/state semantics remain unsupported.

## [2026-08-31] investigation | Document lazy model widget state

Audited commit `2542135be111f53211ac66e39a19591abadde0d6` (2026-08-30). Updated [[widget-system]] and added [[widget-registry-storage]]: nine model-family state groups now use one lazy `Option<Box<ModelWidgetState>>`; absent payloads preserve defaults, mutations allocate on demand, and storage accounting includes boxed strings/vectors without double counting. The settled 45,002-frame estimate fell from 239,185,088 to 213,058,036 bytes under the unchanged 230,000,000-byte budget. Model rendering remains intentionally unsupported. No spec or changelog update was warranted; protected, code, test, manifest, vendor/cache, and `PLAN.md` paths were untouched.

## [2026-08-30] audit | Update maintained rendering-order documentation

Audited commits `dfd997a05` and `178c83bf0`. Updated [[rendering-pipeline]] plus the maintained `rendering-pipeline.md` and `hit-testing.md`: active `toplevel="true"` frames use a separate monotonic show-order sequence; emitted IDs group under the nearest active top-level ancestor across intermediate strata and remain contiguous; explicit `Raise()`/`Lower()` retain same-raw-level semantics. No rendering spec exists under `docs/specs/`, so no spec change was warranted. Protected, code, manifest, vendor/cache, and `PLAN.md` paths were untouched.

## [2026-08-30] investigation | Separate top-level show ordering from explicit Raise

Updated [[world-map-voice-chat-alerts]] for commit `dfd997a05`. The prior `UIParent`/`WorldFrame` boundary repair remained necessary but no longer sufficient after explicit `Raise()`/`Lower()` was correctly constrained to same raw levels: `set_frame_visible()` still reused that path for `toplevel=true`, leaving the low-level `WorldMapFrame` root and split cross-strata descendants around `ChatFrameChannelButton`. Added monotonic active top-level show order in `SimState`, nearest-active-ancestor grouping across intermediate strata, contiguous per-strata groups after regular content, deterministic nested ownership, and full regroup on top-level show. Focused proof is 8/8 state-render tests plus the exact live-like overlap regression 1/1. No spec or new page was needed; protected, vendor/cache, manifest, and `PLAN.md` paths were untouched.

## [2026-08-30] audit | Document bounded profession specialization fixture

Audited commits `f6b1077b8e1740c9efabd44ebda061fab1b5f0f6` and `6141fb0bdbcac4de70d8f85a66905334a1e12523`. Updated [[lua-api]] and the maintained Lua API reference: the temporary `C_ProfSpecs` fixture reports specialization only for modeled Blacksmithing skill line 164, unrelated 165 remains false, and authoritative path/tree semantics remain unsupported because `GetChildrenForPath()` and backing state are unmodeled. The full Professions specialization-panel test remains explicitly ignored; no spec, changelog, or new investigation page was warranted. Protected `src/c_api/c_string_util.rs` and concurrent code changes were untouched.

## [2026-08-30] audit | Document EditMode base-alias initialization

Audited commit `0caee6eb94d672bbadb37bc9bfbea4b84be59c18`. Updated [[frame-data-flow]] and the maintained frame data-flow reference: `EditModeSystemMixin` now seeds seven callable native `Base` aliases during mixin application, before prepended lifecycle handlers run; missing native methods propagate as template-application errors. Updated the index summary; no new page, spec, or changelog was warranted. Protected `src/c_api/c_string_util.rs` and concurrent worktree changes were untouched.

## [2026-08-30] audit | Document Linux workload gating for prefork and performance tests

Audited commits `39f4e1a39`, `82b2b927a`, and `5354fad5a`. Updated [[prefork-test-harness]] and the prefork spec/system docs: Linux test workloads use a poison-recovering process-local `RwLock` plus cross-process `flock`; ordinary timeout, prefork, and full-UI workloads acquire shared access, while performance measurements acquire exclusive access before timeout children and measured timing. The prefork target remains exactly 1,951 retail cases (1,942 marker-generated and 9 manual/nested), with two default workers and zero eligible remaining. Added workload-gate implementation/conformance references. No changelog or new page was warranted; protected, code, test, manifest, vendor, cache, and `PLAN.md` paths were untouched.

## [2026-08-30] investigation | Document Blizzard_TransmogShared inventory-slot scope

Audited commits `4f2d4b779` and `e831f1d98`. Added [[transmog-inventory-slot-scope]], updated [[lua-api]] and [[addon-loading]], and linked the maintained API/loader references: retail 12.1 keeps `GetInventorySlotInfo` nil publicly while `Blizzard_TransmogShared` receives the registered lookup through a retained target-scoped environment, with prior global/environment restoration after success or `LoadError`. No spec or changelog update was needed; protected, code, test, manifest, vendor, cache, and `PLAN.md` paths were untouched.

## [2026-08-30] audit | Document idempotent SettingsPanel reconciliation

Audited commit `5715008bb`. Updated [[lua-api]] and [[addon-loading]] plus the maintained API and loader docs: post-load workarounds reconcile replacement `_G.SettingsPanel`/`Settings` surfaces before category registration/opening and remain idempotent for the same panel identity. `C_SettingsUtil` APIs remain explicitly unsupported. No new page, spec, or changelog was warranted; protected `src/c_api/c_string_util.rs`, code/tests, manifest, vendor/cache, and active agent paths were untouched.

## [2026-08-30] audit | Document retail Guild toggle ownership

Audited commit `427ae6cca58327b02d95d73dc23732778269bbfa`. Updated [[lua-api]], the maintained Lua API reference, and `keybinding-system.md`: retail does not register a simulator `ToggleGuildFrame()` fallback, so `Blizzard_Communities` supplies the canonical implementation after load; non-retail keeps the fallback. Added source and focused-test references. No new page, spec, or changelog was warranted; protected `src/c_api/c_string_util.rs` and concurrent worktree changes were untouched.

## [2026-08-30] investigation | Verify SharedMapDataProviders test-runtime optimization

Audited verified code HEAD `e1f9b210b1790258a6f865cc3df0e85f6364e8e0` and finalized [[test-runtime-optimization]]. The bare root closure was invalid: 16/18 tests passed because top-level Lua relied on normal eager ambient order despite a dependency-free TOC; missing SharedXML color helpers and `CVarMapCanvasDataProviderMixin::Init` surfaced downstream as nil `DelveEntrancePinMixin`. The fix adds ordered `BlizzardAddonOverride` implicit dependencies `Blizzard_SharedXML`, then `Blizzard_MapCanvas`, while retaining a fresh environment and direct root load without panel/full UI or shared mutable state. Parallel proof is 18/18 in 0.83s libtest / 0.85s wall with 1,067,108 KiB RSS versus 6.01s / 6.083s / 6,412,848 KiB (wall −86.03%, RSS −83.36%); sequential proof is 18/18 in 2.96s / 2.97s with 535,624 KiB RSS versus 20.10s / 20.140s / 1,246,460 KiB (wall −85.25%, RSS −57.03%). Updated the index summary; no spec change was warranted. Fixed-run artifacts under `/tmp/pi-shared-map-fixed-*` remain ephemeral evidence.

## [2026-08-29] audit | Correct typed diagnostic forwarding summary

Audited the documentation added for commits `01026c8b6` and `3ecbc33fc`. Corrected the index and log summaries to distinguish nested diagnostic forwarding from top-level runtime diagnostic retention/draining, and linked the typed diagnostic record definitions in `src/lua_api/state_types/runtime.rs`. No API claims or unsupported behavior changed.

## [2026-08-29] system | Separate typed addon-load diagnostics

Updated [[addon-loading]] and its index summary for the typed diagnostic contract: actual loader/XML/Lua/runtime failures remain in `LoadResult.warnings`; regular nil-symbol accesses and missing `C_*` contracts retain addon/source/line/environment attribution in dedicated channels; nested runtime loads forward all three channels exactly once to their parent, while top-level runtime diagnostics remain in SimState until drained once. Startup health now gates only genuine failures while strict unsupported requirements remain inspectable. No unsupported API semantics or runtime-bootstrap hashes changed.

## [2026-08-29] audit | Document active voice channel type query

Audited commit `068ee317e27d6c850d331aaf422d98384e13b2bc`. Updated [[lua-api]] and the maintained Lua API reference: `C_VoiceChat.GetActiveChannelType()` returns the seeded active channel's numeric `ChatChannelType`, or one `nil` result for no/stale active channel IDs. Added source and focused test references. No new page, spec, index entry, code, generated, vendor, cache, `PLAN.md`, or protected-file change was warranted.

## [2026-08-29] audit | Document retail/PTR Recruit-A-Friend surface distinction

Audited commit `928243411`: retail 12.1 retains `C_RecruitAFriend.IsEnabled`, while PTR hides it after startup through `src/ptr/strict_removals.lua`, despite both profiles selecting API epoch 12.1. Updated [[client-profiles]], [[patch-12-1-api-audit]], the client-profile spec, and [StrictRemovalTimingProbe](../addons/StrictRemovalTimingProbe/README.md) so epoch-gated API guidance preserves this evidence-backed channel exception. No code, generated, vendor, cache, `PLAN.md`, or protected-file changes.

## [2026-08-29] audit | Document modeled C_Item queries

Audited commit `d38588beec834a2adb5ca1ea3f2f57a03469ff85`. Updated [[lua-api]] and the maintained Lua API reference to record that `C_Item.IsConsumableItem`, `C_Item.IsEquippableItem`, and `C_Item.IsItemInRange` are state-backed and share the existing legacy-global semantics. No new page, spec, changelog, or stub-inventory update was warranted; the protected C StringUtil file was not touched.

## [2026-08-29] audit | Document backpack bag flag queries

Audited commit `c17b32062`. Updated the maintained C_* capability references to mark `C_Container.GetBagSlotFlag`, `SetBagSlotFlag`, `GetBackpackAutosortDisabled`, and `GetBackpackSellJunkDisabled` as state-backed shared bag-slot queries, and added the C_Container surface to the Lua API references. No spec or new wiki page was warranted; protected `src/c_api/c_string_util.rs` was not touched.

## [2026-08-29] audit | Document CVar-backed spell queue window

Audited commit `8e8f23e05`. Updated [[lua-api]] and the maintained Lua API reference: `C_Spell.GetSpellQueueWindow()` reads the mutable `SpellQueueWindow` CVar, returns a numeric value when parseable, returns `nil` for unavailable/non-numeric values, and preserves identity for the two deprecated spell globals that alias it. No spec or separate page was warranted; protected `src/c_api/c_string_util.rs` was not touched.

## [2026-08-29] system | Promote default retail to API epoch 12.1

Audited commit `3a652e1b2`. Updated [[client-profiles]], [[patch-api-audit-manifest]], and the index: `client-retail` now selects cumulative `retail-12-1-0` / interface `120100`; `client-ptr` remains a separate PTR profile/cache at the same API epoch; historical `profile-retail` epoch selection remains available. Preserved the existing client-profile spec because it already records this contract. No code, Cargo, generated, vendor, cache, PLAN, or protected-file changes.

## [2026-08-29] audit | Document syntactic-global versus explicit-_G nil diagnostics

Audited wow-ui-sim commit `d149b2c8d` and rilua commits `1a7c9de` / `3630419`. Updated [[addon-loading]], [[lua-api]], and the addon-loading pipeline docs: direct syntactic global loads remain startup diagnostics; missing regular globals read through `_G.name` or `_G[name]` are optional probes excluded from nil-symbol records and the dedup cache; all `C_*` namespace/member gaps remain strict. Recorded that rilua scopes lookup provenance to VM execution state and exposes read-only `debug.isglobalindex()`, restoring state across nested lookups, errors, and coroutine swaps. No new page or spec was warranted; no code, generated, vendor, cache, `PLAN.md`, Cargo.lock, or protected file changes were made.

## [2026-08-28] audit | Document live retail 12.1 GlobalStrings slice

Audited commit `72f3ec342`. Updated [[patch-12-1-api-audit]] and [[lua-api]] with live enUS retail `12.1.0.69497` / interface `120100` evidence captured 2026-08-28: 32 exact string globals are registered only under `profile-retail` + `retail-12-1-0`, and 12 warning candidates proven raw `nil` remain unregistered. Added source/test references and the rationale against placeholder publication; private probe evidence remains outside the repository. Updated the existing index summary; no new page or spec was warranted. Protected `src/c_api/c_string_util.rs` was not touched.

## [2026-08-28] investigation | Record live PlayerSpells panel replacement contract

Updated [[playerspells-runtime-load]] from live artifact `/tmp/CharacterPlayerSpellsProbe-live-2026-08-28.lua` (SHA-256 `40dcf028acd5605d675810abbfc9eb8aa63147425ed5f8bb8b3b54b78c997595`). WoW 12.1.0 build 69497/interface 120100 shows successful direct `ShowUIPanel` and real `ToggleCharacter`/`ToggleSpellBookFrame` paths replacing CharacterFrame with PlayerSpells; CharacterFrame expands 338→540 when opened and returns to 338 when replaced. Recorded PlayerSpells panel attributes and marked empty `UIParent:GetUIPanel` slot captures inconclusive because the API belongs to private `FramePositionDelegate`. Documented that production behavior stayed unchanged and commit `38bc75892` uses dependency-aware `C_AddOns.LoadAddOn` in the fixture. Existing index entry remained sufficient; no code, test, spec, or manifest changes.

## [2026-08-28] audit | Preserve top-level runtime addon warnings

Audited commit `f6052114e74354276359d0f12da0c53cf4a5efe2`. Updated [[addon-loading]] and its index summary: finalized warnings from top-level `C_AddOns.LoadAddOn` calls now remain in SimState until the startup/test collector drains them exactly once, while nested warnings retain owner attribution and forward transitively to the immediate parent result. Added source/test references; no new page or spec was warranted.

## [2026-08-28] audit | Document nested warning propagation and recorder encapsulation

Audited commits `88cf327a7` and `8a6b5f61a`. Updated [[addon-loading]] and the top-level addon-loading pipeline documentation: nested runtime-addon warnings are finalized under the nested addon and forwarded exactly once to the immediate parent `LoadResult` (transitively for nested-nested loads, without raw-access reprocessing), while the global-publication recorder is captured in a bootstrap-local upvalue and removed from `_G` so addon Lua cannot forge publication events. Updated the existing index summary and added the runtime loader source link; no spec or new wiki page was warranted.

## [2026-08-28] audit | Document same-addon nil-symbol publication reconciliation

Audited commits `473031857` and `e2545a3d1`. Updated [[addon-loading]] and the top-level addon-loading pipeline documentation to record strict nil-symbol diagnostics with reconciliation limited to regular public globals explicitly published later by the same stable addon index through ordinary Lua assignment or named XML frame creation. Nested `C_AddOns.LoadAddOn` publications do not resolve the outer warning, cleared globals and all `C_*` gaps remain warned, and publication-ledger cleanup follows `LoadingAddonGuard`. Updated the existing index summary; no spec change was warranted.

## [2026-08-28] audit | Document runtime-template lifecycle ordering

Audited commits `0f0c3e40d` through `5189a14ef`. Updated [[xml-template-system]] and [[dropdown-intrinsic-script-chain]] plus the top-level XML template architecture doc. Documentation now records that inline `<Scripts>...</Scripts>` parsing preserves following siblings, XML `<KeyValues>` are initialized before deferred template-child `OnLoad`, intrinsic default scripts use precall dispatch ahead of ordinary style handlers, and animation-group mixins are applied before XML method-script binding and `OnLoad`. Corrected the dropdown regression reference to use real script dispatch rather than `GetScript()`'s normal-binding view. No spec or new index entry was warranted; existing page coverage and cross-links remain sufficient.

## [2026-08-28] audit | Align runtime object contract documentation

Audited commit `41205e19c`. Updated the FunctionContainer audit claims to match `tests/userdata_proxy.rs`: userdata type, method exposure, cancellation/invoke suppression, per-instance fields, and read-only keys are covered; tostring formatting and broader identity, callback, timer, lifecycle, and metadata semantics remain unproven. Updated secureenv documentation for dynamically proven shallow table-reference sharing, and clarified that `debug.getfenv(frame)[1]` is a live per-instance field view via `tests/widget_misc_methods.rs::test_frame_debug_env_exposes_live_newindex_fields`.

## [2026-08-27] audit | Default-suite repair slices

Audited commits `5dfbd222c`, `2a876dc25`, `8adadc959`, `d301bd023`, and `af662c223`. No spec or changelog update was warranted: the QuestAsync TOC resolver, Mists-only test gate, duplicate fallback removal, and corrected fixture expectation are test/maintenance changes. Updated [[playerspells-runtime-load]], [[on-update-dirty]], and the index because the SpellBook helper now requires an observable-frame check before falling back, and the settled solo leave-instance path does not query instance/LFG state.

## [2026-08-27] audit | Refresh patch evidence hashes and accept prefork test references

Audited commits `85b00c5cb` and `da273a149`. Refreshed checkout-byte evidence hashes in the 12.0.0, 12.0.5, and 12.1 patch manifests. The patch-manifest validator now treats generated `prefork_full_ui_case!` marker cases as valid named test references alongside ordinary `#[test]` functions, while unmarked functions remain invalid. Updated the patch-manifest and prefork specs plus their wiki system pages; no evidence files or index summary required changes.

## [2026-08-27] system | Migrate timeout-wrapped full-startup cases

Audited commit `d548d0ffc`. Moved four StoreTree and one GuildMemberList normal-retail full-startup cases from ordinary `test_timeout!` libtest into the existing prefork target. The dedicated default-retail target now lists 1,951 cases: 1,941 migrated full-environment cases and 10 manual/nested prefork cases. The remaining ordinary startup-like scan contains 304 tests with zero eligible cases; the owned-timeout exclusion is removed. These cases retain the 120-second per-child timeout and process-tree cleanup.

## [2026-08-27] audit | Finalize prefork eligibility coverage

Updated [[prefork-test-harness]], the prefork spec, and index from `/tmp/prefork-final-eligibility.json` and `/tmp/prefork-final-registry-list.txt`. The dedicated default-retail target lists 1,946 tests: 1,936 migrated full-environment cases and 10 manual/nested prefork cases. The final ordinary startup-like scan contains 309 tests with zero eligible remaining. Exact exclusions: 75 pre-start custom, 9 non-equivalent lifecycle, 113 partial custom, 13 partial domain, 3 partial template, 55 partial thread-sensitive, 15 alternate-screen, 12 render custom, 5 owned-timeout, 7 profile-specific, 1 post-drop global-state, and 1 version-specific. The 211 partial/custom/glue/render/thread-sensitive setup-family cases and the 14 profile/version/owned-timeout/post-drop cases remain ordinary libtest; no code, tests, cargo commands, or commit were run.

## [2026-08-27] system | Migrate generic post-start LoD cases

Audited commits `3faf7ad57` and `9cd3988a7`. Updated [[prefork-test-harness]] and its index summary: 156 newly migrated default-retail post-start LoadOnDemand cases across 17 modules use borrowed-environment child setup with dependency order preserved. `Blizzard_SharedMapDataProviders` remains excluded because its nine-case fixture loads before post-load workarounds and omits startup events. Two PTR-only GuildBank/ItemUpgrade tests were restored to ordinary libtest with profile-specific full startup. GenericTraitUI listing includes three previously migrated cases, so conformance covers 159 listed cases while the unique new migration is 156. Conformance passed 1/1; `is_addon_loaded` passed 135/135; GenericTraitUI publication passed 4/4; each restored PTR test passed 1/1; no orphan processes remained. The generated ordinary full-UI aggregate is now 1,936 cases.

## [2026-08-27] system | Migrate housing post-start LoD cases

Audited commit `ee6f58cbf`. Updated [[prefork-test-harness]] to record 189 post-start housing LoadOnDemand cases across 13 modules migrated with child-only borrowed-environment setup, exact generated-registry conformance of 189, and the representative `is_addon_loaded` proof passing 119/119 with no orphan processes. The generated ordinary full-UI aggregate is now 1,780 cases. Normal-retail shared-preload cases remain distinct from custom child setup; pre-start, partial/custom, glue, and otherwise non-equivalent fixtures remain excluded.

## [2026-08-27] system | Migrate SpellSearch and post-start explicit/LoD cases

Audited commits `e05310123` and `054067c53`. Updated [[prefork-test-harness]] and its index summary: 15 SpellSearch cases now explicitly load `Blizzard_SpellSearch` in each child after normal preload; 96 additional post-start explicit/LoadOnDemand cases across ten modules use borrowed-environment child setup with dependency order preserved. Generated-registry conformance totals 111 cases including SpellSearch; the `is_addon_loaded` behavior filter passed 106/106, and no orphan processes remained. Pre-start or otherwise behaviorally non-equivalent custom fixtures remain excluded pending separate proof. The generated ordinary full-UI aggregate is 1,591 cases (1,480 + 15 + 96), while manual and fixture/conformance cases remain separately categorized.

## [2026-08-27] system | Generate prefork full-UI case registry

Audited commit `fe0d8f5a1`. Updated [[prefork-test-harness]] and the prefork spec to document explicit `prefork_full_ui_case!` marker bodies, `syn`/`quote` build-generated stable `<module>::<function>` registration, generated integration-tree reuse for mixed modules, and the nested registry/preloaded-startup fixture. The current registry contains 12 full-UI cases: 9 manual keybinding cases, 2 behavioral-messaging cases, and 1 nested fixture. Remaining eligible normal-retail full-environment tests are not yet migrated. The existing index entry remains current; no index change was needed.

## [2026-08-26] audit | Document ItemButton runtime load ordering

Audited commit `9b1ba9bcd`. Updated [[addon-load-order]], `docs/addon-load-order-investigation.md`, and the existing index summary to replace the obsolete claim that `ItemButtonUtil` remained unavailable until after `Blizzard_ItemButton`. `WowLuaEnv::new` does not publish it; loading `Blizzard_UIParent` dispatches `UIParent_OnShow`, which runtime-loads `Blizzard_AccountStore`; runtime `C_AddOns` loads the game foundation lane through `Blizzard_FrameXMLUtil`; and `ItemUtil.lua` publishes `ItemButtonUtil` before eager discovery reaches `Blizzard_ItemButton`. The removed intermediate-state test was obsolete; the production load-order snapshot remains unchanged and green. No new wiki page was required.

## [2026-08-26] audit | Document shared-atlas headless rendering

Audited commit `a3f12b265`. Updated [[rendering-pipeline]] and [[texture-atlas]] plus their source docs to record `render_batches_to_images`: union texture/glyph preloading and sequential rendering through one WGPU device, pipeline, target, and GPU atlas. Related before/after images must share that context to model the live persistent atlas and avoid packing-dependent bilinear/UV edge differences.

## [2026-08-26] audit | Document ClearTarget boolean return

Audited commit `f77dd9d7e`. Updated [[lua-api]] and `docs/lua-api.md` to record that global `ClearTarget()` returns `true` only when it clears an existing target, returns `false` otherwise, and preserves `PLAYER_TARGET_CHANGED`.

## [2026-08-26] audit | Document BC-backed mask resolution

Audited commit `c2647c005`. Updated [[mask-texture]], [[rendering-pipeline]], and [[texture-atlas]] plus their source docs to record that deferred mask requests resolve from either the RGBA tiers or BC1/BC3 atlases, remap UVs into the selected slot, and apply the correct mask coverage path. CircleMask-style BC masks no longer become unresolved pending masks and render unmasked.

## [2026-08-26] audit | Clarify prefork cache-disabled contract

Audited commits `4d8136fc4` through `880d58943` against the current prefork loader. Clarified that cache sealing applies only when bytecode caching is enabled; disabled caching remains a successful no-op, and sealing rejects initialized or populated cache state rather than any bypass API call. Added the related bytecode-cache growth cross-link.

## [2026-08-26] system | Bypass bytecode pack during prefork preload

Updated [[prefork-test-harness]] with process-local `ParentBypass` entered before the parent `WowLuaEnv` exists. Prefork preload now compiles source without reading or writing `pack.bin`, seals empty initialized cache state after startup, and transitions children to read-only without allowing them to reload the pack. Separate conformance preserves the warm read-only contract. The serial nine-case benchmark completed in 10.12 seconds; process maximum RSS fell 33.8% and process-tree PSS fell 12.5% against the retained baseline. Aggregate tree RSS rose because it double-counts shared copy-on-write pages.

## [2026-08-26] performance | Release prefork parent bytecode memory

Updated [[prefork-test-harness]] so the completed full-UI parent drops the bytecode cache's in-memory pack and index before forking while preserving disk state and later parent writes. Nine migrated cases pass; child-phase peak PSS fell 40.4% and final parent PSS fell 47.5%. Whole-command maximum RSS remains unchanged because preload allocates the pack before release.

## [2026-08-26] fix | Skip zero-match prefork setup and propagate cleanup restore errors

Updated [[prefork-test-harness]] so non-list filters selecting zero cases return a successful zero-test result without conformance or full-UI preload. The full-UI preload now propagates `Blizzard_EnvironmentCleanup` restoration failures through the result-returning loader API with explicit addon context.

## [2026-08-26] system | Migrate world-map detail cases to full-UI prefork

Updated [[prefork-test-harness]] with lazy argument selection, isolated conformance-before-preload execution, and the normal default-retail game startup snapshot. Nine `test_keybindings_panels_detail` cases now run as immutable 120-second prefork children with read-only bytecode-cache setup; listing bypasses conformance and preload, and the standard integration harness no longer registers those cases.

## [2026-08-26] system | Add read-only prefork bytecode-cache children

Updated [[prefork-test-harness]] with the generic child setup hook and the process-local one-way Lua bytecode-cache mode. Fresh-subprocess conformance now proves child-only setup state, unchanged parent-prewarmed cache bytes/metadata/directory contents across a unique child compile, and continued parent writability. Focused cache tests cover invalid and oversized removal suppression, torn-pack truncation suppression, legacy in-memory hits without file promotion/migration, and skipped append/replacement/temp-file paths.

## [2026-08-26] fix | Make prefork setup and failure cleanup leak-free

Updated [[prefork-test-harness]] after conformance exposed empty split skips and a real `RLIMIT_NOFILE` failure with active children. Parent-owned pipes and setup sockets now use ownership-based closure; a two-way setup handshake verifies the child process group before test release; every parent-side error kills active groups/direct children, reaps direct children, and drops remaining descriptors. Timeout escalation and grandchild-disappearance behavior remain unchanged.

## [2026-08-26] system | Add prefork test harness core

Added [[prefork-test-harness]] for the Linux-only custom test runner introduced by `prefork_full_ui`. The page records the single-thread pre-fork invariant, immutable parent-state borrowing, bounded process workers, structured child outcomes, capture modes, and process-group timeout escalation. Current proof is runner conformance only; real `WowLuaEnv` migration and benchmarking remain open.

## [2026-08-26] audit | Document model no-op and texture identity contracts

Audited commits `b062bab23`, `802516f34`, `847052f41`, `73caa97fe`, and `f79deb010`. Updated [[lua-api]] and [[widget-system]] to record the permanent model-family boundary: Lua-facing model methods such as `ClearFog` remain callable while 3D visual behavior is intentionally unimplemented. Clarified that known texture paths may expose numeric fileDataIDs through `GetTexture()`/`GetTextureFileID()`, while `GetTextureFilePath()` is the source-path assertion API. Updated existing `index.md` summaries; no new wiki page or feature spec was required. Routine fixture, cursor, ready-check, and quest-blob test changes required no documentation.

## [2026-08-26] fix | Preserve widget-handler Lua tracebacks

Updated [[lua-call-frame-restoration]] for commits `430ac3cb8`, `f32ae4470`, `9bcfa511a`, and `326bcf335`. Rust-driven widget dispatch now reaches `debug.traceback` through rilua's native `xpcall` before failed Lua frames unwind, while retaining variadic arguments, original-error fallback, and the existing policy that handled protected-call failures do not enter `lua_error_counts`. Focused `system_api::`, `error_handler::`, and `game_menu::` modules pass.

## [2026-08-26] update | Document chat-window compatibility state

Updated [[lua-api]] and [[post-load-workaround-audit]] for commit `98a48859f`: temporary `__wow_chat_window_state` now stores `SetChatWindowName()` and `SetChatWindowDocked()` values consumed by `GetChatWindowInfo()`. The fields are compatibility state, not saved-layout persistence, and should retire with a modeled chat-layout subsystem. Test-only third-wave changes required no documentation update.

## [2026-08-25] update | Document final retail runtime recovery boundaries

Updated [[lua-api]], [[taint-system]], and [[post-load-workaround-audit]] after the second-wave retail repairs. Documentation now records canonical Font object field precedence for inherited FontStrings; delimiter-receiver `string.split` handling for empty/equal-length inputs; no-profile `C_ClickBindings` behavior; local Encounter Journal numeric-string coercion; `DEFAULT_CHAT_FRAME` assignment without an edit box; and guarded gamepad cursor-global restoration required by the real Collections Escape close stack. No new page or index entry was needed.

## [2026-08-25] update | Preserve final retail runtime compatibility contracts

Audited the final fixes after `0bb44d828`: `2e06ddba7` and `51ea06091` now document `CreateWindow` as a minimal frame-backed external-tool window contract (size/minimum size, topmost, close, and owner attachment; title/focus are no-ops); `25285d13f` documents inert `Kiosk.GetKioskLoginInfo()` defaults; and `ffccfcb6e` documents preserve-mode EnvironmentCleanup restoration for missing UI strings/constants without clobbering Blizzard assignments. Updated [[lua-api]] and [[post-load-workaround-audit]]. No new page was required.

## [2026-08-25] update | Preserve current loader, XML, and runtime-surface contracts

Audited committed behavior from `a3d0f4f21` through `711f8e9c8` and updated [[addon-loading]], [[lua-api]], and [[xml-template-system]]. Documentation now records: loaded-addon idempotence preserving mutable registries such as `StaticPopupDialogs`; enum child-table reseeding that preserves Blizzard extensions; `toplevel="true"` XML frames retaining implicit UIParent so XML strata survives same-parent `OnLoad`; state-backed legacy Timerunning globals; retail `GetGuildTabardFiles`; `C_StringUtil.EscapeDecimalNonPrintables`; and Browser `NavigateTo`/`NavigateHome` no-result compatibility methods. TOC and symbol fixture refreshes were intentionally skipped. No new wiki page was required; existing system pages remain the SSOT.

## [2026-08-25] update | Document rilua global-slot read modes

Updated `design/track-3-global-slot-abi.md` after commit `f0f5312be` pinned the rilua slot-read fix. The default/no-shadow `GETGLOBAL_SLOT` path now documents current root `_G` coherence after bare assignment, `_G.Name = value`, `rawset`, and nil; the optional live-shadow/freeze mode retains frozen snapshot fallback with shadow overrides taking precedence. Added the design page to `index.md` and recorded the required mode-matrix parity proof.

## [2026-08-25] fix | Bound Lua bytecode cache growth

Commit `39caf2662` updates `investigations/bytecode-cache-growth.md`. A valid `WOWBC002` pack had reached **32,316,662,045 bytes** with **25,256,274 unique hashes**, stalling isolated addon loading because the cap applied only at the next load and `read_to_end` preceded the size check. The cache now checks metadata/bounded reads before parsing, enforces serialized size before append, compacts or rebuilds at the limit, and persists before replacing in-memory state. Focused `bytecode_cache` tests cover bounded oversized-pack rejection, compaction, rebuild, oversized entries, failed-append rollback, and legacy promotion. Added `[[bytecode-cache-growth]]` and cross-linked it to `[[track-3-global-slot-abi]]`.

## [2026-08-24] fix | Run AuthChallenge export patch publicly

Commit `b3324ad06` adds `LoaderEnv::exec_public()` for narrow loader code that must run in the public environment without weakening the loading addon's secure file execution. The addon-specific AuthChallenge workaround uses it to restore five callbacks to `_G`; this is not generic secure-to-public mirroring. Updated `[[lua-api]]`.

## [2026-08-24] fix | Model Chromie Time empty state

Commit `3fcfa7d31` adds the retail/PTR `C_ChromieTime` surface. Expansion-option queries return nil or fresh empty tables, while `CloseUI()` and `SelectChromieTimeOption()` are no-ops. Broader Chromie Time state, selection, and UI behavior remain unmodeled. Updated the Lua API architecture references.

## [2026-08-24] fix | Model Catalog Shop virtual-currency product query

Commit `f0396bb82` adds the modeled `C_CatalogShop.GetVCProductInfos()` surface. Each call returns a fresh empty table when no virtual-currency products are seeded; populated Catalog Shop data and broader purchase/catalog semantics remain unmodeled. Updated the Lua API architecture references.

## [2026-08-24] fix | Replay selected Blizzard libraries into secure environment

Commit `7dfc33c3d` expands the evidence-backed secure replay allowlist with `Blizzard_CombatLogBase` and `Blizzard_CatalogShopSharedUtil`. Their `CombatLogUtil` and `CatalogShopUtil` globals are now re-executed into `__secureenv`; the loader does not mirror `_G` generically. Focused tests verify both public and secure bindings. Updated `[[addon-loading]]` and `[[taint-system]]`.

## [2026-08-24] fix | Reuse engine roots for XML definitions

Commit `e5089fbeb2` makes XML definitions for pre-created `UIParent` and `WorldFrame` configure the existing root objects instead of creating replacements. XML mixins, scripts, event registrations, and lifecycle configuration therefore attach to the same objects later observed through `_G.UIParent` and `_G.WorldFrame`. The duplicate path stranded startup scripts and event registrations on the original UIParent and prevented CombatLog runtime loading. Implementation: `src/loader/xml_frame_codegen.rs`.

## [2026-08-24] fix | Guard runtime addon load transactions

Commit `ce67dc961` uses one `SimState`-owned RAII loading transaction for direct and runtime addon loads. Runtime dependency traversal enters the transaction before loading foundations or TOC dependencies; nested re-entry reports the addon as loading but not loaded until file loading and post-load workarounds complete and the owning transaction commits. Guard cleanup restores the previous loading owner after success or failure. Implementation: `src/c_api/c_addons_runtime.rs`, `src/loader/addon.rs`, and `src/lua_api/state.rs`.

## [2026-08-24] correction | Retail manifest family inventory and texture identity

Commits `f8a34a362` and `34a7f7d19` corrected stale tests and establish the current retail source/runtime boundary. The retail Blizzard manifest mirrors the complete Gethe `live` AddOns tree, including `Classic/` and `Mainline/` family variants; this is source completeness, not a second retail load path. Runtime TOC `[Family]` substitution selects `Mainline` for retail. Known resolved texture paths expose numeric fileDataIDs through `Texture:GetTexture()`: class circles `237669` and the question mark `134400`. This supersedes the 2026-07-13 note describing a filtered retail manifest.

## [2026-08-24] investigation | Resolve default-retail startup error cascade

At HEAD `20f141534`, `target/debug/wow-sim --no-addons --no-saved-vars lua-errors` emits `[]` (0 unique errors, 0 occurrences), artifact `/tmp/claude/wow-sim-lua-errors-after-editmode.out`. `ff01991aa` restored Lua call frames after direct errors; `6ac9e32ee` removed the obsolete exact-string retry; and `7d6051797`, `cb75dd007`, and `20f141534` corrected retail EditMode enum publication. The two missing retail members were `Enum.EditModeEncounterEventsSetting.TooltipAnchor=8` and `Enum.DamageMeterVisibility.InGroup=3`, with metadata boundaries `0/13/14` and `0/3/4`. Their absence aborted EditMode initialization. After the enum correction, the remaining EditMode, CooldownViewer, chat, MicroMenu, compact-raid, WorldFrame, ExtraAbility, and related records disappeared without local fixes, identifying them as startup cascades in this baseline.

## [2026-08-13] investigation | Resolve retail 12.0.0 EventScheduler display-info slice

Retail 12.0.0 EventScheduler proof commit `2d4a320cb50140d0fdf8065e81ffbaa8ac68e402` resolves seven occurrences as **best-effort/behavioral**: `C_EventScheduler.EventDisplayInfo.hideDescription`, `EventDisplayInfo.hideTimeLeft`, `EventDisplayInfo.overrideAtlas`, `EventDisplayInfo.overrideTooltipWidgetSetID`, `OngoingEventInfo.displayInfo`, `ScheduledEventInfo.displayInfo`, and `ScheduledEventInfo.eventID`. Focused proof is `src/lua_api/workarounds/temporary/event_scheduler_state.rs::installs_seeded_events_reminders_and_namespace_fallback`; current checkout-byte SHA-256 is `657720512980428bb72e3241ac5861618507919ece665a4acfed9a595d726f59`. Retail seeded ongoing/scheduled events publish displayInfo tables with `hideDescription=false`, `hideTimeLeft=false`, and nil optional overrides; scheduled event IDs are numeric `2001`/`2002`, remain stable after `RequestEvents`, and displayInfo remains populated after refresh. Claims exclude real scheduling/timing, persistence, filtering, UI/rendering, localization, producers outside seeded state, and broader lifecycle semantics. Current totals are **2240 best-effort, 1168 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 total).

## [2026-08-13] investigation | Resolve retail 12.0.0 DamageMeter field/reset slice

Retail 12.0.0 DamageMeter proof commits `2be7293ba4bcb2d9121bff725b2238a76823431c + 451b69e725df4be8d3ffe7ee8c8f131f57a55ae3 + 89fdfe290823a03215cfd2a26795fd5d60b1daae` resolve ten occurrences as **best-effort/behavioral**: `C_DamageMeter.DamageMeterAvailableCombatSession.name`; `DamageMeterCombatSource.amountPerSecond`, `.classFilename`, `.specIconID`; `DamageMeterCombatSpell.amountPerSecond`, `.totalAmount`; `DamageMeterCombatSpellUnitDetails.amount`, `.classification`, `.unitClassFilename`; and `C_DamageMeter.ResetAllCombatSessions`. Focused proof is `src/lua_api/workarounds/temporary/damage_meter_state.rs::patch_12_0_0_damage_meter_fields_and_reset`. It proves seeded populated values/types/relationships, exact reset state/query behavior including both source lookup APIs, and harmless repeated reset. No real combat aggregation, persistence, events, UI, multi-session lifecycle, or production data ingestion is claimed. Current totals are **2233 best-effort, 1175 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 total). Evidence uses checkout-byte hashes: source register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863` and DamageMeter implementation/test `e261d39cd3f752e67eb6fb001a015703e3b2a03beb84b76970678e3835b4460f`.

## [2026-08-13] investigation | Resolve retail 12.0.0 ColorUtil proof slice

Retail 12.0.0 ColorUtil proof commit `d79c8510cbb8cd4b4d8c6f52159743e47115d23f` resolves five occurrences as **best-effort/behavioral**: `C_ColorUtil.ConvertHSLToHSV`, `C_ColorUtil.ConvertHSVToHSL`, `C_ColorUtil.ConvertHSVToRGB`, `C_ColorUtil.ConvertRGBToHSV`, and `C_ColorUtil.WrapTextInColor`. Focused proof is `src/lua_api/workarounds/temporary/color_defaults.rs::patch_12_0_0_color_util_conversions_and_wrapping`; it proves normalized conversion vectors, primary/fractional/achromatic/boundary behavior, exact three-number returns, the tested missing-argument clamping convention, and the exact direct RGB-table/empty-text wrap strings. No localization, untested out-of-range input contract, consumers, rendering, persistence, or broader color-system semantics are claimed. Current totals are **2223 best-effort, 1185 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 total). Evidence uses checkout-byte hashes: source register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, ColorUtil implementation/test `52410dca483d3dbcee8dab178fc76d3591eee110d96f64f6a1f5735821b27f86

## [2026-08-13] investigation | Resolve retail 12.0.0 combat-log producer slice

Retail 12.0.0 producer proof commit `6bcd6ded2ada37c5f78ba4b98387549f2430369c` resolves ten occurrences as **best-effort/behavioral**: events `COMBAT_LOG_APPLY_FILTER_SETTINGS`, `COMBAT_LOG_ENTRIES_CLEARED`, `COMBAT_LOG_MESSAGE`, `COMBAT_LOG_MESSAGE_LIMIT_CHANGED`, and `COMBAT_LOG_REFILTER_ENTRIES`; APIs `C_CombatLog.ApplyFilterSettings`, `C_CombatLog.ClearEntries`, `C_CombatLog.RefilterEntries`, `C_CombatLog.SetMessageLimit`, and `C_CombatLogSecure.CreateCombatLogMessage`. Focused proof is `src/lua_api/workarounds/temporary/combat_log_state.rs::patch_12_0_0_combat_log_producers_dispatch_exact_notifications`. It proves only exact synchronous producer notification, payload arity/types/values, corresponding temporary-state mutation where asserted, and one notification per repeated explicit call. `Frame:RegisterEventCallback` now retains supplied callbacks so restricted callback delivery is observable. No filtering algorithms/effects, retention, message-limit enforcement, secure/taint restrictions, UI/rendering, persistence, or broader lifecycle semantics are claimed. Current totals are **2218 best-effort, 1190 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 total). Evidence uses checkout-byte hashes: source register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, combat-log implementation/test `da7f5ecf1118b268dce86327d639c76cfc2e389385dfa8bc1d8ea848f8e463d8`, callback bridge `2b2c36966fe5fc1b554dcf781081606e60a441d2b7b73cba04fbffcda1125ba8`, and event registries `5c8daa7a3be7550d53b14733588aa0687f5bb23b2c608a3f13d1360595dac88e` / `a3c4acf8bf96423f3781a2615675c3fd16fa8d7eb597dadc78ca77b306f0028b`.

## [2026-08-12] investigation | Classify 76 removed retail 12.0.0 CVars

## [2026-08-12] investigation | Classify final 11 retail 12.0.0 occurrences

Retail 12.0.0 final 11-row slice: `C_PerksActivities.PerksActivityRequirement.completed`, `C_PerksActivities.PerksActivityRequirement.requirementText`, `HOUSING_DECOR_NUDGE_STATUS_CHANGED`, `LEARNED_SPELL_IN_TAB`, `NewCraftingOrderInfo.reagentItems`, `RegularReagentInfo.itemID`, `math.huge`, and `math.pi` are **evidence-required/unsafe** with `commit: null`, `tests: []`, and exact populated-state/full-LoD or Lua-compatibility probes. `docs.extra_events.COMBAT_LOG_APPLY_FILTER_SETTINGS`, `docs.extra_events.COMBAT_LOG_REFILTER_ENTRIES`, and `docs.extra_script_objects.FrameAPITooltip` are **best-effort/provenance-only** with `commit: null`, `tests: []`, and `assertions: []`; claims are endpoint provenance only. Current totals are **2208 best-effort, 1200 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 total).

Retail 12.0.0 removed CVar slice: 75 rows are **best-effort/behavioral**, bounded only to old-name `GetCVar` and `GetCVarDefault` absence. `activeCUFProfile` and `currencyTokensBackpack2` use proof commit `15a7936e69e00d3efd297a056e92218f717ed7a8`; the other 73 proven names use proof commit `16f4b6956`, all through `src/loader/tests/wow_api_globals/patch_12_0_0_cvar_removals.rs::test_patch_12_0_0_removed_nameplate_cvars`, current test hash `b39e972d1cd9eb8401d9c9f7138804f6819ebd091b5f17b53cb7357cd5761292`. `nameplateShowFriendlyNPCs` is **evidence-required/unsafe**: source removes uppercase NPCs and adds lowercase Npcs, while simulator lookup lowercases keys and currently resolves the old spelling to the new entry; required proof must establish lowercase value/default, uppercase absence, and retail case sensitivity. No replacement/equivalence, domain/UI/rendering, mutation, persistence, consumer, producer, or lifecycle semantics are claimed. Current totals are **2205 best-effort, 1192 evidence-required, 2 exception-requested, and 11 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify removed 12.0.0 global constants

Retail 12.0.0 removed globals `LE_FRAME_TUTORIAL_LINK_TRANSMOG_OUTFIT`, `LE_FRAME_TUTORIAL_TRANSMOG_OUTFIT_DROPDOWN`, `LE_WORLD_ELAPSED_TIMER_TYPE_CHALLENGE_MODE`, `LE_WORLD_ELAPSED_TIMER_TYPE_NONE`, and `LE_WORLD_ELAPSED_TIMER_TYPE_PROVING_GROUND` are **best-effort/behavioral** using proof commit `e385636fe` via `src/loader/tests/wow_api_globals/patch_12_0_0_removed_global_constants.rs::test_patch_12_0_0_removed_global_constants`. The focused test asserts `rawget(_G, name) == nil` for all five in the retail 12.0.0 epoch; runtime preserves shared definitions for excluded profiles through epoch-specific post-compat filtering. Claims are bounded to retail 12.0.0 startup global absence only; no replacement, tutorial, elapsed-timer, challenge/proving-ground, UI, producer, consumer, mutation, persistence, or lifecycle semantics are claimed. Current totals are **2130 best-effort, 1191 evidence-required, 2 exception-requested, and 87 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify changed raid, StatusBar, scrub, and string.trim rows

Retail 12.0.0 exact-row update: `CanBeRaidTarget`, `ClearRaidMarker`, `IsRaidMarkerActive`, `PlaceRaidMarker`, `RemoveRaidTargets`, `SetRaidTarget`, and `StatusBar` are **evidence-required/unsafe** with `commit: null`, `tests: []`, and state-backed/full-LoD probes bounded to signatures, types, state, invalid inputs, errors, and observable delivery. `scrub` is **best-effort/provenance-only** with `commit: null`, `tests: []`, and `assertions: []`; it removes inaccurate metadata without a behavior claim. `string.trim` is **evidence-required/unsafe** with a probe for custom characters, defaults, wrong types, return values, errors, and publication. Raid registration or event firing alone is not behavior proof; current raid-marker state is missing, StatusBar still exposes old string/field behavior, and the bootstrap `string.trim` alias is not contract proof. Claims exclude unsupported producer, lifecycle, UI, rendering, mutation, persistence, and consumer semantics. Current totals are **2125 best-effort, 1191 evidence-required, 2 exception-requested, and 92 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify removed floating-combat-text CVars

Retail 12.0.0 removes 26 rows in this slice: 25 floating-combat-text CVars plus `enablePetBattleFloatingCombatText` are **best-effort/behavioral** because focused proof commit `15a7936e69` via `src/loader/tests/wow_api_globals/patch_12_0_0_cvar_removals.rs::test_patch_12_0_0_removed_nameplate_cvars` asserts both public getters return nil for each old name. `floatingCombatTextAuraFade` is **best-effort/provenance-only** with `commit: null`, `tests: []`, and `assertions: []`: its value `'0'` appears only in an intermediate snapshot and the name is absent at both endpoints. Claims are bounded to old-name getter/default absence for the 26 proven removals and endpoint provenance for AuraFade; no `_v2`/replacement equivalence, rendering, mutation, persistence, consumer, Pet Battle, or lifecycle semantics are claimed. Evidence uses the current source register, CVar defaults, generic CVar implementation, focused removal test, and discovery hashes. Current totals are **2119 best-effort, 1149 evidence-required, 2 exception-requested, and 140 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify Pet Journal, spell-overlay, and tracker CVar formatting

Retail 12.0.0 changed CVar rows `petJournalFilters`, `petJournalSourceFilters`, `petJournalTypeFilters`, `spellActivationOverlayOpacity`, and `superTrackerDist` are **best-effort/behavioral**. Focused proof commit `8b64275a8` via `src/loader/tests/wow_api_globals/patch_12_0_0_pet_journal_overlay_tracker_cvar_values.rs::test_patch_12_0_0_pet_journal_overlay_tracker_cvar_values` asserts both `GetCVar` and `GetCVarDefault` return exact strings `0`, `0`, `0`, `0.650000`, and `0.750000` respectively. Runtime defaults already matched; only focused public getter proof was missing. Claims are bounded to startup getter/default publication only; no Pet Journal/filter, spell-overlay, tracker/rendering, mutation, persistence, consumer, or lifecycle semantics are claimed. Current totals are **2092 best-effort, 1149 evidence-required, 2 exception-requested, and 167 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify nameplate and party CVar formatting

Retail 12.0.0 changed CVar rows nameplateLargerScale, nameplateMaxAlpha, nameplateMaxAlphaDistance, nameplateMaxDistance, nameplateMaxScale, nameplateMaxScaleDistance, nameplateMinAlpha, nameplateMinAlphaDistance, nameplateMinScale, nameplateMinScaleDistance, nameplateOccludedAlphaMult, nameplateOverlapH, nameplateOverlapV, nameplatePlayerLargerScale, nameplatePlayerMaxDistance, nameplateSelectedAlpha, nameplateSelectedScale, nameplateSelfAlpha, nameplateShowSelf, nameplateTargetBehindMaxDistance, partyBackgroundOpacity are **best-effort/behavioral**. Focused proof commit `17e13eb21` via `src/loader/tests/wow_api_globals/patch_12_0_0_nameplate_cvar_changed_values.rs::test_patch_12_0_0_nameplate_cvar_changed_values` asserts both `GetCVar` and `GetCVarDefault` return the exact strings 1.200000, 1.000000, 40.000000, 60.000000, 1.000000, 10.000000, 0.600000, 10.000000, 0.800000, 10.000000, 0.400000, 0.800000, 1.100000, 1.800000, 60.000000, 1.000000, 1.200000, 0.750000, 0, 0.100000, 0.500000 respectively. Eighteen defaults already matched the current runtime; the proof commit added only the missing defaults `nameplateLargerScale='1.200000'`, `nameplatePlayerLargerScale='1.800000'`, and `nameplateSelfAlpha='0.750000'`. Claims are bounded to startup getter/default publication only; no nameplate/party rendering, scale, alpha, overlap, mutation, persistence, consumer, or lifecycle semantics are claimed. Evidence hashes: source register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, `src/cvars.yaml` `21dba80cc85568849bd9512b1c29b6007665fcf539e0ebb6622d2384490ddf10`, `src/cvars.rs` `e4a1e46a35988dd137b95c4afeab0c95f6e65516ef8401f25397f3f2a1a13e6d`, focused test `59ce30f69a07bfe3429a590599ac8485ebbdf3d110e83612f117badaf7beed00`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Current totals are **2087 best-effort, 1149 evidence-required, 2 exception-requested, and 172 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify advanced-flight CVar formatting

The five retail 12.0.0 advanced-flight CVar formatting changes `advFlyKeyboardMaxPitchFactor`, `advFlyKeyboardMaxTurnFactor`, `advFlyKeyboardMinPitchFactor`, `advFlyKeyboardMinTurnFactor`, and `advFlyPitchControlCameraChase` are **best-effort/behavioral**. Focused proof commit `71086bb79` via `patch_12_0_0_adv_fly_cvar_values.rs::test_patch_12_0_0_adv_fly_cvar_values` asserts both `GetCVar` and `GetCVarDefault` return the exact strings `5.000000`, `8.000000`, `2.500000`, `5.000000`, and `20.000000` respectively. Claims are bounded to startup getter/default publication; flight-control, camera, mutation, persistence, consumer, and semantic-equivalence behavior remain unclaimed. Current totals are **2062 best-effort, 1149 evidence-required, 2 exception-requested, and 197 untriaged rows** (3410 total).

Retail 12.0.0 changed events `VOICE_CHAT_TTS_PLAYBACK_STARTED` and `VOICE_CHAT_TTS_PLAYBACK_FINISHED` are **evidence-required/unsafe**. STARTED reduces four payloads `(numConsumers, utteranceID, durationMS, destination)` to one numeric `utteranceID`; FINISHED reduces three `(numConsumers, utteranceID, destination)` to one numeric `utteranceID`, with removed fields absent rather than nullable placeholders. Runtime registers both events but has no TTS playback producer or queue. Full-LoD probes must trigger real playback, verify exact one-argument delivery, identity/timing, repeats/overlap/completion/failure, removed-field absence, and manual-vs-producer dispatch. Claims are bounded to source payload reduction and current producer gap; no TTS lifecycle semantics are claimed. Current totals are **2057 best-effort, 1149 evidence-required, 2 exception-requested, and 202 untriaged rows** (3410 total).

Retail 12.0.0 changed EditModeAuraFrameSetting metadata rows `MaxValue` (7→10) and `NumValues` (8→11) are **best-effort/behavioral**. Focused proof commit `234d1163b` passed 1 test, 0 failed, 1545 filtered and asserts the aura metadata table with numeric `MinValue=0`, `MaxValue=10`, and `NumValues=11`; explicit members publish through `edit_mode.rs`, matching metadata comes from the guarded fallback. Claims are bounded to startup metadata publication only: no Edit Mode aura behavior or consumer semantics are claimed. Current totals are **2054 best-effort, 1147 evidence-required, 2 exception-requested, and 207 untriaged rows** (3410 total).

Retail 12.0.0 changed startup values `EmitterCombatRange`, `NonEmitterCombatRange`, `LE_EXPANSION_LEVEL_CURRENT`, `LE_EXPANSION_LEVEL_PREVIOUS`, `NUM_LE_EXPANSION_LEVELS`, `NUM_LE_FRAME_TUTORIALS`, and `NUM_LE_PET_JOURNAL_FILTERS` are **best-effort/behavioral**. Proof commit `cb2c4abde` passed `1` test, `0` failed, `1545` filtered: globals are Lua numbers with exact values `CURRENT=11`, `PREVIOUS=10`, `NUM_EXPANSION=11`, `NUM_FRAME_TUTORIALS=163`, `NUM_PET_FILTERS=4`; both `GetCVar` and `GetCVarDefault` return `EmitterCombatRange=900.000000` and `NonEmitterCombatRange=6400.000000`. `LE_EXPANSION_LEVEL_CURRENT` uses explicit core-string publication; the other globals use guarded fallback; CVars use `src/cvars.yaml` and generic storage. Claims are bounded to startup numeric publication or exact current/default CVar strings; no expansion, Pet Journal, combat-range, consumer, mutation, or persistence semantics are claimed. Current totals are **2052 best-effort, 1147 evidence-required, 2 exception-requested, and 209 untriaged rows** (3410 total).

Retail 12.0.0 changed `Enum.CraftingOrderResult` rows `MissingItem`, `MissingNpc`, `MissingOrder`, `MissingRecraftItem`, `NoAccountItems`, `NotClaimed`, `NotCrafted`, `NotInGuild`, `NotYetImplemented`, `OutOfPublicOrderCapacity`, `ServerIsNotAvailable`, `ThrottleViolation`, `TargetCannotCraft`, `TargetLocked`, `Timeout`, `TooManyItems`, and `WrongVersion` are **best-effort/behavioral**. Their authoritative numeric values change respectively from `30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46` to `31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,47,48`; guarded `missing_enums.lua` publication matches, with no competing explicit family publication. Focused proof commit `f6db33a61` passed `1` test, `0` failed, `1544` filtered and asserts exact Lua numeric members plus `MinValue=0`, `MaxValue=48`, `NumValues=49`. Claims are bounded to retail 12.0.0 startup enum publication only; no crafting-order production, interpretation, consumer, persistence, transition, or lifecycle semantics are claimed. Current totals are **2045 best-effort, 1147 evidence-required, 2 exception-requested, and 216 untriaged rows** (3410 total).

The retail 12.0.0 changed C API rows `C_Item.CanItemTransmogAppearance`, `C_Item.GetItemInfo`, `C_ItemInteraction.ItemInteractionFrameInfo.flags`, `C_LFGList.DoesEntryTitleMatchPrebuiltTitle`, `C_LFGList.GetPlaystyleString`, `C_LFGList.SetEntryTitle`, `C_PerksActivities.PerksActivityInfo.criteriaList`, `C_PerksActivities.PerksActivityInfo.requirementsList`, `C_Reputation.GetFactionParagonInfo`, `C_SpecializationInfo.GetSpecializationInfo`, and `C_VoiceChat.SpeakText` are **evidence-required/unsafe**; `C_Reputation.IsFactionParagon` is **best-effort/provenance-only** because only its boolean output field name changes. Unsafe rows retain `commit=null` and `tests=[]` with exact full-LoD/state-backed probes for publication, arity, types, nullability, state, invalid inputs, and errors; the provenance-only rename has empty assertions and no behavior claim. Claims are bounded to authoritative source-contract changes and current simulator gaps; no item, LFG, perks, reputation, specialization, voice, or consumer semantics are claimed. Current totals are **2028 best-effort, 1147 evidence-required, 2 exception-requested, and 233 untriaged rows** (3410 total). The pre-update manifest SHA-256 used in these rows is `9e68c96fb0d22ad2f34177feefcae9cefd711cbd6e504021c4b09a0590ae9453`.

Retail 12.0.0 exact-row update: `UnitCastingInfo`, `UnitChannelInfo`, `UnitFullName`, `UnitName`, `UnitNameUnmodified`, and `VOICE_CHAT_TTS_PLAYBACK_FAILED` are **evidence-required/unsafe** with `commit=null`, `tests=[]`, and full-LoD/state-backed probes bounded to output/event count, order, types, nullability, state, invalid inputs, and errors. `UnitIsUnit` is **best-effort/provenance-only** with `commit=null`, `tests=[]`, and `assertions=[]`: only parameter names change from `unitName1/unitName2` to `unit1/unit2`; no behavior claim is made. Current totals are **2027 best-effort, 1136 evidence-required, 2 exception-requested, and 245 untriaged rows** (3410 total). Source/runtime/discovery hashes are recorded on each manifest row; the pre-update manifest SHA-256 was `d171dbaf3ff34774328468dbedca478316efadb936667d54f6f2a07a65813f06`. Claims do not establish unit, naming, casting, channel, or TTS consumer/lifecycle semantics.

The 15 retail 12.0.0 changed event rows TRANSMOG_OUTFITS_CHANGED, UNIT_SPELLCAST_CHANNEL_START, UNIT_SPELLCAST_CHANNEL_STOP, UNIT_SPELLCAST_CHANNEL_UPDATE, UNIT_SPELLCAST_DELAYED, UNIT_SPELLCAST_EMPOWER_START, UNIT_SPELLCAST_EMPOWER_STOP, UNIT_SPELLCAST_EMPOWER_UPDATE, UNIT_SPELLCAST_FAILED, UNIT_SPELLCAST_FAILED_QUIET, UNIT_SPELLCAST_INTERRUPTED, UNIT_SPELLCAST_SENT, UNIT_SPELLCAST_START, UNIT_SPELLCAST_STOP, and UNIT_SPELLCAST_SUCCEEDED are **evidence-required/unsafe**. Exact source contracts: TRANSMOG_OUTFITS_CHANGED changes `nil` to `(newOutfitID: number?)`; CHANNEL_START changes `(unitTarget, castGUID, spellID)` to those fields plus `castBarID: number?`; CHANNEL_STOP changes the same three fields to `(unitTarget, castGUID, spellID, interruptedBy: string, castBarID: number?)`; CHANNEL_UPDATE, DELAYED, EMPOWER_START, EMPOWER_UPDATE, FAILED, and FAILED_QUIET add nullable numeric `castBarID`; EMPOWER_STOP changes `(unitTarget, castGUID, spellID, complete: boolean)` to that tuple plus `interruptedBy: string` and `castBarID: number?`; INTERRUPTED changes the same three fields to `(unitTarget, castGUID, spellID, interruptedBy: string, castBarID: number?)`; SENT changes the first field from `unit: string` to `unitTarget: unit`; START, STOP, and SUCCEEDED declare `(unitTarget, castGUID, spellID, castBarID: number?)`. Current START producers emit only `player, spellID`; current STOP/SUCCEEDED producers emit only three arguments; the other rows have valid event registration but no dedicated producer/payload construction. Required full-LoD/state-backed probes cover real transitions, exact argument count/order/types/nullability, invalid and repeated transitions, GUID/timing, and manual-versus-producer dispatch. Registration and generic/manual dispatch are not behavior proof. Claims are bounded to these source contract changes and current simulator gaps; no spellcast or outfit lifecycle semantics are claimed. Current totals are **2026 best-effort, 1130 evidence-required, 2 exception-requested, and 252 untriaged rows** (3410 total).

The retail 12.0.0 `LE_GAME_ERR_CHARTER_NEIGHBORHOOD_RENAME` change is **best-effort/behavioral**. Source value changes from 1223 to 1224; the prior runtime published 1225, and commit `5789c90d0` corrected the guarded fallback. Focused test `test_patch_12_0_0_le_game_err_charter_neighborhood_rename` observed RED before the correction and GREEN afterward, asserting Lua number 1224. Claims are bounded to startup numeric constant publication; no localized error-message or consumer semantics are claimed. Current totals are **2025 best-effort, 1115 evidence-required, 2 exception-requested, and 268 untriaged rows** (3410 total). Existing references to `src/lua_api/globals/enum_data/missing_constants.lua` were refreshed to SHA-256 `9ff5551f50aa8b3a52628eb2404312b38d41a0c9e242fbdb96665ad1e8b2a0ba`.

The retail 12.0.0 `LE_GAME_ERR_GUILD_NEIGHBORHOOD_BUILT_HOUSE_S` change is **best-effort/behavioral**. Source value changes from 1219 to 1220; prior runtime/discovery published 1221, and commit `5d4c9bda2` corrected the guarded fallback. Focused RED/GREEN proof asserts Lua number 1220. Claims are bounded to startup numeric constant publication; no localized error-message or consumer semantics are claimed.

The retail 12.0.0 `LE_GAME_ERR_GUILD_NEIGHBORHOOD_NEW_SUBDIVISION` change is **best-effort/behavioral**. Source value changes from 1221 to 1222; prior runtime/discovery published 1223, and commit `5d4c9bda2` corrected the guarded fallback. Focused RED/GREEN proof asserts Lua number 1222. Claims are bounded to startup numeric constant publication; no localized error-message or consumer semantics are claimed.

The retail 12.0.0 `LE_GAME_ERR_GUILD_NEIGHBORHOOD_SOLD_HOUSE_S` change is **best-effort/behavioral**. Source value changes from 1220 to 1221; prior runtime/discovery published 1222, and commit `5d4c9bda2` corrected the guarded fallback. Focused RED/GREEN proof asserts Lua number 1221. Claims are bounded to startup numeric constant publication; no localized error-message or consumer semantics are claimed.

The retail 12.0.0 `LE_GAME_ERR_GUILD_NEIGHBORHOOD_RENAME_S` change is **best-effort/behavioral**. Source value changes from 1222 to 1223; the prior runtime and independent discovery published 1224, and commit `2c765200d` corrected the guarded fallback. Focused RED/GREEN proof asserts Lua number 1223. Claims are bounded to startup numeric constant publication; no localized error-message or consumer semantics are claimed. Existing `missing_constants.lua` references use SHA-256 `9ff5551f50aa8b3a52628eb2404312b38d41a0c9e242fbdb96665ad1e8b2a0ba`. Current totals are **2057 best-effort, 1147 evidence-required, 2 exception-requested, and 204 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify restriction and transmog CVar additions

The nine retail 12.0.0 CVar additions `petJournalFilterVersion`, `secretChallengeModeRestrictionsForced`, `secretCombatRestrictionsForced`, `secretEncounterRestrictionsForced`, `secretMapRestrictionsForced`, `secretPvPMatchRestrictionsForced`, `showAllItemsInTransmog`, `showCustomSetDetails`, and `trackedInitiativeTasks` are **best-effort/behavioral**. Focused proof is commit `417521854` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts both `GetCVar` and `GetCVarDefault` return each exact startup/default string, including the empty string for `trackedInitiativeTasks`. Claims are bounded to getter/default publication only: no UI, secret-restriction enforcement, transmog, initiative-task, mutation, persistence, consumer, or later-epoch semantics are claimed. The shared focused-test evidence hash was refreshed for every existing manifest row referencing that file. Current totals are **2014 best-effort, 1110 evidence-required, 2 exception-requested, and 284 untriaged rows** (3410 total).

The retail 12.0.0 rows `COMBAT_LOG_EVENT`, `COMBAT_LOG_EVENT_UNFILTERED`, and `C_Housing.RequestHouseFinderNeighborhoodData` are **evidence-required/unsafe**. The combat-log declarations add callback/restricted metadata while retaining null payload metadata; the simulator models event classification and generic dispatch but lacks producer-backed payload and restricted-delivery proof. The housing API adds a required `neighborhoodName` string; the simulator temporary workaround accepts two arguments but ignores selection/validation and always schedules seeded data. Required full-LoD/state-backed probes cover registration rejection, callback delivery, producer transitions, exact payloads, restricted context, callback versus OnEvent precedence, invalid inputs, event spelling/timing, repeated requests, cancellation, and lifecycle. Claims are bounded to source declarations and current local behavior; no producer, payload, timing, restriction, or consumer semantics are claimed. Current totals are **2005 best-effort, 1110 evidence-required, 2 exception-requested, and 293 untriaged rows** (3410 total).

The eight retail 12.0.0 transmog/LFG/faction CVar additions `lastTransmogCustomSetIDNoSpec, lastTransmogCustomSetIDSpec1, lastTransmogCustomSetIDSpec2, lastTransmogCustomSetIDSpec3, lastTransmogCustomSetIDSpec4, lastTransmogOutfitIDNoSpec, lfgListAdvancedFiltersVersion, majorFactionRenownMap` are **best-effort/behavioral**. Focused proof is commit `e2ac401f5` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts exact public getter/default strings. Claims are bounded to startup getter/default publication; no transmog, LFG, faction-renown, mutation, persistence, or consumer semantics are claimed. Current totals are **1993 best-effort, 1107 evidence-required, 2 exception-requested, and 308 untriaged rows** (3410 total).

The retail 12.0.0 `floatingCombatTextAuraFade` row is **best-effort/provenance-only**: the source register records transient value `"0"` in an intermediate snapshot, but the CVar is absent at both patch endpoints and absent from `src/cvars.yaml`. The distinct `floatingCombatTextAuraFade_v2` CVar is not treated as equivalent. No test, commit proof, or runtime behavior is claimed. Current totals are **1985 best-effort, 1103 evidence-required, 2 exception-requested, and 320 untriaged rows** (3410 total).

The ten retail 12.0.0 raid-frame/spell-diminish/transmog CVar additions `raidFramesCenterBigDefensive, raidFramesDispelIndicatorOverlay, raidFramesDispelIndicatorType, raidFramesDisplayLargerRoleSpecificDebuffs, raidFramesHealthBarColor, scriptWarnings, spellDiminishPVPEnemiesEnabled, spellDiminishPVPOnlyTriggerableByMe, transmogHideIgnoredSlots, transmogrifySetsFilters` are **best-effort/behavioral**. Focused proof is commit `f3ba9786d` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts exact public getter/default strings. Claims are bounded to startup getter/default publication; no raid-frame, spell-diminish, transmog, mutation, persistence, or consumer semantics are claimed. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

The ten retail 12.0.0 nameplate CVar additions `nameplateAuraScale, nameplateCastBarDisplay, nameplateDebuffPadding, nameplateEnemyPlayerAuraDisplay, nameplateFriendlyPlayerAuraDisplay, nameplateInfoDisplay, nameplateShowCastBars, nameplateShowClassColor, nameplateShowFriendlyClassColor, nameplateShowFriendlyNpcs` are **best-effort/behavioral**. Focused proof is commit `bd8ce82d2` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts exact public getter/default strings. Claims are bounded to startup getter/default publication; no nameplate rendering or consumer semantics are claimed. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

The four retail 12.0.0 docs-extra additions `CharCreateAnimTurnType`, `CharSectionCondition`, `COMBAT_LOG_EVENT_INTERNAL_UNFILTERED`, and `FrameAPITooltip` are **provenance-only**: source records transient/null entries with no declared behavior, so no runtime claims are made. The three docs-extra API/event rows `C_CombatLogInternal.GetCurrentEventInfo`, `COMBAT_LOG_APPLY_FILTER_SETTINGS`, and `COMBAT_LOG_REFILTER_ENTRIES` are **evidence-required/unsafe** because full-LoD namespace/callback publication, restriction, firing, and payload behavior remain unproven. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

The retail 12.0.0 `nameplateEnemyNpcAuraDisplay` CVar is **best-effort/behavioral**. Focused proof is commit `16687bd2a` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts both public getters return the authoritative two-byte string `0x02,C`. Claims are bounded to getter/default publication; no nameplate rendering or consumer semantics are claimed. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

The ten retail 12.0.0 floating-combat-text `_v2` CVar additions `floatingCombatTextEnergyGains_v2`, `floatingCombatTextFloatMode_v2`, `floatingCombatTextFriendlyHealers_v2`, `floatingCombatTextHonorGains_v2`, `floatingCombatTextLowManaHealth_v2`, `floatingCombatTextPeriodicEnergyGains_v2`, `floatingCombatTextPetMeleeDamage_v2`, `floatingCombatTextPetSpellDamage_v2`, `floatingCombatTextReactives_v2`, `floatingCombatTextRepChanges_v2` are **best-effort/behavioral**. Focused proof is commit `abee2e56bb` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts both `GetCVar` and `GetCVarDefault` exact startup/default strings. Claims are bounded to public getter/default publication; no rendering, mutation, persistence, consumer, or semantic-equivalence behavior is claimed. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

## [2026-08-12] investigation | Mark security access APIs evidence-required

The five retail 12.0.0 security APIs `canaccessallvalues`, `canaccesssecrets`, `canaccesstable`, `canaccessvalue`, and `dropsecretaccess` are **evidence-required/unsafe**. The simulator provides local shallow/deep access checks for some helpers, while `canaccesssecrets` and `dropsecretaccess` are missing; permissive taint fallbacks are not retail proof. Required probes cover mixed/direct/nested/tainted/zero-argument values, access-policy capability states, non-table inputs and fallback precedence, protected/propagated values, and an observable `dropsecretaccess` state mutation. Claims are bounded to declared signatures and current local/missing behavior, with no complete retail secret, taint, protected-value, propagation, or access-control semantics. Current totals are **1984 best-effort, 1103 evidence-required, 2 exception-requested, and 321 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify viewed transmog and TTS event gaps

The seven retail 12.0.0 viewed-transmog/TTS event rows are **evidence-required/unsafe**: producer-backed transitions and exact handler delivery remain unproven. Required probes assert zero payload for the first four, numeric TransmogOutfitSlot/TransmogType/TransmogOutfitSlotOption for slot save, numeric TransmogOutfitSlot/TransmogOutfitSlotOption for weapon-option changes, and numeric utteranceID/string bookmarkName for TTS bookmarks. Claims are bounded to declarations and registerability; no firing, timing, ordering, lifecycle, transmog, or TTS semantics are claimed. Current totals are **1949 best-effort, 1095 evidence-required, 2 exception-requested, and 364 untriaged rows** (3410 total).

## [2026-08-12] investigation | Mark added unit APIs evidence-required

The twenty retail 12.0.0 unit API/structure rows `UnitCastingDuration`, `UnitChannelDuration`, `UnitClassFromGUID`, `UnitEmpoweredChannelDuration`, `UnitEmpoweredStageDurations`, `UnitEmpoweredStagePercentages`, `UnitHealPredictionValues.totalHealAbsorbs`, `UnitHealthMissing`, `UnitHealthPercent`, `UnitIsLieutenant`, `UnitIsMinion`, `UnitIsNPCAsPlayer`, `UnitNameFromGUID`, `UnitPowerMissing`, `UnitPowerPercent`, `UnitSexBase`, `UnitShouldDisplaySpellTargetName`, `UnitSpellTargetClass`, `UnitSpellTargetName`, and `UnitThreatLeadSituation` are **evidence-required/unsafe**. Current runtime publication is absent, divergent, or temporary: `UnitHealthPercent` and `UnitPowerPercent` return legacy numeric percentages despite declared table results, while `UnitHealPredictionValues.totalHealAbsorbs` is only a synthetic numeric zero field. Required probes must establish state-backed return types, nilability, optional flags/curve/power-type behavior, active cast/channel/empowered states, GUID mappings, unknown-unit behavior, classification, spell-target state, and threat state. Adjacent APIs and synthetic tests do not prove these contracts. Claims are bounded to authoritative declarations and current runtime observations; no complete unit, prediction, cast, security, or lifecycle semantics are claimed. Current totals are **1921 best-effort, 1088 evidence-required, 2 exception-requested, and 399 untriaged rows** (3410 total).

## [2026-08-12] investigation | Mark added interaction APIs evidence-required

The sixteen retail 12.0.0 added API rows `Region.SetVertexColorFromBoolean`, `RegisterEventCallback`, `RegisterUnitEventCallback`, `SetCursorPosition`, `SetTableSecurityOption`, `ShowCloak`, `ShowHelm`, `ShowingCloak`, `ShowingHelm`, `SimpleScriptRegionAPI.IsAnchoringSecret`, `SimulateMouseClick`, `SimulateMouseDown`, `SimulateMouseUp`, `SimulateMouseWheel`, `UnregisterEventCallback`, and `UnregisterUnitEventCallback` are **evidence-required/unsafe**. Current publication is absent, inert, external-only, or a local approximation; no focused proof establishes the requested observable branch, callback, cursor, table-security, transmog, anchoring, or mouse semantics. Required full-LoD probes are recorded per row; claims are bounded to the authoritative declarations and current runtime observations, with no claims about rendering, input, taint/security, transmog, callback, or retail anchoring semantics. Current totals are **1921 best-effort, 1068 evidence-required, 2 exception-requested, and 419 untriaged rows** (3410 total).

The five retail 12.0.0 added rows `GetCollapsingStarCost`, `HouseExteriorTypeOption.houseExteriorTypeID`, `IsRaidMarkerSystemEnabled`, `NewCraftingOrderInfo.reagentInfos`, and `RegularReagentInfo.reagent` are **evidence-required/unsafe**. The first requires a full-LoD numeric return probe; the second requires a producer-backed returned option object with numeric `houseExteriorTypeID`, because temporary fixture data is not retail contract proof; the third requires a Retail callable boolean result and backing state, and the Mists-only false shim does not satisfy Retail; the last two require obtainable crafting-order objects with typed `reagentInfos` entries and typed `CraftingReagent` `reagent` fields. Claims are bounded to authoritative 12.0.0 declarations and current runtime gaps/unmodeled producers; no semantics, values, ordering, lifecycle, or checked-in-absence claims are made. Current totals are **1921 best-effort, 1034 evidence-required, 2 exception-requested, and 453 untriaged rows** (3410 total).

The four retail 12.0.0 removed docs-extra API rows `docs.extra_apis.ShowCloak`, `docs.extra_apis.ShowHelm`, `docs.extra_apis.ShowingCloak`, and `docs.extra_apis.ShowingHelm` are **evidence-required/unsafe**. Source removal and absent checked-in registrations do not prove full-LoD runtime absence or replacement semantics; required probes must distinguish removed docs-extra entries from separate same-name added APIs. Claims are bounded to source removal only. Current totals are **1919 best-effort, 1029 evidence-required, 2 exception-requested, and 460 untriaged rows** (3410 total).

The thirteen retail 12.0.0 removed WorldText CVars are best-effort/behavioral: focused proof commit 58aa3b8de asserts GetCVar and GetCVarDefault return nil for every old name. Distinct _v2 names are not claimed semantically equivalent; claims are bounded to old-name value/default absence. Current totals are **1919 best-effort, 1025 evidence-required, 2 exception-requested, and 464 untriaged rows** (3410 total).

The five retail 12.0.0 removed CVars `ShowClassColorInFriendlyNameplate`, `ShowClassColorInNameplate`, `ShowNamePlateLoseAggroFlash`, `TerrainBlendBakeEnable`, and `TerrainUnlitShaderEnable` are **best-effort/behavioral**. Focused proof is commit `c6702c93d` via `patch_12_0_0_cvar_removals.rs`; it asserts both public `GetCVar` and `GetCVarDefault` return nil for each name. Retail profile filtering removes the Friendly nameplate CVar while preserving non-retail profiles; the other four are omitted. Claims are bounded to CVar/default absence, with no rendering, nameplate, terrain, or other CVar semantics. Current totals are **1906 best-effort, 1025 evidence-required, 2 exception-requested, and 477 untriaged rows** (3410 total).

The four retail 12.0.0 `Enum.SendAddonMessageResult` rows `AddOnMessageLockdown=11`, `TargetOffline=12`, and metadata changes `MaxValue=10→12`, `NumValues=11→13` are **best-effort/behavioral**. Focused proof is commit `ed2345d36` via `patch_12_0_0_send_addon_message_result_enums.rs`; it asserts the complete 13-member Lua-number family, no extras, and metadata `MinValue=0`, `MaxValue=12`, `NumValues=13`. The guarded fallback is authoritative with no drift. Claims are bounded to startup publication and metadata changes; no addon-message transport or consumer semantics are claimed. Current totals are **1906 best-effort, 1025 evidence-required, 2 exception-requested, and 477 untriaged rows** (3410 total).

The removed retail 12.0.0 `SHOW_DELVES_DISPLAY_UI` event is **best-effort/behavioral**. Focused proof is commit `df6f8a5a3`; it asserts registration rejects the removed event while `HOUSE_LEVEL_CHANGED` remains accepted. Claims are bounded to event-name rejection; no producer, payload, replacement, or Delves behavior is claimed.

The ten retail 12.0.0 removed nameplate CVars `NamePlateClassificationScale`, `NamePlateHorizontalScale`, `NamePlateMaximumClassificationScale`, `NamePlateVerticalScale`, `NameplatePersonalClickThrough`, `NameplatePersonalHideDelayAlpha`, `NameplatePersonalHideDelaySeconds`, `NameplatePersonalShowAlways`, `NameplatePersonalShowInCombat`, and `NameplatePersonalShowWithTarget` are **best-effort/behavioral**. Focused proof is commit `64a811ad0` via `patch_12_0_0_cvar_removals.rs`; it asserts both public getters return nil for all ten names. Retail profile filtering removes the two scale CVars while Mists defaults remain unchanged. Claims are bounded to CVar/default absence, with no nameplate behavior or graphics semantics.

The removed retail 12.0.0 CVar `ForceAllowAero` is **best-effort/behavioral**. Focused proof is commit `9431abb53` via `patch_12_0_0_cvar_removals.rs`; both `GetCVar("ForceAllowAero")` and `GetCVarDefault("ForceAllowAero")` return nil. The CVar defaults omit the removed name. Claims are bounded to CVar/default absence, with no graphics semantics. Current totals are **1906 best-effort, 1025 evidence-required, 2 exception-requested, and 477 untriaged rows** (3410 total).

The removed retail 12.0.0 `HOUSING_CATALOG_SEARCHER_RELEASED` event is **best-effort/behavioral**. Focused proof is commit `41a4eeef8` via `housing_event_dispatch.rs`; it rejects RegisterEvent for the removed event while accepting `HOUSE_LEVEL_CHANGED` as a control. Claims are bounded to event-name registration rejection; no producer or replacement-event behavior is claimed.

The six retail 12.0.0 removed `Enum.WMOExteriorID.Invalid=-1`, `DefaultAlliance=9`, `DefaultHorde=87`, and metadata `MaxValue=87`, `MinValue=-1`, `NumValues=3` are **evidence-required/unsafe**. The source register removes the namespace rows; checked-in runtime fallback and enum initialization provide no replacement, but text absence does not prove full-LoD runtime absence. Required proof is a namespace-safe Lua probe for the old namespace/metadata and any replacement. Claims are bounded to source removal only. Current totals are **1906 best-effort, 1025 evidence-required, 2 exception-requested, and 477 untriaged rows** (3410 total).

The removed retail 12.0.0 `HOUSING_CATALOG_SEARCHER_RELEASED` event is **best-effort/behavioral**. Focused proof is commit `41a4eeef8` via `housing_event_dispatch.rs`; it rejects RegisterEvent for the removed event while accepting `HOUSE_LEVEL_CHANGED` as a control. Claims are bounded to event-name registration rejection; no producer or replacement-event behavior is claimed.

The ten retail 12.0.0 removed `Enum.VoiceTtsDestination` members and metadata rows `RemoteTransmission`, `LocalPlayback`, `RemoteTransmissionWithLocalPlayback`, `QueuedRemoteTransmission`, `QueuedLocalPlayback`, `QueuedRemoteTransmissionWithLocalPlayback`, `ScreenReader`, and `Meta.MaxValue`, `MinValue`, `NumValues` are **evidence-required/unsafe**. The source register removes the namespace rows; checked-in fallback, enum initialization, and discovery show no replacement, but text absence does not prove full-LoD runtime absence. Required proof is a namespace-safe Lua probe for the old namespace/member and metadata plus any replacement. Claims are bounded to source removal only. Current totals are **1884 best-effort, 1019 evidence-required, 2 exception-requested, and 505 untriaged rows** (3410 total).

The six retail 12.0.0 removed `Enum.ReportStorageProvider.Aws`, `Alibaba`, `Gcp`, and `Enum.ReportStorageProviderMeta.MaxValue`, `MinValue`, `NumValues` rows are **evidence-required/unsafe**. The source register removes the namespace rows; checked-in fallback, enum initialization, and discovery show no replacement, but text absence does not prove full-LoD runtime absence. Required proof is a namespace-safe Lua probe for both old namespaces and any replacement. Claims are bounded to source removal only. Current totals are **1884 best-effort, 1009 evidence-required, 2 exception-requested, and 515 untriaged rows** (3410 total).

The five retail 12.0.0 removed `Enum.NeighbordhoodInitiativeCategory.Current`, `Legacy`, and `Enum.NeighbordhoodInitiativeCategoryMeta.MaxValue`, `MinValue`, `NumValues` are **evidence-required/unsafe**. The source register removes the misspelled namespace rows, while runtime enum data and discovery provide no full-LoD absence or corrected replacement proof. Required proof is a namespace-safe Lua probe for the old and any corrected `NeighborhoodInitiativeCategory`/`Meta` namespaces; claims are bounded to source removal only. Current totals are **1884 best-effort, 1003 evidence-required, 2 exception-requested, and 521 untriaged rows** (3410 total).

The two retail 12.0.0 removed `C_PerksActivities.PerksActivityCriteria.criteriaID` and `requiredValue` numeric fields are **evidence-required/unsafe**. The source register records removal, but the temporary `C_PerksActivities` workaround does not model or return observable criteria structures. Required proof is a runtime probe obtaining the relevant criteria object, checking old-field absence and any replacement field/object; source-text absence alone is insufficient. Claims are bounded to source removal only. Current totals are **1884 best-effort, 994 evidence-required, 2 exception-requested, and 530 untriaged rows** (3410 total).

The four retail 12.0.0 removed crafting structure fields `CraftingItemSlotModification.itemID`, `CraftingOrderReagentInfo.reagent`, `CraftingReagentInfo.itemID`, and `CraftingResourceReturnInfo.itemID` are **evidence-required/unsafe**. The source register records their removal, while current crafting state does not publish the affected structures; runtime object absence, replacement identity, and nested shape remain unproven. Required probes must obtain each relevant returned object, assert old-field absence, and characterize any replacement separately. Claims are bounded to source removal only. Current totals are **1884 best-effort, 998 evidence-required, 2 exception-requested, and 526 untriaged rows** (3410 total).

The removed retail 12.0.0 `Enum.HousingCatalogEntrySubtype.MarketItem` row is **best-effort/behavioral**. Focused proof is commit `c400d31be` via `patch_12_0_0_housing_catalog_entry_subtype_enums.rs`; it asserts the exact current family `Invalid=0`, `Unowned=1`, `OwnedModifiedStack=2`, `OwnedUnmodifiedStack=3`, metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`, no extras, and `MarketItem` absence. The guarded fallback is authoritative with no drift. Claims are bounded to startup omission and current publication; no semantic replacement is claimed. Current totals are **1884 best-effort, 992 evidence-required, 2 exception-requested, and 532 untriaged rows** (3410 total).

The two retail 12.0.0 removed `Enum.NpcCraftingOrderSetFlags.CraftingOrderFlagAllowMultiple` and `CraftingOrderFlagAllowDuplicate` rows are **best-effort/behavioral**. Focused proof is commit `3de997642` via `patch_12_0_0_npc_crafting_order_set_flags_enums.rs`; it asserts the shortened family `AllowMultiple=1, AllowDuplicate=2`, metadata `MinValue=1, MaxValue=2, NumValues=2`, no extras, and both removed prefixed aliases absent. The guarded fallback is authoritative with no drift. Claims are bounded to startup rename/removal publication; crafting-order flag semantics are not claimed. Current totals are **1883 best-effort, 992 evidence-required, 2 exception-requested, and 533 untriaged rows** (3410 total).

The three retail 12.0.0 removed `Enum.CharCustomizationType` members `Outfit=9`, `Facepaint=10`, and `FacepaintColor=11` are **best-effort/behavioral**. Focused proof is commit `e08b83b6a` via `patch_12_0_0_char_customization_type_enums.rs`; it asserts the exact nine-member replacement family `Skin=0`, `Face=1`, `Hair=2`, `HairColor=3`, `FacialHair=4`, `CustomOptionTattoo=5`, `CustomOptionHorn=6`, `CustomOptionFacewear=7`, `CustomOptionTattooColor=8`, metadata `MinValue=0`, `MaxValue=8`, `NumValues=9`, no extras, and all three removed members absent. The guarded fallback is authoritative and shows no drift. Claims are bounded to startup omission/publication only; no customization or replacement semantics are claimed. Current totals are **1881 best-effort, 992 evidence-required, 2 exception-requested, and 535 untriaged rows** (3410 total).

The four retail 12.0.0 removed `Enum.AccountDataUpdateStatus.AccountDataUpdateCorrupt`, `AccountDataUpdateFailed`, `AccountDataUpdateSuccess`, and `AccountDataUpdateToobig` rows are **best-effort/behavioral**. Focused proof is commit `0910f12fa` via `patch_12_0_0_audit_enums.rs`; it asserts replacement values `Corrupt=2`, `Failed=1`, `Success=0`, `Toobig=3`, metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`, no extras, and all old aliases absent. Claims are bounded to startup rename/removal publication; account-data semantics are not claimed. Current totals are **1878 best-effort, 992 evidence-required, 2 exception-requested, and 538 untriaged rows** (3410 total).

The five retail 12.0.0 removed `C_HousingDecor.HousingLevelInfo` numeric fields `exteriorDecorPlacementBudget`, `exteriorFixtureBudget`, `interiorDecorPlacementBudget`, `level`, and `roomPlacementBudget` are **evidence-required/unsafe**. The source removes the old fields and adds same-named numeric fields under separate `HouseLevelInfo`, but the simulator models neither structure, so runtime identity, old-field absence, and replacement publication remain unproven. Required probe: rawget observable old and replacement objects/fields. Claims are bounded to source removal/replacement metadata only. Current totals are **1874 best-effort, 992 evidence-required, 2 exception-requested, and 542 untriaged rows** (3410 total).

The removed retail 12.0.0 `C_HousingCatalog.HousingCatalogEntryInfo.numStored` row is **best-effort/behavioral**. Focused proof is commit `1e31a40dd` via `patch_12_0_0_housing_catalog_entry_info.rs`; it obtains a seeded catalog entry table, asserts `numStored == nil`, and verifies numeric `totalNumStored`, `totalNumPlaced`, and `destroyableInstanceCount` fields. Runtime seeded catalog state matches the source removal. Claims are bounded to removed field/table shape only; storage semantics remain unclaimed. Current totals are **1874 best-effort, 987 evidence-required, 2 exception-requested, and 547 untriaged rows** (3410 total).

The five retail 12.0.0 removed `C_HousingBasicMode.InvalidPlacementInfo` boolean fields `anyRestrictions`, `invalidCollision`, `invalidTarget`, `notInRoom`, and `tooFar` are **evidence-required/unsafe**. Source metadata records removal, but no runtime object-shape or replacement proof exists; a full-LoD namespace-safe rawget probe is required. Claims are bounded to source removal only. Current totals are **1874 best-effort, 987 evidence-required, 2 exception-requested, and 547 untriaged rows** (3410 total).

The four retail 12.0.0 changed housing events `HOUSE_LEVEL_CHANGED`, `HOUSING_BASIC_MODE_PLACEMENT_FLAGS_UPDATED`, `HOUSING_BASIC_MODE_SELECTED_TARGET_CHANGED`, and `HOUSING_DECOR_PLACE_SUCCESS` are best-effort/behavioral. Focused proof is commit `12d231522` via `housing_event_dispatch.rs`; it verifies event registration and generic dispatch of the exact after-payload arity, order, and Lua types: one representative table; numeric targetType plus numeric activeFlags; boolean hasSelectedTarget, numeric targetType, and boolean isPreview; and string decorGUID, numeric size, boolean isNew, and boolean isPreview. Root cause is registration/generic-dispatch coverage without modeled event producers. Claims are bounded to the startup/event registration contract and generic dispatch only; housing producers, payload fields beyond the tested representatives, timing, lifecycle, ordering, duplicates, and consumer behavior remain unclaimed. Current totals are **1874 best-effort, 987 evidence-required, 2 exception-requested, and 547 untriaged rows** (3410 total).

The eight retail 12.0.0 `Enum.InitiativeMilestoneFlags.*`, `Enum.InitiativeMilestoneFlagsMeta.*`, `Enum.InitiativeRewardFlags.*`, and `Enum.InitiativeRewardFlagsMeta.*` rows are best-effort/behavioral: `FinalMilestone=1` and `PermanentWorldState=1`; both one-member families have metadata `MinValue=1`, `MaxValue=1`, `NumValues=1`. Focused proof is commit `ffb49321e` via `patch_12_0_0_initiative_flags_enums.rs`; it asserts exact complete member sets, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; milestone/reward semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1651 best-effort, 982 evidence-required, 2 exception-requested, and 775 untriaged rows** (3410 total).

The five retail 12.0.0 `Enum.HousingRoomComponentFlags.*` and `Enum.HousingRoomComponentFlagsMeta.*` rows are best-effort/behavioral: `None=0`, `HiddenInLayoutMode=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is commit `b0fd1c338` via the exact HousingRoomComponentFlags startup test; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; room-layout flag semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1641 best-effort, 982 evidence-required, 2 exception-requested, and 785 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.HousingFavorUpdateType.*` and `Enum.HousingFavorUpdateTypeMeta.*` rows are best-effort/behavioral: `Add=0, InitiativeAdd=1, Set=2`; metadata `MinValue=0, MaxValue=2, NumValues=3`. Focused proof is commit `6fbe32520` via the exact HousingFavorUpdateType startup test; it asserts the complete member set, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; favor behavior, producers, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1636 best-effort, 982 evidence-required, 2 exception-requested, and 790 untriaged rows** (3410 total).

The eleven retail 12.0.0 `Enum.HousingFavorUpdateSource.*` and `Enum.HousingFavorUpdateSourceMeta.*` rows are best-effort/behavioral: `Unknown=0, DecorCollection=1, DeferredRewards=2, RetroactiveDecor=3, NewHouseDecorFavor=4, InitiativeTask=5, InitiativeChest=6, Quest=7`; metadata `MinValue=0, MaxValue=7, NumValues=8`. Focused proof is commit `e5377de7c` via the exact HousingFavorUpdateSource startup test; it asserts the complete member set, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; housing-favor behavior, producers, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1636 best-effort, 982 evidence-required, 2 exception-requested, and 790 untriaged rows** (3410 total).

The eight retail 12.0.0 `Enum.HousingDecorPlacementRestriction.*` rows are best-effort/behavioral: added `ChildOutsideBounds=8`, `OutsidePlotBounds=4`, `OutsideRoomBounds=2`; changed `InvalidTarget=4→16`, `InvalidCollision=8→32`, metadata `MaxValue=8→32`, `NumValues=4→6`; removed `NotInsideRoom` (prior value 2). Focused proof is commit `fe3ea6de9` via the exact placement-restriction startup test; it asserts the complete family, Lua numeric types, no extras, metadata `MinValue=1`, `MaxValue=32`, `NumValues=6`, and removed-member absence. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication, exact values/types/metadata, and absence; placement semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1619 best-effort, 982 evidence-required, 2 exception-requested, and 807 untriaged rows** (3410 total).

The seven retail 12.0.0 `Enum.FrameTutorialAccount.*` and `Enum.FrameTutorialAccountMeta.*` rows are best-effort/behavioral: `TransmogOutfits=41`, `TransmogSets=42`, `TransmogCustomSets=43`, `TransmogSituations=44`, `TransmogWeaponOptions=45`; metadata `MinValue=1`, `MaxValue=45`, `NumValues=45` (changed from 40). Focused proof/fix is commit `6f00129a6` via the exact transmog startup test; it asserts the five values/types, exactly 45 unique numeric members in range 1..45, and metadata. Root cause: the guarded fallback is authoritative because `FRAME_TUTORIAL_ACCOUNT` is not included in `EXPLICIT_ENUMS`; fallback carried later-shape values 46/47 and metadata 47, corrected by the retail 12.0.0 override. Claims are bounded to startup publication and exact values/types/metadata; tutorial behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1611 best-effort, 982 evidence-required, 2 exception-requested, and 815 untriaged rows** (3410 total).

The five retail 12.0.0 `Enum.FontStringScaleAnimationMode.*` and `Enum.FontStringScaleAnimationModeMeta.*` rows are best-effort/behavioral: `FontSize=0`, `Vertex=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is commit `462b919fe` via the exact family startup test; it asserts complete member set, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; scaling/animation semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1604 best-effort, 982 evidence-required, 2 exception-requested, and 822 untriaged rows** (3410 total).

The four retail 12.0.0 `Enum.FragmentID.*` and `Enum.FragmentIDMeta.NumValues` rows are best-effort/behavioral: `FPlayerInitiativeInfo=37`, `FNeighborhoodStateData=38`, `FUnitAIGroupLink=39`; metadata `MinValue=0`, `MaxValue=255`, `NumValues=72` (changed from 69). Focused proof is commit `d42484e0a` via the exact additions startup test; it asserts the three numeric members and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; fragment semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1604 best-effort, 982 evidence-required, 2 exception-requested, and 822 untriaged rows** (3410 total).

The retail 12.0.0 `Enum.UnitAuraSortRule.BigDefensive` row is best-effort/behavioral: numeric value `2`. Focused proof is commit `2bc686862`; claims are bounded to startup publication and exact numeric value, not aura-sort semantics or lifecycle. Current totals are **1595 best-effort, 982 evidence-required, 2 exception-requested, and 831 untriaged rows** (3410 total).

The five retail 12.0.0 `Enum.UnitAuraSortDirection.*` rows are best-effort/behavioral: `Normal=0`, `Reverse=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is commit `a3ae227f8`; claims are bounded to startup publication and exact values/types/metadata, not aura sorting semantics or lifecycle. Current totals are **1594 best-effort, 982 evidence-required, 2 exception-requested, and 832 untriaged rows** (3410 total).

The five retail 12.0.0 `Enum.UICovenantDisplayInfoFlags.*` rows are best-effort/behavioral: `DisplayCovenantAsJourney=1`, `UseJourneyRewardTrack=2`; metadata `MinValue=1`, `MaxValue=2`, `NumValues=2`. Focused proof is commit `dcda2eea2` via the exact startup test; it asserts the exact two-member family, numeric types, no later extra member, and metadata. Root cause: shared fallback data contained later `UseJourneyUnlockToastText`; the retail 12.0.0 override corrects publication. Claims are bounded to startup publication and exact values/types/metadata; covenant display semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1589 best-effort, 982 evidence-required, 2 exception-requested, and 837 untriaged rows** (3410 total).

The 41 retail 12.0.0 `Enum.TransmogSituation*` and `Enum.TransmogSituationTrigger*` rows are best-effort/behavioral across five exact families and matching metadata. Focused proof is commit `5f9dcaf31` via the exact situation-family startup test; it asserts complete family membership, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; situation/trigger semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1584 best-effort, 982 evidence-required, 2 exception-requested, and 842 untriaged rows** (3410 total).

The 24 retail 12.0.0 TransmogOutfit slot-save, slot-warning, transaction-flag metadata, and transaction-type rows are best-effort/behavioral. Focused proofs are commits `977b8c972`, `eaf96b3bb`, and `3519919b1`; they assert complete family membership, Lua numeric types, no extras, and exact metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-save/warning semantics, transaction semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1543 best-effort, 982 evidence-required, 2 exception-requested, and 883 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.TransmogOutfitSlotPosition.*` and `Enum.TransmogOutfitSlotPositionMeta.*` rows are best-effort/behavioral: `Left=0`, `Right=1`, `Bottom=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `b7bf42409` via the exact family startup test; it asserts the complete member set, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-position semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1519 best-effort, 982 evidence-required, 2 exception-requested, and 907 untriaged rows** (3410 total).

The three retail 12.0.0 `Enum.TransmogOutfitSlotOptionMeta.*` rows are best-effort/behavioral: metadata `MinValue=0`, `MaxValue=11`, `NumValues=12` for the exact 12-member `TransmogOutfitSlotOption` family. Focused proof is commit `789800edf` via the exact family metadata startup test; it asserts numeric member types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-option semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1513 best-effort, 982 evidence-required, 2 exception-requested, and 913 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.TransmogOutfitSlotOptionFlags.*` and `Enum.TransmogOutfitSlotOptionFlagsMeta.*` rows are best-effort/behavioral: `IllusionNotAllowed=1`, `DynamicOptionName=2`, `DisablesOffhandSlot=4`; metadata `MinValue=1`, `MaxValue=4`, `NumValues=3`. Focused proof is commit `bcb3b459c` via the exact family startup test; it asserts the complete member set, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-option flag semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1510 best-effort, 982 evidence-required, 2 exception-requested, and 916 untriaged rows** (3410 total).

The three retail 12.0.0 `Enum.TransmogOutfitSlotMeta.*` rows are best-effort/behavioral: metadata `MinValue=0`, `MaxValue=14`, `NumValues=15` for the exact 15-member `TransmogOutfitSlot` family. Focused proof is commit `f5b2eae12` via the exact family metadata startup test; it asserts numeric member types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; outfit-slot semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1504 best-effort, 982 evidence-required, 2 exception-requested, and 922 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.TransmogOutfitSlotFlags.*` and `Enum.TransmogOutfitSlotFlagsMeta.*` rows are best-effort/behavioral: `CannotBeHidden=1`, `CanHaveIllusions=2`, `IsSecondarySlot=4`; metadata `MinValue=1`, `MaxValue=4`, `NumValues=3`. Focused proof is commit `86b0e4cb2` via the exact family startup test; it asserts the complete member set, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-flag semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1501 best-effort, 982 evidence-required, 2 exception-requested, and 925 untriaged rows** (3410 total).

The three retail 12.0.0 `Enum.TransmogOutfitSlotErrorMeta.*` rows are best-effort/behavioral: metadata `MinValue=0`, `MaxValue=14`, `NumValues=15` for the exact 15-member `TransmogOutfitSlotError` family. Focused proof is commit `0e81d67ea` via the exact family startup test; it asserts all numeric members, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; slot-error semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1495 best-effort, 982 evidence-required, 2 exception-requested, and 931 untriaged rows** (3410 total).

The four retail 12.0.0 `Enum.TransmogOutfitDataFlags.*` and `Enum.TransmogOutfitDataFlagsMeta.*` rows are best-effort/behavioral: `IsCachedLocally=1`; metadata `MinValue=1`, `MaxValue=1`, `NumValues=1`. Focused proof is commit `2f67fd253` via the exact one-member family startup test; it asserts the numeric type, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; caching semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1492 best-effort, 982 evidence-required, 2 exception-requested, and 934 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.TransmogOutfitSetType.*` and `Enum.TransmogOutfitSetTypeMeta.*` rows are best-effort/behavioral: Equipped=0, Outfit=1, CustomSet=2; metadata MinValue=0, MaxValue=2, NumValues=3. Focused proof is commit `c540bb029` via the exact family startup test; it asserts complete member set, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; set-selection semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1488 best-effort, 982 evidence-required, 2 exception-requested, and 938 untriaged rows** (3410 total).
The nine retail 12.0.0 `Enum.TransmogOutfitEquipAction.*` and `Enum.TransmogOutfitEquipActionMeta.*` rows are best-effort/behavioral: `Equip=0, EquipAndLock=1, Remove=2, RemoveAndLock=3, Unlock=4, Lock=5`; metadata `MinValue=0`, `MaxValue=5`, `NumValues=6`. Focused proof is commit `ad955a3a6` via the exact family startup test; it asserts complete member set, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; equip/remove behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1482 best-effort, 982 evidence-required, 2 exception-requested, and 944 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.TransmogOutfitEntrySource.*` and `Enum.TransmogOutfitEntrySourceMeta.*` rows are best-effort/behavioral: `StampedSource=0`, `AutomaticallyAwarded=1`, `PlayerPurchased=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `320983281` via the exact family startup test; it asserts complete member sets, Lua numeric types, no extras, and metadata. No runtime drift; guarded `missing_enums.lua` is authoritative. Claims are bounded to startup publication and exact values/types/metadata; outfit-source semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1473 best-effort, 982 evidence-required, 2 exception-requested, and 953 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify tooltip and transmog outfit enum rows

The 19 retail 12.0.0 TooltipDataLineType, TransmogOutfitDisplayType, and TransmogOutfitEntryFlags rows are best-effort/behavioral: TooltipDataLineType adds SpellPassive=43 and SpellDescription=44 with metadata MaxValue=44 and NumValues=45; TransmogOutfitDisplayType is Unassigned=0, Assigned=1, Equipped=2, Hidden=3 with metadata MinValue=0, MaxValue=3, NumValues=4; TransmogOutfitEntryFlags is AutomaticallyAwardedOnLogin=1, UseOverrideName=2, OnlyAvailableDuringEvent=4, SortedToTopOfList=8, UseOverrideCostModifier=16 with metadata MinValue=1, MaxValue=16, NumValues=5. Focused proof is commit 891617e4c via the three exact family startup tests; assertions cover complete member sets, Lua numeric types, no extras, and metadata. Root cause: shared fallback data reflected a later enum shape while the retail-12-0-0 override now publishes the exact authoritative 12.0.0 family. Claims are bounded to startup publication and exact values/types/metadata; enum semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1467 best-effort, 982 evidence-required, 2 exception-requested, and 959 untriaged** (3410 rows).

The five retail 12.0.0 `Enum.StatusBarTimerDirection.*` and `Enum.StatusBarTimerDirectionMeta.*` rows are best-effort/behavioral: `ElapsedTime=0`, `RemainingTime=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is at `7f3d2cf1511aa2b19fb6e20a6e200346af7df56c` and asserts the exact complete namespace, Lua numeric types, no extras, and metadata. Runtime is correct because guarded `missing_enums.lua` is authoritative; the reversed `STATUS_BAR_TIMER_DIRECTION` constant in `combat_system.rs` is omitted from `SEQUENTIAL_ENUMS` and therefore inactive. Claims are bounded to startup publication and exact values/metadata; timer direction behavior, consumers, persistence, transitions, and lifecycle are not claimed.
The eight retail 12.0.0 `Enum.LFGEntryGeneralPlaystyle.*` and `Enum.LFGEntryGeneralPlaystyleMeta.*` rows are best-effort/behavioral: `None=0`, `Learning=1`, `FunRelaxed=2`, `FunSerious=3`, `Expert=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused proof is commit `3a00e22f1` via `patch_12_0_0_lfg_entry_general_playstyle_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; playstyle behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current manifest totals are **1659 best-effort, 982 evidence-required, 2 exception-requested, and 767 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.LimitedInputType.*` and `Enum.LimitedInputTypeMeta.*` rows are best-effort/behavioral: `MouseMove=0`, `MouseDown=1`, `MouseUp=2`, `MouseWheel=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `280ffd561` via `patch_12_0_0_limited_input_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; input-dispatch behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1687 best-effort, 982 evidence-required, 2 exception-requested, and 739 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.NamePlateFriendlyPlayerAuraDisplay.*` and `Enum.NamePlateFriendlyPlayerAuraDisplayMeta.*` rows are best-effort/behavioral: `None=0`, `Buffs=1`, `Debuffs=2`, `LossOfControl=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `8859a6fa4` via `patch_12_0_0_nameplate_friendly_player_aura_display_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; nameplate aura rendering behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1687 best-effort, 982 evidence-required, 2 exception-requested, and 739 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.NamePlateInfoDisplay.*` and `Enum.NamePlateInfoDisplayMeta.*` rows are best-effort/behavioral: `None=0`, `CurrentHealthPercent=1`, `CurrentHealthValue=2`, `RarityIcon=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `295c3f811` via `patch_12_0_0_nameplate_info_display_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; nameplate display behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1687 best-effort, 982 evidence-required, 2 exception-requested, and 739 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.LuaCurveType.*` and `Enum.LuaCurveTypeMeta.*` rows are best-effort/behavioral: `Linear=0`, `Step=1`, `Cosine=2`, `Cubic=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `e016d4eeb` via `patch_12_0_0_lua_curve_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the `LUA_CURVE_TYPE` constant is not registered in `SEQUENTIAL_ENUMS`, so the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; curve interpolation behavior, consumers, persistence, transitions, and lifecycle are not claimed.
The two retail 12.0.0 `Enum.ItemCreationContext.TimewalkerLevelUp` and `Enum.ItemCreationContext.TimewalkerMaxLevel` rows are best-effort/behavioral: `TimewalkerLevelUp=22` and `TimewalkerMaxLevel=186`; metadata is `MinValue=0`, `MaxValue=186`, `NumValues=187`. Focused proof is commit `9691f40af` via `patch_12_0_0_item_creation_context_additions.rs`, asserting Lua numeric types, exact values, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; item-creation semantics, consumers, persistence, transitions, and lifecycle remain unclaimed.

The nine retail 12.0.0 `Enum.NeighborhoodInitiativeChestResult.*` and `Enum.NeighborhoodInitiativeChestResultMeta.*` rows are best-effort/behavioral: `NiSuccess=0, NiUnspecifiedFailure=1, NiNoHouseFound=2, NiNoRewards=3, NiThrottled=4, NiServiceDisabled=5`; metadata `MinValue=0, MaxValue=5, NumValues=6`. Focused proof is at `1c17a5e5c0afbee1358d2ad0cd237d1e12cfa659`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/metadata; initiative chest reward/service behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1443 best-effort, 982 evidence-required, 2 exception-requested, and 983 untriaged** (3410 rows).
## [2026-08-10] investigation | Classify neighborhood initiative neighborhood-type enums

The five retail 12.0.0 `Enum.NeighborhoodInitiativeNeighborhoodTypes.*` and `Enum.NeighborhoodInitiativeNeighborhoodTypesMeta.*` rows are best-effort/behavioral: `NiNeighborhoodTypeSingleton=0`, `NiNeighborhoodTypePool=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is at d02a9a8c94b7d2966b0f319f9f570fe4f23d2bff; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/metadata; Neighborhood initiative selection, assignment, behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1429 best-effort, 982 evidence-required, 2 exception-requested, and 997 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify nameplate threat-display enums

The seven retail 12.0.0 `Enum.NamePlateThreatDisplay.*` and `Enum.NamePlateThreatDisplayMeta.*` rows are best-effort/behavioral: `None=0`, `Progressive=1`, `Flash=2`, `HealthBarColor=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is at `ad7cae26a`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/metadata; nameplate threat rendering/display behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1434 best-effort, 982 evidence-required, 2 exception-requested, and 992 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify nameplate stack-type enums

The six retail 12.0.0 `Enum.NamePlateStackType.*` and `Enum.NamePlateStackTypeMeta.*` rows are best-effort/behavioral: `None=0`, `Enemy=1`, `Friendly=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is at `eece809a72b113d512b828e0d8163c4adb4296de`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/metadata; nameplate stacking/layout/rendering, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1424 best-effort, 982 evidence-required, 2 exception-requested, and 1002 untriaged** (3410 rows).
The five retail 12.0.0 `Enum.NamePlateType.*` and `Enum.NamePlateTypeMeta.*` rows are best-effort/behavioral: Friendly=0, Enemy=1; metadata MinValue=0, MaxValue=1, NumValues=2. Focused proof is at `ebc47b1665df258f2a9399f18555470e5fbce941`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/metadata; nameplate type/rendering/selection, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1434 best-effort, 982 evidence-required, 2 exception-requested, and 992 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify nameplate style enums

The nine retail 12.0.0 `Enum.NamePlateStyle.*` and `Enum.NamePlateStyleMeta.*` rows are best-effort/behavioral: `Modern=0`, `Thin=1`, `Block=2`, `HealthFocus=3`, `CastFocus=4`, `Legacy=5`; metadata `MinValue=0`, `MaxValue=5`, `NumValues=6`. Focused proof is at `e9564865c38688e2c6a59bdb70a739b2293be488`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found; explicit member publication and matching fallback metadata are authoritative. Claims are bounded to startup publication and exact values/metadata; nameplate rendering/styling, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1411 best-effort, 982 evidence-required, 2 exception-requested, and 1015 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter timeline track enums

The eight retail 12.0.0 `Enum.EncounterTimelineTrack.*` and `Enum.EncounterTimelineTrackMeta.*` rows are best-effort/behavioral: `Queued=0`, `Short=1`, `Medium=2`, `Long=3`, `Indeterminate=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused proof is at `1abf8c72f3adabf9c8f50c53870c2a2f643b9fb6`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; queueing, timing, track behavior, ordering, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1411 best-effort, 982 evidence-required, 2 exception-requested, and 1015 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify enemy-player nameplate aura-display enums

The seven retail 12.0.0 `Enum.NamePlateEnemyPlayerAuraDisplay.*` and `Enum.NamePlateEnemyPlayerAuraDisplayMeta.*` rows are best-effort/behavioral: `None=0, Buffs=1, Debuffs=2, LossOfControl=3`; metadata `MinValue=0, MaxValue=3, NumValues=4`. Focused proof is at `cc2e085ff2e15b6a617df21e19833db20417524b`; it asserts the exact complete set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; nameplate aura rendering, cast-bar behavior, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify enemy-NPC nameplate aura-display enums

The seven retail 12.0.0 `Enum.NamePlateEnemyNpcAuraDisplay.*` and `Enum.NamePlateEnemyNpcAuraDisplayMeta.*` rows are best-effort/behavioral: `None=0, Buffs=1, Debuffs=2, CrowdControl=3`; metadata `MinValue=0, MaxValue=3, NumValues=4`. Focused proof is at `6f44ef24d370a7b292a389b6e2dc634d56746671`; it asserts the exact complete set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; nameplate aura rendering, cast-bar behavior, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify nameplate cast-bar display enums

The nine retail 12.0.0 `Enum.NamePlateCastBarDisplay.*` and `Enum.NamePlateCastBarDisplayMeta.*` rows are best-effort/behavioral: `None=0, SpellName=1, SpellIcon=2, SpellTarget=3, HighlightImportantCasts=4, HighlightWhenCastTarget=5`; metadata `MinValue=0, MaxValue=5, NumValues=6`. Focused proof is at `bc15bc6b4814c0bc7f28ebd815697454f4cc5f60`; it asserts the exact complete set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; nameplate rendering, cast-bar behavior, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter timeline icon-set enums

The nine retail 12.0.0 `Enum.EncounterTimelineIconSet.*` and `Enum.EncounterTimelineIconSetMeta.*` rows are best-effort/behavioral: `TankAlert=1, HealerAlert=2, DamageAlert=3, Deadly=4, Dispel=5, Enrage=6`; metadata `MinValue=1, MaxValue=6, NumValues=6`. Focused proof is at `061aabfece4c8efb103100e343f1d1b648f5b151`; it asserts the exact complete set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; icon selection, rendering, timeline behavior, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter timeline event-source enums

The seven retail 12.0.0 `Enum.EncounterTimelineEventState.*` rows are best-effort/behavioral: `Active=0`, `Paused=1`, `Finished=2`, `Canceled=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is at `171ce8632e17cd4d0141a704daf3d7cc99c888e3`; it asserts the exact four-member set, Lua numeric types, and metadata. The runtime state enum matches retail 12.0.0 with no drift. Claims are bounded to startup publication and exact values/metadata; state transitions, timing, producers, consumers, persistence, and lifecycle remain unclaimed.

The six retail 12.0.0 `Enum.EncounterTimelineEventSource.*` and `Enum.EncounterTimelineEventSourceMeta.*` rows are best-effort/behavioral: `Encounter=0`, `Script=1`, `EditMode=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is at `4fd68742c715f44d43f2afc017816fa198cedc40`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; source routing, event producers, ordering, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter-timeline sort-direction enums

The five retail 12.0.0 `Enum.EncounterTimelineEventSortDirection.*` rows are best-effort/behavioral: `Descending=0`, `Ascending=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is at `16469db1abdc879db2d2c0617a49583d86a7b4b8`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; sorting behavior, order semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1372 best-effort, 982 evidence-required, 2 exception-requested, and 1054 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter events visibility enums

The six retail 12.0.0 `Enum.EncounterEventsVisibility.*` and `Enum.EncounterEventsVisibilityMeta.*` rows are best-effort/behavioral: Always=0, InEncounter=1, DeprecatedHidden=2; metadata MinValue=0, MaxValue=2, NumValues=3. Evidence is bounded to startup publication, Lua numeric types, exact complete member set, and metadata. Focused proof is at bf17614b988cba98d936c789b30c919bf20f5952. No runtime drift was found. Encounter-event visibility behavior, transitions, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1322 best-effort, 982 evidence-required, 2 exception-requested, and 1104 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter-events orientation enums

Classified exactly five retail 12.0.0 `Enum.EncounterEventsOrientation.*` rows as best-effort/behavioral: Horizontal=0, Vertical=1; metadata MinValue=0, MaxValue=1, NumValues=2. Evidence is bounded to startup publication, Lua numeric types, exact complete member set, and metadata. Focused proof is at f59d146fd340eddd9761c1828e00f672afcd4ba3. No runtime drift was found. Layout/orientation behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1316 best-effort, 982 evidence-required, 2 exception-requested, and 1110 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter-event severity enums

The seven retail 12.0.0 `Enum.EncounterEventsIconDirection.*` rows are best-effort/behavioral: `Top=0`, `Bottom=1`, `Left=0`, and `Right=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=4`. Focused proof is at `49b052e36826a5ab70241fd242f4031e23b57805`; it asserts the exact complete member set, Lua numeric types, no extras, and metadata. No runtime drift was found. Claims are bounded to startup publication and exact values/metadata; encounter layout/direction behavior, consumers, persistence, transitions, and lifecycle remain unclaimed.

The six retail 12.0.0 `Enum.EncounterEventSeverity.*` rows are best-effort/behavioral: `Low=0`, `Medium=1`, `High=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is at `6a0012bc8a670c04e8a5413a06903b35d4d5e090`; it asserts the exact complete member set, Lua numeric types, and metadata. The guarded runtime fallback matches retail 12.0.0 with no drift. Claims are bounded to startup publication and exact values/metadata; encounter severity interpretation, producers, consumers, ordering, transitions, persistence, and lifecycle remain unclaimed. Current totals are **1311 best-effort, 982 evidence-required, 2 exception-requested, and 1115 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter event iconmask enums

Classified exactly thirteen retail 12.0.0 `Enum.EncounterEventIconmask.*` and `Enum.EncounterEventIconmaskMeta.*` rows as best-effort/behavioral: DeadlyEffect=1, EnrageEffect=2, BleedEffect=4, MagicEffect=8, DiseaseEffect=16, CurseEffect=32, PoisonEffect=64, TankRole=128, HealerRole=256, DpsRole=512; metadata MinValue=1, MaxValue=512, NumValues=10. Evidence is bounded to startup publication, Lua numeric types, exact bitmask member values, and metadata. Focused proof is at `0de2bc41145abf8518ff9ec602c7428cc3afc728`. The guarded runtime fallback matches retail 12.0.0 with no drift. Icon-mask interpretation/operations, encounter behavior, consumers, persistence, ordering, transitions, and lifecycle remain unclaimed. Current totals are **1298 best-effort, 982 evidence-required, 2 exception-requested, and 1128 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter-event cast-state enums

The six retail 12.0.0 `Enum.EncounterEventCastState.*` and `Enum.EncounterEventCastStateMeta.*` rows are best-effort/behavioral: `Casting=1`, `NotCasting=2`, `Expired=3`; metadata `MinValue=1`, `MaxValue=3`, `NumValues=3`. Focused proof is at `23852ce7b301bffecc3688984eab8b615a6df02f`; it asserts the exact three-member set, Lua numeric types, and metadata. The guarded runtime fallback matches retail 12.0.0 with no drift. Claims are bounded to startup publication and exact values/metadata; encounter-event cast behavior, producers, transitions, consumers, persistence, ordering, and lifecycle remain unclaimed. Current totals are **1298 best-effort, 982 evidence-required, 2 exception-requested, and 1128 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode unit-frame settings

The three retail 12.0.0 `Enum.EditModeUnitFrameSetting.*` rows are best-effort/behavioral: `AuraOrganizationType=18`, `IconSize=19`, and `Opacity=20`. Focused proof is at `509daf308b3e76d5f9768f8e694c887902190ca1`; it asserts startup table existence, Lua numeric types, exact values, absence of later `BigDefensiveIconSize`, and metadata `MinValue=0`, `MaxValue=20`, `NumValues=21`. Root cause: shared/current sequential enum data includes later `BigDefensiveIconSize=21` and fallback metadata `MaxValue=21`/`NumValues=22`; the retail 12.0.0 override removes that member and restores the metadata boundary without shifting the three target values. Claims are bounded to startup publication, type, exact values, and namespace boundary; unit-frame behavior, aura organization, sizing, opacity rendering, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1298 best-effort, 982 evidence-required, 2 exception-requested, and 1128 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode system enums

Classified exactly three retail 12.0.0 `Enum.EditModeSystem.*` rows as best-effort/behavioral: `PersonalResourceDisplay=21`, `EncounterEvents=22`, and `DamageMeter=23`. Evidence is bounded to startup table publication, Lua numeric type, exact values, absence of later `TotemActionBar=24`, and metadata `MinValue=0`, `MaxValue=23`, `NumValues=24`. Focused proof is at `4d883c9647cf972d6b12b7b4982b9de634d293c5`. Root cause: shared sequential enum data included later `TotemActionBar=24`; the retail 12.0.0 override removes it without shifting the three classified values. Edit Mode behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1276 best-effort, 982 evidence-required, 2 exception-requested, and 1150 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode personal-resource settings

The five retail 12.0.0 `Enum.EditModePersonalResourceDisplaySetting.*` and `Enum.EditModePersonalResourceDisplaySettingMeta.*` rows are best-effort/behavioral, bounded to startup exact namespace publication, Lua numeric type, and exact values (`HideHealthAndPower=0`, `OnlyShowInCombat=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`). Focused proof is at `d302a414f6392978a1ccec9997cedf14345907bc`. Root cause: shared runtime used a later fifteen-member namespace with different names (`HideHealth`, `DeprecatedOnlyShowInCombat`, etc.) and metadata `0/14/15`; retail 12.0.0 epoch initialization replaces it with the exact two-member namespace and metadata `0/1/2`. Display behavior, persistence, consumers, transitions, and lifecycle remain unclaimed. Current totals are **1273 best-effort, 982 evidence-required, 2 exception-requested, and 1153 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode encounter-events settings

Classified exactly thirteen retail 12.0.0 `Enum.EditModeEncounterEventsSetting.*` and `Enum.EditModeEncounterEventsSettingMeta.*` rows as best-effort/behavioral: `Orientation=0`, `IconDirection=1`, `ShowSpellName=2`, `IconSize=3`, `OverallSize=4`, `BackgroundTransparency=5`, `Transparency=6`, `Visibility=7`, `ShowTooltips=8`, `ShowTimer=9`; metadata `MinValue=0`, `MaxValue=9`, `NumValues=10`. Proof is at `f08638e9fe2b4d0144091eeeb61ccd48647001ac`. Shared runtime leaked later `ViewType=10`, `FlipHorizontally=11`, `BarWidth=12`, and `Padding=13` with metadata `0/13/14`; an earlier member was `TooltipAnchor` rather than authoritative `ShowTooltips`. Commit `d75584c79` corrected `ShowTooltips`, and the retail 12.0.0 epoch override removes later members and restores `0/9/10`. Claims are bounded to startup enum/metadata publication, Lua numeric type, exact values, and non-12.0.0 member absence; setting behavior, persistence, consumers, transitions, and lifecycle remain unclaimed. Current totals are **1273 best-effort, 982 evidence-required, 2 exception-requested, and 1153 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode damage-meter settings

Classified exactly sixteen retail 12.0.0 `Enum.EditModeDamageMeterSetting.*` and `Enum.EditModeDamageMeterSettingMeta.*` rows as best-effort/behavioral: `Visibility=0`, `Style=1`, `Numbers=2`, `FrameWidth=3`, `FrameHeight=4`, `Padding=5`, `Transparency=6`, `ObsoleteReuse1=7`, `ShowSpecIcon=8`, `ShowClassColor=9`, `BarHeight=10`, `TextSize=11`, `BackgroundTransparency=12`; metadata `MinValue=0`, `MaxValue=12`, `NumValues=13`. Focused proof is at `ce163833d2e0d984abd2baf0de6e498ca662f939`. Edit Mode damage-meter setting behavior, persistence, consumers, transitions, and lifecycle remain unclaimed. Current totals are **1255 best-effort, 982 evidence-required, 2 exception-requested, and 1171 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Edit Mode account/aura setting enums

Classified exactly six retail 12.0.0 `Enum.EditModeAccountSetting.*` rows as best-effort/behavioral: added `ShowPersonalResourceDisplay=29`, `ShowEncounterEvents=30`, `ShowDamageMeter=31`, and `ShowExternalDefensives=32`; changed metadata `MaxValue=28→32` and `NumValues=29→33`. The three `Enum.EditModeAuraFrameSetting.*` claims remain unchanged: `VisibleSetting=8`, `Opacity=9`, and `ShowDispelType=10`. Focused proof is at `0dfaa313be82192f753688ecc1dc2547df75bcea`. Root cause: shared sequential/current runtime included later `ShowTotemActionBar=33` and metadata `MaxValue=33`/`NumValues=34`; the retail 12.0.0 epoch override removes that member and restores metadata `MinValue=0`, `MaxValue=32`, `NumValues=33`. Claims are bounded to startup enum/metadata publication, Lua numeric type, exact values, and later-member absence; Edit Mode behavior, persistence, consumers, transitions, and lifecycle remain unclaimed. Current totals remain **1255 best-effort, 982 evidence-required, 2 exception-requested, and 1171 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DurationTimeModifier enums

Classified exactly five retail 12.0.0 `Enum.DurationTimeModifier.*` and `Enum.DurationTimeModifierMeta.*` rows as best-effort/behavioral: `RealTime=0`, `BaseTime=1`; metadata `MinValue=0`, `MaxValue=1`, and `NumValues=2`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `b846f4833e508d4f440b5a89c3194cc91a5f0384`. Duration calculations, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1230 best-effort, 982 evidence-required, 2 exception-requested, and 1196 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DungeonEncounterXCreatureFlags enums

Classified exactly six retail 12.0.0 `Enum.DungeonEncounterXCreatureFlags.*` and `Enum.DungeonEncounterXCreatureFlagsMeta.*` rows as best-effort/behavioral: `BossCreature=1`, `DropLootImmediately=2`, `DoNotDespawnOnSuccess=4`; metadata `MinValue=1`, `MaxValue=4`, and `NumValues=3`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `8cb0dda90867aed71a06eb707b2c836dcc2d872c`. Creature flag interpretation, loot timing, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1225 best-effort, 982 evidence-required, 2 exception-requested, and 1201 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DungeonEncounterTriggerType enums

Classified exactly eight retail 12.0.0 `Enum.DungeonEncounterTriggerType.*` and `Enum.DungeonEncounterTriggerTypeMeta.*` rows as best-effort/behavioral: `Invalid=0`, `OnStart=1`, `OnComplete=2`, `OnEnd=3`, `PreviouslyCompleted=4`; metadata `MinValue=0`, `MaxValue=4`, and `NumValues=5`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `17cd655bc64490cb39b0377f669160d876ef8af8`. Dungeon-encounter trigger behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1219 best-effort, 982 evidence-required, 2 exception-requested, and 1207 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DungeonEncounterFlags enums

Classified exactly thirteen retail 12.0.0 `Enum.DungeonEncounterFlags.*` and `Enum.DungeonEncounterFlagsMeta.*` rows as best-effort/behavioral: `StickyNews=1`, `GuildNews=2`, `RaidLockPlayers=4`, `AutoEnd=8`, `Cosmetic=16`, `Unused=32`, `HideUntilCompleted=64`, `NoAutoStart=128`, `IgnoreSpawnLimit=256`, `DisableEncounterEvents=512`; metadata `MinValue=1`, `MaxValue=512`, and `NumValues=10`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `9084f476893fd4be8275e05ac1c5f38e185bdb44`. Dungeon-encounter flag interpretation, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1211 best-effort, 982 evidence-required, 2 exception-requested, and 1215 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterVisibility enums

Classified exactly six retail 12.0.0 `Enum.DamageMeterVisibility.*` and `Enum.DamageMeterVisibilityMeta.*` rows as best-effort/behavioral: `Always=0`, `InCombat=1`, `Hidden=2`; metadata `MinValue=0`, `MaxValue=2`, and `NumValues=3`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `7652c9bbee3fe0f41a924ec47689ae311afa34a0`. Visibility behavior, combat transitions, consumers, persistence, and lifecycle remain unclaimed. Current totals are **1198 best-effort, 982 evidence-required, 2 exception-requested, and 1228 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterType enums

Classified exactly twelve retail 12.0.0 `Enum.DamageMeterType.*` and `Enum.DamageMeterTypeMeta.*` rows as best-effort/behavioral: `DamageDone=0`, `Dps=1`, `HealingDone=2`, `Hps=3`, `Absorbs=4`, `Interrupts=5`, `Dispels=6`, `DamageTaken=7`, `AvoidableDamageTaken=8`; metadata `MinValue=0`, `MaxValue=8`, and `NumValues=9`; later `Deaths` and `EnemyDamageTaken` are absent. Evidence is bounded to startup enum/metadata publication, Lua numeric type, exact values, and later-member absence. Focused proof is at `5395715a0ff9b71f8d66700cd0fb3eaf87322d42`. Root cause: shared sequential/runtime data also published the later members with `MaxValue=10`/`NumValues=11`; the retail 12.0.0 epoch override removes them and restores 12.0.0 metadata. Aggregation, calculations, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1192 best-effort, 982 evidence-required, 2 exception-requested, and 1234 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterStorageType enums

Classified exactly ten retail 12.0.0 `Enum.DamageMeterStorageType.*` and `Enum.DamageMeterStorageTypeMeta.*` rows as best-effort/behavioral: `Damage=0`, `HealingAndAbsorbs=1`, `Absorbs=2`, `Interrupts=3`, `Dispels=4`, `DamageTaken=5`, `AvoidableDamageTaken=6`; metadata `MinValue=0`, `MaxValue=6`, and `NumValues=7`; later `Deaths` and `EnemyDamageTaken` are absent. Evidence is bounded to startup enum/metadata publication, Lua numeric type, exact values, and later-member absence. Focused proof is at `786cbe81e73e50fbba65a53a50a461c80a536d40`. Root cause: shared generated runtime data also published the later members with `MaxValue=8`/`NumValues=9`; the retail 12.0.0 epoch override removes them and restores 12.0.0 metadata. Storage behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1180 best-effort, 982 evidence-required, 2 exception-requested, and 1246 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterSpellDetailsDisplayType enums

Classified exactly six retail 12.0.0 `Enum.DamageMeterSpellDetailsDisplayType.*` and `Enum.DamageMeterSpellDetailsDisplayTypeMeta.*` rows as best-effort/behavioral: `SpellCasted=0`, `UnitSpecificSpellCasted=1`, `SpellAffected=2`; metadata `MinValue=0`, `MaxValue=2`, and `NumValues=3`; later `Deaths` and `EnemyDamageTaken` are absent. Evidence is bounded to startup enum/metadata publication, Lua numeric type, exact values, and later-member absence. Focused proof is at `5fce4c9814293c336b3b580c868ee992a00c79f1`. Root cause: shared generated runtime data also published the later members with `MaxValue=4`/`NumValues=5`; the retail 12.0.0 epoch override removes them and restores 12.0.0 metadata. Spell-details display selection, formatting, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1180 best-effort, 982 evidence-required, 2 exception-requested, and 1246 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterSessionType enums

Classified exactly six retail 12.0.0 `Enum.DamageMeterSessionType.*` and `Enum.DamageMeterSessionTypeMeta.*` rows as best-effort/behavioral: `Overall=0`, `Current=1`, `Expired=2`; metadata `MinValue=0`, `MaxValue=2`, and `NumValues=3`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `319d5c3ee5837ffe5a7bf1b8ba92d0892a00a0bf`. Damage-meter session selection, aggregation, expiration, persistence, consumers, transitions, and lifecycle remain unclaimed. Current totals are **1164 best-effort, 982 evidence-required, 2 exception-requested, and 1262 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterOverrideType enums

Classified exactly eight retail 12.0.0 `Enum.DamageMeterOverrideType.*` and `Enum.DamageMeterOverrideTypeMeta.*` rows as best-effort/behavioral: `Ignore=0`, `AllowFriendlyFire=1`, `RedirectSourceToOwner=2`, `RedirectSourceToAuraCaster=3`, and `IgnoreForAbsorbSpell=4`; metadata `MinValue=0`, `MaxValue=4`, and `NumValues=5`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `cbbb9064bab0a7aebe4451834bbe77e89d5b951d`. Damage-meter override interpretation, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1158 best-effort, 982 evidence-required, 2 exception-requested, and 1268 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify DamageMeterNumbers enums

Classified exactly six retail 12.0.0 `Enum.DamageMeterNumbers.*` and `Enum.DamageMeterNumbersMeta.*` rows as best-effort/behavioral: `Minimal=0`, `Compact=1`, `Complete=2`; metadata `MinValue=0`, `MaxValue=2`, and `NumValues=3`. Evidence is bounded to startup enum/metadata publication, Lua numeric type, and exact values. Focused proof is at `fdcfc288a2048f1c29eb541e6699018cb56804fe`. Damage-meter number formatting, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1150 best-effort, 982 evidence-required, 2 exception-requested, and 1276 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify crafting-order result enums

Classified exactly two retail 12.0.0 `Enum.CraftingOrderResult.*` rows as best-effort/behavioral: `MissingCurrency=30` and `TooManyCurrencies=46`. Evidence is bounded to startup enum publication, Lua numeric type, and exact values. Focused proof is at `ac04f9fc55381073c01ba07d794d0ba0d06b0b91`. Crafting-order result production, interpretation, persistence, transitions, consumers, and lifecycle remain unclaimed. Current totals are **1144 best-effort, 982 evidence-required, 2 exception-requested, and 1282 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify ItemCreationContext removals

Classified exactly two removed retail 12.0.0 `Enum.ItemCreationContext.*` rows as best-effort/behavioral: `Placeholder_12_0_0` (prior 186) and `Timewalker` (prior 22) are absent at startup. Focused proof is at `4919858d04ce4b3e4418ed334f4c228de8129daa`. Item-creation context selection, item provenance, consumers, transitions, and lifecycle remain unclaimed. Current totals are **1142 best-effort, 982 evidence-required, 2 exception-requested, and 1284 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify removed expansion landing-page enums

Classified exactly six removed retail 12.0.0 `Enum.ExpansionLandingPageType.*` and `Enum.ExpansionLandingPageTypeMeta.*` rows as best-effort/behavioral. The retail-12.0.0 post-compat epoch override proves `None`, `Dragonflight`, `WarWithin`, `MinValue`, `MaxValue`, and `NumValues` are absent. Focused proof is at `54416879b5c6305ed956f347fe0a17e9fb2ccb28`; landing-page selection, UI behavior, transitions, and lifecycle remain unclaimed. Current totals are **1140 best-effort, 982 evidence-required, 2 exception-requested, and 1286 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify crafting-order item type

Classified exactly seven retail 12.0.0 `Enum.CraftingOrderItemType.*` and `Enum.CraftingOrderItemTypeMeta.*` rows as best-effort/behavioral: added `Item=0`, `Deprecated=4`, and `Currency=5`; changed metadata `MaxValue=5` and `NumValues=6`; removed `NpcProvided` (prior value 4) and `Reagent` (prior value 0) are absent. Focused proof is at `a72ad8574f25a5e7ed6e58065ee5da615a1c2233`, with unchanged `Recraft=1`, `CraftedResult=2`, `RemoveReagent=3`, and metadata `MinValue=0` asserted as context. Crafting-order item classification, payload behavior, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1134 best-effort, 982 evidence-required, 2 exception-requested, and 1292 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify crafting-order item flags

Classified exactly six retail 12.0.0 `Enum.CraftingOrderItemFlags.*` and `Enum.CraftingOrderItemFlagsMeta.*` rows as best-effort/behavioral, bounded to startup enum/metadata numeric publication and exact values (`None=0`, `NpcProvided=1`, `HasEnchantmentData=2`, `MinValue=0`, `MaxValue=2`, `NumValues=3`). Focused proof is at `3bacb64a269eecc6c1b74838f256b6e0a4473911`. Flag assignment, enchantment/NPC item semantics, order payload behavior, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1127 best-effort, 982 evidence-required, 2 exception-requested, and 1299 untriaged** (3410 rows).

## [2026-08-10] architecture | Separate retail profile selection from API epochs

Documented the internal `profile-retail` Cargo feature as the retail profile marker. It selects the retail Blizzard cache without forcing the current API epoch; public `client-retail` remains the current-retail bundle and enables `profile-retail` plus cumulative `retail-12-0-7`. `RetailApiEpoch` and `ACTIVE_RETAIL_API_EPOCH` select the highest enabled cumulative retail epoch. Historical 12.0.0 audit tests use `cargo test --no-default-features --features profile-retail,retail-12-0-0`, and profile-specific runtime behavior gates use `profile-retail` so historical retail tests retain retail semantics. No 12.1, 12.0.7, or 12.0.5 audit-status changes.

## [2026-08-10] investigation | Classify cooldown-viewer alert-type enums

Classified exactly five retail 12.0.0 `Enum.CooldownViewerAlertType.*` and `Enum.CooldownViewerAlertTypeMeta.*` rows as best-effort/behavioral, bounded to startup enum/metadata numeric publication and exact values (`Sound=1`, `Visual=2`, `MinValue=1`, `MaxValue=2`, `NumValues=2`). Focused final-runtime proof is at `5fe329d4d5d852365e81839df992d904d291305d`. Alert triggering, sound/visual effects, configuration, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1121 best-effort, 982 evidence-required, 2 exception-requested, and 1305 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-log message-order enums

Classified exactly five retail 12.0.0 `Enum.CombatLogMessageOrder.*` and `Enum.CombatLogMessageOrderMeta.*` rows as best-effort/behavioral, bounded to startup enum/metadata numeric publication and exact values (`Newest=0`, `Oldest=1`, `MinValue=0`, `MaxValue=1`, `NumValues=2`). Focused proof is at `2d6495a732f7cb598cf67871eae507c28fc7c3bb`. Combat-log storage order, query behavior, persistence, UI consumers, transitions, and lifecycle remain unclaimed. Current totals are **1116 best-effort, 982 evidence-required, 2 exception-requested, and 1310 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio unit enums

Classified exactly five retail 12.0.0 `Enum.CombatAudioAlertUnit.*` and `Enum.CombatAudioAlertUnitMeta.*` rows as best-effort/behavioral, bounded to startup enum/metadata numeric publication and exact values (`Player=0`, `Target=1`, `MinValue=0`, `MaxValue=1`, `NumValues=2`). Focused proof is at `5f5635e7b99954ae0f057deb93292e39ddba3bfa`. Unit selection, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1111 best-effort, 982 evidence-required, 2 exception-requested, and 1315 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio type enums

Classified exactly five retail 12.0.0 `Enum.CombatAudioAlertType.*` and `Enum.CombatAudioAlertTypeMeta.*` rows as best-effort/behavioral, bounded to startup enum/metadata numeric publication and exact values (`Health=0`, `Cast=1`, `MinValue=0`, `MaxValue=1`, `NumValues=2`). Focused proof is at `0c51d4c32a8ae89fffdf3ca67bf80d03926496d2`. Health/cast classification behavior, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1106 best-effort, 982 evidence-required, 2 exception-requested, and 1320 untriaged** (3410 rows).

## [2026-08-10] investigation | Correct and classify combat-audio throttle enums

Classified exactly ten retail 12.0.0 `Enum.CombatAudioAlertThrottle.*` and `Enum.CombatAudioAlertThrottleMeta.*` rows as best-effort/behavioral, bounded to retail 12.0.0 startup enum/metadata Lua numeric publication, exact values (`Sample=0`, `PlayerHealth=1`, `TargetHealth=2`, `PlayerCast=3`, `TargetCast=4`, `PlayerResource1=5`, `PlayerResource2=6`; metadata `MinValue=0`, `MaxValue=6`, `NumValues=7`), and absence of later-epoch `PlayerHealthSamePercent`, `TargetHealthSamePercent`, `PlayerResource1SamePercent`, and `PlayerResource2SamePercent` members after the retail-12.0.0 override. Focused proof is at `8096cc84cb9e200840507937c0b379bb6f0ca656`. Throttle timing, coalescing, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed.

Current totals are **1101 best-effort, 982 evidence-required, 2 exception-requested, and 1325 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio target-health format enums

Classified exactly twelve retail 12.0.0 `Enum.CombatAudioAlertTargetHealthFormatValues.*` and `Enum.CombatAudioAlertTargetHealthFormatValuesMeta.*` rows as best-effort/behavioral. Focused startup proof at `3c320a56d387e716e299f114972e1c9746abc2f0` is limited to exact Lua numeric publication: `NoHealthFull=0`, `NoHealthNoPercent=1`, `NoHealthNoPercentDiv10=2`, `HealthFull=3`, `HealthNoPercent=4`, `HealthNoPercentDiv10=5`, `TargetFull=6`, `TargetNoPercent=7`, and `TargetNoPercentDiv10=8`, with metadata `MinValue=0`, `MaxValue=8`, and `NumValues=9`. Target-health state detection, percent/divisor formatting, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1091 best-effort, 982 evidence-required, 2 exception-requested, and 1335 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio target-death behavior enums

Classified exactly five retail 12.0.0 `Enum.CombatAudioAlertTargetDeathBehavior.*` and `Enum.CombatAudioAlertTargetDeathBehaviorMeta.*` rows as best-effort/behavioral. Focused startup proof at `a9860867b49d156f3d84535843b7807cb1bf646b` is limited to exact Lua numeric publication: `Default=0`, `SayTargetDead=1`, with metadata `MinValue=0`, `MaxValue=1`, and `NumValues=2`. Target-death detection, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1079 best-effort, 982 evidence-required, 2 exception-requested, and 1347 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio target-cast format enums

Classified exactly ten retail 12.0.0 `Enum.CombatAudioAlertTargetCastFormatValues.*` and `Enum.CombatAudioAlertTargetCastFormatValuesMeta.*` rows as best-effort/behavioral. Focused startup proof at `523c8cb79ebdb304c64884ff86a79a19d018d161` is limited to exact Lua numeric publication: `TargetCastingSpellname=0`, `TargetCastSpellname=1`, `CastingSpellname=2`, `CastSpellname=3`, `Casting=4`, `Cast=5`, `Spellname=6`, with metadata `MinValue=0`, `MaxValue=6`, and `NumValues=7`. Target/cast state detection, spell-name formatting, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1074 best-effort, 982 evidence-required, 2 exception-requested, and 1352 untriaged** (3410 rows).

## [2026-08-10] investigation | Correct and classify combat-audio spec-setting enums

Corrected the retail 12.0.0 `Enum.CombatAudioAlertSpecSetting` fallback by removing later-epoch `Resource1Voice`, `Resource1Volume`, `Resource2Voice`, and `Resource2Volume` members and restoring the checked-in values: `Resource1Percent=0`, `Resource1Format=1`, `Resource2Percent=2`, `Resource2Format=3`, and `SayIfTargeted=4`; metadata is `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused startup proof is committed at `5ff1d782f8c83aa8d972a608c31b62c6d89e92a6`. The eight rows are best-effort/behavioral, limited to exact numeric publication, metadata, and absence of those later-epoch members; combat-audio configuration persistence, narration, resource/target state, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1064 best-effort, 982 evidence-required, 2 exception-requested, and 1362 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio targeted-state enums

Classified the seven `Enum.CombatAudioAlertSayIfTargetedType.*` and `Enum.CombatAudioAlertSayIfTargetedTypeMeta.*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; aggro/target detection, narration, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1056 best-effort, 982 evidence-required, 2 exception-requested, and 1370 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio player-resource format enums

Classified the nine `Enum.CombatAudioAlertPlayerResourceFormatValues.*` and `Enum.CombatAudioAlertPlayerResourceFormatValuesMeta.*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; resource narration, percent/divisor formatting, resource selection, configuration, sound side effects, and lifecycle remain unclaimed. Current totals are **1049 best-effort, 982 evidence-required, 2 exception-requested, and 1377 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio player-health format enums

Classified the nine `Enum.CombatAudioAlertPlayerHealthFormatValues.*` and `Enum.CombatAudioAlertPlayerHealthFormatValuesMeta.*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; health narration, percent/divisor formatting, configuration, sound side effects, and lifecycle remain unclaimed. Current totals are **1040 best-effort, 982 evidence-required, 2 exception-requested, and 1386 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio player-cast format enums

Classified the eight `Enum.CombatAudioAlertPlayerCastFormatValues.*` and `Enum.CombatAudioAlertPlayerCastFormatValuesMeta.*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; narration, cast detection, spell-name formatting, configuration, sound side effects, and lifecycle remain unclaimed. Current totals are **1031 best-effort, 982 evidence-required, 2 exception-requested, and 1395 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify combat-audio party-percent enums

Classified the 14 `Enum.CombatAudioAlertPartyPercentValues.*` and `Enum.CombatAudioAlertPartyPercentValuesMeta.*` rows, plus the nine `Enum.CombatAudioAlertPercentValues.*` and `Enum.CombatAudioAlertPercentValuesMeta.*` rows, as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; combat-audio threshold behavior, triggers, configuration, sound side effects, transitions, and lifecycle remain unclaimed. Current totals are **1023 best-effort, 982 evidence-required, 2 exception-requested, and 1403 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify chat-lockdown and combat-audio enums

Classified the three `Enum.ChatMessagingLockdownReason.*`, three `Enum.ChatMessagingLockdownReasonMeta.*`, three `Enum.CombatAudioAlertCastState.*`, and three `Enum.CombatAudioAlertCastStateMeta.*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; chat-lockdown enforcement, secret/taint behavior, errors, and lifecycle; combat-audio triggers, configuration, sound side effects, and lifecycle remain unclaimed. Current totals are **1000 best-effort, 982 evidence-required, 2 exception-requested, and 1426 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify bulk purchase/refund enums

Classified the 20 `Enum.BulkPurchaseResult*` and `Enum.BulkRefundResult*` rows as best-effort/behavioral. Evidence is limited to exact startup enum/metadata numeric publication and values; purchase/refund transaction state, services, side effects, errors, and lifecycle remain unclaimed. Current totals are **988 best-effort, 982 evidence-required, 2 exception-requested, and 1438 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify 12.0.0 enum rows

Classified the four `Enum.AccountDataUpdateStatus.*`, three `Enum.AddOnRestrictionState.*`, three `Enum.AddOnRestrictionStateMeta.*`, five `Enum.AddOnRestrictionType.*`, three `Enum.AddOnRestrictionTypeMeta.*`, three `Enum.AuraFrameVisibleSetting.*`, and three `Enum.AuraFrameVisibleSettingMeta.*` rows as best-effort/behavioral. Evidence is bounded to exact startup enum/metadata numeric publication and values; restriction policy, enforcement, events, lifecycle, AuraContainer behavior, and visibility transitions remain unclaimed. Current totals are **968 best-effort, 982 evidence-required, 2 exception-requested, and 1458 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify UnitPowerSpellIDs constants

Classified the five `Constants.UnitPowerSpellIDs` rows as best-effort/behavioral: focused exact type/value proof at `ce4ea294bff9f41401f74e3e9e5ca40d42ea4516` covers startup publication only; spell, aura, alternate-power, consumer, mutation, and lifecycle semantics remain unclaimed. Current totals are **944 best-effort, 982 evidence-required, 2 exception-requested, and 1482 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Damage Meter events

Classified `DAMAGE_METER_COMBAT_SESSION_UPDATED`, `DAMAGE_METER_CURRENT_SESSION_UPDATED`, and `DAMAGE_METER_RESET` as evidence-required/unsafe: event names are registered, but no Damage Meter session/reset producer or state/timing model exists. Payload/order/duplicates, UI refresh, persistence, and lifecycle semantics remain unresolved. Current totals are **939 best-effort, 982 evidence-required, 2 exception-requested, and 1487 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify encounter timeline events

Classified `ENCOUNTER_STATE_CHANGED`, the eight `ENCOUNTER_TIMELINE_*` events, and `ENCOUNTER_WARNING` as evidence-required/unsafe: names are registered and addons load, but no encounter timeline/warning producer or state machine exists. Payloads, synchronous/unique/delayed timing, order, post-dispatch query state, UI refresh, persistence, and lifecycle semantics remain unresolved. Current totals are **939 best-effort, 982 evidence-required, 2 exception-requested, and 1487 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify criteria tracking fields

Classified `CriteriaRequiredValue.criteriaID`, `.requiredValue`, `CriteriaRequirement.completed`, and `.requirementText` as evidence-required/unsafe: the tracking namespace emits nil/empty defaults and has no criteria/requirement payload producer; field/type/value/progress/completion/localization/event/persistence/lifecycle semantics remain unresolved. Current totals are **939 best-effort, 969 evidence-required, 2 exception-requested, and 1500 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify crafting structure fields

Classified the 19 crafting structure fields from `CraftingItemSlotModification.reagent` through `CraftingVariableQuantities.reagent` as evidence-required/unsafe: no typed runtime producers cover reagent, quality/icon, variable-quantity, or resource-return fields; type/value/nesting/validation/update/event/persistence/lifecycle semantics remain unresolved. Current totals are **939 best-effort, 965 evidence-required, 2 exception-requested, and 1504 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify grouped constants

Classified 38 grouped constants as best-effort/behavioral: six misc target/currency/item/profession/transmog constants, 12 EncounterTimeline constants, six TTS constants, 13 UICharacterClasses constants, and `UnitEventConstants.MAX_UNIT_TOKENS_IN_EVENT`. Focused grouped test proof at `1895cacebd8fa8f881f053694ec61de9a645824c` establishes startup Lua numeric type/value only; consumer, mutation, protection, and subsystem semantics remain unclaimed. Current totals are **939 best-effort, 946 evidence-required, 2 exception-requested, and 1523 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify WeeklyRewards progress rows

Classified `C_WeeklyRewards.GetSortedProgressForActivity` and `WeeklyRewardActivityTierProgress.activityTierID`, `.difficulty`, and `.numPoints` as evidence-required/unsafe: the API is absent, and current Great Vault state lacks tier IDs, difficulty, points, and shared-difficulty sorting. Field/result/event/persistence/lifecycle semantics remain unresolved. Current totals are **901 best-effort, 946 evidence-required, 2 exception-requested, and 1561 untriaged** (3410 rows).

## [2026-08-10] investigation | Classify Prey Hunt widget visualization

Classified `C_UIWidgetManager.GetPreyHuntProgressWidgetVisualizationInfo` and its 16 `PreyHuntProgressWidgetVisualizationInfo` fields as evidence-required/unsafe: the API is absent and no Prey Hunt widget state or payload producer exists. Widget lookup, typed fields, timer/progress/animation/tooltip/texture/model values, refresh, events, persistence, and lifecycle remain unresolved. Current totals are **901 best-effort, 942 evidence-required, 2 exception-requested, and 1565 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_Tutorial.GetCombatEventInfo

Classified `C_Tutorial.GetCombatEventInfo` as evidence-required/unsafe: the explicit implementation is a zero-result no-op while the checked-in source provides no output schema; authoritative return semantics, combat state/producer/timing, persistence, and lifecycle remain unresolved. Current totals are **901 best-effort, 925 evidence-required, 2 exception-requested, and 1582 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify catalog and combat-log constants

Classified `Constants.CatalogShopVirtualCurrencyConstants.HEARTHSTEEL_VC_CURRENCY_CODE` (`XVV`), `TRADERS_TENDER_VC_CURRENCY_CODE` (`XWP`), both `CombatLogMessageLimits` values (`300`, `1000`), and all five `CombatLogObjectMasks` values (`15`, `768`, `240`, `-65536`, `64512`) as best-effort/behavioral. Focused proof at `a5a3d167b3e0e747e02e7c711b4255efe0488cef` establishes startup Lua type/value only; consumer, mutation, protection, and broader semantics remain unclaimed. Current totals are **901 best-effort, 924 evidence-required, 2 exception-requested, and 1583 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_TransmogSets APIs

Classified `C_TransmogSets.GetAvailableSets`, `GetSetsFilter`, `IsUsingDefaultSetsFilters`, `SetDefaultSetsFilters`, `SetSetsFilter`, and `C_TransmogSets.TransmogSetInfo.grantAsPrecedingVariant` as evidence-required/unsafe: the temporary surface has empty/default compatibility only, with no wardrobe set inventory, filter state, or variant relationship. API results/mutations/field/event/persistence/lifecycle semantics remain unresolved. Current totals are **892 best-effort, 924 evidence-required, 2 exception-requested, and 1592 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_TooltipComparison.CompareItem

Classified `C_TooltipComparison.CompareItem` as evidence-required/unsafe: no native C_TooltipComparison implementation or comparison-data model exists. Protected-call behavior, item selection, tooltip rendering/anchors/deltas, cleanup, and lifecycle remain unresolved. Current totals are **892 best-effort, 918 evidence-required, 2 exception-requested, and 1598 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_TooltipInfo APIs

Classified `C_TooltipInfo.GetOutfit`, `GetUnitAuraByAuraInstanceID`, `GetRecipeResultItem`, and `GetRecipeResultItemForOrder` as evidence-required/unsafe: GetOutfit is absent; aura implementation is player-only and ignores filter; recipe methods ignore reagent/order/recraft/level/quality inputs and return static output-item tooltips. Full payload/secret/lifecycle semantics remain unresolved. Current totals are **892 best-effort, 917 evidence-required, 2 exception-requested, and 1599 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_TaskQuest.GetQuestUIWidgetSetByType

Classified `C_TaskQuest.GetQuestUIWidgetSetByType` as evidence-required/unsafe: explicit implementation produces synthetic widget-set IDs from static world-quest fixtures; authoritative per-quest/type mapping, enum/nil behavior, refresh/widget/event/persistence/lifecycle remain unresolved. Current totals are **892 best-effort, 913 evidence-required, 2 exception-requested, and 1603 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_Transmog.TransmogApplyWarningInfo fields

Classified `C_Transmog.TransmogApplyWarningInfo.itemLink` and `.text` as evidence-required/unsafe: removed parent structure/fields lack runtime absence and exact removal/load timing proof; source-token coverage is insufficient. Current totals are **892 best-effort, 912 evidence-required, 2 exception-requested, and 1604 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_StableInfo.IsBonusPetSlotAvailable

Classified `C_StableInfo.IsBonusPetSlotAvailable` as evidence-required/unsafe: implementation has `IsAtPetStable` only; no Beast Master Animal Companion or active-combat-configuration availability model exists. Boolean results, configuration transitions, stable UI refresh, events, persistence, and lifecycle remain unresolved. Current totals are **892 best-effort, 910 evidence-required, 2 exception-requested, and 1606 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_SpellBook lookup and duration APIs

Classified `C_SpellBook.FindBaseSpellByID`, `FindFlyoutSlotBySpellID`, `FindSpellOverrideByID`, `GetSpellBookItemChargeDuration`, `GetSpellBookItemCooldownDuration`, and `GetSpellBookItemLossOfControlCooldownDuration` as evidence-required/unsafe: mapping fallbacks are nil/input-ID compatibility only and no flyout/override model exists; duration APIs lack slot/bank cooldown state and a LuaDurationObject producer. Known/unknown mappings, required/nilable results, valid slots/banks, timing/expiration, refresh, object lifetime, and lifecycle remain unproven. Current totals are **892 best-effort, 909 evidence-required, 2 exception-requested, and 1607 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_Sound.PlaySound

Classified `C_Sound.PlaySound` as evidence-required/unsafe: the silent no-op fallback proves callability only and ignores inputs and returns; no-audio builds are not a scope exception. Input defaults, invalid IDs, success/failure, handles, duplicate suppression, finish callbacks, priority, backend behavior, and lifecycle remain unproven. Current totals are **892 best-effort, 903 evidence-required, 2 exception-requested, and 1613 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_SettingsUtil APIs

Classified `C_SettingsUtil.NotifySettingsLoaded` and `C_SettingsUtil.OpenSettingsPanel` as evidence-required/unsafe: current settings defaults model partial Settings categories/panel helpers only; no C_SettingsUtil namespace, SETTINGS_LOADED dispatch, or category+element scrolling implementation exists. Event timing/order, nil/valid/unknown category and element targets, repeated opens, visibility/scrolling, and lifecycle remain unproven. Current totals are **892 best-effort, 902 evidence-required, 2 exception-requested, and 1614 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify secure transfer APIs

Classified `C_SecureTransfer.Cancel`, `C_SecureTransfer.CompleteHousingPurchase`, `C_SecureTransfer.CompleteHousingVCPurchase`, `C_SecureTransfer.GetHousingPurchaseCost`, and `C_SecureTransfer.GetHousingVCPurchaseProductID` as evidence-required/unsafe: current temporary state exposes counters/lastAction and manually injected numeric query values only; no secure transaction, pricing/product producer, validation, authorization, purchase mutation, cancellation/rollback, callbacks/events exists. Exact costs/products and valid/invalid/repeated transaction lifecycle remain unproven. Current totals are **892 best-effort, 900 evidence-required, 2 exception-requested, and 1616 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify restricted-action policy APIs

Classified `C_RestrictedActions.CheckAllowProtectedFunctions`, `C_RestrictedActions.GetAddOnRestrictionState`, and `C_RestrictedActions.IsAddOnRestrictionActive` as evidence-required/unsafe: constant-true and constant-Inactive defaults are compatibility scaffolding, the active query is absent, and no per-object protected-function policy or per-type restriction state machine exists. Object and silent semantics, enum types, transitions, taint/restriction enforcement, refresh, events, persistence, and lifecycle remain unproven. Current totals are **892 best-effort, 895 evidence-required, 2 exception-requested, and 1621 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify current-player faction paragon eligibility

Classified `C_Reputation.IsFactionParagonForCurrentPlayer` as best-effort/behavioral: explicit faction-paragon state and focused tests prove registered eligible factions return true, while level-gated and missing factions return false. Live service population, refresh, events, persistence, malformed inputs, and lifecycle remain outside the claim. Current totals are **892 best-effort, 892 evidence-required, 2 exception-requested, and 1624 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify quest favor and active prey APIs

Classified `C_QuestInfoSystem.GetQuestLogRewardFavor` and `C_QuestLog.GetActivePreyQuest` as evidence-required/unsafe: no quest-specific favor or cycle-cap model and no active-prey quest state exist; current quest surfaces cover other classification, log, and reward behavior only. Quest IDs, favor amounts and clamping, inactive nil and active prey IDs, transitions, refresh, events, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 892 evidence-required, 2 exception-requested, and 1625 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_PvP BattlegroundInfo and Training Grounds APIs

Classified all 23 added C_PvP rows as evidence-required/unsafe: the simulator only has legacy positional battleground probes/state, not C_PvP.BattlegroundInfo or Training Grounds availability/catalog/rewards/queue/match/daily-win models. Required/nilable fields, structure-return contract, eligibility/reasons, lists/rewards, match/win transitions, join validation/events/persistence/lifecycle remain unproven. Current totals are **891 best-effort, 890 evidence-required, 2 exception-requested, and 1627 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_Ping.IsPingSystemEnabled

Classified `C_Ping.IsPingSystemEnabled` as evidence-required/unsafe: the source requires a boolean enabled result, but temporary ping defaults have no ping availability state or explicit method; generic fallback does not model it. Enabled/disabled states, context changes, transitions, refresh, events, and lifecycle remain unproven. Current totals are **891 best-effort, 867 evidence-required, 2 exception-requested, and 1650 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative access, task-removal, request, and viewing APIs

Classified `C_NeighborhoodInitiative.PlayerHasInitiativeAccess`, `C_NeighborhoodInitiative.PlayerMeetsRequiredLevel`, `C_NeighborhoodInitiative.RemoveTrackedInitiativeTask`, `C_NeighborhoodInitiative.RequestInitiativeActivityLog`, `C_NeighborhoodInitiative.RequestNeighborhoodInitiativeInfo`, `C_NeighborhoodInitiative.SetActiveNeighborhood`, and `C_NeighborhoodInitiative.SetViewingNeighborhood` as evidence-required/unsafe: access/level queries lack eligibility state and fallback nil; tracked-task removal is a no-op; request and active/viewing setters lack request/producers/context state. Boolean eligibility, valid/invalid IDs/GUIDs, mutations, request timing/results/events, transitions, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 866 evidence-required, 2 exception-requested, and 1651 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.NeighborhoodInitiativeInfo fields

Classified `C_NeighborhoodInitiative.NeighborhoodInitiativeInfo` fields `currentCycleID`, `currentProgress`, `description`, `duration`, `initiativeID`, `isLoaded`, `milestones`, `neighborhoodGUID`, `playerTotalContribution`, `progressRequired`, `tasks`, and `title` as evidence-required/unsafe: the source requires 12 scalar/nested initiative payload fields, but no NeighborhoodInitiativeInfo model or populated producer exists; generic GetNeighborhoodInitiativeInfo fallback returns nil. Loaded/nil states, scalar values, ordered milestone/task arrays, progress/cycle/contribution transitions, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 859 evidence-required, 2 exception-requested, and 1658 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative tracked state and queries

Classified `C_NeighborhoodInitiative.InitiativeTasksTracked.trackedIDs`, `C_NeighborhoodInitiative.IsInitiativeEnabled`, `C_NeighborhoodInitiative.IsPlayerInNeighborhoodGroup`, and `C_NeighborhoodInitiative.IsViewingActiveNeighborhood` as evidence-required/unsafe: trackedIDs currently proves only an empty table shape with no tracked state/add-remove synchronization; IsInitiativeEnabled is constant false scaffolding; group/view queries lack backing state and return fallback nil. Numeric IDs, query booleans, valid/invalid/duplicate transitions, ordering, context changes, refresh, persistence, events, and lifecycle remain unproven. Current totals are **891 best-effort, 847 evidence-required, 2 exception-requested, and 1670 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeTaskInfo fields

Classified `C_NeighborhoodInitiative.InitiativeTaskInfo` fields `ID`, `completed`, `criteriaList`, `description`, `inProgress`, `progressContributionAmount`, `requirementsList`, `rewardQuestID`, `sortOrder`, `supersedes`, `taskName`, `taskType`, `timesCompleted`, and `tracked` as evidence-required/unsafe: the source requires 14 task fields including nested criteria/requirements arrays and a task-type enum, but no InitiativeTaskInfo model, task state, or populated producer exists; the explicit default returns nil and the current test proves nil only. Field types/values, absent/valid/unknown IDs, arrays/order, completion/progress/tracking transitions, refresh, persistence, events, and lifecycle remain unproven. Current totals are **891 best-effort, 843 evidence-required, 2 exception-requested, and 1674 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeMilestoneInfo.rewards and InitiativeMilestoneRewardInfo fields

Classified `C_NeighborhoodInitiative.InitiativeMilestoneInfo.rewards` plus `C_NeighborhoodInitiative.InitiativeMilestoneRewardInfo.decorID`, `decorQuantity`, `description`, `favor`, `money`, `rewardQuestID`, and `title` as evidence-required/unsafe: the source defines a required reward array and seven required fields, but no InitiativeMilestoneInfo/InitiativeMilestoneRewardInfo runtime model or initiative payload producer exists; generic fallback produces no milestones. Reward arrays/order, entry shape/types/values, empty/populated milestones, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 829 evidence-required, 2 exception-requested, and 1688 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeMilestoneInfo.requiredContributionAmount

Classified `C_NeighborhoodInitiative.InitiativeMilestoneInfo.requiredContributionAmount` as evidence-required/unsafe: the source defines a required numeric requiredContributionAmount field, but no InitiativeMilestoneInfo runtime model or initiative payload producer exists; generic fallback produces no milestones. Contribution thresholds, empty/populated milestones, ordering/progress comparisons, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 821 evidence-required, 2 exception-requested, and 1696 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeMilestoneInfo.milestoneOrderIndex

Classified `C_NeighborhoodInitiative.InitiativeMilestoneInfo.milestoneOrderIndex` as evidence-required/unsafe: the source defines a required numeric milestoneOrderIndex field, but no InitiativeMilestoneInfo runtime model or initiative payload producer exists; generic fallback produces no milestones. Numeric indexes, ordering, empty/populated milestones, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 820 evidence-required, 2 exception-requested, and 1697 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogInfo.taskActivity

Classified `C_NeighborhoodInitiative.InitiativeActivityLogInfo.taskActivity` as evidence-required/unsafe: the source defines a required array of InitiativeActivityLogEntry structures, but no activity-log info/entry model or producer exists; generic fallback produces no payload. Absent/present behavior, array shape/order, entry fields, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 819 evidence-required, 2 exception-requested, and 1698 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogInfo.neighborhoodGUID

Classified `C_NeighborhoodInitiative.InitiativeActivityLogInfo.neighborhoodGUID` as evidence-required/unsafe: the source defines a required string neighborhoodGUID field, but no InitiativeActivityLogInfo runtime model or producer exists; generic fallback produces no payload. GUID presence and values, absent/present logs, refresh/update transitions, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 817 evidence-required, 2 exception-requested, and 1700 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogInfo.nextUpdateTime

Classified `C_NeighborhoodInitiative.InitiativeActivityLogInfo.nextUpdateTime` as evidence-required/unsafe: the source defines a required numeric nextUpdateTime field, but no InitiativeActivityLogInfo runtime model or producer exists; generic fallback produces no payload. Numeric timing presence and values, absent/present logs, update/refresh transitions, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 818 evidence-required, 2 exception-requested, and 1699 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogInfo.isLoaded

Classified `C_NeighborhoodInitiative.InitiativeActivityLogInfo.isLoaded` as evidence-required/unsafe: the source defines a required boolean isLoaded field, but no InitiativeActivityLogInfo runtime model or producer exists; generic fallback produces no payload. Loaded-state presence/values, absent/present logs, refresh/update transitions, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 816 evidence-required, 2 exception-requested, and 1701 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogEntry.taskName

Classified `C_NeighborhoodInitiative.InitiativeActivityLogEntry.taskName` as evidence-required/unsafe: the source defines a required string taskName field, but no InitiativeActivityLogEntry runtime model or activity-log producer exists; generic fallback produces no payload. String field presence/values, empty/present logs, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 815 evidence-required, 2 exception-requested, and 1702 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogEntry.taskID

Classified `C_NeighborhoodInitiative.InitiativeActivityLogEntry.taskID` as evidence-required/unsafe: the source defines a required numeric taskID field, but no InitiativeActivityLogEntry runtime model or activity-log producer exists; generic fallback produces no payload. Numeric task-ID presence/values, invalid/unknown tasks, empty/present logs, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 814 evidence-required, 2 exception-requested, and 1703 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogEntry.playerName

Classified `C_NeighborhoodInitiative.InitiativeActivityLogEntry.playerName` as evidence-required/unsafe: the source defines a required string playerName field, but no InitiativeActivityLogEntry runtime model or activity-log producer exists; generic fallback produces no payload. String field presence/values, empty/present logs, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 813 evidence-required, 2 exception-requested, and 1704 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogEntry.completionTime

Classified `C_NeighborhoodInitiative.InitiativeActivityLogEntry.completionTime` as evidence-required/unsafe: the source defines a required numeric completionTime field, but no InitiativeActivityLogEntry runtime model or activity-log producer exists and generic fallback produces no payload. Numeric time presence/values, empty/present logs, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 812 evidence-required, 2 exception-requested, and 1705 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.InitiativeActivityLogEntry.amount

Classified `C_NeighborhoodInitiative.InitiativeActivityLogEntry.amount` as evidence-required/unsafe: the source defines a required numeric amount field, but no InitiativeActivityLogEntry runtime model or activity-log producer exists and generic fallback produces no payload. Numeric field presence/values, empty/present logs, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 811 evidence-required, 2 exception-requested, and 1706 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetTrackedInitiativeTasks

Classified `C_NeighborhoodInitiative.GetTrackedInitiativeTasks` as evidence-required/unsafe: the source returns InitiativeTasksTracked with numeric trackedIDs, but the fallback always returns an empty table shape and the focused probe proves only that shape. No tracked-task state exists, so add/remove, duplicate, invalid/unknown ID, ordering, refresh, persistence, events, and lifecycle remain unproven. Current totals are **891 best-effort, 810 evidence-required, 2 exception-requested, and 1707 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetRequiredLevel

Classified `C_NeighborhoodInitiative.GetRequiredLevel` as evidence-required/unsafe: the source requires one numeric level, but no explicit method or initiative required-level state exists; namespace defaults omit it and generic fallback returns nil. No-initiative behavior, valid levels, transitions, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 809 evidence-required, 2 exception-requested, and 1708 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo

Classified `C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo` as evidence-required/unsafe: the source returns a nilable NeighborhoodInitiativeInfo payload with progress, milestone, task, contribution, timing, and neighborhood fields, but no explicit method or state model exists, namespace defaults omit it, and generic fallback returns nil. Nil alone does not establish populated payloads, field types/values, transitions, refresh, persistence, or lifecycle. Current totals are **891 best-effort, 808 evidence-required, 2 exception-requested, and 1709 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetInitiativeTaskInfo

Classified `C_NeighborhoodInitiative.GetInitiativeTaskInfo` as evidence-required/unsafe: the source returns a nilable InitiativeTaskInfo by numeric task ID, but the current explicit fallback always returns nil with no task records or payload model; the focused probe proves only nil for ID 1. Valid/unknown IDs, exact fields/types, updates, tracked-task interaction, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 807 evidence-required, 2 exception-requested, and 1710 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetInitiativeTaskChatLink

Classified `C_NeighborhoodInitiative.GetInitiativeTaskChatLink` as evidence-required/unsafe: the source requires a string chat link by numeric task ID, but no explicit method or task/chat-link model exists, namespace defaults omit it, and generic fallback returns nil. Valid/unknown IDs, exact link format, absent-task behavior, updates, refresh, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 807 evidence-required, 2 exception-requested, and 1710 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetInitiativeActivityLogInfo

Classified `C_NeighborhoodInitiative.GetInitiativeActivityLogInfo` as evidence-required/unsafe: the source returns a nilable InitiativeActivityLogInfo payload, but no explicit method or activity-log state model exists, namespace defaults omit it, and generic fallback returns nil. Nil alone does not establish absent/present behavior, payload fields/entries, update timing, refresh/events, persistence, or lifecycle. Current totals are **891 best-effort, 805 evidence-required, 2 exception-requested, and 1712 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.GetActiveNeighborhood

Classified `C_NeighborhoodInitiative.GetActiveNeighborhood` as evidence-required/unsafe: the source requires one neighborhoodGUID string, but no explicit method or active-neighborhood model exists; namespace defaults omit it and generic fallback returns nil. No-active behavior, valid GUID output, state transitions, refresh, events, persistence, and lifecycle remain unproven. Current totals are **891 best-effort, 804 evidence-required, 2 exception-requested, and 1713 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_NeighborhoodInitiative.AddTrackedInitiativeTask

Classified `C_NeighborhoodInitiative.AddTrackedInitiativeTask` as evidence-required/unsafe: the source accepts a numeric initiativeTaskID with no declared return, but the current temporary fallback is a no-op and `trackedIDs` stays empty; focused probes prove only callability and nil return. Task mutation, valid/unknown IDs, duplicate handling, ordering, removal interaction, task info, persistence, event dispatch, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 803 evidence-required, 2 exception-requested, and 1714 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.ShouldUseJourneyRewardTrack

Classified `C_MajorFactions.ShouldUseJourneyRewardTrack` as evidence-required/unsafe: the source requires a boolean by faction ID, but the temporary fallback is constant false with no per-faction journey reward-track model. Focused probes prove only fallback callability/default; positive/negative faction policy, unknown IDs, argument/security restrictions, transitions, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 802 evidence-required, 2 exception-requested, and 1715 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.ShouldDisplayMajorFactionAsJourney

Classified `C_MajorFactions.ShouldDisplayMajorFactionAsJourney` as evidence-required/unsafe: the source requires a boolean by faction ID, but the temporary fallback is constant false; an existing probe proves fallback only. Per-faction policy, positive cases, unknown IDs, argument/security restrictions, transitions, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 801 evidence-required, 2 exception-requested, and 1716 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.RenownHighlightInfo.title

Classified `C_MajorFactions.RenownHighlightInfo.title` as evidence-required/unsafe: the source requires a string, but no `RenownHighlightInfo` model or highlights producer exists and `GetMajorFactionData` omits highlights. Required titles, schema/order, empty/non-empty behavior, per-faction values, unknown behavior, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 800 evidence-required, 2 exception-requested, and 1717 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.RenownHighlightInfo.level

Classified `C_MajorFactions.RenownHighlightInfo.level` as evidence-required/unsafe: the source requires a number, but no `RenownHighlightInfo` model or highlights producer exists; existing `RenownLevelInfo.level` is a different structure. Numeric values, schema/order, empty/non-empty behavior, per-faction values, unknown behavior, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 799 evidence-required, 2 exception-requested, and 1718 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.RenownHighlightInfo.description

Classified `C_MajorFactions.RenownHighlightInfo.description` as evidence-required/unsafe: the source requires a string, but no `RenownHighlightInfo` model or highlights producer exists and `GetMajorFactionData` omits highlights. Required strings, element schema/order, empty/non-empty behavior, per-faction values, unknown behavior, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 798 evidence-required, 2 exception-requested, and 1719 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.MajorFactionRenownRewardInfo.rewardType

Classified `C_MajorFactions.MajorFactionRenownRewardInfo.rewardType` as evidence-required/unsafe: the source requires a nullable number, but no `MajorFactionRenownRewardInfo` model or producer exists and `C_MajorFactions.GetRenownRewardsForLevel` returns an empty table. Present/nil values, numeric meanings, per-level rewards, ordering, unknown behavior, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 797 evidence-required, 2 exception-requested, and 1720 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.MajorFactionData.playerCompanionID

Classified `C_MajorFactions.MajorFactionData.playerCompanionID` as evidence-required/unsafe: the source requires a nullable number, but current `MajorFactionData` has no companion association and `GetMajorFactionData` omits the field. Present/nil behavior, authoritative IDs, per-faction values, unknown behavior, mutation, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 796 evidence-required, 2 exception-requested, and 1721 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.MajorFactionData.highlights

Classified `C_MajorFactions.MajorFactionData.highlights` as evidence-required/unsafe: the source requires an array of `RenownHighlightInfo`, but the current `MajorFactionData` has no highlights state and `GetMajorFactionData` emits no array. Element schema/order, empty/non-empty behavior, per-faction values, mutation, refresh, and lifecycle remain unproven. Current totals are **891 best-effort, 795 evidence-required, 2 exception-requested, and 1722 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_MajorFactions.MajorFactionData.description

Classified `C_MajorFactions.MajorFactionData.description` as evidence-required/unsafe: the source requires a string field, but the current state model lacks description and `GetMajorFactionData` omits it. Exact strings/defaults/per-faction values/unknown behavior/mutation/refresh/lifecycle remain unproven; `name` and `unlockDescription` are separate fields. Current totals are **891 best-effort, 794 evidence-required, 2 exception-requested, and 1723 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LimitedInput.LimitedInputAllowed

Classified `C_LimitedInput.LimitedInputAllowed` as evidence-required/unsafe: the source requires an `Enum.LimitedInputType` input and boolean result, but no allowance, budget, or policy state or namespace method exists; generic fallback returns nil. Authorization/taint rules, budget exhaustion, validation, boolean results, transitions, and lifecycle remain unproven. Current totals are **891 best-effort, 793 evidence-required, 2 exception-requested, and 1724 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LFGList.LfgSearchResultData.generalPlaystyle

Classified `C_LFGList.LfgSearchResultData.generalPlaystyle` as best-effort/behavioral, bounded to seeded `GetSearchResultInfo()` payload field presence and numeric publication. Nullable omission, exact enum validation, invalid values, mutation, serialization, filtering effects, persistence, refresh, and full retail lifecycle remain unclaimed. This search-result field is distinct from `C_LFGList.LfgEntryData.generalPlaystyle` and `C_LFGList.LfgListingCreateData.generalPlaystyle`. Current totals are **891 best-effort, 792 evidence-required, 2 exception-requested, and 1725 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LFGList.LfgListingCreateData.generalPlaystyle

Classified `C_LFGList.LfgListingCreateData.generalPlaystyle` as evidence-required/unsafe: the source requires an `Enum.LFGEntryGeneralPlaystyle` input with default 0, but current `C_LFGList` has no `CreateListing`/`UpdateListing` implementation; generic fallback returns nil and seeded `PremadeListing` output is a different contract. Input parsing/defaults, validation, create/update state, result propagation, errors, and lifecycle remain unproven. Current totals are **890 best-effort, 792 evidence-required, 2 exception-requested, and 1726 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LFGList.LfgEntryData.generalPlaystyle

Classified `C_LFGList.LfgEntryData.generalPlaystyle` as evidence-required/unsafe: the source field is nullable `Enum.LFGEntryGeneralPlaystyle`, but no active-listing model exists and `GetActiveEntryInfo()` always returns nil. The separate search-result `generalPlaystyle` field is a different structure; field presence, nilability, enum values, mutation, serialization, validation, and lifecycle remain unproven. Current totals are **890 best-effort, 791 evidence-required, 2 exception-requested, and 1727 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LFGList generalPlaystyle4 field

Classified `C_LFGList.AdvancedFilterOptions.generalPlaystyle4` as best-effort/behavioral, bounded to modeled boolean publication and exact false default through `C_LFGList.GetAdvancedFilter()`; the focused test proves it. Mutation, serialization, validation, search semantics, persistence, refresh, and broader retail LFG behavior remain unclaimed. Current totals are **890 best-effort, 790 evidence-required, 2 exception-requested, and 1728 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_LFGList generalPlaystyle3 field

Classified `C_LFGList.AdvancedFilterOptions.generalPlaystyle3` as best-effort/behavioral, bounded to modeled boolean publication and exact false default through `C_LFGList.GetAdvancedFilter()`; the focused test proves it. Mutation, serialization, validation, search semantics, persistence, refresh, and broader retail LFG behavior remain unclaimed. Current totals are **889 best-effort, 790 evidence-required, 2 exception-requested, and 1729 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify LFG general playstyle fields

Classified `C_LFGList.AdvancedFilterOptions.generalPlaystyle1` as best-effort/behavioral, bounded to `C_LFGList.GetAdvancedFilter()` publishing a boolean field with exact default false from modeled `LfgAdvancedFilter`; `tests/test_premade_groups.rs::get_advanced_filter_default_is_permissive` proves the default. Mutation, serialization, validation, search semantics, persistence, refresh, and broader retail LFG behavior remain unclaimed. Classified `C_LFGList.AdvancedFilterOptions.generalPlaystyle2` with the same bounded best-effort/behavioral claim: `GetAdvancedFilter()` publishes a boolean field with exact false default from modeled state, proven by the existing focused test; mutation, serialization, validation, search filtering, persistence, refresh, and broader retail LFG semantics remain unclaimed. Current totals are **888 best-effort, 790 evidence-required, 2 exception-requested, and 1730 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify two C_InstanceEncounter rows

Classified `C_InstanceEncounter.IsEncounterInProgress` as best-effort/behavioral: its state-backed boolean query shares `world.encounter_in_progress` with the legacy global, and a focused test proves false by default and true when enabled; encounter producers, events, and lifecycle remain unclaimed. Classified `C_InstanceEncounter.IsEncounterLimitingResurrections` as evidence-required/unsafe because no resurrection-limiting state or explicit method is modeled, and the generic nil fallback does not satisfy the required boolean contract. Current totals are **886 best-effort, 787 evidence-required, 2 exception-requested, and 1735 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_Item account-binding row

Classified `C_Item.IsItemBindToAccount` as evidence-required/unsafe: the target method is absent; related `IsBound` and `IsBoundToAccountUntilEquip` reuse constant false; current item records have numeric bonding metadata but no bonding 7 or 8 positive account-bound fixture; and generic missing-method fallback returns nil. ItemInfo parsing, true/false classification, unknown-item behavior, and binding fidelity remain unproven. Current totals are **886 best-effort, 790 evidence-required, 2 exception-requested, and 1732 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify two additional C_InstanceEncounter rows

Classified `C_InstanceEncounter.IsEncounterSuppressingRelease` and `C_InstanceEncounter.ShouldShowTimelineForEncounter` as evidence-required/unsafe because no explicit methods or backing state exist, and generic nil fallback does not satisfy their required boolean contracts. Current totals are **886 best-effort, 789 evidence-required, 2 exception-requested, and 1733 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify six housing preview-state rows

Classified `C_HousingDecor.EnterPreviewState`, `C_HousingDecor.ExitPreviewState`, `C_HousingDecor.GetNumPreviewDecor`, and `C_HousingDecor.IsPreviewState` as best-effort/behavioral only for focused temporary false/0 → true/1 → false/0 transitions; actual decor cardinality, placement/cleanup, events, persistence, refresh, and retail lifecycle remain unclaimed. Classified `C_HousingCustomizeMode.IsHouseExteriorDoorHovered` as evidence-required/unsafe because the method and door-hover state are absent. Classified `C_HousingDecor.IsModeDisabledForPreviewState` as evidence-required/unsafe because constant false ignores mode and lacks mode-aware state. Current totals are **885 best-effort, 786 evidence-required, 2 exception-requested, and 1737 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify four preview-cart/refund rows

Classified `C_HousingCatalog.IsPreviewCartItemShown`, `C_HousingCatalog.PromotePreviewDecor`, and `C_HousingCatalog.SetPreviewCartItemShown` as best-effort/behavioral only for focused in-memory unknown=false, boolean setter true/false round-trip, promotion returning true, and shown-state mutation. No GUID/decor validation, failure/repeat behavior, events/refresh, persistence, or lifecycle claims are made. Classified `C_HousingCatalog.RequestHousingMarketRefundInfo` as evidence-required/unsafe because its no-op has no refund request/list state, population, repeated-request behavior, or `HOUSING_REFUND_LIST_UPDATED` lifecycle. Current totals are **881 best-effort, 784 evidence-required, 2 exception-requested, and 1743 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify eleven HousingPreviewItemData fields

Classified `HousingPreviewItemData.bundleCatalogShopProductID`, `decorGUID`, `decorID`, `icon`, `id`, `isBundleChild`, `isBundleParent`, `name`, `price`, `productID`, and `salePrice` as evidence-required/unsafe. No typed `HousingPreviewItemData` producer exists; related catalog/decor/bundle fixture fields do not establish these specific fields, nullability, identities, bundle relationships, pricing, validation, event/preview-list production, refresh, persistence, or lifecycle. No approval or exception applies. Current totals are **878 best-effort, 783 evidence-required, 2 exception-requested, and 1747 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three HousingCatalogEntryInfo fields

Classified `HousingCatalogEntryInfo.isUniqueTrophy` and `HousingCatalogEntryInfo.itemID` as best-effort/behavioral only for focused seeded entry 1001 proof of boolean false and numeric 1001; exact trophy classification, nullable/missing item IDs, other entries, authoritative item data, validation, mutation, persistence, refresh, and lifecycle remain unclaimed. Classified `HousingCatalogEntryInfo.dyeIDs` as evidence-required/unsafe because the required numeric-array field is absent and no dye-ID state exists. Current totals are **878 best-effort, 772 evidence-required, 2 exception-requested, and 1758 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify two HousingBundleInfo fields

Classified `HousingBundleInfo.canPreview` as best-effort/behavioral only for focused seeded bundle 5001 proof that the field is boolean true; dynamic preview eligibility, other-bundle behavior, mutation, validation, persistence, refresh, and lifecycle remain unclaimed. Classified `HousingBundleInfo.originalPrice` as evidence-required/unsafe because the current payload publishes nil only and lacks numeric original-price, discount, and currency semantics. Current totals are **876 best-effort, 771 evidence-required, 2 exception-requested, and 1761 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify four housing catalog rows

Classified `GetBundleInfo` as best-effort/behavioral only for seeded bundle 5001 lookup, two entries, and `wasViewed` false→true proof; exact schema, unknown IDs, clone isolation, pricing/product semantics, validation, persistence, refresh, and lifecycle remain unclaimed. Classified `GetCartSizeLimit` as best-effort/behavioral only for callable publication and seeded value 20; dynamic enforcement/configuration, consumers, persistence, and lifecycle remain unclaimed. Classified `GetCatalogEntryRefundTimeStampByRecordID` as evidence-required/unsafe because it ignores arguments and always returns nil without keyed refund state/window semantics. Classified `HasFeaturedEntries` as evidence-required/unsafe because it hardcodes true rather than deriving from featured catalog contents/state transitions. Current totals are **875 best-effort, 770 evidence-required, 2 exception-requested, and 1763 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three housing placement/cart gaps

Classified `C_HousingBasicMode.SetFreePlaceEnabled`, `C_HousingBasicMode.StartPlacingPreviewDecor`, and `C_HousingCatalog.DeletePreviewCartDecor` as evidence-required/unsafe. All are published no-ops without mutable free-placement state, preview placement/decor/bundle state, or observable cart deletion; validation, repeated/unknown calls, requests/events, persistence, reset/isolation, and lifecycle remain unmodeled. Current totals are **873 best-effort, 768 evidence-required, 2 exception-requested, and 1767 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three C_HouseExterior mutation/debug gaps

Classified `C_HouseExterior.GetSelectedFixtureDebugInfo`, `SetHouseExteriorSize`, and `SetHouseExteriorType` as evidence-required/unsafe. The debug API retains only a name without signature/returns and lacks selected-fixture debug state; both setters are published no-ops that do not update getter-visible selected state, validate values, resolve names, persist, refresh, reset/isolate, or model lifecycle. Current totals are **873 best-effort, 765 evidence-required, 2 exception-requested, and 1770 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify two C_HouseExterior rows

The bounded two-row `C_HouseExterior` slice classifies `GetHouseExteriorTypeOptions` as best-effort/behavioral only for focused callable publication, one returned table, `selectedExteriorType` 1, and tested `Cottage`/1 plus `Manor`/2 options; metadata, mutation, persistence, refresh, validation, and lifecycle semantics remain unclaimed. `GetHoveredFixtureDebugInfo` is evidence-required/unsafe because only its API name is retained, its signature and returns are unknown, and the nil fallback has no hovered-fixture debug state. Current totals are **873 best-effort, 762 evidence-required, 2 exception-requested, and 1773 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three Delves/housing rows

The bounded three-row 12.0.0 Delves/housing slice classifies `C_HouseExterior.GetHouseExteriorSizeOptions` as best-effort/behavioral only for focused proof of callable publication, one table return, `selectedSize` 3, and exactly Medium/3 plus Large/4 options; exact metadata, enum fidelity, mutation, persistence, refresh, validation, and lifecycle remain unclaimed. `C_DelvesUI.IsTraitTreeForCompanion` is evidence-required/unsafe because it is absent without trait-tree ownership/classification state. `C_Housing.OnHouseFinderClickPlot` is evidence-required/unsafe because it is absent without selected-plot request/event/state side effects or validation.

## [2026-08-09] investigation | Classify four C_DelvesUI and housing unsafe rows

Classified `C_DelvesUI.GetLockedTextForCompanion`, `C_HouseExterior.GetFixtureDebugInfoForGUID`, `C_Housing.IsHousingMarketShopEnabled`, and `C_HousingBasicMode.IsFreePlaceEnabled` as evidence-required/unsafe. `GetLockedTextForCompanion` is absent and lacks companion lock-state/text behavior; `GetFixtureDebugInfoForGUID` is absent, lacks GUID-indexed fixture-debug state, and the checked-in source preserves only its API name without signature/return semantics; `IsHousingMarketShopEnabled` is absent without dedicated boolean state; and `IsFreePlaceEnabled` hardcodes true while `SetFreePlaceEnabled` is a no-op, so mutable state and exact semantics are unmodeled. None has approval or an exception. Current totals are **872 best-effort, 761 evidence-required, 2 exception-requested, and 1775 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify four C_EventUtils, C_HouseExterior, C_CreatureInfo, and C_GameRules rows

Classified `C_EventUtils.IsCallbackEvent` as best-effort/behavioral from focused positive `COMBAT_LOG_EVENT` and negative `PLAYER_LOGIN` proof; exact registry completeness, argument validation, dynamic behavior, and lifecycle remain unclaimed. Classified `C_HouseExterior.GetCurrentHouseExteriorType` as best-effort/behavioral only for callable publication and the seeded two-return shape/types/values `1` and `Sunspire Cottage`; retail selection, mutation, persistence, refresh, and lifecycle remain unclaimed. Classified `C_CreatureInfo.GetCreatureID` as evidence-required/unsafe because it is absent and lacks a GUID/creature identity model. Classified `C_GameRules.IsPersonalResourceDisplayEnabled` as evidence-required/unsafe because the current fallback returns nil rather than the required boolean and has no backing state. Current totals are **871 best-effort, 755 evidence-required, 2 exception-requested, and 1782 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify eight C_EventScheduler rows

Classified `C_EventScheduler.CanShowEvents` as best-effort/behavioral only for simulator visibility derivation from explicit override, suppression, event-list state, and request repopulation; retail availability, refresh timing, persistence, full lifecycle, and edge semantics remain unclaimed. Classified `EventDisplayInfo.hideDescription`, `EventDisplayInfo.hideTimeLeft`, `EventDisplayInfo.overrideAtlas`, `EventDisplayInfo.overrideTooltipWidgetSetID`, `OngoingEventInfo.displayInfo`, `ScheduledEventInfo.displayInfo`, and `ScheduledEventInfo.eventID` as evidence-required/unsafe because temporary empty/seeded compatibility payloads do not establish documented typed field values, shape, producers, or lifecycle. Current totals are **869 best-effort, 753 evidence-required, 2 exception-requested, and 1786 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three C_CooldownViewer gaps

Classified `C_CooldownViewer.CooldownViewerCooldown.category`, `C_CooldownViewer.CooldownViewerCooldown.cooldownID`, and `C_CooldownViewer.GetValidAlertTypes` as `evidence-required`/`unsafe`. The current temporary surface returns nil/empty defaults and has no typed cooldown producer, category/ID records, ordered alert-type arrays, validation, routing, Settings UI behavior, or lifecycle. Current totals are **868 best-effort, 746 evidence-required, 2 exception-requested, and 1794 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify seven C_ChatInfo, C_CombatText, and C_Commentator API gaps

Classified the seven-row `C_ChatInfo`/`C_CombatText`/`C_Commentator` API-gap slice as `evidence-required`/`unsafe`. Current fallbacks are absent, no-op, constant-false, or adjacent-state only and do not model lockdown, emote, active-unit, combat-text, or commentator event state, restrictions, result contracts, transitions, events, ordering, or lifecycle. Current totals are **868 best-effort, 743 evidence-required, 2 exception-requested, and 1797 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_CharacterServices.AssignFCMDistribution

Classified `C_CharacterServices.AssignFCMDistribution` as `evidence-required`/`unsafe`. The source register provides no signature/result metadata, and the simulator has no FCM validation/assignment model; account/realm/character checks, validation-only behavior, exact results, state transitions, persistence, and events remain unproven. Current totals are **868 best-effort, 736 evidence-required, 2 exception-requested, and 1804 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify 23 C_CatalogShop API and structure rows

Classified `C_CatalogShop.HasNewProducts` as `best-effort`/`behavioral` only for publication and the exact constant-false boolean result proven by `test_startup_service_namespaces_exist`. Classified the other 22 rows as `evidence-required`/`unsafe` because current CatalogShop state is absent, no-op, incomplete, seeded, wrong-typed, or untested and does not establish purchase, category/product, refundable-decor, currency, session, refresh, restriction, payload, event, or lifecycle semantics. Current totals are **868 best-effort, 735 evidence-required, 2 exception-requested, and 1805 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three C_BattleNet mutation and transport APIs

Classified `C_BattleNet.SendGameData`, `C_BattleNet.SendWhisper`, and `C_BattleNet.SetCustomMessage` as `evidence-required`/`unsafe`. No current registration, backing transport/state, restriction enforcement, result model, or focused side-effect proof exists; exact account/text validation, return values, persistence, events, and consumer behavior remain unproven. Current totals are **867 best-effort, 713 evidence-required, 2 exception-requested, and 1828 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify C_AdventureMap quest portrait info

Classified the six-row `C_AdventureMap.GetQuestPortraitInfo` slice—the API plus five portrait fields—as bounded `best-effort`/`behavioral` claims from focused `tests/c_adventure_map/quests.rs` proof. Claims cover injected-state lookup, typed five-field publication, unknown/nonnumeric zero-value returns, nullable `modelSceneID`, and tested display-ID gating. Retail data population, localization, full validation/edge behavior, assets/rendering, and lifecycle remain unclaimed. `modelSceneID` is treated strictly as data under the existing permanent no-3D scope; no 3D implementation or new exception is requested. Current totals are **867 best-effort, 710 evidence-required, 2 exception-requested, and 1831 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify nine C_ActionBar charge/cooldown structure fields

Classified the nine `C_ActionBar.ActionBarChargeInfo`/`ActionBarCooldownInfo` structure fields as `evidence-required`/`unsafe`. `GetActionCharges` returns static placeholder charge fields; `GetActionCooldown` returns a partial table with fixture-backed start/duration and constant enabled/mod-rate. Neither establishes authoritative typed payload fidelity, slot-dependent charges, field relationships, invalid-slot behavior, progression, secrets, or lifecycle. Current totals are **861 best-effort, 710 evidence-required, 2 exception-requested, and 1837 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify ten CatalogShop, encounter-chat, combat-log, and commentator events

Classified the five combat-log producer/event occurrences `COMBAT_LOG_APPLY_FILTER_SETTINGS`, `COMBAT_LOG_ENTRIES_CLEARED`, `COMBAT_LOG_MESSAGE`, `COMBAT_LOG_MESSAGE_LIMIT_CHANGED`, and `COMBAT_LOG_REFILTER_ENTRIES`, plus `C_CombatLog.ApplyFilterSettings`, `C_CombatLog.ClearEntries`, `C_CombatLog.RefilterEntries`, `C_CombatLog.SetMessageLimit`, and `C_CombatLogSecure.CreateCombatLogMessage`, as `best-effort`/`behavioral` under proof commit `6bcd6ded2ada37c5f78ba4b98387549f2430369c`. Focused proof is limited to exact synchronous notification, payload arity/types/values, corresponding temporary-state mutation where asserted, and repeated explicit calls; filtering, retention, message-limit enforcement, secure/taint, UI, persistence, and broader lifecycle semantics remain unclaimed. The remaining CatalogShop, encounter-chat, and commentator rows remain `evidence-required`/`unsafe`. Current totals are **2218 best-effort, 1190 evidence-required, 2 exception-requested, and 0 untriaged rows** (3410 rows).

## [2026-08-09] investigation | Classify six additional CAA CVar-default rows

Classified six additional target-health, voice, and volume CAA CVar rows as bounded `best-effort`/`behavioral` claims using `test_patch_12_0_0_cvar_defaults`. Consolidated with the prior 27-row slice, the cumulative CAA CVar-default slice covers 33 rows. The focused test proves only startup `GetCVar`/`GetCVarDefault` exact string defaults; CAA behavior, UI/audio effects, mutation, persistence, events, flags, consumers, and later-epoch semantics remain unclaimed. Current totals are **861 best-effort, 691 evidence-required, 2 exception-requested, and 1856 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify 20 additional CAA CVar-default rows

Classified 20 additional player-, resource-, speech-, and target-cast CAA CVar rows as bounded `best-effort`/`behavioral` claims using `test_patch_12_0_0_cvar_defaults`. Consolidated with the prior seven-row slice, the cumulative CAA CVar-default slice covers 27 rows. The focused test proves only startup `GetCVar`/`GetCVarDefault` exact string defaults; CAA behavior, UI/audio effects, mutation, persistence, events, flags, consumers, and later-epoch semantics remain unclaimed. Current totals are **855 best-effort, 691 evidence-required, 2 exception-requested, and 1862 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify two 12.0.0 event rows

Classified `ADDON_RESTRICTION_STATE_CHANGED` and `BULK_PURCHASE_RESULT_RECEIVED` as `evidence-required`/`unsafe`. Registration and enum publication exist, but no modeled transition/purchase producer or focused proof establishes exact payload values/structures/arity, synchronous timing, ordering, duplicate behavior, lifecycle, or consumers. Current totals are **835 best-effort, 691 evidence-required, 2 exception-requested, and 1882 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify three global-utility rows

Classified `AbbreviateLargeNumbers` and `AbbreviateNumbers` as `evidence-required`/`unsafe` because temporary fallbacks ignore `NumberAbbrevOptions` and do not model abbreviation, localization, or validation. Classified `AddSourceLocationExclude` as bounded `best-effort`/`behavioral` for nil-guarded global publication and successful string-argument no-op invocation through `installs_debug_environment_defaults`; exclusion, filtering, and debug semantics remain unclaimed. Current totals are **835 best-effort, 689 evidence-required, 2 exception-requested, and 1884 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify seven CAA CVar-default rows

Classified `CAAEnabled`, `CAAInterruptCast`, `CAAInterruptCastSuccess`, `CAAPartyHealthFrequency`, `CAAPartyHealthPercent`, `CAAPlayerCastFormat`, and `CAAPlayerCastMinTime` as bounded `best-effort`/`behavioral` claims using `test_patch_12_0_0_cvar_defaults`. The focused test proves only startup `GetCVar`/`GetCVarDefault` exact string defaults; CAA behavior, UI/audio effects, mutation, persistence, events, flags, consumers, and later-epoch semantics remain unclaimed. Current totals are **834 best-effort, 687 evidence-required, 2 exception-requested, and 1887 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify 23 CVar-default rows

Classified 23 added CVar-default rows as bounded `best-effort`/`behavioral` claims using `test_patch_12_0_0_cvar_defaults`. The focused test proves only startup `GetCVar`/`GetCVarDefault` exact string defaults; mutation, events, persistence, secure/read-only flags, consumers, and later-epoch behavior remain unclaimed. Current totals are **827 best-effort, 687 evidence-required, 2 exception-requested, and 1894 untriaged** (3410 rows).

## [2026-08-09] investigation | Classify four unit/heal-prediction rows

Classified `UnitCreatureID`, `UnitIsHumanPlayer`, `UnitIsSpellTarget`, and `UnitHealPredictionValues.totalDamageAbsorbs` as bounded `best-effort`/`behavioral` claims. Existing focused tests cover the token/GUID/vendor-shim cases; `unit_detailed_heal_prediction_populates_calculator` now asserts the numeric zero absorb field. Full retail semantics, invalid inputs, lifecycle, and untested states remain unclaimed. Current totals are **804 best-effort, 687 evidence-required, 2 exception-requested, and 1917 untriaged**.

## [2026-08-09] investigation | Classify miscellaneous payload fields

Classified eight ExpansionDisplayInfo, LuaColorCurvePoint, PrivateAuraIconInfo, and SpellCooldownInfo structure fields as `evidence-required`/`unsafe`. Current behavior is absent, nil-only, generic, or placeholder-backed and does not establish exact contracts, state/security, or consumer semantics; tests remain empty with null commit, approval, and scope exception. Current totals are **800 best-effort, 687 evidence-required, 2 exception-requested, and 1921 untriaged**.

## [2026-08-09] investigation | Classify number-abbreviation fields

Classified eight `NumberAbbrevData`/`NumberAbbrevOptions` structure fields as `evidence-required`/`unsafe`. Generic `AbbreviateConfig` proxy round-tripping does not establish typed contracts, defaults/nullability, validation, ordering, or formatting behavior; tests remain empty with null commit, approval, and scope exception. Current totals are **800 best-effort, 679 evidence-required, 2 exception-requested, and 1929 untriaged**.

## [2026-08-08] investigation | Classify conflicting 12.0.0 error globals

Classified 12 added `LE_GAME_ERR_*` globals as `evidence-required`/`unsafe` because checked-in 12.0.0 source-register values conflict with current nil-guarded fallback publication. Authoritative epoch/value reconciliation is required before changing runtime behavior; tests remain empty with null commit, approval, and scope exception. Current totals are **800 best-effort, 671 evidence-required, 2 exception-requested, and 1937 untriaged**.

## [2026-08-09] investigation | Classify 12.0.0 tutorial and pet constants

Classified five tutorial/pet globals as bounded `best-effort`/`behavioral` startup claims. `test_patch_12_0_0_ui_global_constant_values` proves numeric Lua publication and exact source-register values; tutorial and pet-journal behavior, consumers, lifecycle, and historical load timing remain unclaimed. Current totals are **800 best-effort, 659 evidence-required, 2 exception-requested, and 1949 untriaged**.

## [2026-08-09] investigation | Classify 12.0.0 housing payload fields

Classified 16 exterior size/type, house-level, and decor-refund structure fields as `evidence-required`/`unsafe`. Current temporary housing data is absent or fixture-backed and does not establish exact contracts, authoritative values, state transitions, ordering/localization, or consumer behavior; tests remain empty with null commit, approval, and scope exception. Current totals are **795 best-effort, 659 evidence-required, 2 exception-requested, and 1954 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 general events

Classified nine faction, initiative, loot-rule, nameplate, and neighborhood events as `evidence-required`/`unsafe`. Retail registration exists, but no modeled producer or focused proof establishes each source payload contract, timing, lifecycle, ordering, or duplicate behavior; tests remain empty with null commit, approval, and scope exception. Current totals are **795 best-effort, 643 evidence-required, 2 exception-requested, and 1970 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 housing events

Classified 12 added housing events as `evidence-required`/`unsafe`. The retail event registry accepts each name, but no modeled producer or focused proof establishes its source payload contract, timing, lifecycle, ordering, or duplicate behavior; tests remain empty with null commit, approval, and scope exception. Current totals are **795 best-effort, 634 evidence-required, 2 exception-requested, and 1979 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 VAS result enum

Classified `Enum.VasTransactionPurchaseResult.DbHouseOwnerRestriction=20096` as a bounded `best-effort`/`behavioral` startup claim. `test_patch_12_0_0_vas_transaction_purchase_result_value` proves namespace publication, numeric Lua type, and the exact source-register value; VAS transaction behavior, validation, consumers, lifecycle, and historical load timing remain unclaimed. Current totals are **795 best-effort, 622 evidence-required, 2 exception-requested, and 1991 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 aura sort enums

Classified nine `UnitAuraSortRule` member/metadata rows as bounded `best-effort`/`behavioral` startup claims. `test_patch_12_0_0_unit_aura_sort_rule_enum_values` proves namespace/metadata publication, numeric Lua types, and all exact source-register values; aura ordering, filtering, consumers, lifecycle, validation, and historical load timing remain unclaimed. Current totals are **794 best-effort, 622 evidence-required, 2 exception-requested, and 1992 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 heal-prediction enums

Classified 21 `UnitDamageAbsorbClampMode`, `UnitHealAbsorbClampMode`, `UnitHealAbsorbMode`, and `UnitIncomingHealClampMode` member/metadata rows as bounded `best-effort`/`behavioral` startup claims. `test_patch_12_0_0_heal_prediction_enum_values` proves namespace/metadata publication, numeric Lua types, and all exact source-register values; heal prediction, absorb/clamp calculations, incoming-heal state, UI behavior, lifecycle, and consumer semantics remain unclaimed. Current totals are **785 best-effort, 622 evidence-required, 2 exception-requested, and 2001 untriaged**.

## [2026-08-08] investigation | Fix and classify UI enum metadata epochs

Added exact 12.0.0 startup coverage for 15 metadata rows across `CooldownViewerAddAlertStatusMeta`, `CooldownViewerAlertEventTypeMeta`, `DamageMeterStyleMeta`, `EditModeEncounterEventsSystemIndicesMeta`, and `HouseExteriorWMODataFlagsMeta`. Epoch fix `890ae4afd2b2f5a6c368c8601e332044937135d5` removes later `OnAuraApplied`/`OnAuraRemoved` members and restores `CooldownViewerAlertEventTypeMeta` to `MaxValue=4`, `MinValue=1`, `NumValues=4`. Updated the four existing CooldownViewerAlertEventType member rows to the corrected implementation and absence/metadata proof. Current totals are **764 best-effort, 622 evidence-required, 2 exception-requested, and 2022 untriaged**.

## [2026-08-08] investigation | Classify cooldown, housing, and Edit Mode enums

Classified 20 added `CooldownViewerAddAlertStatus`, `CooldownViewerAlertEventType`, `DamageMeterStyle`, `EditModeEncounterEventsSystemIndices`, and `HouseExteriorWMODataFlags` member rows as bounded `best-effort`/`behavioral` startup publication/value claims. A new focused 12.0.0 test covers the 12 cooldown/housing rows; the existing Edit Mode profile-option enum test covers the eight DamageMeter/Edit Mode rows. At that classification point the current CooldownViewerAlertEventType table still contained later members; epoch fix `890ae4afd2b2f5a6c368c8601e332044937135d5` subsequently removed them for 12.0.0 and added exact four-member membership proof. Consumer semantics remain unclaimed. Current totals are **749 best-effort, 622 evidence-required, 2 exception-requested, and 2037 untriaged**. Moved two existing classified-slice narratives before the inventory source/footer section.

## [2026-08-08] investigation | Classify 12.0.0 replacement enum keys

Added focused 12.0.0 startup coverage for five same-value enum replacements: `GossipNpcOption.TieredEntrance=66`, `ItemRecraftFlags.Invalid=1`, `PerksVendorCategoryType.RefundUnused=24`, `PlayerInteractionType.TieredEntrance=79`, and `QuestTagType.Prey=19`. Classified those five current names as bounded `best-effort`/`behavioral` publication/type/value claims. Their five paired old names remain `evidence-required`/`unsafe`; current bootstrap omission does not prove full-LoD dynamic absence, historical removal timing, alias compatibility, or semantic replacement identity. Current totals are **729 best-effort, 622 evidence-required, 2 exception-requested, and 2057 untriaged**.

## [2026-08-08] investigation | Extend 12.0.0 enum metadata coverage

Classified 10 account-state, HousingResult, TransmogSituation, and SecretAspect metadata rows as bounded `best-effort`/`behavioral` claims. Epoch override `31172606b3f3b8b61bea63a81c457219546789fa` removes the two 12.0.1 SecretAspect members from 12.0.0 and restores exact metadata while retaining later values. Focused tests now cover 98 account-state rows, 90 HousingResult rows, 25 TransmogSituation rows, and 43 ItemCollectionType/SecretAspect rows. Normalized 33 provenance-only structure inventory rows to the declared five-column table. Current totals are **724 best-effort, 617 evidence-required, 2 exception-requested, and 2067 untriaged**.

## [2026-08-08] investigation | Fix EncounterEventFlags epoch drift

Corrected the 12.0.0 runtime to publish the one-value `Enum.EncounterEventFlags` table while retaining the later two-value table from 12.0.5 onward. Focused tests prove `Disabled=1`, exact metadata, absence of later `IgnoreCastConsume` under 12.0.0, and preservation of later values. Classified those four rows plus 12 TooltipDataType, TraitNodeFlag, UICursorType, and UIWidgetVisualizationType rows as bounded `best-effort`/`behavioral` namespace/type/value claims. Current totals are **714 best-effort, 617 evidence-required, 2 exception-requested, and 2077 untriaged**.

## [2026-08-08] investigation | Extend small 12.0.0 enum coverage

Classified 15 added/changed CraftingReagentItemFlag, EditModeAuraFrameSystemIndices, HousingItemToastType, MapIconUIWidgetSetType, and SurveyDeliveryMoment rows as `best-effort`/`behavioral` by extending `src/loader/tests/wow_api_globals/patch_12_0_0_small_enums.rs::test_patch_12_0_0_small_enum_values`. The test now covers 35 rows across 11 families with exact namespace, numeric-type, and source-register-value assertions. Consumer, server/state, bitwise composition, mutation/protection, lifecycle, and edge semantics remain unclaimed. Current totals are **698 best-effort, 617 evidence-required, 2 exception-requested, and 2093 untriaged**.

## [2026-08-08] investigation | Classify six small 12.0.0 enum families

Classified 20 added/changed AccountTransType, CurrencyDestroyReason, CurrencySource, EditModeCooldownViewerSetting, GameRule, and HousingDecorActionFlags rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/patch_12_0_0_small_enums.rs::test_patch_12_0_0_small_enum_values`. The focused retail 12.0.0 startup test asserts namespace publication, exact numeric Lua types, and exact source-register values. Claims exclude consumer, server/state, bitwise composition, mutation/protection, lifecycle, and edge semantics. Current totals are **683 best-effort, 617 evidence-required, 2 exception-requested, and 2108 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 CombatLogObject enums

Classified 35 added `Enum.CombatLogObject`, `Enum.CombatLogObjectMeta`, `Enum.CombatLogObjectTarget`, and `Enum.CombatLogObjectTargetMeta` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/combat_log_object.rs::test_patch_12_0_0_combat_log_object_enum_values`. The focused retail 12.0.0 startup test asserts all four namespace tables, exact numeric Lua types, and exact source-register values, including positive high-bit `None` and `RaidNone`. Claims exclude combat-log bitmask operations, filter matching, consumers, signed host representations, mutation/protection, lifecycle, and edge semantics. Implementation ancestor is `b14f2a854ba`; current source/test hashes are recorded per row. Totals are now **663 best-effort, 617 evidence-required, 2 exception-requested, and 2128 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 TransmogSituation enum

Classified all 22 added `Enum.TransmogSituation.*` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/transmog_situation.rs::test_patch_12_0_0_transmog_situation_enum_values`. The focused retail 12.0.0 startup test asserts namespace publication, exact numeric Lua type, and exact source-register value for every entry; transmog behavior, flags/combinations, consumers, validation, persistence, mutation/protection, and lifecycle remain unclaimed. Implementation ancestor is `339424faf1`; current source/test hashes are recorded per row. Totals are now **628 best-effort, 617 evidence-required, 2 exception-requested, and 2163 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 transmog outfit enums

Classified 52 added `Enum.TransmogOutfitSlot`, `Enum.TransmogOutfitSlotError`, `Enum.TransmogOutfitSlotOption`, and `Enum.TransmogOutfitTransactionFlags` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/transmog_outfit_enums.rs::test_patch_12_0_0_transmog_outfit_enum_values`. The focused retail 12.0.0 test asserts startup namespace publication, exact numeric Lua type, and exact source-register value for all 52 entries. Claims exclude transmog operations, validation, slot compatibility, transaction semantics, persistence, mutation/protection, lifecycle, and edge semantics. Totals are now **606 best-effort, 617 evidence-required, 2 exception-requested, and 2185 untriaged**.

## [2026-08-08] investigation | Classify ItemCollectionType and SecretAspect enums

Classified 40 added/changed `Enum.ItemCollectionType`, `Enum.ItemCollectionTypeMeta`, and `Enum.SecretAspect` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/item_collection_secret_aspects.rs::test_patch_12_0_0_item_collection_and_secret_aspect_values`; claims are limited to startup namespace publication, numeric type, and exact value. Classified 13 removed ItemCollectionType aliases/meta rows as `evidence-required`/`unsafe`; bootstrap omission does not prove full runtime/dynamic publication absence, historical timing, replacement semantics, or all-LoD removal. Totals are now **554 best-effort, 617 evidence-required, 2 exception-requested, and 2237 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 HousingResult enum

Classified 88 added/changed `Enum.HousingResult.*` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/housing_result.rs::test_patch_12_0_0_housing_result_values`; the focused retail 12.0.0 test asserts namespace publication, exact numeric Lua type, and exact source-register value. Claims exclude housing operations, result/error text mapping, consumers, persistence, mutation/protection, and lifecycle. Classified removed `FixtureNotOwned` and `MissingTheme` as `evidence-required`/`unsafe`; bootstrap omission does not prove full runtime/dynamic publication absence, historical timing, replacement semantics, or all-LoD absence. Totals are now **514 best-effort, 604 evidence-required, 2 exception-requested, and 2290 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 account-state enum families

Classified 96 added `Enum.AccountStateLoadedFlags.*`/`Enum.CreateAllAccountData.*` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/account_state_flags.rs::test_patch_12_0_0_account_state_enum_values`. The focused retail 12.0.0 startup test asserts both namespaces, exact Lua string type, and exact source-register value for all 96 entries; bitflag combinations, consumers, mutation/protection, persistence, aliases, and lifecycle remain unclaimed. Classified 94 removed legacy alias rows as `evidence-required`/`unsafe`; bootstrap omission does not prove full runtime/dynamic publication absence, historical timing, replacement semantics, or all-LoD absence. Totals are now **426 best-effort, 602 evidence-required, 2 exception-requested, and 2380 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 HousingCatalog constant removals

Classified the 16 removed `Constants.HousingCatalogConsts.*` rows as `evidence-required`/`unsafe` using checked-in 12.0.0 source-register evidence plus current `src/lua_api/globals/enum_data/constants_values.lua` bootstrap evidence. The simulator bootstrap omits the keys while retaining the namespace, but source/bootstrap absence is insufficient to prove full runtime or dynamic publication, historical load-order timing, replacement semantics, or exact 12.0.0 removal. Tests/assertions remain empty with null commit, approval, scope exception, load_addon, and provenance_only. Totals are now **330 best-effort, 508 evidence-required, 2 exception-requested, and 2570 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 CAA constants

Classified exactly 32 added `Constants.CAAConstants` rows as `best-effort`/`behavioral` using `src/loader/tests/wow_api_globals/caa_constants.rs::test_patch_12_0_0_caa_constants_publish_exact_values`. The focused startup test asserts namespace publication, exact Lua type, and exact value for all 32 rows. Source implementation ancestor is `cf0908682a897f314da15dc3ae4f9c12c03cf6f0`; current source/test hashes are recorded per row. Claims exclude CVar linkage, UI behavior, localization, mutation/protection, and consumer semantics. Totals are now **330 best-effort, 492 evidence-required, 2 exception-requested, and 2586 untriaged**.

## [2026-08-08] investigation | Classify remaining structure declarations

Classified the nine remaining 12.0.0 structure declarations as `evidence-required`/`unsafe`: three added typed-structure rows (`LuaColorCurvePoint`, `NumberAbbrevData`, `NumberAbbrevOptions`) and six removed/removal-sensitive rows. Related proxy/curve tests do not prove exact added typed fields or payloads; source metadata, method absence, and auxiliary token checks do not prove removed structure/replacement/mixin identity. Tests/assertions remain empty with null commit, approval, and scope exception. Totals are now **298 best-effort, 492 evidence-required, 2 exception-requested, and 2618 untriaged**.

## [2026-08-08] investigation | Migrate remaining pure structure declarations

Migrated exactly 23 added structure declarations to best-effort/provenance-only. Their field/API rows remain separate; no runtime behavior is claimed. Nine structure declarations remain untriaged because they are behavior-linked or removal-sensitive. Totals are now **298 best-effort, 483 evidence-required, 2 exception-requested, and 2627 untriaged**.

## [2026-08-08] Migrate 12.0.0 structure declaration provenance rows

Migrated ten structure declarations to the machine-validated `best-effort`/`provenance-only` contract: the six `C_DamageMeter` declarations, `C_ActionBar.ActionBarChargeInfo`, `C_ActionBar.ActionBarCooldownInfo`, `C_CatalogShop.BulkPurchaseIndividualProductResult`, and `C_CatalogShop.RefundableDecorInfo`. Their field/API rows remain untriaged; no payload or runtime behavior is claimed. Totals are now **275 best-effort, 483 evidence-required, 2 exception-requested, and 2650 untriaged**.

## [2026-08-08] investigation | Migrate 12.0.0 typedef provenance rows

Migrated all 21 untriaged `typedef.*` rows to the machine-validated `best-effort`/`provenance-only` contract. Each retains owner/category and source-register evidence only, sets `provenance_only: true`, has empty runtime proof fields, and uses exact notes `Provenance-only: no runtime behavior claimed.` This bookkeeping status claims no simulator-visible runtime behavior and is completion-eligible without a commit. Totals are now **265 best-effort, 483 evidence-required, 2 exception-requested, and 2660 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 UnitHealPredictionCalculator luaobject methods

Classified `added:UnitHealPredictionCalculator.GetDamageAbsorbClampMode` as `best-effort`/`behavioral` using `tests/userdata_proxy.rs::heal_prediction_set_get_roundtrip` and implementation ancestor `4d7dbe1da`; the claim is limited to the tested local setter/getter round-trip. Classified the other 13 currently untriaged UnitHealPredictionCalculator luaobject-method rows as `evidence-required`/`unsafe` with empty tests/assertions and null commit/approval/scope exception because the generic proxy does not establish exact absorb/mode/default/reset/predicted-payload, secret, lifecycle, validation, or edge semantics. All 68 luaobject-method rows are now non-untriaged. Totals are now **244 best-effort, 483 evidence-required, 2 exception-requested, and 2681 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 script-object rows

Classified five added script-object rows as `best-effort`/`behavioral` using existing direct proxy/state tests: `script_object.AbbreviateConfigAPI`, `script_object.LuaColorCurveObjectAPI`, `script_object.LuaCurveObjectAPI`, `script_object.LuaDurationObjectAPI`, and `script_object.UnitHealPredictionCalculatorAPI`. Claims are limited to tested table/method shape, state round-trips, scalar evaluation/copy behavior, per-instance fields/tostring, duration identity, and fixture prediction state as recorded per row in `data/patch-api/12.0.0.json`; retail abbreviation/curve/timing/clock/secret/heal-prediction lifecycle and edge fidelity remain unproven. Classified `script_object.FrameAPITooltip` and `script_object.LuaCurveObjectBaseAPI` as `evidence-required`/`unsafe` because construction/base contracts remain unmodeled or incompletely tested; generated documentation and indirect concrete-object tests are not runtime proof. All seven script-object rows are now non-untriaged. Totals are now **243 best-effort, 470 evidence-required, 2 exception-requested, and 2695 untriaged**.

## [2026-08-08] investigation | Classify remaining 12.0.0 UI-method rows

Classified `changed:StatusBar.SetMinMaxValues` as `best-effort`/`behavioral` using `tests/widget_methods_colorselect.rs::test_statusbar_set_min_max_values_clamps_existing_value`. The test stores values before narrowing/shifting ranges and verifies `(10, 50)` clamps 80 to 50 and `(30, 40)` clamps 20 to 30; the claim excludes interpolation-target behavior, rendering/events, invalid/reversed ranges, and edge semantics. Classified `changed:StatusBar.GetFillStyle`, `changed:StatusBar.SetFillStyle`, and `added:TextureBase.SetSpriteSheetCell` as `evidence-required`/`unsafe`: the fill-style getter returns constant STANDARD and breaks nonstandard round-trips, while SetSpriteSheetCell is a no-op. Exact styles, validation, sprite-cell mapping, optional dimensions, rendering, and edge semantics remain unproven. Totals are now **238 best-effort, 468 evidence-required, 2 exception-requested, and 2702 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 secret/no-3D UI methods

Classified `added:Model.SetUseGBuffer` as `exception-requested`/`impossible` under the already-decided permanent no-3D project scope; this is not a user approval request and no model-callability behavior is claimed. Classified `added:Region.IsAnchoringSecret`, `added:UIObject.HasAnySecretAspect`, `added:UIObject.HasSecretAspect`, `added:UIObject.HasSecretValues`, `added:UIObject.IsPreventingSecretValues`, and `added:UIObject.SetPreventSecretValues` as `evidence-required`/`unsafe`. Current local flag/aspect behavior and frame-state tests establish simulator consistency only, not authoritative secret/taint semantics, aspect mapping/aggregation, propagation, authorization, anchoring relationships, or lifecycle; tests remain empty with null commit, approval, and scope exception. Totals are now **237 best-effort, 465 evidence-required, 2 exception-requested, and 2706 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 UI-method state batch

Classified `added:Frame.IsIgnoringChildrenForBounds`, `added:Frame.SetIgnoringChildrenForBounds`, and `added:Region.SetAlphaFromBoolean` as `best-effort`/`behavioral` using the focused frame-state and alpha tests. Claims are limited to false/true/false stored-state mutation, true/false alpha branch selection, and same-value no-op dirty behavior; actual bounds/layout effects, full alpha defaults/clamping/propagation, rendering, invalid arguments, lifecycle, and edge semantics remain unproven. Classified `added:Cooldown.SetPaused`, `added:FontString.GetScaleAnimationMode`, `added:FontString.SetScaleAnimationMode`, and `added:LayeredRegion.SetVertexColorFromBoolean` as `evidence-required`/`unsafe` because no matching simulator implementation/test was found; exact contracts require authoritative evidence or a correct model/test, with empty tests/assertions and null commit/approval/scope exception. The checked-in manifest records both FontString rows as `added` occurrences. Totals are now **237 best-effort, 459 evidence-required, 1 exception-requested, and 2713 untriaged**.

## [2026-08-08] investigation | Correct ResetTexCoord audit evidence

Updated `added:TextureBase.ResetTexCoord` after commit `090af1ec8` corrected `tests/texture_methods_port.rs::test_reset_tex_coord_restores_defaults` to assert WoW's eight-corner `GetTexCoord` result `(0,0, 0,1, 1,0, 1,1)` instead of the obsolete four-value contract. Refreshed the test hash and manifest wording; best-effort scope remains limited to SetTexCoord-then-ResetTexCoord default reset, with atlas-specific reset, rendering, invalid arguments, and edge semantics unproven.

## [2026-08-08] investigation | Classify tested 12.0.0 UI method batch

Classified six rows as `best-effort`/`behavioral` using existing direct tests: actual register IDs are `added:GameTooltip.GetLeftLine`, `added:GameTooltip.GetRightLine`, `added:TextureBase.ResetTexCoord`, `changed:StatusBar.SetValue`, `added:Frame.RegisterEventCallback`, and `added:Frame.RegisterUnitEventCallback`. The manifest records source/test SHA-256 hashes plus exact implementation ancestors `4ef55ce2cf0737da018279923edd49f075eda820`, `2591694d10dca666a09593cc5ffff121193171f9`, and `7a7a440402f4b03a89fe341956b6cb5e2051f465`; test ancestors are `fcc633ce286002f95b758635427fa1ba7720838a`, `c473e49cdaac6fbb882c43e4bb2a24456debe5e5`, `0c2c6a966d0d5351d42b8d227d1ee35747e92fde`, `7819ea6620139616fc208fc9d0b30d307adf636c`, and `118a2c9a3b56c5d700f0de0994edfac0df082e7f`. Claims are limited to tested tooltip line lookup, default texture-coordinate reset, StatusBar immediate/Smooth value state, and event/unit-callback dispatch. `added:TextureBase.SetSpriteSheetCell`, `changed:StatusBar.GetFillStyle`, `changed:StatusBar.SetFillStyle`, and `changed:StatusBar.SetMinMaxValues` remain untriaged because existing tests do not assert their contracts. Rendering/layout/animation, invalid/edge inputs, lifecycle/validation, and taint/security semantics remain unproven. Totals are now **234 best-effort, 455 evidence-required, 1 exception-requested, and 2720 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 GameTooltip.SetText

Classified `GameTooltip.SetText` as `best-effort`/`behavioral` using `tests/tooltip_basic.rs::test_settext_clears_and_sets_first_line`. Implementation ancestor is `fa8cd0e2fc`; test-file ancestor is `fcc633ce28`; current evidence hashes are `src/lua_api/frame/methods/text_attribute_event/text.rs`=`ae628de726915e32c5a4c1472e9d0395c7a9253b1e9a546c900bcc6b911d90c2` and `tests/tooltip_basic.rs`=`a0ea03f766f01b53131a9eea7c18c2d55451faa60cf1c06b1ce8840598c001f9`. The claim is limited to clearing existing tooltip lines, inserting supplied text as the first line, and the focused `NumLines() == 1` assertion; formatting, wrapping, colors, localization, rendering, invalid arguments, and edge semantics remain unproven. Totals are now **228 best-effort, 455 evidence-required, 1 exception-requested, and 2726 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 TextureBase atlas methods

Classified `TextureBase.GetTexCoord` and `TextureBase.SetAtlas` as `best-effort`/`behavioral` using the focused tests in `tests/methods_texture.rs`. The claim is limited to atlas remapping including partial UVs, known/unknown lookup, direct tile-slice selection, tiling flags and clearing, and render-preferred 2x path selection. Implementation ancestors are `50e028c9ef` for full-coordinate GetTexCoord behavior and `2591694d10` for the current atlas-method file; tiling and 2x path behavior are retained from `8bbadbf8c8` and `0c63d580ce`; test-file ancestor is `257f13c34e`. Current evidence hashes are `src/lua_api/frame/methods/widgets/texture/coords.rs`=`0aa45b65e0f427602ec88569039d3b5cc44ee5a4636f010aa51cdbc3c63f4d95`, `src/lua_api/frame/methods/widgets/texture/atlas.rs`=`ec7681bfe7e11c4692a1e5290423fd1c2332874555147964910bff488742a625`, and `tests/methods_texture.rs`=`419c30bfd97df89dbc9bb960de00690f769d6ab60e35354a3da9938a0e78361c`. Complete atlas fidelity, CASC/texture loading beyond these assertions, filtering/wrap edge cases, invalid arguments, and rendering correctness remain unproven. Totals are now **227 best-effort, 455 evidence-required, 1 exception-requested, and 2727 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 GameTooltip layout methods

Classified `GameTooltip.GetMinimumWidth`, `GameTooltip.SetMinimumWidth`, `GameTooltip.GetPadding`, and `GameTooltip.SetPadding` as `best-effort`/`behavioral` using `tests/tooltip_basic.rs::test_setminimumwidth_and_getminimumwidth` and `tests/tooltip_basic.rs::test_setpadding_and_getpadding`. Implementation ancestor is `4a4621c2e`; test-file ancestor is `fcc633ce2`. Current evidence hashes are `src/lua_api/frame/methods/widgets/tooltip/line_data.rs`=`469dd6b58b37b0a47bc800a5e6fe08eb3b0a27b438b0365c193100ce3e7d28b2` and `tests/tooltip_basic.rs`=`a0ea03f766f01b53131a9eea7c18c2d55451faa60cf1c06b1ce8840598c001f9`. The claim is limited to setter/getter state round-trips for minimum width 150 and padding 8; tooltip rendering/layout effects, clamping, invalid arguments, and edge semantics remain unproven. Totals are now **225 best-effort, 455 evidence-required, 1 exception-requested, and 2729 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 Cooldown methods

Classified `Cooldown.GetCountdownFontString`, `Cooldown.SetCooldownFromDurationObject`, and `Cooldown.SetCooldownFromExpirationTime` as `best-effort`/`behavioral` using `tests/cooldown_widget.rs::cooldown_widget_methods_persist_runtime_state`. Implementation ancestors are `d7a3cf21b` for countdown-font/expiration behavior and `a1f638733` for duration-object behavior. Current evidence hashes are `src/lua_api/frame/methods/widgets/cooldown.rs`=`527213c02fce0cda3bc7c6fd5f0874d3eea3d4e28dc59de2fb086cb32ac2e2b7` and `tests/cooldown_widget.rs`=`07a66162ff33fd6245879fc0d962ffa10cb9ca5026e8a9bf3fdcd651b4d19b8d`. The claim is limited to FontString creation/type, expiration-to-start/duration conversion, duration-object start/total-duration/mod-rate access, and zero-duration clearing; retail rendering, time progression, formatting, invalid arguments, and edge semantics remain unproven. Totals are now **221 best-effort, 455 evidence-required, 1 exception-requested, and 2733 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 StatusBar interpolation methods

Classified `StatusBar.GetInterpolatedValue`, `StatusBar.IsInterpolating`, and `StatusBar.SetToTargetValue` as `best-effort`/`behavioral` using `tests/widget_methods_colorselect.rs::test_statusbar_interpolation_methods_track_target_and_displayed_value` and ancestor implementation commit `24e44f3f0`. The claim is limited to the tested interpolation state machine: Smooth target assignment leaves the displayed value unchanged, `GetValue` is target-facing, `GetInterpolatedValue` returns the displayed value, `IsInterpolating` reports target presence, and `SetToTargetValue` snaps and clears interpolation. Timing/animation progression, repeated-target behavior, invalid modes, render, and event fidelity remain unproven. Totals are now **218 best-effort, 455 evidence-required, 1 exception-requested, and 2736 untriaged**.

## [2026-08-08] investigation | Classify 12.0.0 legacy combat-log cursor globals

Classified `CombatLogAdvanceEntry` and `CombatLogSetCurrentEntry` as `evidence-required`/`unsafe`. The checked-in 12.0.0 register records both removals; pinned retail/PTR `Blizzard_DeprecatedCombatLog` sources do not publish them; the temporary simulator model retains fixture-only cursor mutation for Wrath/Mists `Blizzard_CombatLog` callers; and no equivalent replacement contract or authoritative legacy semantics is established. Tests remain empty with null commit, approval, and scope exception. Totals are now **215 best-effort, 455 evidence-required, 1 exception-requested, and 2739 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 SpellGetVisibilityInfo vendor wrapper

Classified `SpellGetVisibilityInfo` as `best-effort`/`vendor-present` using `patch-tests/patch_12_1/vendor_deprecated_chat_spell.rs::vendor_deprecated_chat_spell_globals_are_published_and_forward` at `ed3ad9d87`. The focused full-LoD proof checks publication under enabled `loadDeprecationFallbacks`, string-to-Enum translation for `RAID_INCOMBAT`, sentinel forwarding, and unknown visibility-name nil forwarding; it does not claim complete `C_Spell` visibility semantics. `CombatLogAdvanceEntry` and `CombatLogSetCurrentEntry` remain untriaged. Totals are now **215 best-effort, 453 evidence-required, 1 exception-requested, and 2741 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 deprecated chat/spell globals

Classified `CancelEmote`, `DoEmote`, `SpellIsPriorityAura`, and `SpellIsSelfBuff` as `best-effort`/`vendor-present` using `patch-tests/patch_12_1/vendor_deprecated_chat_spell.rs::vendor_deprecated_chat_spell_globals_are_published_and_forward` at `02db1895a`. The focused full-LoD proof checks publication under enabled `loadDeprecationFallbacks`, CancelEmote forwarding, DoEmote named and nil branches, and spell-wrapper forwarding; it does not claim complete legacy semantic fidelity. `CombatLogAdvanceEntry`, `CombatLogSetCurrentEntry`, and `SpellGetVisibilityInfo` remain untriaged. Totals are now **214 best-effort, 453 evidence-required, 1 exception-requested, and 2742 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 simulator legacy compatibility globals

Classified exactly seven removed globals (`FindBaseSpellByID`, `FindFlyoutSlotBySpellID`, `FindSpellOverrideByID`, `GetBattlegroundInfo`, `PlaySound`, `SetPortraitToTexture`, and `strtrim`) as `best-effort`/`compat` using the focused full-LoD proof `patch-tests/patch_12_1/legacy_compat.rs::simulator_legacy_compat_globals_preserve_tested_behavior` at `abba2bd2a`. The claim is limited to tested wrapper forwarding, seeded battleground and unknown-ID behavior, numeric sound acceptance without audio fidelity, portrait circular masking without duplicates, and default/custom trimming. `SpellGetVisibilityInfo` and the other unresolved simulator-published legacy globals remain untriaged. Totals are now **210 best-effort, 453 evidence-required, 1 exception-requested, and 2746 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 vendor deprecated globals

Classified exactly 56 removed legacy global API rows as `best-effort`/`vendor-present` using the committed full-LoD proof `patch-tests/patch_12_1/strict_removals.rs::vendor_deprecated_globals_are_published_and_forward` at `a26692e00`. The 35 ActionBar, 3 BattleNet, 10 CombatLog, 2 CombatText, 3 DeathRecap, and 3 InstanceEncounter wrappers are published by Blizzard deprecated addons when `loadDeprecationFallbacks` is enabled; representative forwarding/alias checks prove publication ownership, not complete legacy semantic fidelity. Fourteen other diagnosed published globals remain untriaged. Totals are now **203 best-effort, 453 evidence-required, 1 exception-requested, and 2753 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 legacy global removals

Classified exactly four removed legacy global API rows (`IsConsumableSpell`, `SetRaidTargetProtected`, `SpellIsAlwaysShown`, and `StripHyperlinks`) as `best-effort`/`behavioral` using the committed full-LoD `rawget(_G, name)` probe `patch-tests/patch_12_1/strict_removals.rs::removed_legacy_global_apis_are_absent_after_full_lod_load` at `3471c5a4c`. The claim is limited to current publication absence; no source-scanner, replacement-behavior, or historical timing claim is made. The other 70 diagnosed published removed globals remain untriaged pending vendor/simulator provenance. Totals are now **147 best-effort, 453 evidence-required, 1 exception-requested, and 2809 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 C_Transmog removals

Classified exactly 18 removed `C_Transmog` API rows as `best-effort`/`behavioral` using the committed full-LoD runtime probe `patch-tests/patch_12_1/strict_removals.rs::removed_transmog_methods_are_absent_after_full_lod_load` at `2975b0ad6`. The probe uses `rawget` to prove current publication absence while five retained C_Transmog APIs remain callable; source scanning is auxiliary only. Removed the two obsolete simulator registrations and obsolete direct test expectations. The three removed `TransmogApplyWarningInfo` structure/field rows remain untriaged because runtime rawget does not prove metadata removal; the 11 added/changed slot-visual rows remain evidence-required. Totals are now **124 best-effort, 453 evidence-required, 1 exception-requested, and 2832 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 C_TransmogCollection removals

Classified exactly 10 removed `C_TransmogCollection` outfit API rows as `best-effort`/`behavioral` using the committed full-LoD runtime probe `patch-tests/patch_12_1/strict_removals.rs::removed_transmog_collection_outfit_methods_are_absent_after_full_lod_load` at `c3ae90c26`. The probe uses `rawget` to prove current publication absence while retained appearance methods remain callable; source scanning is auxiliary only. Removed the three obsolete simulator outfit placeholders and their tests. The 23 added/changed custom-set and appearance-source rows remain evidence-required; custom-set replacement APIs are not claimed implemented, and no clean replacement surface, historical load-order timing, or broad scanner completeness is claimed. Totals are now **106 best-effort, 453 evidence-required, 1 exception-requested, and 2850 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_CombatLog slice

Classified all 11 added `C_CombatLog` API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the shared permissive/fixture-backed `src/lua_api/workarounds/temporary/combat_log_state.rs` model. Tests remain empty with null commit, approval, and scope exception. Filter schema/matching, restriction state, retention/message-limit bounds, clear/refilter lifecycle, and entry semantics remain unproven; no approval can close these rows. Totals are now **77 best-effort, 453 evidence-required, 1 exception-requested, and 2879 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_CombatLogSecure slice

Classified all nine added `C_CombatLogSecure` secure-only API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the permissive/fixture-backed `src/lua_api/workarounds/temporary/combat_log_state.rs` model. Tests remain empty with null commit, approval, and scope exception. Secure/taint enforcement, filtering rules, event/message payload shape, navigation semantics, and entry lifecycle remain unproven; no approval can close these rows.

## [2026-08-08] investigation | Bound 12.0.0 C_UnitAuras slice

Classified exactly 10 added/changed `C_UnitAuras` API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the current `src/lua_api/globals/auras.rs` plus temporary `unit_auras_state.rs` seeded aura model. Tests remain empty with null commit, approval, and scope exception. Source signatures/defaults and adjacent seeded aura behavior do not establish defensive classification, expiration/display formatting, duration objects, refresh calculations, color curves, sorted instance IDs, private callback dispatch, or GetUnitAuras sort semantics; no approval can close these rows. Totals are now **77 best-effort, 433 evidence-required, 1 exception-requested, and 2899 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_UnitAurasPrivate slice

Classified all 10 added/changed `C_UnitAurasPrivate` secure-only API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the current `src/lua_api/workarounds/temporary/private_aura_state.rs` permissive/partial model. Tests remain empty with null commit, approval, and scope exception. Secure enforcement, private-aura visibility, callback/anchor lifecycle, callback payloads, and the two absent APIs remain unproven; existing tests prove simulator-only seeded state/callback behavior and are intentionally not attached, so no approval can close these rows. Totals are now **77 best-effort, 423 evidence-required, 1 exception-requested, and 2909 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_SpellDiminish slice

Classified exactly 14 added `C_SpellDiminish` API, structure, and structure-field rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the current `src/c_api/c_spell_diminish.rs` static eight-category fixture. Tests remain empty with null commit, approval, and scope exception. The source register establishes signatures and field names only; the fixture and local tests do not establish authoritative 12.0.0 category contents, ruleset tracking semantics, or tracker payload fields, and no approval can close these rows. Totals are now **77 best-effort, 423 evidence-required, 1 exception-requested, and 2909 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_DeathRecap slice

Classified exactly four added `C_DeathRecap` API/structure rows as `evidence-required`/`unsafe` using checked-in source-register evidence, pinned API documentation/vendor-consumer evidence, and the current `src/c_api/c_death_recap.rs`/`src/lua_api/state_types/mythic_plus_scenario.rs` killing-blow-only model. Tests remain empty with null commit, approval, and scope exception. Pinned retail/PTR/Wowless documentation exposes no `DeathRecapEventInfo` fields; vendor consumers reveal only partial amount/timestamp/sourceGUID usage; recap-event fields, link format, recap-ID selection, default/no-argument behavior, unknown-ID handling, and event-presence semantics remain unproven. `HasRecapEvents` is not classified best-effort by inference, and no approval can close these rows. Totals are now **77 best-effort, 399 evidence-required, 1 exception-requested, and 2933 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_TransmogCollection slice

Classified exactly 23 added/changed `C_TransmogCollection` custom-set and appearance-source rows as `evidence-required`/`unsafe` using checked-in source-register evidence, the seeded/partial `src/lua_api/globals/missing_surface/transmog_collection.rs` surface, and `src/lua_api/state_types/collections.rs`; tests remain empty with null commit, approval, and scope exception. The 10 removed outfit rows remain untriaged because removal direction alone does not establish replacement behavior. Custom-set lifecycle, hyperlinks, persistence, validation, and exact `TransmogAppearanceSourceInfoData` semantics remain unproven, and related local tests do not establish authoritative retail 12.0.0 behavior; no approval can close these rows. Totals are now **77 best-effort, 395 evidence-required, 1 exception-requested, and 2937 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_TransmogOutfitInfo slice

Classified exactly 115 added `C_TransmogOutfitInfo` API, structure, and structure-field rows as `evidence-required`/`unsafe` using checked-in source-register evidence and `src/lua_api/globals/transmog_outfit_info.rs`; tests remain empty with null commit, approval, and scope exception. The two present lock queries use only local state behavior, the other 113 rows are unmodeled, and local tests do not establish authoritative retail 12.0.0 semantics. Authoritative live evidence or a correct modeled transmog-outfit subsystem with focused tests is required, and no approval can close these rows. Totals are now **77 best-effort, 355 evidence-required, and 2978 untriaged**.

# Wiki Log

## [2026-08-29] audit | Document secure nil-symbol publication and synthetic attribution fixes

Audited commits `2da0dc1de` and `584b84c9e`. Updated [[addon-loading]] and [[lua-api]] plus the maintained addon-loading/Lua API references: same-addon nil-symbol reconciliation now records and resolves public and secure publications separately by stable addon index; secure assignments and secure frame exports cannot resolve public misses; precompiled lifecycle dispatch uses raw `_G.self` snapshot/restore; and post-cleanup `C_StoreSecure` restoration reads raw `_G` state to avoid attributing simulator bootstrap lookups to Blizzard code. No new page or spec was warranted; `docs/wiki/index.md` required no catalog change.\n\n## [2026-08-29] audit | Document state-backed C_GuildInfo management methods

Audited commit `d64bbb8b3`. Updated [[lua-api]] and the maintained Lua API reference to record that `C_GuildInfo.Invite`, `Uninvite`, `Promote`, and `Leave` share the existing state-backed guild handlers and deprecated-global identity. Explicitly documented that `Demote`, `Disband`, `SetLeader`, and `RemoveFromGuild` remain unsupported because their distinct semantics are not modeled. No spec or new wiki page was warranted; protected `src/c_api/c_string_util.rs` was not touched.

## [2026-08-08] investigation | Bound 12.0.0 C_Transmog slice

Classified exactly 11 non-removed `C_Transmog` structure/API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the current `src/lua_api/globals/missing_surface/transmog.rs` partial surface; tests remain empty with null commit, approval, and scope exception. The source records only names and the `GetSlotVisualInfo` signature transition, while no direct slot-visual/pending/apply state model or behavioral tests establish authoritative retail 12.0.0 semantics. The 21 removed C_Transmog rows remain untriaged because removal direction alone does not establish replacement behavior; no unrelated collection/outfit tests were used and no approval can close these rows. Totals are now **77 best-effort, 372 evidence-required, 1 exception-requested, and 2960 untriaged**.

## [2026-08-08] investigation | Prove 12.0.0 C_NamePlate removals

Classified six added 2D `C_NamePlate`/`C_NamePlateManager` APIs as `evidence-required`/`unsafe` because the current permanent shim has no modeled nameplate-manager state and exact 12.0.0 semantics remain unproven. Classified `C_NamePlateManager.IsNamePlateUnitBehindCamera` as `exception-requested`/`impossible` under the already-decided permanent no-3D project scope; this is not a user approval request. Classified all 19 removed `C_NamePlate` rows as `best-effort`/`behavioral` using the committed full-LoD runtime probe `patch-tests/patch_12_1/strict_removals.rs::removed_nameplate_methods_are_absent_after_full_lod_load` at `2c2c5ad1d`. The probe uses `rawget` to prove current publication absence while retained APIs remain callable; source scanning is auxiliary only and does not claim historical load-order timing or broad scanner completeness. Totals are now **96 best-effort, 453 evidence-required, 1 exception-requested, and 2860 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_EncounterTimeline slice

Classified exactly 55 added `C_EncounterTimeline` API, structure, and structure-field rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the current `src/lua_api/workarounds/temporary/encounter_state.rs` partial seeded fixture; tests remain empty with null commit, approval, and scope exception. Nine APIs are present only as fixture-backed behavior and 46 rows are absent; exact encounter state, script-event lifecycle, timers, feature flags, payload values, and structure-field semantics require authoritative live evidence or a correct modeled subsystem with focused tests, and no approval can close these rows. Totals are now **77 best-effort, 240 evidence-required, and 3093 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_PingSecure slice

Classified exactly 15 changed `C_PingSecure` API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and clean current `src/c_api/c_ping_secure.rs`; tests remain empty with null commit, approval, and scope exception. The source contract is secure-only while current behavior is no-op/inert callback storage/partial or absent; exact secure-call enforcement, targeting, frame/error/audio/UI dispatch, callback invocation, and `PingResult` semantics require authoritative live evidence or a correct ping/security model and direct tests, and no approval can close these rows. Totals are now **77 best-effort, 185 evidence-required, and 3148 untriaged**.

## [2026-08-08] investigation | Bound 12.0.0 C_Secrets slice

Classified exactly 23 added `C_Secrets` API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the examined current `src/lua_api/globals/register.rs` surface; tests remain empty with null commit, approval, and scope exception. Exact secrecy levels and action/aura/cooldown/totem/unit health/identity/power/cast restriction semantics require authoritative live evidence or a correct taint/security model and tests; secret behavior must not be guessed or approved closed. Totals are now **77 best-effort, 170 evidence-required, and 3163 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_EncounterWarnings slice

Classified exactly 19 added `C_EncounterWarnings` structure/API rows as `evidence-required`/`unsafe` using checked-in source-register evidence and clean current `encounter_warnings.rs`; tests remain empty with null commit, approval, and scope exception. `GetEditModeWarningInfo`/current structure fields are fabricated preview/static payload behavior, `PlaySound` is a no-op, and the other three methods lack examined registration; exact state, payload meanings, feature flags, severity sound mapping, and audio playback require authoritative evidence or a correct modeled subsystem/test. Totals are now **77 best-effort, 147 evidence-required, and 3186 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_CombatAudioAlert slice

Classified exactly 12 added `C_CombatAudioAlert` rows as `evidence-required`/`unsafe` using checked-in source-register evidence and the examined current `src/lua_api/globals/register.rs` surface; tests remain empty with null commit, approval, and scope exception. exact combat-audio settings, speech scheduling/audio output, and enable/throttle semantics require authoritative evidence or a correct modeled subsystem, and no approval can close the rows. Totals are now **77 best-effort, 128 evidence-required, and 3205 untriaged**.

## [2026-08-07] investigation | Bound remaining 12.0.0 C_ActionBar runtime slice

Classified four C_ActionBar rows as best-effort/behavioral using only source-register, current action-bar implementation/registration, and the named direct/end-to-end tests; claims are limited to exact tested seeded/empty/malformed profession quality, modeled action-slot presence/texture, and modeled outfit-lock slot behavior. The remaining 22 action queries/registration rows are evidence-required/unsafe with no approval path; the 11 ActionBarChargeInfo/ActionBarCooldownInfo structure/field rows remain untriaged. Totals are now **77 best-effort, 116 evidence-required, and 3217 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_ActionBar page/state-query slice

Classified exactly 13 `C_ActionBar` page/state-query rows as `best-effort`/`behavioral` from the source register, current action-bar implementation/registration, ancestor commits `dacb23419`, `bdf741cf4`, `0859e07be`, `06bf6616a`, `5b9497307`, `6425b3f0b`, `080358cdb`, `6d13156ae`, and `167e23297`, and only the named page/state/default/transition tests; claims remain limited to tested state/default/transition behavior, with exact retail paging, vehicle/override/bonus precedence, skins, secure state, and lifecycle semantics unproven. Classified exactly three rows as `evidence-required`/`unsafe` with source-register and current implementation evidence, empty tests, and null commit/approval/scope exception; authoritative semantics or a correct model/test are required, and no approval can close them. Totals are now **73 best-effort, 94 evidence-required, and 3243 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_DamageMeter triage

Classified exactly 19 C_DamageMeter rows as `best-effort`/`behavioral` from the source register, clean temporary seeded-state implementation, ancestor commit `e005f99e`, and only the named seeded/empty/zero-ID/unit-detail tests; claims remain limited to exact seeded/empty lookup and shape/type assertions, with no complete retail aggregation/lifecycle/secret fidelity. Classified exactly 10 rows as `evidence-required`/`unsafe` with source-register and current implementation evidence, empty tests, and null commit/approval/scope exception; seeded-but-unasserted fields and the unimplemented reset lifecycle require authoritative semantics or a correct model/test, and no approval can close them. Six metadata-only structure rows remain untriaged. Totals are now **60 best-effort, 91 evidence-required, and 3259 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_TradeSkillUI slice

Classified `added:C_TradeSkillUI.GetDependentReagents` as `best-effort`/`behavioral` from source-register, current professions implementation/registration, exact tests, and ancestor commit `36f425fb2`; the claim is limited to table return/iteration safety and nil/malformed/unknown-reagent behavior. Classified exactly 11 quality/recraft/reagent-link rows as `evidence-required`/`unsafe` with source-register and current professions implementation/registration evidence, empty tests, and null commit/approval/scope exception; current evidence distinguishes absent methods, placeholder empty-table/true behavior, and unproven removal behavior. Authoritative profession semantics or a correct model/test are required, and no approval can close them. Totals are now **41 best-effort, 81 evidence-required, and 3288 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_Spell triage

Classified exactly 12 added `C_Spell` rows as `evidence-required`/`unsafe` using only the checked-in source register and clean current `src/c_api/c_spell.rs` evidence. Duration lifecycle, spell metadata, and boolean semantics remain unproven; `IsSelfBuff` has a same-name internal implementation, but no matching 12.0.0 publication/behavioral contract is proven. All tests/assertions are empty with null commit, approval, and scope exception; no approval can close these rows. Totals are now **40 best-effort, 70 evidence-required, and 3300 untriaged**.

Chronological record of wiki operations.

## [2026-08-07] investigation | Bound 12.0.0 C_ColorUtil slice

Classified `added:C_ColorUtil.GenerateTextColorCode` and `added:C_ColorUtil.WrapTextInColorCode` as best-effort behavioral from the checked-in source register, current color defaults, exact `installs_color_defaults` test, and ancestor commit `1abe6088d`; claims remain limited to tested RGB-to-`ffRRGGBB` conversion and explicit color-code wrapping. Classified four conversion rows and `added:C_ColorUtil.WrapTextInColor` as evidence-required unsafe with empty tests and no approval path. The register now totals **40 best-effort, 58 evidence-required, and 3312 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_Timer signature slice

Classified `changed:C_Timer.NewTimer` and `changed:C_Timer.NewTicker` as best-effort behavioral from checked-in signatures, current timer/proxy paths, ancestor commits `330e521be`/`3d767dbd2`, and the five named timer-container tests; claims remain limited to function/container acceptance, returned container identity/proxy equality, cancellation, and independent ticker counts. Classified `changed:C_Timer.After` as evidence-required unsafe because only an ignored focused-looking test exists; callback/lifecycle semantics require a correct modeled implementation and executable behavioral proof, with no approval path. The register now totals **38 best-effort, 53 evidence-required, and 3319 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 C_StringUtil slice

Classified `added:C_StringUtil.EscapeQuotedCodes` as best-effort behavioral from the checked-in source register, current `src/c_api/c_string_util.rs`, exact focused test, and ancestor commit `b3f579f70`; the claim is limited to quoted-code pipe escaping for tested plain/color-code cases. Classified eight unpublished `C_StringUtil` rows as evidence-required unsafe with no approval path. The register now totals **36 best-effort, 52 evidence-required, and 3322 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 curve-family triage

Classified nine curve factory/object rows as best-effort behavioral from the checked-in source register, temporary proxy factory, ancestor commit `22ab64e5e`, and only the relevant `userdata_proxy` tests; claims remain limited to tested factory/table shape, scalar interpolation/copy behavior, and color-object/copy shape. Classified 23 unresolved curve contracts as evidence-required unsafe with empty tests and no approval path because the generic proxy omits or does not faithfully establish their contracts; they cannot be approved closed. 17 curve-family metadata rows remain untriaged. The register now totals **35 best-effort, 44 evidence-required, and 3331 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 duration-object audit slice

Classified five duration/factory/StatusBar rows as best-effort behavioral from checked-in source, ancestor commits `35e39a58f`/`0a737bfbc`, and exact focused tests. Classified 21 duration-time/lifecycle/secret rows as evidence-required unsafe; current behavior is constant/no-op/incomplete and needs authoritative semantics or a correct modeled implementation, with no approval path. The register now totals **26 best-effort, 21 evidence-required, and 3363 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 FunctionContainer classification

Classified `changed:C_FunctionContainers.CreateCallback` and four `LuaFunctionContainer` rows as best-effort behavioral from checked-in source-register/proxy evidence and the named `userdata_proxy` tests. Tested userdata type, method exposure, cancellation/invoke suppression, per-instance fields, and read-only keys are covered; exact retail callback validation, metatable/equality identity, tostring formatting, timer integration, lifecycle/GC, and API metadata fidelity remain unproven. The register now totals **21 best-effort and 3389 untriaged**.

## [2026-08-07] investigation | Bound 12.0.0 classifications

Classified four `AbbreviateConfig` rows and twelve `UnitHealPrediction` rows as best-effort from bounded, evidence-backed tests. The 12.0.0 register now totals **16 best-effort and 3394 untriaged** (3410 occurrences); these limited-fidelity classifications do not complete the audit or satisfy `--complete`.

## [2026-08-07] architecture | Document 12.0.0 occurrence payloads

Documented the current 12.0.0 generator/source-register change: categorized occurrence objects may preserve optional typed `before`/`after` payloads containing normalized value and metadata for exact enum, constant, signature, and structure triage. Added/removed/changed and transient lifecycle rows carry the corresponding sides; row identity remains `direction+symbol`, unknown fields remain rejected, and counts/statuses/source SHA are unchanged.

## [2026-08-07] investigation | Document neutral 12.0.0 audit artifacts

Added [[patch-12-0-0-api-audit]] for the reproducible wowless snapshot register: boundary 11.2.7 build `65299` → final explicit 12.0.0 build `65727`, six 12.0.0 snapshots, 8 transient lifecycle rows, 3410 total occurrences, and all rows untriaged. Recorded the explicit limit: wowless schema provenance only, with no historical 12.0.0 FrameXML or live runtime claim; the active retail cache manifest is validation metadata rather than historical provenance.

## [2026-08-07] decision | Separate evidence-required from exception-requested

Migrated the 12.1 behavior register to **33 best-effort, 21 evidence-required, 0 exception-requested, 0 untriaged** and the 12.0.5 probe register to **33 best-effort, 4 evidence-required, 1 approved provenance-only exception-requested, 0 untriaged**. `evidence-required` marks triaged unresolved unsafe/impossible behavior requiring item-specific authoritative/live evidence; it requires no approval, commit, or focused test and cannot pass `--complete`. The 12.0.7 repository-authorized no-3D exception remains unchanged.

## [2026-08-07] investigation | Add strict-removal timing evidence guidance

Audited commit `12ed1355b`. Documented `StrictRemovalTimingProbe` as a collector for addon-visible lifecycle timing. Its presence does not resolve or close any 12.1 behavior row; strict-removal timing gaps remain open until raw retail/PTR SavedVariables captures are obtained and interpreted. The existing `ForbiddenAspectsProbe` remains the live evidence path for all six forbidden-aspect restrictions.

## [2026-08-07] investigation | Add private-object and forbidden-aspect probe guidance

Documented `PrivateScriptObjectProbe` and `ForbiddenAspectsProbe` as addon-tainted live-evidence tools. Their corresponding 12.1 rows remain open until raw captures are retained and interpreted; neither proves secure-caller behavior or unsupported internal/input paths.

## [2026-08-07] investigation | Add 12.1 live-client probe guidance

Documented [UnitAuraSecretProbe](../addons/UnitAuraSecretProbe/README.md) and [DurationTextBindingProbe](../addons/DurationTextBindingProbe/README.md) as evidence paths. UnitAura constrains addon-tainted AuraData and `UNIT_AURA` behavior but cannot establish Blizzard-secure caller access; DurationTextBinding constrains representation, identity, and lifetime observations but cannot prove native finalization. Corresponding 12.1 rows remain open until raw retail/PTR SavedVariables captures are retained and interpreted.

## [2026-08-07] decision | Reopen unverified 12.0.5 behavior exceptions

Applied the correct-behavior-only policy: reopened `ScaleEventProbe.SameSizeDuplicatePair` and `StoreForbiddenProbe.ForbiddenDescendants` by clearing their approvals. Kept `XmlFrameLevelProbe.RawCaptureProvenance` approved as provenance-only because its behavior is independently regression-tested. The 12.0.5 audit remains open at 33 best-effort, 5 exception-requested, 0 untriaged: 1 approved provenance-only exception and 4 open behavior exceptions (1 impossible same-size boundary and 3 unsafe Store/security gaps).

## [2026-08-07] investigation | Approve two 12.0.5 exceptions

Recorded explicit approvals for `ScaleEventProbe.SameSizeDuplicatePair` and `XmlFrameLevelProbe.RawCaptureProvenance`. Both remain `exception-requested` with `impossible` resolution; the three unsafe Store exceptions remained unapproved at this point.

## [2026-08-07] decision | Approve Store descendant exception

Recorded explicit approval for `StoreForbiddenProbe.ForbiddenDescendants` as an unsafe exception because the retained probe never captured the `/sfp` descendant matrix. Two unsafe Store exceptions remain unapproved; the 12.0.5 register remains 33 best-effort, 5 exception-requested, and 0 untriaged.

## [2026-08-07] investigation | Reclassify Texture radial progress rows

Corrected the three 12.1 RadialProgress rows to Texture-backed behavioral best-effort contracts. Focused Texture surface/state proof covers method availability, receiver dispatch, defaults, setters/getters, visual mode, and Clear reset; exact retail clamping and visual rendering remain best-effort. The broader register now has 33 best-effort, 21 exception-requested, and 0 untriaged rows, with no impossible candidates.

## [2026-08-07] investigation | Itemize remaining 12.1 exceptions

Converted all 24 remaining 12.1 broader behavior rows to item-specific `exception-requested` entries: 21 unsafe and 3 impossible. Each row has current repository source evidence, empty tests/assertions, `commit: null`, and `approval_id: null`; no exception is approved. The broader register now has 30 best-effort, 24 exception-requested, and 0 untriaged rows.

## [2026-08-07] investigation | 12.0.5 pending exception register

Converted the five remaining 12.0.5 probe rows to item-specific `exception-requested` entries: three unsafe Store/protection gaps and two impossible window/provenance gaps. Each row has hashed repository evidence, `approval_id: null`, empty tests/assertions, and awaits separate informed approval or new live evidence. The register now has 33 best-effort, 5 exception-requested, and 0 untriaged rows.

## [2026-08-07] investigation | TieredEntrance payload classification

Corrected the final 12.1 safe candidate from a tiered-aura label to `TieredEntrance` after confirming pinned PTR exposes `C_DelvesUI` `TieredEntranceTierInfo` / `TieredEntranceRewardInfo` and no corresponding aura API. Focused proof classifies deterministic tier/reward rows as best-effort; live reward IDs, quantities, unlock timing, eligibility, and economics remain outside the claim. The broader register now has 30 best-effort and 24 untriaged rows, with zero safe-best-effort rows remaining.

## [2026-08-07] investigation | Protected descendant-anchor probe replay

Classified `IsProtectedProbe.DescendantAnchorPropagation` as best-effort from focused behavioral evidence. The directly protected root returns true/true; child, grandchild, frames anchored to the root or child, and the root-keyed anchored frame remain false/false. The 12.0.5 register now has 33 best-effort and 5 untriaged rows.

## [2026-08-07] investigation | PlayerChoice and strict-load classification

Classified the state-backed `C_PlayerChoice` payload and mutator-intent contract plus the pre-removal pinned-PTR load window as best-effort. Focused proof covers default and seeded nested choice payloads and confirms representative compatibility symbols remain callable after the complete all-LoD load. The broader register now has 23 best-effort and 31 untriaged rows.

## [2026-08-07] architecture | PTR C_PlayerChoice local model

Documented commit `c64472e6e`: patch 12.1 `C_PlayerChoice` uses `SimState.player_choice` for deterministic query payloads and mutator-intent markers, including nested choice options and currency/item/reputation rewards. Explicit boundary: no claim of retail timing, server validation, reroll economics, or live service values.

## [2026-08-07] investigation | Cooldown, pet, and LFG payload classification

Added focused 12.1 payload tests for active/inactive spell cooldowns, seeded/unknown pet species, and seeded/unknown LFG search results. Classified all three local compatibility contracts as best-effort; secret fields and server-backed semantics remain unmodeled. The broader register now has 21 best-effort and 33 untriaged rows.

## [2026-08-07] investigation | Mouse-focus probe replay

Added a GUI-path replay of the retained two-frame DIALOG scenario. `GetMouseFocus()` and `GetMouseFoci()[1]` both retain the higher raw-level frame before and after `Raise`/`Lower`, then clear when both frames hide. The 12.0.5 register now has 32 best-effort and 6 untriaged rows.

## [2026-08-07] investigation | Battle.net service payload classification

Classified three broader 12.1 Battle.net rows as best-effort from focused local-state tests: deduplicated verified friend invites and returned fields, title-friend names/tags/feature and presence state, and the explicitly unsupported deterministic unit-invite result. Exact service validation, persistence, events, and eligibility remain unmodeled. The broader register now has 18 best-effort and 36 untriaged rows.

## [2026-08-07] investigation | Duration binding identity split

Split stable Lua-table identity from unknown Blizzard representation fidelity. Focused reference-retention/identity proof classifies lifetime and stable identity as best-effort; exact type/metatable/finalization fidelity remains a separate unsafe candidate. The broader register now has 54 rows: 15 best-effort and 39 untriaged.

## [2026-08-07] investigation | First 12.1 behavior classifications

Classified 13 broader 12.1 rows only where exact existing tests directly prove the simulator contract: aura-frame creation, DurationTextBinding formatter/color/FontString behavior, Discord state, four housing models, Encounter Journal difficulty guesses, and post-startup strict removal. Forty rows remain untriaged.

## [2026-08-07] investigation | 12.1 broader behavior register

Created a separate 53-row machine register for non-FrameXML 12.1 fidelity boundaries. All rows remain neutral; candidate disposition is 30 safe best-effort, 20 unsafe, and 3 impossible, with family summaries no longer serving as approval units.

## [2026-08-07] investigation | UI-scale CVar event ordering

Modeled successful `SetCVar` event dispatch and the retail `uiScale`/`useUiScale` boundary: new effective scale first, then `DISPLAY_SIZE_CHANGED` and `UI_SCALE_CHANGED` while the old CVar remains visible, then storage and `CVAR_UPDATE`. The 12.0.5 register now contains 31 best-effort and 7 untriaged rows.

## [2026-08-07] investigation | MessageFrame region replay

Added focused MessageFrame and ScrollingMessageFrame owner, region, anchor, and TextInsets proof. The 12.0.5 register now contains 30 best-effort and 8 untriaged rows.

## [2026-08-07] investigation | FontString size, anchor, and EditBox replays

Added focused no-size/partial-size/full-size FontString checks, explicit TOP/BOTTOM/LEFT/RIGHT/TOPLEFT anchor controls, and no-size/sized/inset EditBox backing-region proof. The 12.0.5 register now contains 29 best-effort and 9 untriaged rows.

## [2026-08-07] investigation | Frame-layer FontString anchor evidence

Mapped the retained unanchored frame-layer `justifyH`/`justifyV` matrix to its exact regression test. The 12.0.5 register now contains 26 best-effort and 12 untriaged rows.

## [2026-08-07] investigation | Panel pulse and DevTools dump replays

Added exact two-panel `ShowUIPanel`/`CloseAllWindows` lifecycle proof and a loaded-Blizzard-UI `DevTools_Dump` frame-array metadata replay. The 12.0.5 register now contains 25 best-effort and 13 untriaged rows.

## [2026-08-07] investigation | XML protected-frame replay

Added an exact XML `protected="true"` frame replay covering protection, forbidden state, absent legacy setters, failing setter calls, and retained protection. The 12.0.5 register now contains 23 best-effort and 15 untriaged rows.

## [2026-08-07] investigation | Retail protection probe replays

Added focused checks for absent retail `CreateForbiddenFrame` and the complete plain-frame protection/forbidden/legacy-setter sequence. Both probe rows are now best-effort, bringing the 12.0.5 register to 22 best-effort and 16 untriaged rows.

## [2026-08-07] correction | HookScript binding probe contract

Corrected the 12.0.5 source register to the retained retail behavior: the normal binding slot succeeds and chains, while explicit slots 0 and 2 return false and remain absent from `GetScript`. Existing focused tests directly prove that contract; classification totals are unchanged.

## [2026-08-07] investigation | Invalid SetAtlas argument evidence

Classified the retained nil, no-argument, boolean, numeric, empty-string, and unknown-atlas matrix from three focused behavioral tests. The 12.0.5 register now contains 20 best-effort and 18 untriaged rows.

## [2026-08-07] investigation | Identity and protection probe evidence

Classified surrogate identity dispatch, absent legacy `Protect`/`SetProtected`, and secure-template protection from exact focused tests. The 12.0.5 register now contains 19 best-effort and 19 untriaged rows.

## [2026-08-07] investigation | Animation script-handler probe replay

Added a focused Frame/AnimationGroup/nine-animation-subtype handler-matrix regression. The initial RED exposed a test misunderstanding—unsupported `HasScript` returns false without error, while `SetScript` rejects—then the corrected test passed and classified `AnimScriptProbe.HandlerMatrix` as best-effort. Current 12.0.5 totals: 16 best-effort and 22 untriaged.

## [2026-08-07] investigation | First evidence-backed 12.0.5 probe classifications

Classified 15 probe rows as best-effort only where focused behavioral tests directly exercise the recorded contract: scalar attributes, forbidden state, unit-event filters, wildcard attributes, frame ordering/identity, HookScript bindings, ButtonText anchors, texture path/clear behavior, and XML frame-level semantics. Twenty-three rows remain neutral; broad subsystem or register-shape tests are not accepted as probe evidence.

## [2026-08-07] investigation | 12.0.7 duration proof and conservative status correction

Extracted a focused duration clock/object/text-binding regression that directly exercises the modeled method families and verifies the stale formatting-options/raw-value factories remain unavailable. Corrected five overclaimed additive rows to best-effort, classified `ModelSceneActorBase.GetModelUnitGUID` as an impossible no-3D scope exception candidate, and pinned the register at 29 implemented, 101 best-effort, one exception-requested, and zero untriaged rows.

## [2026-08-07] system | repository scope exception for 12.0.7 no-3D gap

Migrated `changed:ModelSceneActorBase.GetModelUnitGUID` from redundant user-chat approval to the repository scope-exception mechanism. The row remains `exception-requested`/`impossible`; `AGENTS.md#intentional-gaps` is the validated authority for the permanent no-3D project boundary.

## [2026-08-06] system | Non-Lua exception evidence

Allowed `unsafe` and `impossible` manifest rows to omit fabricated Lua-path assertions only when item-specific evidence concerns provenance or another non-Lua boundary. Item-specific evidence and unique per-row informed approval remain mandatory for completion.

## [2026-08-06] investigation | 12.0.7 occurrence-level re-triage

Classified all 131 named 12.0.7 occurrences: 34 implemented and 97 best-effort, with no untriaged or exception-requested rows. Focused profile-gated tests cover safe globals, duration methods, CVar defaults, event registration, widget compatibility, and retained/removed compatibility surfaces. Exact service payload, secrecy, and load-order fidelity limits remain explicit best-effort notes; unnamed crawler claims remain metadata.

## [2026-08-06] investigation | 12.0.7 named CVar classification

Classified the six CVar additions actually named by the checked-in 12.0.7 crawler source as implemented. `patch_12_0_7_cvar_defaults_match_retail` verifies each profile-gated runtime/default value. The register now contains six implemented and 125 untriaged rows; unnamed crawler claims remain source metadata outside row classification.

## [2026-08-06] investigation | 12.0.7 approval-language correction

Removed stale claims that unnamed CVar source gaps, stale duration extraction rows, and Minimap removals were already approved exceptions. The 131-row machine manifest remains fully neutral/untriaged. Crawler-omitted CVar claims stay source metadata, and potential unsafe/impossible rows remain candidates until evidence-backed classification is presented for informed approval.

## [2026-08-06] investigation | Neutral 12.0.5 probe register

Created the checked-in 38-row probe source, machine manifest, generated checklist, and human inventory. Prior documentation states are preserved as 30 resolved, four best-effort, and four unresolved, while every machine row remains neutral/untriaged pending item-specific evidence. Removed stale broad-approval wording: Store dropdown/descendant evidence and XmlFrameLevel provenance remain unresolved, while same-size window transitions are an unapproved impossible exception candidate.

## [2026-08-06] system | Test-backed behavioral audit resolution

Added a generic `behavioral` resolution for patch occurrences that are not truthfully observable as Lua global/table paths, including event registration, widget methods, CVar defaults, and probe outcomes. Behavioral rows require hashed test evidence plus a focused named test, allow implemented/best-effort status, forbid Lua presence assertions, and consume no synthetic runtime observation.

## [2026-08-06] investigation | Neutral 12.0.7 occurrence register

Created the checked-in 12.0.7 machine manifest, generated checklist, and human inventory from the categorized source register. All 131 named occurrences remain neutral/untriaged: 79 added, 29 changed, and 23 removed. Changed occurrences preserve valid normalized symbol paths plus exact change details; crawler-omitted CVar names remain unresolved metadata and are not invented.

## [2026-08-06] system | Categorized 12.0.7 occurrence source

Added a generic categorized patch-source format and the raw 12.0.7 occurrence register. The source contains 79 added, 29 changed, and 23 removed named occurrences (131 total), grouped from globals, script-object methods, events, widgets, and the six CVar additions actually named by the crawler excerpt. The unresolved claims for fourteen unnamed additions and five unnamed removals remain explicit metadata rather than invented symbols.

## [2026-08-06] system | Generic changed-occurrence patch manifests

Extended the patch-audit manifest schema to preserve `changed:symbol` occurrences alongside added and removed rows. Source JSON may include a `changed` array; manifest metadata records `changed_count`, defaulting to zero for the existing 12.1 added/removed-only register. Focused tests cover direction IDs, count mismatches, source ordering, and backward compatibility.

## [2026-08-06] investigation | 12.1 broader fidelity re-triage

Reconciled the completed 432-row FrameXML register with eight broader 12.1 fidelity families that are outside that manifest. Aura containers, DurationTextBinding, and service-backed structures retain explicit best-effort contracts. UnitAura secrecy, private/forbidden security enforcement, standalone RadialProgress construction, and strict-removal timing are individually identified as unapproved unsafe/impossible exception candidates. Removed contradictory wording that described the same candidates as both approved and pending; no approval was requested or recorded.
## [2026-09-02] update | Blizzard_CombatLogBase replays into secureenv

Updated `systems/taint-system.md`: `Blizzard_CombatLogBase` joined the secure replay library list. PTR startup raised `Blizzard_CombatLogProcessor.lua:337: attempt to index global 'CombatLogUtil'` from `ChatConfigFrame`'s PLAYER_ENTERING_WORLD handler because the secure processor reads `CombatLogUtil`, `COMBATLOG_FILTER_MINE` and `COMBATLOG_DEFAULT_COLORS` from its plain dependency, which had only reached `_G`; the handler aborted before `hasEnteredWorld`. A generalized rule (replay every direct `## Dependencies` of a `UseSecureEnvironment` addon) was implemented first and rejected: secure TOCs declare `Blizzard_FrameXMLBase`, `Blizzard_ObjectAPI`, `Blizzard_SharedXMLGame`, `Blizzard_VisualAlerts` and others as `## Dep`, and re-running those in secureenv re-executed their top-level code — `Constants.lua` and `VisualAlertTemplates.lua` load errors doubled and `PrivateVisualAlertsManager` OnLoad gained a new `attempt to call a nil value`.

## [2026-07-16] update | FrameStrata before/after observations

Updated FrameStrata documentation from `/tmp/FrameStrataProbe-parent-retail.lua` (retail 12.0.7 build 68453, captured `2026-07-16T01:21:08`). During XML `OnLoad`, the actual parent and direct `PARENT` child both reported `DIALOG`, while the literal sibling reported `LOW`. Under an actual `DIALOG` parent, base `HIGH` reported `HIGH`, derived literal `LOW` reported `LOW`, and derived `PARENT` reported `HIGH`. After the tested parent-strata and reparent operations, every tested non-fixed child and grandchild reported `LOW`, including explicit XML `MEDIUM` fixtures. Documentation avoids claims about the client's internal resolution or propagation mechanism; the capture did not test `BLIZZARD`.

## [2026-07-14] investigation | 12.0.7 widget compatibility matrix

Focused 12.0.7 proof verifies six retained Minimap texture setters, four Button methods, four ScrollFrame methods, and five font-bearing `SetFont` methods. `ModelSceneActorBase:GetModelUnitGUID` is absent under the intentional permanent no-3D scope and remains an explicit exception candidate; no approval requested yet.

## [2026-07-14] investigation | 12.0.7 removal and event matrix

Added exact 12.0.7 startup proof for all 17 proposed global removals and 17 added/changed events. Eleven removed names are nil, six remain compatibility functions, and every event registers. This corrects the earlier blanket claim that all removed wrappers remained available.

## [2026-07-14] investigation | 12.1 final publication matrix

Closed the final 106 FrameXML rows with an exact all-LoD publication matrix: seven proposed additions remain nil and 99 proposed removals remain functions. Every mismatch reports symbol, expected type, and observed type. Full target: 15 tests passed in 22.35 seconds (26.048 seconds wall time). Final 12.1 FrameXML inventory: 1 implemented, 431 best-effort, 0 exception-requested, and 0 untriaged rows.

## [2026-07-14] investigation | 12.1 conservative source-absence batch

Classified 175 proposed additions individually as stale snapshot entries. Selection requires the bare method/global token to be absent from every PTR Lua/XML/TOC file; the focused generated test then loads the complete game-compatible addon closure, including LoD roots, and reports any exact global/namespace publication. Stronger source patterns cover dot, colon, bracket, and `rawset` forms for earlier namespace families. Some all-LoD addons emit recorded Lua errors, so this remains explicitly best-effort source-plus-runtime evidence rather than an exact fidelity claim. Full target: 14 tests passed in 21.82 seconds (24.836 seconds wall time). Current inventory: 1 implemented, 325 best-effort, 0 exception-requested, and 106 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 utility namespace and colon-publication audit

Corrected source scanning to cover both `Namespace.Method` and `Namespace:Method` Lua publications. Earlier stale families remain absent under the stronger falsifier. Classified 29 utility additions as stale and `PingUtil.GetContextualPingTypeForUnit` as vendor-present with tested `C_Ping` forwarding. Current full target: 13 tests passed in 15.20 seconds (18.309 seconds wall time). Current inventory: 1 implemented, 150 best-effort, 0 exception-requested, and 281 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 GuildControl snapshot mismatch

Classified ten proposed `GuildControlUI_*` additions as stale snapshot globals. Shared-corpus PTR source proof finds no occurrences and startup runtime keeps all ten nil. The first post-build target took 64.43 seconds wall time; the unchanged warm complete target passed 11 tests in 15.65 seconds (17.272 seconds wall time), satisfying the 60-second gate. Current inventory: 1 implemented, 120 best-effort, 0 exception-requested, and 311 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 Narration snapshot mismatch

Classified all 14 proposed `NarrationUtil` additions as stale qualified names. Shared-corpus PTR source proof and startup runtime enumeration keep the namespace nil. Full grouped audit target: 10 tests passed in 13.82 seconds (20.495 seconds wall time). Current inventory: 1 implemented, 110 best-effort, 0 exception-requested, and 321 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 SocialUI snapshot mismatch and source-scan budget

Classified all 13 proposed `SocialUIUtil` additions as stale qualified names. Exact-qualified PTR source proof and runtime enumeration keep the namespace nil. Adding a third recursive source scan first pushed complete-target wall time to 76.304 seconds; a shared `OnceLock` source corpus reduced the verified nine-test target to 14.49 seconds test time and 19.023 seconds wall time. Current inventory: 1 implemented, 96 best-effort, 0 exception-requested, and 335 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 friends-list snapshot mismatch

Classified all 29 proposed `FriendsListUtil` additions as stale qualified names. The falsifier first caught similarly named `FriendsFrame_*` globals; corrected exact-qualified PTR source proof shows no `FriendsListUtil.*` publications, and runtime proof confirms the namespace remains nil. Full grouped audit target: 8 tests passed in 10.32 seconds (12.541 seconds wall time). Current inventory: 1 implemented, 83 best-effort, 0 exception-requested, and 348 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 combat audio snapshot mismatch

Classified ten proposed `CombatAudioAlertUtil` interrupt/start/end/death additions as stale snapshot entries. Recursive PTR source proof scans Lua/XML/TOC files; runtime proof verifies the active namespace and representative real method exist while all ten proposed names remain nil. Full grouped audit target: 7 tests passed in 13.94 seconds (22.816 seconds wall time). Current inventory: 1 implemented, 54 best-effort, 0 exception-requested, and 377 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 retained UI geometry globals

Classified the proposed removals of `UIDoFramesIntersect`, `GetNotchHeight`, and `GetUIParentOffset` as vendor-present. Focused PTR proof covers overlap/separation/edge-touch behavior, physical-to-UI notch normalization, and maximum debug-bar/notch offset selection. Full grouped audit target: 6 tests passed in 13.438 seconds. Current inventory: 1 implemented, 44 best-effort, 0 exception-requested, and 387 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 input utility snapshot reversal

Classified five proposed `InputUtil` additions as stale snapshot namespace moves and four proposed global removals as vendor-present. Focused PTR proof verifies the namespace members remain nil while the legacy globals perform cursor scaling, frame-scale forwarding, mouse-offset forwarding, and inspect-cursor selection. Full grouped audit target: 5 tests passed in 9.232 seconds. Current inventory: 1 implemented, 41 best-effort, 0 exception-requested, and 390 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 screen-scale snapshot reversal

Classified the proposed `InterfaceUtil.GetScreenHeightScale` and `InterfaceUtil.GetScreenWidthScale` additions as stale snapshot entries and the proposed global removals as vendor-present. Focused PTR proof verifies `InterfaceUtil` is absent, both globals remain functions, and a 1024×768 fixture returns `1.0` for each. Full grouped audit target: 4 tests passed in 10.873 seconds. Current inventory: 1 implemented, 32 best-effort, 0 exception-requested, and 399 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 PTRFeedback quest-progress helper

Classified `GetTimeSinceLastQuestProgress` as vendor-present best-effort behavior. Focused PTR proof verifies publication by PTRFeedback and pins the current upstream nil-arithmetic invocation defect caused by undefined `lastProgressTime`; simulator adds no guessed correction. Full grouped audit target: 3 tests passed in 6.625 seconds. Current inventory: 1 implemented, 28 best-effort, 0 exception-requested, and 403 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 shake namespace mismatch

Classified proposed legacy `ShakeFrame` and `ShakeFrameRandom` additions as stale snapshot entries. Focused PTR proof verifies both globals remain nil while the distinct `ScriptAnimationUtil` methods exist and return cancellation functions for safe no-op conditions. Full grouped audit target: 2 tests passed in 5.774 seconds. Current inventory: 1 implemented, 27 best-effort, 0 exception-requested, and 404 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 PlayerChoice toggle LoD lifecycle

Classified both `PlayerChoiceToggle_TryShow` snapshot occurrences as best-effort load-on-demand vendor behavior. Focused PTR proof verifies absence before `Blizzard_PlayerChoice`, publication after explicit load, eligible-button visibility, explicit plus OnShow state updates, and nil return. Current inventory: 1 implemented, 25 best-effort, 0 exception-requested, and 406 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 smooth-progress snapshot reversal

Classified the proposed `InterpolatorUtil.GetSmoothProgressChange` addition as a reversed snapshot and the proposed global `GetSmoothProgressChange` removal as vendor-present. Focused PTR proof verifies the namespace member remains nil, the global remains a function, and representative input returns `70`. Current inventory: 1 implemented, 23 best-effort, 0 exception-requested, and 408 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 Macro save lifecycle

Classified both `MacroFrame_SaveMacro` snapshot occurrences as best-effort vendor-present behavior. Focused PTR proof verifies the eager UIParent no-op placeholder is harmless and explicit `Blizzard_MacroUI` loading replaces it with the `MacroFrame:SaveMacro()` delegate. Current inventory: 1 implemented, 21 best-effort, 0 exception-requested, and 410 neutral untriaged rows.

## [2026-07-14] system | Active-TOC patch source reachability

Added `--active-tocs` to the full-tree Lua publication index. The active compiled profile uses `find_toc_file` to select each addon's TOC, applies per-file environment rules, recursively follows XML script/include paths through the loader's addon-root fallback and case-insensitive resolver, records unresolved Lua/XML reference paths, and excludes source files selected only by other flavor TOCs. This corrects false candidate matches such as Mists-only helpers found by the raw all-files scan. Source selection remains candidate evidence; dependency order and LoD timing still require lifecycle tests.

## [2026-07-14] system | Full-tree patch source candidates

Added deterministic `--index-lua-tree` scanning with relative paths, per-file hashes, and first-directory addon ownership. Pre-lexer candidate counts were discarded after review exposed comment/string/local-scope false positives; corrected counts must come from a fresh scan. Results remain candidate-only. The later active-TOC entry applies active-profile source reachability; dependency order and LoD timing remain unapplied.

## [2026-07-14] system | Patch source candidates and initialization observations

Added `--observe-initialization`, which writes actual active-profile Lua observations and rejects manifest/profile mismatches. Added `--index-lua-source` for file/line direct-publication candidates plus explicit mixin/metatable/dynamic-global/factory ambiguity records. Candidate source evidence never changes final statuses automatically. Full manifest-driven post-load/LoD/reset orchestration remains open.

## [2026-07-14] system | Patch observation primitive

Added a production observation primitive that resolves actual Lua global/table paths in `WowLuaEnv` and records active profile, presence, and Lua type while carrying caller-supplied phase/addon labels. Focused coverage observes present and absent symbols, a real identity-matched TOC load transition, and exact manifest-byte hashing. A concrete post-reset runtime operation and full manifest-driven phase orchestration remain open.

## [2026-07-14] investigation | Preserve 12.0.7 API-change source

Moved the 12.0.7 Warcraft Wiki source snapshot from temporary storage into `data/patch-api/sources/12.0.7-api-changes.txt` and linked the patch audit to the checked-in evidence. The 12.0.7 manifest/register remains open work.

## [2026-07-14] system | Patch API audit manifest

Created `systems/patch-api-audit-manifest.md` and the 432-row `data/patch-api/12.1-framexml.json` register. Reviewer correction removed false blanket exception requests: 412 pending rows now have null status and neutral `untriaged` resolution. Repository validation recomputes source/evidence hashes, verifies tests and commit ancestry, and rejects checklist/inventory drift. Completion requires an exact-manifest observation artifact and per-item unsafe/impossible approval provenance; real per-row observation generation remains open.

## [2026-07-14] investigation | 12.1 CustomerOrders hide-wrapper mismatch

Updated the 12.1 FrameXML inventory after a recursive PTR source scan found no `HideProfessionsCustomerOrdersFrame` definition. A focused PTR test loads ProfessionsTemplates and AuctionHouse dependencies, explicitly loads `Blizzard_ProfessionsCustomerOrders`, verifies its frame exists, and confirms the snapshot-only wrapper remains nil. Current inventory: 1 implemented, 19 best-effort, 0 exception-requested, and 412 neutral untriaged rows.

## [2026-07-14] investigation | 12.1 Garrison hide-wrapper mismatches

Updated the 12.1 FrameXML inventory after confirming `HideGarrisonMissionFrames` and `HideGarrisonShipyardFrame` have no definitions in local PTR Blizzard sources. A focused PTR test loads `Blizzard_GarrisonUI` with its LoD dependencies and verifies both snapshot-only wrappers remain nil. At that intermediate stage, 1 row was implemented, 18 were best-effort, and 413 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] investigation | 12.1 BlackMarket hide-name mismatch

Updated the 12.1 FrameXML inventory after confirming PTR defines `BlackMarketFrame_Hide` with `HideUIPanel(BlackMarketFrame)` plus close-sound behavior, while snapshot entry `HideBlackMarketFrame` is absent. A focused PTR test explicitly loads `Blizzard_BlackMarketUI`, verifies the authoritative helper, and confirms the reversed-name wrapper remains nil. At that intermediate stage, 1 row was implemented, 16 were best-effort, and 415 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] investigation | 12.1 ItemUpgrade hide-name mismatch

Updated the 12.1 FrameXML inventory after confirming PTR defines `ItemUpgradeFrame_Hide` as the authoritative `HideUIPanel(ItemUpgradeFrame)` helper, while snapshot entry `HideItemUpgradeFrame` is absent. A focused PTR test explicitly loads `Blizzard_ItemUpgradeUI` and verifies the reversed-name wrapper remains nil. At that intermediate stage, 1 row was implemented, 15 were best-effort, and 416 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] investigation | 12.1 GuildBank hide wrapper mismatch

Updated the 12.1 FrameXML inventory after confirming `HideGuildBankFrame` has no definition in local PTR Blizzard sources. A focused PTR test explicitly loads `Blizzard_GuildBankUI` and verifies the snapshot-only wrapper remains absent. At that intermediate stage, 1 row was implemented, 14 were best-effort, and 417 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] investigation | 12.1 AuctionHouse hide wrapper mismatch

Updated the 12.1 FrameXML inventory after confirming `HideAuctionHouseFrame` has no definition in the local PTR Blizzard sources. A focused PTR runtime test loads `Blizzard_AuctionHouseUI` and verifies the snapshot-only wrapper remains absent instead of adding guessed close semantics. At that intermediate stage, 1 row was implemented, 13 were best-effort, and 418 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] investigation | 12.1 Mists-only time helper excluded

Updated the 12.1 FrameXML inventory after proving `GetTimeStringFromSeconds` is defined only by `Mists/UIParent.lua` and excluded from the PTR mainline TOC. It is classified best-effort as cross-flavor snapshot contamination rather than implemented behavior. PTR tests verify absence during environment initialization, after Blizzard loading/post-load compatibility, and after startup events. At that intermediate stage, 1 row was implemented, 12 were best-effort, and 419 unresolved rows were still mislabeled as exception requests; the latest manifest entry corrects them to neutral untriaged state.

## [2026-07-14] update | 12.1 DifficultyUtil delegates modeled

Updated the 12.1 audit and FrameXML inventory after adding five epoch-scoped, post-load `DifficultyUtil` color delegates. The delegates dynamically call the authoritative vendor globals, preserving arguments, both return values, later hotfix replacement, and explicit missing-global errors. Focused tests cover namespace reset/preservation and dynamic dispatch; full PTR Game UI startup verifies vendor threshold behavior, while older-retail startup verifies non-exposure. Inventory now records 1 implemented, 11 best-effort, and 420 pending strict exception re-triage.

## [2026-07-14] decision | Patch API audit exception approval superseded

A broad approval for documented 12.1, 12.0.7, and 12.0.5 exceptions was recorded, then superseded after review found that the full itemized checklist was not presented in chat and 12.1 FrameXML entries were mass-deferred without individual unsafe/impossible justification. Audits remain open pending re-triage and informed per-item approval.

## [2026-07-13] create | Mastery spells modeled, alias identity fix

Created `systems/specialization-mastery-spells.md` and
`investigations/deprecated-specialization-alias-identity.md` after modeling
`C_SpecializationInfo.GetSpecializationMasterySpells` from ChrSpecialization.db2
(retiring the empty-table temporary shim) and fixing three pre-existing test
failures: deprecated alias identity broken by post-cleanup re-registration, and
legacy specialization globals / UIWidgetContainerMixin duplicated in
`c_api/c_spec.rs` after their move to the Lua globals layer.

## [2026-07-13] update | Retail-only CASC isolation test

Updated `systems/casc-asset-cache.md` after adding
`scripts/test-retail-casc-isolation.py`. Documented Bubblewrap masking of all
non-retail WoW flavor directories, isolated writable caches, preserved failure
logs, exact missing-entry reporting, and the verified workflow for adding or
removing retail manifest entries.

## [2026-07-13] update | Docker headless release build

Updated `reference/addon-compatibility.md` and `systems/rendering-pipeline.md` after
Docker CI for v0.1.29 failed because the headless release build omitted the
required `client-retail` profile and `frame_collect` depended on the GUI-only
`hit_grid` module. Recorded the fixed build contract
(`--no-default-features --features client-retail`) and the shared `HitOrderKey`
ownership split; the next tag will carry the fix.

## [2026-07-13] update | Retail Blizzard UI manifest curation

Historical note superseded by the 2026-08-24 retail manifest correction above. The earlier 3,591-entry filtered snapshot and its legacy-profile exclusion claim are no longer the retail source-inventory contract.

## [2026-07-09] investigation | Patch 12.0.5 API audit

Created `investigations/patch-12-0-5-api-audit.md` to consolidate the probe-driven 12.0.5 work. Recorded that retail `12.0.5.67823` findings for forbidden frames, invalid unit-event filters, wildcard false attributes, Raise/Lower ordering, frame identity slot `[0]`, XML frame-level semantics, and display/scale event pairs are already modeled with focused tests. Documented that no `patch_12_0_5_inert_defaults` module exists and no obvious safe, already-backed 12.0.5 inert default remains unconverted. Updated with all 13 retained SavedVariables probe families, explicit best-effort boundaries, and exception requests for missing exact regressions, Store lifecycle evidence, XmlFrameLevel raw provenance, same-size window transitions, and the absent patch API-diff source.

## [2026-07-06] investigation | Patch 12.0.7 API audit

Created `investigations/patch-12-0-7-api-audit.md` after bridging compatible 12.0.7 API gaps and pausing exact-behavior work. Recorded additive/inert API bridges, verification logs, and blocked areas requiring live behavior: restricted unit-token returns, `ENCOUNTER_END` payloads, EncounterEvents color state, SimulateMouse taint/focus restrictions, debug secret propagation, secure raidtarget actions, M+ CalendarTime returns, aura security changes, widget secret aspects, and deprecated/removal timing. Updated after `C_BattleNet.InviteFriend` moved from inert bridge to modeled `SimState.bnet_friends` mutation. Updated again after ready-check behavior moved from inert `C_PartyInfo` bridge to modeled state: `DoReadyCheck`/`ReadyCheck`, `ConfirmReadyCheck`, `GetReadyCheckStatus`, `GetReadyCheckTimeLeft`, and immediate ready-check event dispatch. Updated after `C_UIFileAsset` moved from inert Lua defaults to best-effort limited-listfile lookup, after timeline event colors started mirroring the existing `C_EncounterEvents` color state, after `DurationTextBinding` gained documented non-secret state methods plus best-effort duration-object storage, after `GameTooltip_AddMoneyLine` started formatting money through `GetMoneyString` instead of appending raw copper, after `C_PartyInfo.IsGUIDInGroup` moved to the simulator party roster model, after C_PartyInfo leader/assistant mutators started updating simulator group-role state, after `C_PingSecure.ClearPendingPingOffScreenCallback` moved to the Rust shared callback table, after 12.0.7 CPU usage globals moved to the shared performance-metric defaults module, after `C_DurationUtil.CreateManualClock` moved to the Rust `C_DurationUtil` surface, after Delves/Housing/MerchantFrame/QuestHub trivial namespace defaults plus `C_PartyInfo.UninviteUnit` moved from the 12.0.7 Lua patch shim to Rust-backed best-effort/model-backed surfaces, after `C_EncounterTimeline.GetEventColor` moved to the Rust encounter-events surface while `GameTooltip_AddMoneyLine` moved to the shared formatting defaults, after secure pending button/ping/toggle callback globals moved to Rust-backed shared PingSecure callback storage, and after the recovered exact 12.0.7 CVar delta (17 adds, five removals, one default change) moved into profile-gated defaults with an explicit source-integrity exception for three unnamed claimed additions, and after `LuaDurationObject` gained best-effort clock storage plus deterministic `HasExpired`/`HasStarted`/`IsActive` methods for the documented 12.0.7 duration-object surface, and after the DurationText no-argument regression established that the stale `C_DurationUtil.CreateDurationTextFormattingOptions` / `CreateDurationTextRawValue` extraction names are universal-fallback nil functions rather than documented factories.

## [2026-07-06] investigation | Patch 12.1 API audit

Created `investigations/patch-12-1-api-audit.md` after bridging compatible 12.1 API gaps and pausing exact-behavior work. Recorded committed bridge points, verification logs, and the blocked areas that require live PTR behavior: UnitAura secrecy, Private Script Objects/Forbidden Partition, full ForbiddenAspect enforcement, AuraContainer/AuraButton/ManagedAuraContainer, DurationTextBinding/RadialProgress script objects, and exact structure payloads. Updated in the second pass to add an explicit implementation matrix and record 12.1 `DurationTextBinding` color-curve compatibility methods while leaving standalone `RadialProgress` paused. Updated again after Battle.net title-friend custom names/tags moved from inert Lua defaults to a best-effort `SimState.bnet_friends` model, after Encounter Journal difficulty helpers moved to generated-instance-data guesses, after `C_Discord.IsEnabled` started reflecting `discordClientEnabled`, after pending Battle.net friend invites gained a best-effort state model, after Battle.net feature probes started returning true for modeled friend-list/title-friend/tag support, after `C_Housing` owned-house/plot probes plus `ResetHouse` moved to local `SimState.housing` state pending replacement with probe-backed service semantics, and after safe `C_HousingBlueprint` share-code/import/export calls moved to local blueprint intent state pending exact PTR/service payload probes, and after housing editor/customize/decor/layout probes moved to local `SimState.housing` state with remaining blueprint availability calls itemized as pending exception requests until PTR/service payloads are known. Updated after `SetAppearOffline` moved to `SimState.bnet_appear_offline` and `BNCheckTitleFriendInviteToUnit` moved out of Lua inert defaults as a deterministic false best-effort probe pending title-friend service data. Updated again after Discord OAuth/link/settings/server/channel probes moved to local `SimState.discord` state and the final 12.1 Lua inert defaults were removed. Updated after housing blueprint availability probes moved from nil placeholders to local `SimState.housing` result codes pending exact service enum probes. Added explicit 12.1 exception requests for security-sensitive aura/private/forbidden/aspect behavior, standalone RadialProgress fidelity, full DurationTextBinding fidelity, exact service payloads, and strict-removal timing. Best-effort 12.1 bridges are explicitly temporary: keep them only while they are backed by existing simulator state and documented tests, then replace them with PTR/service probe-backed semantics once exact behavior is known. Updated after `LoadAddOnWithErrorHandling` was added as a tested canonical wrapper around `UIParentLoadAddOn`, and after the local 12.1 FrameXML snapshot was expanded into an exhaustive 320-added/112-removed inventory: the wrapper is implemented; the remaining 431 entries are explicit exception requests pending ownership/lifecycle evidence. Two names occur in both source lists, so the 432 entries represent 430 distinct names.

## [2026-07-06] update | Retail API epoch features

Updated `systems/client-profiles.md` after introducing cumulative retail API epoch features (`retail-12-0-7`, `retail-12-1-0`) alongside mutually-exclusive `client-*` profile features. Recorded that API surface gates belong on epoch features while PTR cache, CASC product, install paths, and vendor manifest behavior remain `client-ptr` profile concerns.

## [2026-07-02] update | XML method binding timing

Updated `systems/xml-template-system.md` after live PTR probing and simulator
regressions clarified XML `method="..."` behavior: XML binding installs the
currently composed method function as the script handler, object fields and
`GetScript` storage diverge after `frame.X = ...` or `SetScript`, and private
methods under `useForbiddenObjectTable` resolve from the forbidden object table
for AuraContainer-style XML.

## [2026-07-02] update | 12.1 XML partitioned mixins

Updated `systems/xml-template-system.md` after implementing PTR 12.1
`ScopedModifier useForbiddenObjectTable` and partitioned mixin semantics for
AuraContainer-style XML. Recorded public vs forbidden frame partitions,
partition-aware KeyValues, `targetPartition`, `inboundPartition` self
substitution, and `secureDelegates` public delegate behavior.

## [2026-07-02] update | Blizzard UI CDN missing chunks

Updated `systems/casc-asset-cache.md`, `systems/addon-loading.md`, and `PLAN.md`
after wiring Blizzard UI sync to fetch missing authoritative CASC chunks from
Blizzard CDN by encoding key via public `Osso/casc-extract` when local streaming
install archives are incomplete. Recorded that repo/source mirrors remain
disabled and CDN archive indexes persist under `~/.cache/casc-extract/`.

## [2026-07-01] update | Per-profile Blizzard UI manifests

Updated `systems/addon-loading.md`, `systems/casc-asset-cache.md`, and `index.md`
after splitting the Blizzard UI cache manifest into profile-specific files under
`data/blizzard-ui-files/`. Recorded that the active `client-*` profile selects
both the manifest and CASC product (`wow` vs `wowt`) so PTR-only addon files no
longer have to be shoehorned into the retail manifest. Documented that Blizzard
UI cache population has no repo-source fallback; tracked listfile overrides are
used only to teach CASC path→FDID mappings until upstream listfiles catch up.

## [2026-07-01] update | Bootstrap TOC semantics

Updated `systems/addon-loading.md` after correcting `[Bootstrap]` semantics:
annotated entries stay in normal TOC order, no standalone bootstrap pass runs,
and LoadOnDemand addons execute those files only during normal `C_AddOns.LoadAddOn`.
Updated `index.md` summary wording.

## [2026-06-29] investigation | Retail/PTR full startup Lua errors

Created `investigations/retail-ptr-full-startup-lua-errors.md` after full GUI
startup logs exposed handler-time errors missed by `lua-errors`. Recorded the
PVPUI API gaps (`C_WeeklyRewards`, `C_PvP`, legacy PVP role globals,
`ClearBattlemaster`), PTR cursor gap, Store inbound nil `StoreFrame` fallback
issue, and verification with clean retail/PTR startup logs. Updated `index.md`.

## [2026-06-29] update | Mists 5.5.4 lua-errors cleanup

Updated `investigations/mists-world-map-startup.md` after reducing Mists
5.5.4 startup `lua-errors` to `[]`. Recorded the root causes: escaped
`Interface/AddOns` XML paths, missing profile-cache manifest files
(`Blizzard_UIParent/Classic/*`, `Blizzard_SharedXML/Classic/GameTooltipTemplate.lua`),
`QuestUtil` being reset by `Blizzard_FrameXMLUtil/Classic/QuestUtils.lua`,
Mists XML referencing unshipped `WorldStateProvingGrounds_*` helpers, missing
native EditMode frame methods, and missing `UNIT_LEVEL_NON_ATTACKABLE` color.
Updated `index.md`.

## [2026-06-19] update | Bag button OnLoad load-order resolved

Marked `investigations/addon-load-order.md` RESOLVED. The historical
`PaperDollItemSlotButton_OnLoad`-before-definition failure no longer reproduces:
OnLoad completes (verified via `CharacterBag0Slot` event registration and clean
`lua-errors`), and the replay workaround was removed (`70fca4e25`, `d4f1287f9`).
Fixed dead `workarounds_bags.rs` / `l.rs` path references in both the wiki page
and the legacy `docs/addon-load-order-investigation.md`. Documented the root
cause: transitive `LoadFirst` — `Blizzard_EnvironmentCleanup` (`LoadFirst: 1`)
depends on `Blizzard_UIPanels_Game`, and the eager two-pass loader emits a
LoadFirst addon's deps first, so the definer loads before the bag buttons.

## [2026-06-19] update | PTR client profile

Updated `systems/client-profiles.md` after adding the `client-ptr` profile for
12.1 PTR. Documented the new `ptr` Blizzard UI cache scope, `120100` interface
version, mainline TOC/game-type behavior, and fallback preference for the
cache-managed `Gethe/wow-ui-source@ptr` archive before live/beta.
Updated again after the first PTR cache sync investigation: PTR sync now selects
the `wowt` CASC product automatically and filters legacy-profile manifest entries
before cache-completeness checks.

## [2026-06-19] update | Blizzard UI profile cache migration

Updated `systems/addon-loading.md` after moving runtime Blizzard UI loading to
profile-scoped user-cache roots under
`~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`. Documented
`wow-cli casc sync-blizzard-ui` as the canonical cache population path, the
completion/provenance marker pair, and the setup-script compatibility wrappers.

## [2026-06-12] investigation | Lib test failure sweep

Created `investigations/lib-test-failure-sweep-2026-06.md` after root-causing
nine accumulated `cargo test --lib` failures: runtime foundation order
(Blizzard_ScriptErrors), the silently-refused
`hooksecurefunc(C_AddOns, "LoadAddOn")` target (two workarounds rewired through
`apply_blizzard_post_load_patches`; one dead hook remains in
`runtime_surface_bootstrap.lua`), two more classic-rebase code losses, drifting
duplicated Lua installers, and tests stale against deliberate semantic changes.
Updated `index.md`.

## [2026-06-10] investigation | Retail core behavior probes

Created `investigations/retail-core-behavior-probes.md` after adding and
installing `CoreBehaviorProbe`. Recorded the retail `12.0.5.67823`
observations for `SetForbidden`, `CreateForbiddenFrame`, invalid
`RegisterUnitEvent`, wildcard false attributes, and the improved Raise/Lower
probe that needs a fresh live-client capture before simulator behavior changes.
Updated it after the fresh reload capture: `Raise()` and `Lower()` succeeded,
but simple shown sibling frames kept `GetFrameLevel()` at `1`/`10` and
`GetRaisedFrameLevel()` at `0` before and after under both a private parent and
`UIParent`.
Updated it again after fixing wow-ui-sim so `GetRaisedFrameLevel()` returns the
retail-observed `0` for the simple sibling probe while keeping internal
`raise_order` as render/hit-order bookkeeping.
Updated it again after adding explicit regression coverage for wildcard
`GetAttribute(prefix, name, suffix)` preserving a stored false value.
Updated it again after the hit-order capture showed the higher raw frame level
kept mouse focus before and after `Raise()`/`Lower()`; wow-ui-sim now treats
`raise_order` as a same-level tie-breaker instead of adding it to
`frame_level`.

## [2026-06-10] update | Frame identity token userdata

Updated `investigations/frame-surrogate-identity-slot.md` after switching `frame[0]` from a tiny backed table to a `FrameIdentity` userdata token. Recorded that DevTools dumping also needs raw frame iteration plus `dumpobject` returning nil so `[0]` renders as opaque userdata.
Updated it again after making `extract_frame_id` dispatch-aware and adding `native_frame_id_from_val` for the rare cases that need the original table backing.
Updated it again after real-client and wowless duplicate-name probes showed replacement frames get fresh identity and do not migrate custom Lua fields; recorded the simulator fix that removed old-global field copying during `CreateFrame` registration.
Updated `systems/addon-loading.md` after fixing third-party addon enable-state merging and startup keybinding import. Recorded that local `AddOns.txt` overlays real WTF state, required-dependency disables apply to default-enabled dependents, and `bindings-cache.wtf` imports before addon loading.
Updated it again after extending the existing `ServerSnapshot` addon to capture AddOn List enable state and keybindings. Recorded that `ServerSnapshotDB` is the preferred live-client overlay for addon state because `AddOns.txt` alone is not reliable for the per-character UI state.

## [2026-06-08] update | Mists 5.5.4 EditMode

Updated `investigations/editmode-layout.md` after bumping the Mists Blizzard UI
source pin to 5.5.4. Startup was clean with the existing `C_EditMode` fallback,
but a real frame tick exposed missing `MIRRORTIMER_NUMTIMERS` in
`WorldFrame_OnUpdate`; the mirror-timer Rust surface now publishes the constant.
The update also records that Mists now loads `Blizzard_UIParentPanelManager`
through its `_Classic.toc`, so the simulator no longer excludes that addon and
the cache manifest now carries the Classic panel-manager files.

## [2026-06-08] investigation | PlayerSpells runtime load

Created `investigations/playerspells-runtime-load.md` after fixing the retail `TOGGLETALENTS` keybind path. The note records the `C_AddOns.LoadAddOn` call-frame preservation issue plus the temporary PlayerSpells ModelScene/PvP talent backfills needed for `Blizzard_PlayerSpells` demand-load.

## [2026-06-08] investigation | ModelScene player actor stub

Created `investigations/modelscene-player-actor-stub.md` after Collectionator's transmog recovery helper crashed while calling `GetPlayerActor():SetModelByUnit("player")`. Documented the compatibility boundary: 3D rendering remains intentionally stubbed, but ModelScene actor object methods must exist for addon probes.

## [2026-06-08] update | FontString default anchors

Updated `investigations/fontstring-default-anchors.md` after follow-up retail JustifyProbe data showed XML `ButtonText` uses the same `justifyH` implicit anchor as layer `FontString`, explicit vertical-only anchors suppress the default, and EditBox backing FontStrings remain unanchored even with XML `TextInsets`.

## [2026-06-08] investigation | FontString default anchors

Created `investigations/fontstring-default-anchors.md` after real-client testing showed unanchored XML `FontString` layer children use `justifyH` for their implicit anchor point. Updated `index.md` with the new investigation page.

## [2026-05-28] investigation | action button icon mask coverage

Created `investigations/action-button-icon-mask.md` after tracing vanished main
action-bar icons to mask sampling, not action state. The prior minimap mask fix
made the shader sample RGB mask intensity; action-bar icon masks store coverage
in alpha, so their black visible regions went transparent. The renderer now
marks alpha-backed masks with a shader flag while retaining RGB coverage for
opaque black/white masks.

## [2026-05-26] update | EditMode cache with no saved vars

Updated `investigations/editmode-layout.md` with the `--no-saved-vars` status
tracking regression. EditMode profile cache files are not Lua SavedVariables, so
startup now loads them through a separate WTF cache path when Lua SavedVariables
are disabled.

## [2026-05-18] investigation | ElvUI tooltip skin ordering

Updated `investigations/tooltip-double-shell.md` after tracing Character panel item tooltips under ElvUI to direct `GameTooltip` skin textures rendering after the tooltip frame's internal text emitter. `GameTooltip` now renders direct texture regions before the tooltip frame/text while still deferring direct FontStrings above it.

## [2026-05-18] investigation | Mists full-addon login profile

Updated `investigations/talent-performance.md` with a fresh full-addon Mists login profile. Startup is currently dominated by third-party Lua compilation (`16.73s` compile out of `25.64s` third-party addon load, bytecode cache `1/1395` hits), with AllTheThings and RaiderIO DB addons as the largest contributors.

## [2026-05-17] investigation | Mists ElvUI startup compatibility

Created `investigations/mists-elvui-startup-compat.md` after the full-addon Mists probe isolated ElvUI startup failures to trim aliases, overexposed MessageFrame methods on plain frames, Mists AuraUtil tuple shape, and unanchored Slider label fontstrings.
Updated it after tracing ElvUI install text displacement and raid-control centering to fixed physical screen-size globals under `UIParent` scale; runtime screen-size changes now also dispatch display/scale events.
Updated it again after fixing ElvUI Chat initialization: Mists now provides `RedockChatWindows`, and the shared runtime surface provides `GetPlayerInfoByGUID`.
Updated it again after fixing ElvUI Tooltip font initialization: Font objects now share an object-type metatable, so ElvUI `FontTemplate` additions made through `GameFontNormal` are visible on `GameTooltipText`.
Updated it again after adding `GetInventoryItemDurability`; ElvUI DataTexts now sees the expected inventory durability global and the full-addon probe no longer reports that startup error.
Updated it again after tracing the ElvUI static-popup `OnUpdate` error to simulator-driven layout dirtying. The size/layout dirty path now snapshots and restores existing custom `OnUpdate` handlers across recursive layout-parent `MarkDirty()` calls, so `ElvUI_StaticPopup1` keeps ElvUI's handler after `E:StaticPopup_Show`.
Updated it again after tracing the ElvUI Installation close-button click failure to unscaled `SetHitRectInsets` in hit-grid construction. Hit rectangles now scale insets by the frame's effective scale before subtracting them from scaled layout rects.
Updated it again after finding the remaining ElvUI installer and raid-control placement issue: startup screen globals reported 1024x768 while `SimState`/`UIParent` still defaulted to 1600x1200, so ElvUI anchored a correctly sized parent inside the wrong root canvas.

## [2026-05-17] update | SetAtlas empty clear semantics

Updated `systems/texture-atlas.md` after tracing missing Character panel paper-doll elements to `SetAtlas("")` resolving the generated empty-name atlas entry (`Interface\castingbar\uicastingbarstandardflipbook`). Empty atlas names now clear texture/atlas state and clear propagated parent button slots, which restores ElvUI-stripped equipment slot rendering.

## [2026-05-17] update | frame field environment numeric slot

Updated `systems/frame-data-flow.md` after tracing the ElvUI/oUF aura `button:SetSize` startup error to frame field storage occupying raw numeric key `1`. Frame refs now keep addon array slots free while `debug.getfenv(frame)[1]` remains a compatibility view onto normal frame fields.

## [2026-05-17] update | Mists talent first-open latency

Updated `investigations/talent-performance.md` after tracing full-addon Mists talent first-open latency to a deferred AceAddon enable queue. BlizzMove's skipped `PLAYER_LOGIN` left 27 queued Ace addons until `ADDON_LOADED("Blizzard_TalentUI")`; allowing BlizzMove to receive login again makes the talent load itself sub-second, with remaining ElvUI login cost tracked separately.

## [2026-05-17] investigation | Mists panel stack overflow layout cycle

Created `investigations/mists-panel-stack-overflow-layout-cycle.md` after reproducing the Achievements/Talents abort through the real GUI click path. Documented that the root cause was active layout resolution re-entering through parent/anchor cycles, not the Lua open-panel path, and recorded the new `headless-click-probe` regression check.

## [2026-05-17] investigation | unanchored frame render leak

Created `investigations/unanchored-frame-render-leak.md` after tracing the startup stray editbox/dropdown to render-list fallback geometry for unanchored frames. The render path now matches Lua rect validity and skips descendants of unanchored frames too.

## [2026-05-17] update | minimap mask clipping

Updated `investigations/minimap-map-ring-alignment.md` after fixing minimap rendering to use the stored/default minimap mask texture instead of synthetic circle clipping. The shader now treats RGB mask intensity as coverage for opaque black/white masks.

## [2026-05-17] investigation | minimap map/ring correction

Created `investigations/minimap-map-ring-alignment.md` to record the correction that the active minimap bug is the map texture/mask/ring alignment, not the SimCommands minimap button. The note documents the reasoning error and directs future debugging at minimap mask/clip/ring geometry.

## [2026-05-15] add | Mists heirloom tooltip

Created `investigations/mists-heirloom-tooltip.md` after the Mists Collections
heirloom button path threw on missing `GameTooltip:SetHeirloomByItemID`.
Documented that the fix belongs on the tooltip widget/data surface and routes
through `C_TooltipInfo.GetHeirloomByItemID`, reusing item tooltip data.

## [2026-05-15] add | Mists addon-panel resume mistake

Created `investigations/mists-addon-panel-resume-error.md` after incorrectly
rerunning already-proven Mists addon panel rows from `AllTheThings`. The page
records the direct rule: resume from the first unproven addon, which was
`Plater` for this run, and use `--start-at` / stable artifact roots instead of
discarding retained evidence. The resumed `Plater` and `SimpleItemLevel` rows
now have retained pass artifacts under the shared cache audit root.

## [2026-05-15] update | Mists backpack slot chrome

Updated `investigations/backpack-background-texture.md` with the Mists-specific
container path: bag ID 0 uses `UI-BackpackBackground` and its item buttons still
keep the authored `UI-Quickslot2` normal texture. Documented that clearing those
normal textures in post-load code is the wrong fix for Mists slot chrome.

## [2026-05-14] add | Hybrid scrollbar thumb texture

Created `investigations/hybrid-scrollbar-thumb-texture.md` after replacing the
HybridScrollBar-specific placeholder fallback with XML-backed slider
`<ThumbTexture>` application. Also documented the SharedXML test helper bug:
tests were pointed at removed `Interface/BlizzardUI` instead of the simulator's
Blizzard UI cache.

## [2026-05-13] update | EditMode active profile fallback

Updated `investigations/editmode-layout.md` with the C_EditMode active profile
state bug: the bootstrap fallback always returned `activeLayout = 1` and had a
no-op `SetActiveLayout()`, so Blizzard selected the first preset layout instead
of a saved profile. Documented the in-memory fallback state model and regression
coverage.

Follow-up: documented the real WTF import path. EditMode layouts come from
`edit-mode-cache-account.txt`; active per-spec selection comes from
`edit-mode-cache-character.txt`. Startup now imports those cache files before
Blizzard addons load, and Blizzard addons use the saved-variable-aware loader.

Follow-up: documented the second-stage layout mapping bug. `C_EditMode` selected
the saved layout, but `EditModeManagerFrame.layoutInfo` prepended presets and
kept the old index, activating `Classic`; startup now remaps the active saved
layout after prepending presets.

Follow-up: documented the visible action-bar anchoring bug. The Widescreen
layout was active and systemInfo was seeded, but action bars stayed at the
temporary `TOPLEFT, 0, 0` anchor because the startup fast path skipped
`ApplySystemAnchor`.

Follow-up: documented main action-bar side art as the `HideBarArt` EditMode
setting. Sparse saved layouts now merge missing defaults from Blizzard's modern
preset map, and the action-bar bootstrap path applies those values without
calling handlers that require an initialized `actionButtons` Lua array.

Follow-up: documented broader action-bar saved setting replay. The startup
fast path now applies safe runtime effects for saved visibility, icon
count/scale/padding, page number visibility, show-grid state, and button art
without repacking saved anchors.

Follow-up: documented raw/display conversion for saved action-bar profile
settings. The cache stores compact raw slider values, so the action-bar fast
path now reads through Blizzard's `GetSettingValue()` before applying display
values such as icon-size percentages.

Follow-up: documented account-level EditMode setting application. Startup now
invokes Blizzard's `InitializeAccountSettings()` after rebuilding layout info,
so saved account toggles are applied rather than merely copied into
`accountSettings`.

Follow-up: documented sparse account cache reconciliation. Imported account
settings now merge over defaults so older profiles still receive default values
for newer account-level EditMode toggles before Blizzard initializes account
settings.

Follow-up: documented cast-bar lock fidelity. Startup no longer overwrites the
active saved profile's `LockToPlayerFrame` / `CastBarUnderneath` settings.

Follow-up: documented saved setting replay for seeded EditMode systems. Startup
now applies saved settings for systems that skip full `UpdateSystem()`, while
deferring unit-frame `BuffsOnTop` if the seeded frame has no `UpdateAuras()`
method and would otherwise add a ScriptErrors entry.

## [2026-05-13] update | CASC font path fallback

Updated `investigations/casc-fdid-1579624-root-debug.md` after the standard
font FDIDs (`615960`, `615958`, `615971`) failed through asset-resolver's
path fallback with lowercase `fonts/...` paths. Documented the fix: preserve
canonical `Fonts/...` casing in the bundled listfile, normalize lookup keys
without losing entry path casing, and skip noisy `resolve_bytes(fdid)` calls
when the CASC resolution cache has no font FDID entry.

Follow-up: documented the real rendering failure after the noise fix. Skipping
missing font FDIDs suppressed the error but selected a system fallback font.
Font loading now reads standard font bytes from local CASC archives by
path-to-encoding resolution or known encoding-key fallback.

## [2026-05-09] update | Wrath vendor source switched to Gethe

Updated `systems/addon-loading.md` and `systems/client-profiles.md` after
standardizing the Wrath 3.3.5 source on `Gethe/wow-ui-source` tag `3.3.5`
(`c4e0255f`). Documented that Wrath's symlink points at the checkout root
because Gethe's 3.3.5 tag stores `AddOns/` and `FrameXML/` at repo root,
unlike newer profiles that keep sources under `Interface/`.
Recaptured the Wrath startup snapshot against the Gethe source (128 distinct
startup messages). The snapshot file was later removed from the Mists-only
merge scope.

## [2026-05-08] update | Windows default GUI build and headless CI compile

Updated `investigations/windows-port-build.md` after reproducing MSVC `LNK1189`
in the default `cargo build --bin wow-sim` path. The root cause remains the
local `iced-dynamic` DLL, but the current fix is to make `fast-build` opt-in
rather than part of default features. Also documented the no-default test
compile contract: GUI/render tests need `cfg(feature = "gui")`, GUI benchmark
binaries need `required-features = ["gui"]`, and CASC examples need
`required-features = ["casc"]`.

## [2026-05-08] update | CASC Friz Quadrata root probe

Updated `investigations/casc-fdid-1579624-root-debug.md` with known-good
FDID `615960` / `fonts/frizqt__.ttf` resolution data, hashes, and extraction
proof for Windows root parser debugging.

## [2026-05-08] add | Windows CASC Blizzard taint

Created `investigations/windows-casc-blizzard-taint.md` after fixing Windows
startup against the CASC-synced Blizzard UI cache. Documented the TOC and
`Blizzard_` folder-name taint semantics, plus the runtime `C_AddOns.LoadAddOn`
stack-taint clearing needed for Blizzard/secure TOCs loaded under a tainted
caller.

## [2026-05-08] ingest | CASC FDID 1579624 root debug

Created `investigations/casc-fdid-1579624-root-debug.md` with the verified
FDID-to-path mapping, content key, encoding key, CRLF hash proof against Gethe
`12.0.5`, extraction proof, local build caveat, and root parser debugging
checklist.

## [2026-05-08] update | CASC resolution cache location

Updated `systems/casc-asset-cache.md` after moving generated CASC resolution
metadata out of repo `data/casc` and into the asset-resolver user cache. The
page now documents product/build-key scoped cache paths and automatic rebuild
behavior for missing or stale `resolution.sqlite`.

## [2026-05-03] add | PVE tabs direct offset

Added `investigations/pve-tabs-direct-offset.md` after tracing the Dungeons &
Raids bottom tab placement to XML direct `<Offset x="..." y="..."/>`
attributes being ignored unless they used nested `<AbsDimension>`.

## [2026-05-02] add | Root-region render order

Added `investigations/root-region-render-order.md` after tracing an inverted
root-region tie breaker. Documented that root-level regions should use ascending
creation order inside the same draw layer, matching child regions, and that
`Reverse(id)` made newer root regions draw underneath older ones.

## [2026-05-02] update | Dropdown base mouse handling

Updated `investigations/dropdown-intrinsic-script-chain.md` after shared input
dispatch gained `RegisterForMouse` state and `SetPropagateMouseClicks` parent
dispatch. Documented why dropdown-like widgets should not need per-dropdown
click shims when the issue is physical mouse registration or child hit targets.

## [2026-05-02] update | LFD queue verbs

Updated `investigations/lfd-dungeon-list-empty.md` after the LFD join path got
past the group-size gate and hit missing `ClearAllLFGDungeons`. Documented the
new `ClearAllLFGDungeons` / `SetLFGDungeon` / `JoinLFG` / `GetLFGInfoServer`
surface and queued-mode state through Blizzard's Lua `GetLFGMode`.

## [2026-05-02] update | LFD normal dungeon group-size gate

Updated `investigations/lfd-dungeon-list-empty.md` after a five-player party saw
`You need a group of 1 players` when joining LFD. Documented that normal dungeon
entries must return nil for `GetLFGDungeonInfo` slot 17 (`minPlayers`) because
Blizzard treats a non-nil value as an exact group-size requirement.

## [2026-05-02] update | LFD reward cap info missing

Updated `investigations/lfd-dungeon-list-empty.md` after the Join as Party path
advanced into `LFGRewardsFrame_EstimateRemainingCompletions()` and hit missing
`GetLFGDungeonRewardCapInfo`. Documented the inert 11-nil return shape Blizzard
uses as the no-cap path.

## [2026-05-02] update | LFD Join as Party format error

Updated `investigations/lfd-dungeon-list-empty.md` after clicking `Join as Party`
raised `bad argument #2 to 'string.format' (number expected)`. Documented that
`GetLFGDungeonInfo` returned `mapName` in Blizzard's `minPlayers` slot, so
`ERR_LFG_MEMBERS_REQUIRED` received a dungeon name where `%d` expected a number.

## [2026-05-02] add | Adventure Guide disabled tabs

Added `investigations/adventure-guide-disabled-tabs.md` after tracing the
apparently unclickable Adventure Guide abilities tab. Blizzard intentionally
shows the tab disabled until a boss is selected, but simulator model stubs had
overwritten the shared `SetDesaturated` / `SetDesaturation` implementation, so
the disabled tab art stayed saturated and looked active.

## [2026-05-01] update | Journeys breadcrumb overlap

Updated `investigations/journeys-midnight-empty.md` with the later Midnight
renown-card overlap root cause. The issue was XML property order:
`setAllPoints=true` was applied after explicit `$parentInset` anchors, clearing
them and stretching `EncounterJournalJourneysFrame` to the full parent.

## [2026-05-01] update | EditBox child background render ordering

Updated `investigations/editbox-render-text-cache.md` after tracing the SimCommands search box typed text to render ordering: the search box's opaque child `BACKGROUND` texture rendered after the EditBox frame, while the EditBox frame emitter owns the internal input text and caret. Documented the new EditBox-specific strata DFS rule that child regions render before the EditBox frame emitter.

## [2026-05-01] update | Tooltip Lua NineSlice center fill

Updated `investigations/tooltip-double-shell.md` after the Journeys renown-card tooltip showed underlying card text through the tooltip body. Documented that the Lua-owned tooltip `NineSlice` should suppress the Rust fallback border/shell, but not the solid center fill needed while the simulator does not have a renderable opaque Lua center.

## [2026-05-01] ingest | Appearances Wardrobe API baseline

Created `investigations/appearances-wardrobe-api.md` after opening Collections Journal > Appearances in the simulator and auditing Blizzard Wardrobe/Transmog call sites against the current `C_TransmogCollection`, `C_Transmog`, and `C_TransmogSets` surfaces. Documented that the panel opens with no Lua errors, but real browsing/filtering/search/favorite behavior needs stateful source, visual, filter, and search backing rather than no-op filter setters and empty Lua bootstrap fallbacks.

## [2026-05-01] update | Wardrobe weapon slot switch crash

Updated `investigations/appearances-wardrobe-api.md` with the root cause for the `Blizzard_Wardrobe.lua:687` nil-index crash. The simulator was reporting main/offhand appearance slot location metadata as weapon collection categories, which made Blizzard treat weapons as armor setup slots; weapon slot metadata now uses `Enum.TransmogCollectionType.None` so the weapon-category path handles them.

## [2026-05-01] update | Wardrobe invalid appearance overlays

Updated `investigations/appearances-wardrobe-api.md` after Wardrobe rendered every head appearance card with the red invalid overlay. Root cause was missing displayability/usability fields on simulator appearance rows; `canDisplayOnPlayer`, usability, source validity, hidden/favorite defaults, name, and quality now come from the `C_TransmogCollection` row backing instead of forcing Blizzard's invalid-card path.

## [2026-05-01] ingest | Adventure Guide boss icon fallback

Created `investigations/adventure-guide-boss-icons.md` after tracing blank Encounter Journal boss icons to `EJ_GetCreatureInfo` returning `0` for missing creature icon fileDataIDs. Documented that Blizzard's boss button Lua relies on nil to select `UI-EJ-BOSS-Default`; `0` is truthy and makes `SetTexture(0)` clear the texture.

## [2026-05-13] update | asset-resolver cache root decoupled from game-engine

Updated `systems/casc-asset-cache.md` after moving `asset-resolver` path selection behind an explicit resolver config. wow-ui-sim now constructs the resolver with `$ASSET_RESOLVER_CACHE_DIR` or the normal user cache root and no longer sets `GAME_ENGINE_SHARED_ROOT` or assumes a local game-engine checkout for CASC/listfile cache data.

## [2026-05-01] update | Adventure Journal dungeon click stack overflow

Updated `investigations/lfd-dungeon-list-empty.md` after reproducing the stack overflow from `EncounterJournal_DisplayInstance(1271)`. Documented the split between modern `C_EncounterJournal.GetInstanceInfo` slot 9 (`linkDungeonID`) and legacy `EJ_GetInstanceInfo` slot 9 (`shouldDisplayDifficulty`) plus the same-button `Button:Click()` reentry guard that prevents the programmatic overview-tab click loop from aborting the process.

## [2026-05-01] update | LFD Join as Party leadership gating

Updated `investigations/lfd-dungeon-list-empty.md` after `JOIN_AS_PARTY` was still greyed out with valid role and dungeon selection state. Documented that party-size fixtures had made `party1` the leader, causing Blizzard's `LFD_IsEmpowered()` to reject the local player; `A_Admin.SetPartySize` and GUI party-size changes now default to local-player leadership.

## [2026-05-01] update | Adventure Journal LFD dungeon handoff

Updated `investigations/lfd-dungeon-list-empty.md` after the Adventure Journal dungeon click path exposed another LFD id/state gap. Documented that `AJ_DUNGEON_ACTION` depends on `DungeonAppearsInRandomLFD` and on Encounter Journal `linkDungeonID` values using the LFD id family, not Encounter Journal instance ids.

## [2026-05-01] update | Crafting cast duration

Updated `investigations/crafting-cast-bar.md` after the crafting bar was found to finish too quickly. The simulator had reused a 1.5 second GCD-style duration for `C_TradeSkillUI.CraftRecipe`; normal profession crafts now use a 2.0 second default, with a regression in `tests/test_crafting.rs`.

## [2026-04-30] ingest | CASC asset cache layers and measured costs

Created `systems/casc-asset-cache.md` after measuring the three stacked caches end-to-end with `examples/casc_bench.rs`. The doc covers (a) the resolution sqlite shared with the game-engine repo via `GAME_ENGINE_SHARED_ROOT`, (b) the per-listfile-path BLP byte cache at `~/.cache/wow-ui-sim/casc-extract/`, and (c) the per-process `TextureManager` in-memory cache, with concrete timings (~300 ms one-time CASC init, ~10 ms steady-state extract, ~1 ms disk hit, ~2 µs mem hit). Records that `Installation::initialize` no longer parses `root.bin`/`encoding.bin` — that work is permanently delegated to the resolution sqlite — so the per-extract cost stays in the millisecond range.

## [2026-04-29] ingest | Backpack body renders gray, not textured

Created `investigations/backpack-background-texture.md`. User showed a retail
screenshot of an open `Backpack` (combined-bags) window with a tan/brown
textured body and reported the simulator was missing it. Render-time tracing
confirmed the sim emits a solid `PANEL_BACKGROUND_COLOR` quad on the
`Bg.TopSection`/`Bg.BottomEdge` textures — i.e. exactly what
`FlatPanelBackgroundTemplate` authors. Both the pinned `12.0.5` vendor and the
`Gethe/wow-ui-source` `live` HEAD (verified via WebFetch + Codex
gpt-5.5/high) define `ContainerFrameCombinedBags` with no body atlas/file and
a no-op `UpdateBackground`. Bank's tan body comes from a separate
`bank-frame-background` atlas declared on `BankFrame` itself, not shared with
the bag panel. Conclusion: the textured retail look is applied outside the
public Blizzard source we have (addon overlay, unmirrored patch, or an
unknown runtime path); closed without a sim-side change.

## [2026-04-30] ingest | client-profiles system page

Created `systems/client-profiles.md` documenting the five-profile cargo-feature layout (retail/wrath/mists/era/anniversary), vendor pinning, profile-aware TOC suffix and gametype tables, per-profile compat bootstraps (`src/wrath/`, `src/mists/`, `src/era/` shared by era + anniversary), wrath's synthetic FrameXML addon, and the CI matrix. Updated `systems/addon-loading.md` to fix the now-stale "Mainline-only TOC suffix" claims and link to the new page; updated `index.md` with a row in `systems/`. Source: PLAN.classic.md Phase 7.x landings (cargo features, profile-aware loader/toc, src/era/ bootstrap), commits 2915f2b9..6b320417.

## [2026-04-30] update | addon-loading per-profile vendor structure

Expanded `systems/addon-loading.md` with a new "Vendor sources & per-profile setup" section: documents the gitignored `Interface/BlizzardUI/<Profile>/` symlink layout, the `setup-blizzard-ui.sh <profile> [ref]` and `init-worktree.sh` scripts, and the canonical vendor-repo + pinned-SHA table. Added per-profile addon-set notes to "Blizzard Addon Load Order" (24 wrath + synthetic FrameXML, ~112 mists, ~35 era/anniversary discovered after `_Vanilla.toc` filtering), and listed `[[client-profiles]]` under See Also. Source: `scripts/setup-blizzard-ui.sh`, `scripts/init-worktree.sh`, vendor-pin commits 73ba3465..afc7189b.

## [2026-04-28] ingest | Windows port build unblock

Created `investigations/windows-port-build.md` after the Windows smoke pass. The root cause was the local `iced-dynamic` re-export crate forcing a huge `iced_dynamic.dll` link, which hit MSVC `LNK1189`; the build now depends on upstream `iced` directly. Verified `wow-sim`, `wow-cli`, GUI startup, and screenshot output on Windows. Updated the note after adding shared WoW resource discovery for install root, CASC `Data`, extracted Interface art, AddOns, and WTF. Live WTF is documented and tested as read-only import; simulator-local SavedVariables take precedence once present.

## [2026-04-28] update | EditMode group-frame startup skip

Updated `investigations/startup-createframe-profile.md` with the resolved
remaining `apply_system_anchors` hotspot. File-based Lua profiling showed full
`UpdateSystem()` on `CompactRaidFrameContainer`, `PartyFrame`, and
`CompactArenaFrame` dominated the first pass; the workaround now seeds those
frames and applies anchors without running full roster/unit layout work.

## [2026-04-28] update | remaining EditMode startup hotspot narrowed

Updated `investigations/startup-createframe-profile.md` with follow-up probes
on the remaining post-load `apply_system_anchors` cost. `InitSystemAnchors`
and the registered-system loop were effectively free; the cost stayed in
`UpdateBottomActionBarPositions()` / managed-frame layout. Recorded the
discarded forbidden-attribute no-op idea because it broke
`tests/frame_positions.rs` by shifting `ObjectiveTrackerFrame`.

## [2026-04-28] update | duplicate post-event EditMode pass

Updated `investigations/startup-createframe-profile.md` with the startup
workaround duplication found during dump-tree profiling. `PLAYER_ENTERING_WORLD`
already ran post-event workarounds through `env_events.rs`, then
`fire_startup_events()` and `settle_headless_startup()` ran the same pass again.
Added one-shot state for `WowLuaEnv::apply_post_event_workarounds`; local
dump-tree logs now show two `apply_system_anchors` passes instead of four.

## [2026-04-28] ingest | three-slice button tiling

Created `investigations/three-slice-button-tiling.md`. Escape menu red button
stripes came from standard `HighlightTexture` children rendering while the
buttons were not hovered. The red-button center atlas special case was removed;
atlas tiling uses source size, and highlight children now render only when the
parent button is hovered or highlight-locked. Additive overlays also skip the
shader brightness boost so active highlights do not amplify low-alpha atlas edge
pixels into visible stripes.

## [2026-04-28] update | eager animation_frame_ids_for_group

Updated `investigations/talent-performance.md` with the panel-open 1.3 FPS root
cause. `advance_animation_group()` called `animation_frame_ids_for_group` on
every group every tick (full linear scan of `anim_frame_to_anim`), but the
result was only consumed inside the `if group_finished` branch. Moved the call
into the conditional. Live-process flamegraph went from 63.9% CPU in that
function to 0.00%; total animation-tick cost dropped from dominant to 3.5%.

## [2026-04-28] update | talent strata repair skip

Updated `investigations/talent-performance.md` with the discarded strata repair
root cause. `set_frame_visible()` was building same-strata repair plans during
talent frame show even when `strata_buckets` was `None`, so the work was thrown
away. Added the guard in `try_repair_strata_buckets_after_show`; release
`bench_talents` subsequent opens dropped from roughly 211-282ms to 112-150ms in
this worktree.

## [2026-04-28] ingest | menu pool SetToDefaults size/anchor reset

Created `investigations/menu-pool-set-to-defaults.md`. Guild roster Mythic+ Rating dropdown rendered as a screen-spanning stripe because `Frame:SetToDefaults` did not reset size or clear anchors. `Menu.lua` `MeasureFrameExtents` reads `frame:GetSize()` from pooled element frames, so previous-user widths inflated each menu measurement. Two `SetToDefaults` registrations existed on the shared frame metatable; `map_frames::register_all` runs after `misc::register_all` so the map_frames version is the active one (the misc registration is dead code). Extended `map_frames::set_to_defaults` to call `frame.clear_all_points()`, `frame.set_size(0.0, 0.0)`, clear `width_is_text_auto`, clear `layout_rect`, and `remove_all_anchor_dependents_for(id)` — matching real WoW semantics documented in `Compositor.lua`. Verified: Guild dropdown now stays at 180×103 even after a 900px synthetic dropdown is opened first; previously it grew to 1036→1172. All 8 minimap_specialized tests still pass.

## [2026-04-27] update | shallow `issecretvalue` for pool releases

Updated `investigations/talent-performance.md` with a new "Spec→Talents Tab Switch (~3.5s)" section. The Spec→Talents tab switch was multiple seconds because `LoadTalentTreeInternal` rebuilds the tree on every Show (`refreshOnShow=true`), and `talentButtonCollection:ReleaseAll()` calls `issecretvalue(frame)` 3× per button. The Rust fallback recursed into the entire frame's table tree (~7.4ms/call). Added `value_is_secret_shallow` in `src/lua_api/globals/security/secret_values.rs` that only inspects direct slot taints on tables, used by `issecretvalue`/`canaccessvalue`/`canaccessallvalues`. `canaccesstable` keeps the deep walk so its accessibility semantics still detect nested secret strings. Result: ReleaseAll 2159ms → 2.6ms; tab switch 3500ms → ~90ms; all 45 security_api tests pass.

## [2026-04-27] ingest | hero spec dialog anchor investigation

Created `investigations/hero-spec-dialog-anchors.md` documenting two anchor resolution bugs in `HeroTalentsSelectionDialog`. (1) `xml_layer_batch.rs` emitted all textures before all fontstrings into the same Lua chunk, breaking XML document order so SpecImage's `relativeKey="$parent.SpecName"` ran before SpecName existed and fell back to the spec frame. (2) Runtime template path lacked the loader's `resolve_named_anchor_targets_for_frame` re-pass, leaving NodesContainer's `$parent.Description` anchor as an unresolved string. Both fixed; talent node icons now render inside their LIGHTSMITH/TEMPLAR panels instead of at the screen bottom.

## [2026-04-27] update | CASC migration: textures and fonts

Migrated texture and font loading off bundled extracts onto direct CASC reads from the live WoW install at `/syncthing/World of Warcraft/Data` (via the `asset-resolver` crate). Removed `./textures` (~1740 WebPs), the `~/Projects/wow/Interface` BLP fallback path, the `disk_cache_dir` field, and `src/texture_cache.rs`. `WowFontSystem::new()` became no-arg and pulled FRIZQT__/ARIALN/frizqt___cyr from CASC; a short-lived embedded FRIZ fallback was later replaced by CASC encoding-key fallback so the repo does not ship font bytes. Updated `docs/wiki/systems/texture-atlas.md`, `docs/texture-atlas-system.md`, `docs/rendering-pipeline.md`, `docs/wiki/reference/cli-commands.md`, and `docs/wiki/index.md` to drop references to the old curated paths and the now-obsolete `convert-texture` / `extract-textures` add-a-missing-texture flow. CASC is gated by the `casc` feature (default-on); set `WOW_SIM_CASC=0` to disable.

## [2026-04-26] ingest | dropdown intrinsic script chain investigation

Created `investigations/dropdown-intrinsic-script-chain.md` to document the ReputationFrame dropdown root cause: style dropdown templates replaced intrinsic `DropdownButton` scripts, so Blizzard's `OnMouseDown_Intrinsic` was not in the click path. Recorded the simulator-side fix, the fake menu fallback removal, and the two regression tests.

## [2026-04-26] update | api-coverage refresh, FUTURE.md retired

Folded `FUTURE.md` into `docs/wiki/reference/api-coverage.md` and deleted the root file. The old "three-layer" stub architecture (Hand-written / `c_stubs_api*.rs` / `generated_stubs.rs (~19K lines)`) has been replaced by per-namespace modules under `src/c_api/` with explicit `permanent_shims/` and `temporary_shims/` subtrees, matching the C-API boundary policy in CLAUDE.md. The wiki page now describes the actual module layout, calls out the shim sub-trees, and points to `wow-cli audit-api --gaps --format plan` as the live source of remaining work instead of a hand-curated task list. Removed the `FUTURE.md` source link.

## [2026-04-26] update | architecture-overview refresh, DESIGN.md retired

Folded `DESIGN.md` into `docs/wiki/design/architecture-overview.md` and deleted the root file. Corrected several stale claims: rilua provides native taint tracking (the previous "stubbed as always-secure" line was wrong — `issecure`/`issecurevariable` work; what's missing is `SetAttribute`/`SetForbidden` enforcement, now spelled out as a non-goal). Removed the dropped `generated_stubs.rs (~19K lines)` reference. Expanded the module diagram beyond `lua_api`/`widget`/`render` to also cover `c_api`, `iced_app`, `loader`, `event`, `xml`, `lua_bridge`, `texture`, and `sound`, with `c_api` called out as a peer of `lua_api` per CLAUDE.md. Updated `addon-compatibility.md` and `development-phases.md` to drop the fixed "127+ addons" number and reflect that the Blizzard UI tree under `Interface/BlizzardUI/` (`SharedXMLBase`/`SharedXML`/`SharedXMLGame`/`FrameXMLBase`/`FrameXMLUtil`/`FrameXML` + per-feature addons) loads, not just `SharedXML`. Removed source links to the deleted `DESIGN.md`.

## [2026-04-26] update | scaling-coordinates refresh

Verified `docs/wiki/design/scaling-coordinates.md` against current source and folded the standalone `SCALING.md` into the wiki page (root `SCALING.md` deleted). Most open items from the original note are done: `GetScreenWidth`/`GetScreenHeight`/`GetPhysicalScreenSize` are now installed dynamically by `install_screen_size_globals()` in `src/lua_api/env_runtime.rs` and re-run from `set_screen_size()`; the hardcoded `TOPLEFT (10, -10)` override in `main.rs` and the debug purple border are gone; layout `size` flows through `src/iced_app/render/rebuild.rs`. Updated file paths (`src/iced_app/` is a directory; `src/lua_api/globals.rs` no longer exists), clarified that the renderer runs in iced top-left Y-down with Y flipped in `Uniforms::new`, and trimmed open items to anchor Y-axis end-to-end docs and a `CENTER`-anchor resize regression.

## [2026-04-24] add | achievement panel hide investigation

Added `investigations/achievement-panel-hide.md` to document the achievement
panel hide fix. The simulator workaround now delegates to Blizzard's real
`AchievementFrame_ToggleAchievementFrame()` / managed `HideUIPanel` path, and
animation advancement now fires child animation `OnFinished` handlers so XML
outro hide scripts can run. Updated `index.md` with the new page.

## [2026-04-24] update | taint-system doc refresh

Updated `systems/taint-system.md` to match the current rilua implementation and added a Blizzard `issecure()` call-site matrix with test coverage references.
SecureHandler APIs now document fallback frame-ref/snippet/wrap/unwrap behavior,
state and attribute drivers now document shallow driver application,
secret-value accessors now document marked/tainted value tracking, and source
links now point at `security.rs` plus current frame-method helpers instead of
removed `security_api.rs`, `secure_env.rs`, and `combat_lockdown.rs` paths.
Refreshed the `index.md` summary.

## [2026-04-24] update | spell description token resolver

Updated `systems/lua-api.md` to record the shared spell-description token
resolver used by both `C_Spell.GetSpellDescription()` and
`C_TooltipInfo.GetSpellByID()`. The note covers the supported DB2 token
families (`$s1`, `$<damage>`, `$<shield>`, `${...}`, `$STR`, `$INT`, `$AP`)
and the reason for the shared path: keep spellbook/tooltips from exposing raw
placeholders or drifting from the C API result. Refreshed the `index.md`
`[[lua-api]]` summary.

## [2026-04-24] update | SimulationCraft spell coefficients

Updated `systems/lua-api.md` with the local SimulationCraft source used for
spell-description coefficients and variables. Recorded the concrete formulas
now mirrored by `src/spell_description_resolver.rs`: Avenger's Shield
`1.55 * AP`, Crusader Strike `1.4 * AP`, Shield of the Righteous `0.95 * AP`,
Eye Beam `$<dmg>` as `10 * 0.4026 * AP`, and Shield of Vengeance `$<shield>`
as `30% max health * (1 + versatility damage)`.

## [2026-04-23] add | quest scrollbar partial XML size investigation

Added `investigations/quest-scrollbar-partial-size.md` to document the
QuestScrollFrame scrollbar alignment bug. Root cause was the direct XML
property path ignoring partial `<Size x="..."/>` / `<Size y="..."/>` values
unless both dimensions were present. Updated `systems/xml-template-system.md`
to record per-dimension size resolution and added the new investigation to
`index.md`.

## [2026-04-22] add | layout lock inventory reference page

Added `reference/layout-lock-inventory.md` as the consolidated source of truth
for UI layout lock coverage. The page inventories baseline frame rect locks
(`tests/frame_positions.rs`) plus subsystem-specific lock tests for objective
tracker, main action bar, status tracking bars, bag bar, micro menu, chat
frame, compact raid manager, character/reputation panels, and buff
icons/durations. Updated `index.md` with a new `[[layout-lock-inventory]]`
entry under `reference/` for discoverability.

## [2026-04-22] update | buff aura onupdate perf lock-down to 0.5ms

Updated `investigations/on-update-dirty.md` with the AuraButton
`OnUpdate <=0.5ms` lock-down pass. Recorded the focused perf harness
(`tests/buff_aura_onupdate_perf.rs`), the pre-fix max (`~0.86ms`),
the dominant hotspot (`SetFormattedText` inside `UpdateDuration`), and the
post-fix max (`31.44us`). Documented the engine-side fixes:
`SetFormattedText` width-hint no-op fast path,
`securecall` direct multi-return fast path with protected fallback, and
non-visual `FrameRef.__newindex` bookkeeping updates.

## [2026-04-22] update | setpoint no-op fast-path optimization

Updated `investigations/on-update-dirty.md` with the post-optimization
`SetPoint` rerun after moving the same-anchor no-op bail-out to
`set_point()` before `ensure_no_anchor_cycle(...)`. Recorded the measured
before/after no-op totals (`3.626955us -> 1.392966us`, about `61.6%` lower)
and updated the control-flow notes to reflect that cycle detection now runs
only on real anchor changes.

## [2026-04-22] update | formattedtext and fontobject no-op fast paths

Updated `investigations/on-update-dirty.md` with the post-optimization
measurements for `SetFormattedText` and `SetFontObject` after
`text/formatting.rs` changes. Documented the new same-signature
`SetFormattedText` cache guard (enabled only while global `format` matches the
captured default), confirmed behavior parity for overridden `format`
(`format_call_probe_same_text_calls=100`), and recorded the large
`SetFontObject` no-op drop (`steady_same_total_us` from `6.104us` baseline to
`1.201us` in the rerun).

## [2026-04-22] update | setalpha no-op fast-path optimization

Updated `investigations/on-update-dirty.md` with the `SetAlpha` fast-path
optimization follow-up in `core_state/alpha.rs`. Recorded a post-change
rerun of the existing `setalpha_bench.lua` microbenchmark and the key no-op
delta metric (`same - get`) showing lower measured no-op overhead than the
earlier baseline, plus the reminder that `noop_hot_setters` behavioral
contracts still pass.

## [2026-04-22] update | no-world-map retained trace after bc-negative cache

Updated `investigations/world-map-texture-loading-budget.md` with a fresh
no-world-map retained GUI trace captured after the BC-negative cache change.
Recorded the exact command and before/after peak timings from the trace logs:
`draw textures` dropped `482.2ms -> 283.2ms` and `bc_parse` dropped
`240.6ms -> 139.2ms`. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` with the new no-world-map baseline deltas.

## [2026-04-22] update | setalpha no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetAlpha` microbenchmark baseline from a headless `--exec-lua` run. Recorded
batch timings (`empty`, `GetAlpha`, same-value `SetAlpha(1)`, and alternating
state-change `SetAlpha`) and the required split:
Lua->Rust call overhead, same-value fast-path overhead, and real
state-change overhead.

## [2026-04-22] update | setformattedtext no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetFormattedText` microbenchmark baseline from a headless `--exec-lua` run.
Recorded the three required splits (argument formatting/parsing cost,
text-equality fast-path cost, and real text-change cost) and added a runtime
probe proving global `format()` is still called on same-text no-op updates.

## [2026-04-22] update | setpoint no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetPoint` differential microbenchmark baseline. Recorded split timings for
implicit-noop parse baseline, explicit target normalization/lookup overhead,
string target lookup overhead, a no-op equivalence-check proxy, and full
relayout/dirty extra cost. Also documented the static control-flow ordering in
`anchors.rs` proving anchor resolution and `ensure_no_anchor_cycle` run before
the no-op `unchanged` bail-out in `apply_set_point`.

## [2026-04-22] update | fontobject shown vertexcolor no-op baselines

Updated `investigations/on-update-dirty.md` with measured no-op and change-path
splits for `SetFontObject`, `SetShown`, and `SetVertexColor`
(`dispatch`, `pre-bail`, `true state-change`). Recorded the steady-state
comparison against the earlier `SetAlpha` baseline and pinned
`SetFontObject` as the highest remaining no-op cost in this primitive set.

## [2026-04-22] update | world-map Lua retry simplification

Updated `investigations/world-map-texture-loading-budget.md` with the
follow-up Lua pin retry cleanup after the request-handle refactor. The
`MapExplorationPinMixin` workaround now keeps only one deferred refresh at a
time instead of recursively re-arming the old timer loop, and the world-map
detail tests still pass. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` to mention the simplified Lua retry layer.

## [2026-04-22] update | world-map live retained trace recapture

Updated `investigations/world-map-texture-loading-budget.md` again with the
current HEAD live-GUI recapture. Recorded the retained startup sequence from
`ToggleWorldMap()` through `tick -> draw -> prepare -> present`, using the
exact `iced-debug` socket emitted by the process for the screenshot burst. The
first world-map tick was `dirty=0x1ff pending=true ready=6`; later draws
advanced atlas-ready `6 -> 24 -> 33 -> 282 -> 335`; the first post-present
world-map screenshot was already textured; and redraws continued after
`pending=false` because `strata_dirty` remained `0x1c`. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget`.

## [2026-04-22] update | world-map RedrawAll verification

Updated `investigations/world-map-texture-loading-budget.md` with the
timer-path `RedrawAll` verification from the same live retained-GUI repro.
Recorded that the first post-warmup frame reached `draw -> prepare -> present`
without any extra presentation gate after texture warmup, with
`ready=38 -> 335` as `prepare()` drained the atlas backlog. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget` to mention the
verified post-warmup frame path.

## [2026-04-22] update | world-map atlas tier pressure audit

Updated `investigations/world-map-texture-loading-budget.md` with the atlas
tier pressure audit from the same live GUI repro. Recorded that every observed
`prepare()` pass kept `retry=0` and `force_rgba_retry=0`, so there were zero
RGBA fallback failures and zero BC upload rejections while the world-map tile
paths drained queued work and reached atlas-ready completion. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget` with the
no-rejection conclusion.

## [2026-04-22] update | world-map retained GPU buffer reupload audit

Updated `investigations/world-map-texture-loading-budget.md` with the retained
GPU buffer reupload audit from the same live GUI repro. Recorded that
`upload_strata` showed the dirty world-map strata being re-written with
`pending_tex_vertices=0` and resolved sample `tex_index` / UVs, proving the
first-open retained path was re-uploading the affected vertex buffers after the
atlas transition. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` with the buffer-reupload conclusion.

## [2026-04-22] update | world-map retained texture display follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the latest
live-GUI follow-up. Documented that the screenshot path could render the world
map on first pass while the retained GUI still missed random tiles/overlays,
then recorded the four later bugs behind that gap: wrong warmup request source,
full-rebuild sentinel clobbering, inverted world-map request priority, and the
remaining `textures_pending` ownership conflict between queued preload and
draw-time retained recovery. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget`.

Later that day, updated the same page again after the deeper state-model fix:
`gpu_uploaded_textures` was only a draw-staging set populated before
`prepare()`, not proof that a path was ready in the atlas. Recorded the new
atlas-ready tracker populated from `WowUiPrimitive::prepare()` and the switch
away from using the staging set for display-readiness checks.

## [2026-04-22] update | buffframe slow onupdate interpretation

Updated `investigations/on-update-dirty.md` with the current BuffFrame
slow-handler interpretation. Documented that logs like
`addon=Blizzard_BuffFrame handler=OnUpdate frame=#21946` map to anonymous
`AuraButtonTemplate` children, not the named `BuffFrame` root, using the
current `BuffFrame.lua` creation path, `handler_timing.rs` fallback formatting,
and a live `dump-tree --filter-key BuffFrame --visible-only` run that showed
five visible anonymous timed buff buttons. Also corrected the stale audit note
to match the current `onupdate_handler_audit.rs` regression coverage.

## [2026-04-21] update | world-map exploration non-current map surface

Updated `investigations/world-map-fog-of-war-overlay-model.md` with a
follow-up regression where `runtime_surface_bootstrap.lua` was still installing
synthetic `C_MapExplorationInfo` handlers and limiting explored overlays to
`currentMapID` / map `1`. Documented the new Rust-backed
`src/c_api/c_map_exploration_info.rs` implementation, fallback-only bootstrap
stubs, and focused non-current-map regressions in
`tests/c_map_exploration_info.rs`. Refreshed the `index.md` summary row for
`world-map-fog-of-war-overlay-model`.

## [2026-04-21] add | class talents edge frame levels

Added `investigations/class-talents-edge-frame-levels.md` documenting the
class-talents edge-over-icon regression and the fix in
`src/lua_api/workarounds.rs` that patches both the mixin and the live
`PlayerSpellsFrame.TalentsFrame` method, then re-levels active edges.
Recorded regression coverage in
`test_class_talent_edges_render_below_visible_talent_buttons`
(`tests/hero_talents.rs`) and updated `index.md`.

## [2026-04-21] update | hero talents visibility and edges

Updated `investigations/class-talents-trait-loadout-state.md` with the hero
subtree rendering regression where only one node appeared and connector edges
were missing. Documented root cause in
`check_spec_conditions_met()` (`src/lua_api/globals/missing_surface/traits.rs`):
spec-set conditions were treated as `AND` instead of `OR` across shared hero
node groups. Recorded the fix and new regression test
`test_active_hero_subtree_exposes_multiple_visible_nodes_and_edges` in
`tests/hero_talents.rs`. Updated `index.md` summary for the investigation page.

## [2026-04-20] investigate | tooltip layout timing

Added `investigations/tooltip-layout-timing.md` to capture the tooltip
one-frame mismatch caused by sizing after layout resolution. Documented that
`update_tooltip_sizes()` runs too late in the live render path, so the current
frame can render from stale `layout_rect` data even though tooltip line data is
fresh.

## [2026-04-20] ingest | tooltip double shell

Added `investigations/tooltip-double-shell.md` to capture the duplicate
tooltip chrome bug. Documented the two-layer root cause: a bootstrap-created
fake `NineSlice` surface on the Lua side plus an unconditional Rust fallback
tooltip background. Recorded the fix: remove the bootstrap injection, repair
tooltip `NineSlice` post-load with the real template-backed surface, and gate
the Rust fallback shell on whether the frame already owns `NineSlice`.

## [2026-04-20] update | blizzard ui test lanes

Added `reference/blizzard-ui-test-lanes.md` and updated
`reference/addon-compatibility.md` to document the explicit split between
Blizzard UI unit tests and addon-bootstrap coverage.

## [2026-04-20] update | blizzard ui addon closure resolver

Added `loader::discover_blizzard_addon_closure_for_screen()` and switched the
render-order test helpers to use it. The resolver walks TOC `Dependencies` and
`OptionalDeps` across the full screen-allowed Blizzard TOC set, so tests can
resolve explicit closures for load-on-demand roots instead of relying on a fake
monolithic Blizzard bundle.

## [2026-04-20] update | blizzard ui smoke targets

Updated `reference/blizzard-ui-test-lanes.md` with the first four explicit
addon-bootstrap smoke targets: combat log, macro UI, world map, and
settings panel. Added the shared smoke-target manifest and harness coverage in
`tests/common/blizzard_addon_manifest.rs`,
`tests/common/blizzard_addon_harness.rs`, and
`tests/blizzard_addon_smoke_targets.rs`.

## [2026-04-20] update | blizzard ui smoke target startup shape

Updated the smoke-target lane so it asserts target startup shape instead of
just "closure loads". The harness now preloads shared panel support, clears
that preload noise, then loads only the target closure; each target asserts no
recorded Lua errors, the expected global/frame pair, and one or two
representative behaviors.

## [2026-04-20] update | blizzard ui addon-bootstrap test home

Documented that addon-bootstrap regressions stay in `cargo test`, while
`wow-sim run-tests` remains the lane for addon-authored Lua suites. The key
tradeoff is that the smoke harness needs direct Rust access to loader state,
recorded Lua errors, and frame-tree assertions, which is easier to maintain in
the normal Rust test binaries than in a Lua-only wrapper.

## [2026-04-20] update | keybinding spellbook direct dispatch

Updated `investigations/keybinding-system.md` with the spellbook follow-up.
Documented that the simulator now routes `S` back to Blizzard-owned
`PlayerSpellsUtil.ToggleSpellBookFrame()`, removes the bootstrap spellbook
fallback wrapper, and dispatches simple zero-arg binding targets directly
instead of always compiling a Lua chunk first. Recorded the key finding that
raw Blizzard spellbook toggles were already correct and the remaining
first-open regression only happened through binding dispatch.

## [2026-04-20] add | class talents trait loadout state

Added `investigations/class-talents-trait-loadout-state.md` to capture the
remaining `C_Traits` restore that unblocked `PlayerSpells`. Documented the root
cause (live talent state existed, but Blizzard-facing trait queries still
returned placeholder config IDs / staged-change surfaces), the new
working-vs-committed diff model exposed through `TalentState`, and the focused
regression coverage for config mapping, purchase gating, and staged edits.
Updated `index.md` with the new page.

## [2026-04-20] ingest | partyframe portrait composition

Added `investigations/partyframe-portrait-composition.md` to capture the
party portrait sizing/composition result from live queries and Blizzard XML.
Documented that the class icon is the `37x37` `Portrait` texture, while the
visible ring/surround is not a separate widget and instead comes from the
larger `UI-HUD-UnitFrame-Party-PortraitOn` frame-art texture (`120x49` in the
live master GUI tree). Updated `index.md` with the new page.

## [2026-04-19] ingest | partyframe status-bar texture drop

Added `investigations/partyframe-statusbar-textures.md` to capture the root cause for the party health/mana bar `MISSING` render. The XML loader creates the bar child correctly, but `SetStatusBarTexture(bar)` passes a userdata frame into a setter that only accepts strings/numbers, so the status-bar source is cleared. Updated `index.md` with the new page.

## [2026-04-18] update | Track 2 metatable handle threading

Updated `investigations/track-2-intern-audit.md` with the current
handle-threading progress: a shared `hot_metatable_key(...)` accessor
now reuses the prewarmed registry handle for `__index` / `__newindex`
lookups in `methods.rs`, `globals/create_frame/helpers.rs`,
`globals/security.rs`, and `env_init/freeze_globals.rs`, with a static
fallback for bootstrap-skipping tests.

## [2026-04-17] ingest | rilua vs mlua gap audit

Added `investigations/rilua-mlua-gap-audit.md` after comparing the current
rilua registration path against `master`'s mlua-era Lua API surface. Recorded
the highest-signal missing handling buckets: no-op sandbox cleanup in
`env_init`, dropped `MessageFrame` method registration, unfinished
attribute/event/text widget parity, and the unwired `patch_namespace_stubs()`
runtime hook. Updated `index.md` with the new investigation page.

## [2026-04-17] update | startup XML loader fast-path follow-up

Updated `investigations/startup-createframe-profile.md` with the current XML
loader fast-path state after the runtime `CreateFrame` work. Recorded the
safe widening steps in `xml_frame/setup.rs` and the split
`template_chain/` machinery, the current clean `xml fast path` counters
(`hits=1868`, `slow=350`, `scripts=234`), the shared-worktree debug startup
range (~`4.8s`-`5.8s` on `--no-addons --no-saved-vars`), and the failed
generic global-method literal-arg experiment that still regresses real
startup. Refreshed the `index.md` summary for the page.

## [2026-04-16] update | intern_string perf re-profile correction

Corrected the earlier PLAN/wiki summary for the post-migration interning
profile. Fresh release `perf` on `wow-sim --no-saved-vars lua-errors` shows
`Gc::intern_string` at 179.5M cycles (2.98%), `StringTable::intern_hashed`
at 169.2M (2.81%), and inline `lua_hash` at only ~4.8M (0.08%). The original
"lua_hash essentially flat" note was wrong; the hash primitive is no longer
the bottleneck, and the remaining cost sits in bucket traversal / dedup work.

## [2026-04-16] update | intern_string_static mid-cycle fix + migration landed

Found the root cause of the earlier migration breakage. `intern_string_static`
inserts into `static_intern_cache`, but `mark_gc_roots` only scans that cache
at cycle start — a mid-Propagate insert is still pre-flip current-white, gets
swept at cycle end, and the cache ends up pointing at a freed slot. Fixed in
rilua by colouring the new ref Black during Propagate/Sweep/Finalize; added
`intern_string_static_mid_cycle_survives_sweep` regression test.

Applied the migration for `registry_get/set/table_or_create`, `registry_table`,
`attach_frame_metatable`, and fan-out callers — intern counter 1,250,287 →
1,096,266 (−12%), release startup 1.18s → 1.15s median (n=5). `frame_ref_cache`
(the biggest single call site, 286K/startup) remains deferred: even with the
GC fix, migrating that path cascades into "OnLoad (a nil value)" failures
for ~300 addons and the root cause isn't yet understood.

## [2026-04-16] ingest | intern_string call-site ranking

Added `investigations/intern-string-ranking.md` and the rilua
`intern-stats` feature. Startup runs 1.25M `intern_string` calls with
top 5 literals accounting for 40%. Attempted migrating the
registry/frame helpers to `intern_string_static` (counter dropped
−74.5%) but release triggered 22 new Lua errors — frames lose their
metatable methods when `registry_set` uses `intern_string_static`.
Reverted the migration; filed as a rilua follow-up.

## [2026-04-16] ingest | layout computation profile

Added `investigations/layout-profile.md`. Release `perf` on `lua-errors`
showed layout-attributed samples at 7.5% of total (470M of 6.3B), with
the biggest single contributor being `LayoutCache::get` via siphash —
the cache was a default `HashMap<u64, CachedFrameLayout>`. Swapped to
`FxHashMap`. Layout 7.5% → 5.0%, total siphash 295M → 76M, release
startup median 1.21s → 1.18s (n=10). Remaining layout cost is real
anchor arithmetic and `resolve_parent_rect` recursion — no further
easy wins.

## [2026-04-16] update | table rehashing fix #2 declined

Tried fix #2 (short-circuit `raw_set_impl` for sequential integer keys
when hash is empty). Two variants both net-negative:
- `array.push` (grow by 1): rehashes 69K → 86K (+25%), wall time tied.
- `array.resize(next_power_of_two)`: rehashes 69K → 57K (−18%) but
  wall time +30% from eager nil-fill on each boundary.

Conclusion documented in `investigations/table-rehashing.md`: the
existing rehash path's `compute_sizes` already does good amortization;
naive replacements either skip the over-allocation (more rehashes
later) or pay the over-allocation eagerly (worse wall time). Status
quo retained.

## [2026-04-16] update | table rehashing fix #1 applied

Applied fix #1 from `investigations/table-rehashing.md`: rilua
`OpCode::NewTable` now pre-allocates 4 hash slots when both size hints
are zero. Measured: 97,340 → 69,105 rehashes (−29%); release startup
median 1.31s → 1.21s (−8%, n=5 each). All 685 rilua tests pass (includes
new `op_newtable_empty_hint_preallocates_hash_to_avoid_first_rehash`).

## [2026-04-16] ingest | table rehashing investigation

Added `investigations/table-rehashing.md`. Profiled startup rehash counts via a
new `rehash-stats` feature in rilua. Found 97K rehashes — 98% from non-frame
tables (`OP_NEWTABLE(0,0)` from addon `local t = {}` pattern), 81% landing at
hash size ≤ 16. Frame-table pre-sizing (64 slots) is working: only 1.6K frame
rehashes. Documented three candidate fixes; none applied in this card.

## [2026-04-14] ingest | world-map exploration seed follow-up

Updated `investigations/world-map-fog-of-war-overlay-model.md` with the current
map exploration follow-up. After removing synthetic fog, Isle of Dorn still
showed fully explored because every default-visible overlay was treated as
discovered. Documented the temporary seed that leaves one real overlay chunk
(`WorldMapOverlay.ID = 4885`, The Three Shields / Skolzgal Mill) unexplored
until per-character exploration state exists, and refreshed the `index.md`
summary.

## [2026-04-14] ingest | world-map fog-of-war overlay model correction

Updated `investigations/world-map-fog-of-war-overlay-model.md` to correct the
previous diagnosis. The current world map does not have a `UiMapFogOfWar` DB
row, so the bug was not a wrong irregular fog shape; it was the simulator
inventing fog for any map art and rendering synthetic geometry from exploration
overlay gaps. Documented the DB-backed fog lookup, the removal of the synthetic
renderer, and the new API/render regressions. Updated `index.md` with the
corrected summary.

## [2026-04-14] ingest | world-map fog-of-war overlay model

Created `investigations/world-map-fog-of-war-overlay-model.md` for the third
world-map fog bug: exploration APIs were already using real irregular overlay
chunks, but the fog renderer still assumed a synthetic half-map model.
Documented the root cause, the `FogOfWarFrame` `uiMapID` plumbing, the new
overlay-complement fog geometry, and the focused API/render regressions.
Updated `index.md` with the new investigation page.

## [2026-04-14] ingest | world-map texture loading budget follow-up

Updated `investigations/world-map-texture-loading-budget.md` with a first-frame
world-map follow-up: BC-preloaded tiles were landing in `bc_cache`, but
`TextureManager::is_cached()` only consulted the RGBA cache. That caused
budgeted draw to pause early after the first BC upload and could make the
world map open with an apparent quarter-map fog/exploration artifact. Added
the BC-cache root cause, fix, and regression coverage to the investigation
page.

## [2026-04-14] ingest | world-map fog-of-war first-open size

Created `investigations/world-map-fog-of-war-first-open-size.md` for the fog
overlay bug where first-open world-map fog could keep a stale size even though
the map tiles were already correct. Documented the missing
`FogOfWarPinMixin:OnCanvasSizeChanged()` handling, the simulator-side
workaround that patches both the mixin and existing fog pins, and the focused
regression tests. Updated `index.md` with the new investigation page.

## [2026-04-14] investigations | world map 90s OnUpdate recapture

Updated `investigations/world-map-onupdate-hover-polling.md` with the fresh
90s world-map profile after the recent OnUpdate fixes. Recorded the new
`/tmp/worldmap-onupdate-20260414.log` numbers (`485` total `fire_on_update`
spikes, `31` steady-state handlers, `64.73ms` post-90s average), added the
new `world_map_onupdate_inventory` handler-ceiling regression test, and
refreshed the `index.md` summary for the page.

## [2026-04-14] investigations | startup XML lifecycle frame-id threading

Updated `investigations/startup-createframe-profile.md` with the loader
follow-up that removes repeated `name -> id -> frame_ref` lifecycle resolution
during XML finalize. Recorded the new `xml_frame.rs` / `xml_lifecycle.rs`
threaded-frame-id path, the focused regression test that fires lifecycle
handlers with a wrong display name but the correct frame id, and refreshed the
`index.md` summary for the page.

## [2026-04-14] investigations | world map UIParent empty worklist follow-up

Updated `investigations/world-map-onupdate-hover-polling.md` with the
`UIParent_OnUpdate` fan-out follow-up: `FCF_OnUpdate`, `ButtonPulse_OnUpdate`,
and `AnimatedShine_OnUpdate` were still doing empty-list Lua dispatch every
tick. Recorded the new post-load wrappers in `workarounds.rs`, the focused
`uiparent_onupdate_worklists` regression tests, and refreshed the `index.md`
summary for the page.

## [2026-04-14] investigations | on-update dirty GameTimeFrame calendar atlas follow-up

Updated `investigations/on-update-dirty.md` with the `GameTimeFrame_SetDate()`
follow-up: same-day calendar atlas updates were still dirtying render because
the plain button texture setter took visual mutable borrows before checking for
real changes. Recorded the new no-op fast path in
`apply_set_button_texture_path()`, the focused atlas-backed button regression
test, the full-UI `GameTimeFrame_SetDate()` regression test, and refreshed the
`index.md` summary for the page.

## [2026-04-14] investigations | on-update dirty handler audit follow-up

Updated `investigations/on-update-dirty.md` with focused handler-audit results:
`LeaveInstanceGroupButton` now shows pure query/dispatch cost once its mutators
settle, while the remaining BuffFrame button cost comes from
`AuraButtonMixin:OnUpdate` doing duration formatting and font-threshold work on
every tick before the no-op setters bail out. Refreshed the `index.md` summary
for the page.

## [2026-04-14] investigations | on-update dirty solo compact raid manager follow-up

Updated `investigations/on-update-dirty.md` with the compact-raid follow-up:
`A_Admin.SetPartySize(0)` now fires `GROUP_ROSTER_UPDATE`, so solo transitions
hide `CompactRaidFrameManager` and remove `LeaveInstanceGroupButton` from the
visible `OnUpdate` handler set. Refreshed the `index.md` summary for the page.

## [2026-04-14] investigations | startup CreateFrame profiling ActionButtonTemplate regions

Updated `investigations/startup-createframe-profile.md` with the direct
`ActionButtonTemplate` layer/fontstring/button-texture fast path: the new
Rust-side region creation in `template/elements*.rs`, the focused regression
test that proves the hot path avoids Lua region fallback, and isolated
`WOW_SIM_PROFILE_CREATE_FRAME` numbers showing another `-27.36%` drop in
explicit template time across the profiled action-bar button families.

## [2026-04-14] investigations | startup CreateFrame profiling nested SpellFX follow-up

Updated `investigations/startup-createframe-profile.md` with the nested `ActionButtonSpellFXTemplate` follow-up: the remaining `ActionButtonInterruptTemplate` / `ActionButtonCastingAnimFrameTemplate` child creation fallback, the widened direct-child selector in `template/children.rs`, and new `WOW_SIM_PROFILE_CREATE_FRAME` numbers showing another `-28.6%` drop in explicit template time across action-bar button families.

## [2026-04-14] investigations | startup CreateFrame profiling MinimalScrollBar recursive fast path

Updated `investigations/startup-createframe-profile.md` with the `MinimalScrollBar` follow-up: the missed `Track -> Thumb` Lua `CreateFrame` fallback inside `apply_inline_frame_content()`, the recursive direct-child propagation change, the new focused regression test, and the smaller but measurable no-addons startup improvement after the fix.

## [2026-04-13] ingest | startup CreateFrame profiling

Created `investigations/startup-createframe-profile.md` to record runtime `CreateFrame` profiling results for Blizzard startup. Documented the new `WOW_SIM_PROFILE_CREATE_FRAME` instrumentation, the measured dominance of action-bar button template expansion (~4.1s across 34 runtime-created buttons), and the link to the planned pure-Rust template child creation work. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map preload API follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the remaining explored-overlay delay root cause: Blizzard's `MapTexturePreloader.lua` was calling `C_Map.RequestPreloadMap()`, but the simulator stubbed that API as a no-op. Recorded the new queued preload path for map art + exploration overlays, the focused `request_preload_map_warms_map_art_and_overlay_textures` regression test, and refreshed the `index.md` summary for that page.

## [2026-04-13] ingest | chat frame scrollbar anchor reapply

Created `investigations/chatframe-scrollbar-anchor-reapply.md` to document the `ChatFrame1` scrollbar/edit-box layout bug. Recorded the real root cause in `reapply_inline_anchors()`: inherited child-frame anchors were resolving `$parent...` against the child name instead of the actual parent frame name, which broke `relativeTo="$parentBackground"` lookups and pushed the resize/scrollbar chain to screen-relative layout. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map texture loading budget follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the second root cause behind the remaining world-map stalls: preload cleared `textures_pending` after CPU cache warmup even while the GPU atlas still lacked most tiles. Recorded the new `gpu_uploaded_textures`-based pending check, the focused `budgeted_preload` regression tests, and refreshed the `index.md` summary for that page.

## [2026-05-01] investigation | crafting cast bar

Created `investigations/crafting-cast-bar.md` to document the missing professions spellbar root cause. `C_TradeSkillUI.CraftRecipe` performed inventory changes but never started player casting or fired `UNIT_SPELLCAST_START`; successful crafts now populate `SimState.casting`, notify spellcast listeners, and emit `UPDATE_TRADESKILL_CAST_STOPPED` on completion. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map CreateTexture sublevel investigation

Created `investigations/world-map-create-texture-sublevel.md` to document the follow-up world-map open ordering churn: `CreateTexture(..., subLevel)` ignored its fourth argument, pooled textures started at sublevel 0, and Blizzard immediately repaired them with `SetDrawLayer()`. Recorded the new regressions for `CreateTexture(..., subLevel)` and no-op `SetDrawLayer()`, plus the traced repro where post-open `SetDrawLayer()` invalidations dropped from 150 to 0. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map voice chat alert investigation

## [2026-05-01] investigation | Journeys Midnight empty

Created `investigations/journeys-midnight-empty.md` to document the empty Journeys tab on Midnight. Root cause: current expansion constants were 11, but default major-faction data only seeded War Within expansion 10 rows, so Blizzard's `JourneysFrameMixin:Refresh()` received an empty `C_MajorFactions.GetMajorFactionIDs(11)` result. Updated `index.md` with the new investigation page.

Created `investigations/world-map-voice-chat-alerts.md` to document the reduced-stack world-map overlay where voice prompt frames appeared above the panel. Recorded the two harness prerequisites behind it: `Blizzard_Channels` needs `Blizzard_SocialToast` for `SocialToastTemplate hidden="true"`, and alert positioning needs the real chat-alert addons instead of the `ChatAlertFrame` stub. Updated `index.md` with the new investigation page.

## [2026-04-27] investigation | explicit XML parent anchors

Created `investigations/explicit-xml-parent-anchors.md` to document the PaperDoll sidebar tab positioning bug. Root cause: nested XML frame creation preferred the containing frame over an explicit child `parent="..."`, so implicit anchors resolved to `PaperDollFrame` instead of `CharacterFrameInsetRight`. Added the page to `index.md`.

## [2026-04-13] ingest | world map OnUpdate hover polling investigation

Created `investigations/world-map-onupdate-hover-polling.md` to document the post-texture-fix `UIParent_OnUpdate` cost: `FCF_OnUpdate` hover polling, the unnecessary mutable borrow in `IsMouseOver()`, the new immutable-borrow regression test, and the runtime repro where verbose OnUpdate logs stayed quiet after startup. Updated `index.md` with the new investigation page.

## [2026-05-02] investigation | Adventure Guide layout

Created `investigations/adventure-guide-layout.md` for the Suggested Content overlap bug. Root cause: `resolve_rect_if_dirty` fast-path geometry queries recomputed only the queried resized frame, leaving sibling frames anchored to it with stale cached rects. The fix now recomputes anchor dependents for direct dirty roots and dirty ancestor roots, and queues those moved dependents for hit-grid updates; regression coverage is `querying_resized_anchor_target_updates_dependent_siblings`.

Updated `investigations/adventure-guide-layout.md` with the follow-up Suggested Content text overlap root cause: tooltip `GetNumLines` overwrote the shared FrameRef method slot, so regular FontStrings returned zero lines. FontStrings now compute wrapped line count from measured text height while GameTooltip keeps tooltip-backed line counts; regression coverage is `fontstring_get_num_lines_reports_wrapped_line_count`.

## [2026-05-02] investigation | Adventure Guide SimpleHTML markup

Created `investigations/adventure-guide-simplehtml-markup.md` for the boss
overview text rendering raw `|c...|Hspell...|h...|r` and `|n` escapes. Root
cause: Encounter Journal uses `SimpleHTML`, whose stripped-text path only
removed HTML tags and bypassed WoW markup cleanup. Updated `index.md` with the
new investigation page.

## [2026-04-13] ingest | world map texture loading budget investigation

Created `investigations/world-map-texture-loading-budget.md` to document the post-rebuild-fix world-map stalls: hidden BC tile uploads, preload/draw source-cache mismatch, the new BC cache in `TextureManager`, and the smaller draw/tick texture budgets. Updated `index.md` with the new investigation page.

## [2026-05-03] investigation | LFD role icon slowness

Created `investigations/lfd-role-icon-slowness.md` for the LFD load pause around role selection icons. Root cause: role icons/backgrounds are button atlas crops from `Interface\lfgframe\uilfgprompts`, and crop extraction decoded the full 2048x2048 BLP before producing small sub-regions. Documented the persistent crop cache added in `TextureManager::load_sub_region` and updated `index.md`.

## [2026-04-13] ingest | world map frame-level rebuild investigation

## [2026-04-13] investigations | startup CreateFrame profiling follow-up

Updated `investigations/startup-createframe-profile.md` with section-level template profiling, the method-only XML script fast path, widened direct-child creation for `ActionButtonSpellFXTemplate` / `MinimalScrollBar`, and current shared-worktree startup numbers showing `36.79s -> 28.89s` on `--no-addons --no-saved-vars`.

Created `investigations/world-map-frame-level-rebuilds.md` to document the world-map performance bug where map pins repeatedly called `SetFrameLevel()` with the same value, forcing unnecessary `strata_buckets` invalidation and bucket rebuilds. Updated `index.md` with the new investigation page.

## [2026-04-29] investigation | LFD dungeon list empty

Created `investigations/lfd-dungeon-list-empty.md` for the Dungeons & Raids panel populating empty when "Specific Dungeons" was selected. Root causes: missing `GetLFDChoiceCollapseState`/`GetLFDChoiceEnabledState`/`GetLFGLockList` globals breaking `LFGDungeonList_Setup`; `LFG_UPDATE_RANDOM_INFO` never fired at startup so `LFDQueueFrame.Specific` stayed hidden and `OnShow=LFDQueueFrame_Update` never ran; `is_random=true` on the negative-id header in `default_lfd_dungeons` routed `GetRandomDungeonBestChoice` to `-1`. Updated `index.md` with the new investigation page.

## [2026-04-25] ingest | micro-menu atlas revert investigation

Created `investigations/micro-menu-atlas-revert.md` to document the micro-menu hover/leave icon disappearance root cause: button atlas setters populated child `tex_coords` but not `atlas_tex_coords`, so restored normal textures could miss the atlas-crop render path. Updated `index.md` with the new investigation page.

## [2026-05-02] update | Adventure Guide portrait masks and icons

Updated `investigations/adventure-guide-layout.md` with the follow-up Adventure Guide portrait bug: `Texture:SetMask` was still a no-op, so card icons did not clip to the gold portrait rings, and two seeded Adventure Journal icon paths did not resolve. Documented the mask wiring and manifest-backed icon replacements.

## [2026-05-28] ingest | Paladin aura stance bar

Created `investigations/paladin-aura-stance-bar.md` to document the root cause behind the missing Paladin aura bar: default Paladin state had zero shapeshift forms, so Blizzard `StanceBarMixin:Update()` hid the bar before rendering. Added the state-backed fix and regression coverage notes for the raw shapeshift globals and Blizzard StanceBar layer.

## [2026-05-08] update | Blizzard UI cache-only runtime

Simplified the Blizzard UI runtime model: `data/blizzard-ui-files.txt` is the committed file list, CASC is the extraction source, and `~/.cache/wow-ui-sim/blizzard-ui` is the only runtime Blizzard UI addon root. Removed the old runtime discovery language for `Interface/BlizzardUI`, `vendor/wow-ui-source`, and local `BlizzardInterfaceCode` fallback.

## [2026-05-08] update | Blizzard UI CASC source cache

Updated `systems/casc-asset-cache.md` and `reference/cli-commands.md` for `wow-cli casc sync-blizzard-ui`, the `~/.cache/wow-ui-sim/blizzard-ui` source cache, and GUI startup fallback behavior when the GitHub checkout is missing.

## [2026-04-29] ingest | dialog background DXT3 stripes

Created `investigations/dialog-background-dxt3-stripes.md` to document the escape-menu background stripe root cause: DXT3 BLPs were incorrectly mapped to BC3 on the raw compressed upload path. Updated `index.md` with the new investigation page.

## [2026-06-09] update | Mists 5.5.4 EditMode action-bar and UnitIsCivilian

Updated `investigations/editmode-layout.md` with the Mists 5.5.4 action-bar root cause: default managed action bars can remain at Blizzard's temporary `TOPLEFT UIParent` anchor when the manual EditMode layout pass aborts before clearing `layoutApplyInProgress`. Documented the Rust-side finalizer that clears the guard and replays `UpdateActionBarPositions()`, the modeled `UnitIsCivilian` fallback for Classic TargetFrame, and the ObjectiveTracker phantom duplicate root cause: bundled-addon startup exposed legacy `WatchFrame` alongside modern `ObjectiveTrackerFrame`.

## [2026-06-09] investigation | frame surrogate identity slot

Created `investigations/frame-surrogate-identity-slot.md` after replacing the simulator-only frame surrogate `[1]` dispatch path with a `[0]` identity token model. Updated `index.md` with the new investigation page.

## [2026-05-01] ingest | editbox render text cache investigation

Created `investigations/editbox-render-text-cache.md` to document the SimCommands search-box input bug: keyboard input updated `Frame.text` but left `text_stripped` stale after `SetText("")`, causing glyph rendering to shape an empty string. Updated `index.md` with the new investigation page.

## [2026-05-01] ingest | LFD checkbox and role-state follow-up

Updated `investigations/lfd-dungeon-list-empty.md` with the follow-up Group Finder bug where Specific Dungeons populated but dungeon selections were empty and `GetLFGRoles` was still a false stub, leaving "Join as Party" disabled. Documented the state-backed role and LFD checkbox fix plus regression coverage.

## [2026-05-01] update | Wardrobe filter click dispatch

Updated `investigations/appearances-wardrobe-api.md` with the follow-up Wardrobe filter bug where menu descriptions and callbacks were valid, but clicks could be swallowed by decorative child regions during GUI hit testing. Documented the fix: final mouse targets must be mouse-enabled frames, while decorative children only guide hit-test descent.

## [2026-05-01] update | Wardrobe class dropdown and set fallback

## [2026-05-18] investigation | ElvUI tooltip scale clipping

## [2026-06-08] ingest | ServerSnapshot action bar import

Created `systems/server-snapshot-action-bars.md` after adding startup import for action-bar spell slots captured by the ServerSnapshot addon. Updated `index.md` with the new systems page.

Updated `investigations/tooltip-double-shell.md` with the follow-up ElvUI tooltip clipping root cause. Tooltip frame bounds were scaled by ElvUI effective scale, but internal `GameTooltip` glyph emission was not; tooltip text now scales font size, line spacing, and text insets with the frame effective scale.

Updated `investigations/appearances-wardrobe-api.md` with the Wardrobe class dropdown casing/color contract and the `C_TransmogSets.GetBaseSets()` nil fallback stack overflow. Documented that class display names come from localized `className`, colors from uppercase `classFile`, and empty set surfaces must return tables.

## [2026-06-11] create | Class talent edge lines

Created `investigations/class-talents-edge-lines.md`. Talent connector lines were missing because `IsRectValid()` reported dirty-but-resolvable talent buttons as invalid, causing Blizzard's edge positioning to skip `Line:SetStartPoint()` / `SetEndPoint()`. A second render-list gate also filtered endpoint-positioned `Line` widgets and arrowhead textures under anchorless edge-frame parents. Fixed by resolving dirty rects inside `IsRectValid()` and allowing parent-independent line/anchor geometry through render-list filtering while preserving the ordinary unanchored-parent guard.

## [2026-04-12] ingest | transparent wrapper render-order investigation

Created `investigations/transparent-wrapper-render-order.md` for the world map / quest log render-order fix. Updated it after a follow-up regression to document the depth-aware transparent-wrapper hoist in `state_render.rs`, including both world-map visibility coverage (`world_map_tiles_render_after_tiled_background`) and world-quest pin ordering coverage.

## [2026-04-09] ingest | systems/ pages created (10 pages)

Created all 10 systems/ pages from source docs in docs/:

- systems/layout-system.md — from layout-system.md + anchor-resolution.md
- systems/rendering-pipeline.md — from rendering-pipeline.md
- systems/widget-system.md — from widget-system.md + button-text-rendering.md
- systems/lua-api.md — from lua-api.md
- systems/event-system.md — from event-system.md
- systems/xml-template-system.md — from xml-template-system.md
- systems/addon-loading.md — from addon-loading-pipeline.md
- systems/texture-atlas.md — from texture-atlas-system.md
- systems/frame-data-flow.md — from frame-data-flow.md
- systems/taint-system.md — from protected-frame-enforcement.md + src/lua_api/frame/methods/methods_helpers.rs + src/lua_api/globals/security.rs + tests/protected_frame_enforcement.rs + tests/secure_handler_fallback.rs + tests/security_api.rs

Updated index.md systems/ table.

## [2026-05-28] ingest | Mists WorldMap startup failure cluster

Created `investigations/mists-world-map-startup.md` after fixing Mists startup errors around `WorldMapFrame`, `WorldMapTrackQuest`, `UpdateUIPanelPositions`, `FogOfWarFrameMixin`, and MapCanvas provider `OnAdded` defaults. Updated `index.md` with the new investigation page.

## [2026-04-10] ingest | Initial bulk ingest from 30+ existing docs

Bootstrapped wiki from root-level documentation files. Created pages across systems/, design/, investigations/, and reference/ categories.

## [2026-04-09] ingest | design/ and reference/ pages created

Created 7 pages from DESIGN.md, SCALING.md, docs/debug-tools.md, FUTURE.md, docs/c-api-signature-audit.md, docs/c-api-stub-audit.md, AGENTS.md, and PLAN.md.

Pages created:
- design/architecture-overview.md
- design/scaling-coordinates.md
- design/debug-tools.md
- reference/api-coverage.md
- reference/cli-commands.md
- reference/addon-compatibility.md
- reference/development-phases.md
## [2026-05-16] ingest | addon startup Settings and item-load investigation

Created `investigations/addon-startup-settings-and-item-load.md` to capture the root causes behind addon startup errors: registered Settings canvases must start hidden, forbidden attribute delegates need secure dispatch, item subclasses must return enUS keyword-compatible names or nil, and positive live item IDs need synthetic placeholder item info so item-load callbacks terminate.

## [2026-06-19] create | Retail Store secure pool constructor mismatch

Created `investigations/store-secure-pool-constructors.md` after fixing the retail Store blank/red card state. Root cause: Store code runs in `__secureenv`, whose `CreateFramePoolCollection` still pointed at the simulator fallback after `Blizzard_SharedXMLBase` installed Blizzard's proxy-backed constructor in `_G`. The fix syncs real pool/factory constructors into `__secureenv` after SharedXMLBase loads and pins the behavior with Store tree, pool surface, and text-cache corruption tests.

## [2026-05-26] update | C API temporary shim module retired

Updated `reference/api-coverage.md` after moving the last `src/c_api/temporary_shims` surface (`C_TransmogOutfitInfo` slot/outfit defaults) into `src/lua_api/workarounds/temporary/` and deleting the empty C API temporary-shim module. Temporary unmodeled `C_*` defaults now belong in Lua workaround modules; `src/c_api/permanent_shims/` remains only for intentional unsupported domains.

## [2026-05-23] update | transmog sets shim boundary

Updated `investigations/appearances-wardrobe-api.md` after moving `C_TransmogSets` empty/default set APIs out of `runtime_surface_bootstrap.lua` and into `src/c_api/temporary_shims/c_transmog_sets.rs`. The investigation still records the same wardrobe contract: `GetBaseSets()` and related set APIs must return empty tables rather than nil until real set inventory state exists.

## [2026-05-29] ingest | Mists Syndicator and Baganator startup cleanup

Created `investigations/mists-syndicator-baganator-startup.md` after fixing full-profile Mists startup errors. The investigation records the Mists-gated item taxonomy overrides needed by Syndicator and the minimal CharacterFrame/TokenUI bootstrap path needed by Baganator.

## [2026-06-10] update | class talent config-scoped visibility

Updated `investigations/class-talents-trait-loadout-state.md` after fixing a full-addon/SavedVariables talent panel regression where `C_Traits.GetNodeInfo(configID, nodeID)` discarded `configID` and evaluated Protection nodes against a stale view spec.

## [2026-06-11] ingest | CASC root v2 misparse dropped 89% of fdids

Created `investigations/casc-root-v2-parsing-missing-textures.md` after the magic-dispel debuff border atlas resolved correctly but its texture (`interface/hud/uidebuffframes.blp`) could not be extracted. cascette-rs 0d0e79a misparsed 12.0.5 TSFM v2 root blocks (split content-flags fields, wrong NoNameHash bit), silently dropping 2.8M of 3.19M root records. Fixed by pinning cascette-rs c5de2b9 / asset-resolver 3ab8a14 (wow-ui-sim 598a29909), rebuilding the resolution cache (346K → 1.88M entries), and clearing 904 stale `.missing` markers.

## [2026-06-11] update | EditMode first-apply dirtiness + player debuff modeling

Updated `investigations/casc-root-v2-parsing-missing-textures.md` with parts 2-3: the dispel swirly was also blocked by the EditMode startup workaround seeding frames before UpdateSystem (destroying first-apply setting dirtiness, so ShowDispelType never applied), and by the absence of any player-debuff modeling. Fixed in c527f05fa (lookup-then-UpdateSystem flow mirroring EditModeManagerFrameMixin) and 2ac3057b6 (A_Admin.AddDebuff + UNIT_AURA isFullUpdate payloads + nilable dispelName).

## [2026-06-11] create | XML scale attribute ignored

Created `investigations/xml-scale-attribute.md`. The hero talents box didn't encompass its node buttons: `FrameXml` had no `@scale` field, so all 127 `scale="..."` attributes in Blizzard XML were silently dropped — including `HeroTalentsTreeNodesContainerTemplate`'s `scale="0.85"`, whose absence left only 212 of the needed 272 local units inside the fixed 284×362 backplate. Fixed in 91835d898 by parsing `@scale` and applying it through the template chain in both the XML loader and runtime CreateFrame paths, mirroring alpha.

## [2026-06-11] ingest | Journeys renown card text anchor fallback

Created `investigations/journeys-renown-card-text-anchor.md`. Reported as text z-ordering, but the Journeys "Renowns" card name/level FontStrings were anchored to the wrong target: their `relativeKey="$parent.IconFrame"` failed eager resolution (loader creates Layers before child Frames) and SetPoint silently fell back to the parent card, pushing the text under the adjacent column. Fixed in 7c0c0f987 by storing unresolved $parent key expressions on the anchor for the existing post-children lazy resolution pass.

## [2026-06-11] ingest | Mouse dead at frozen 50 FPS (probe blockers + idle tick stall)

Created `investigations/mouse-dead-probe-blockers-idle-ticks.md`. CoreBehaviorProbe (loaded from a renamed `.disabled` folder) left two full-screen mouse-enabled DIALOG blockers over UIParent because its 3-deep C_Timer.After cleanup chain stalls: pending C_Timers do not wake the tick loop once the app idles (open bug), and the frozen tick loop also freezes the FPS display at its last value. Loader fixed in b580ea005 to only accept TOCs naming their folder.

## [2026-06-11] update | tick-subscription churn root cause fixed

Updated `investigations/mouse-dead-probe-blockers-idle-ticks.md`: the idle timer stall was subscription churn — compute_tick_interval returned the raw shrinking remaining-time of the next C_Timer, changing the iced time::every identity on every update, so continuous input (mouse moves) recreated the tick stream before it could fire. Fixed in 397742569 with quantized interval buckets.

## [2026-06-12] create | DISPLAY_SIZE_CHANGED / UI_SCALE_CHANGED firing conditions

Created `investigations/display-size-ui-scale-events.md`. A live retail probe (`docs/addons/ScaleEventProbe`, 12.0.5.67823) disproved the sim's "resize never fires UI_SCALE_CHANGED" assumption: retail fires the two events as an ordered pair (display-first) on every display/scale recalculation — drag resize emits repeated ordered pairs during the drag, maximize/restore and resolution/fullscreen transitions can emit double-pairs even when dimensions are unchanged, scale slider and useUiScale changes also emit the pair, and startup fires the pair twice pre-PLAYER_LOGIN. Events fire before CVAR_UPDATE with GetEffectiveScale() already updated. Fixed `set_screen_size` to fire the pair and inverted the resize regression test to assert order. Startup ordering (sim fires post-login) and no-op dedupe remain divergent.

## [2026-06-12] update | Startup display/scale event ordering matched to retail

Updated `investigations/display-size-ui-scale-events.md`. Moved the `DISPLAY_SIZE_CHANGED`/`UI_SCALE_CHANGED` pair from `fire_post_login_events` into `fire_login_sequence` (after `VARIABLES_LOADED`, before `PLAYER_LOGIN`) — retail fires both pairs pre-login and none post-login. With the `set_screen_size` pair during GUI canvas startup this yields two pre-login pairs, matching the probe capture. Same-size window transitions reclassified as an observability limit (iced exposes no OS window-state signal). Verified: lib failure set unchanged, `lua-errors` clean with and without addons; new regression test pins pair-before-login ordering.

## [2026-06-12] update | hit-grid ordered insertion (the core mouse-freeze fix)

Updated `investigations/mouse-dead-probe-blockers-idle-ticks.md` with cause 4: full hit-grid rebuilds per hover transition (180-470ms each) saturated the main thread; the rebuilds were themselves a workaround for append-only HitGrid::insert breaking render order. Fixed with per-frame render-order keys + binary insertion, producer coverage for level/strata/raise changes, hover-time coalescing, and dev opt-level 1 (a7b131f65, telemetry 0c2668a3a).

## [2026-06-12] create | Mount Journal clicks never switched selection

Created `investigations/mount-journal-click-selection.md`. Real mouse clicks on mount rows fired OnMouseDown/OnMouseUp but never OnClick while `row:Click()` worked, because the startup-XML fast path's `parse_single_string_literal` stripped only the outer quotes — the generic MethodWithStringArg parser fused `RegisterForClicks("LeftButtonUp", "RightButtonUp")` into one garbage registration entry that no click edge can match. Fixed by rejecting interior quotes so multi-arg calls fall through to the dedicated parser. Added a `headless-click-probe mounts` panel (dotted-path frame resolution for anonymous ScrollBox rows + post-click `verify_lua`) and a `WOW_SIM_DEBUG_CLICK_DISPATCH=1` dispatch trace.

## [2026-06-12] create | Micro menu clicks missed from stale quadrant anchor

Created `investigations/micro-menu-click-offset.md`. GUI clicks on micro-menu buttons (LFD/Group Finder) did nothing: MicroMenuMixin:Layout anchors the menu inside MicroMenuContainer by screen quadrant, but the sim ran it before saved anchors and the real window size were applied, so the first press snapped the whole bar one QueueStatusButton slot (~46.5px) between mouse-down and mouse-up and the same-frame guard skipped OnClick. Fixed by replaying Blizzard's `InvokeOnAnyEditModeSystemAnchorChanged(force)` at the end of init_edit_mode_layout and from set_screen_size after the DISPLAY_SIZE_CHANGED/UI_SCALE_CHANGED pair. Added `headless-click-probe micromenu` regression panel.

## [2026-06-19] create | Post-load workaround audit

Created `investigations/post-load-workaround-audit.md` while auditing retail post-load hooks. The page records duplicate loader-side hooks retired for AccountStore, MapCanvas, and FrameXMLUtil, and classifies the remaining sampled hooks with current temporary rationale plus retirement paths.

## [2026-06-20] update | Third-party addon metadata pre-registration

Updated `systems/addon-loading.md` after fixing `!BugGrabber`'s false no-display warning when `BugSack` is present. The system page now records the invariant that discovered third-party addon metadata must be registered in `C_AddOns` before eager third-party Lua executes, while actual file loading still follows enabled, non-`LoadOnDemand`, dependency-sorted order.

## [2026-06-21] update | secureenv no-fallback retail probe

Updated `systems/taint-system.md` and `investigations/store-secure-pool-constructors.md` after a retail PrivateAurasUI cooldown-wrapper probe showed secure code does not hit late `_G` overrides. The simulator now models secureenv as a separate shallow copy without `__index = _G`; `Blizzard_SharedXMLBase` Lua is replayed into secureenv to populate shared secure symbols directly instead of copying constructors from `_G` after load.

## [2026-06-30] update | Initial TOC `[Bootstrap]` support

Added parser and loader support for TOC entries annotated with `[Bootstrap]`, motivated by LoD bootstrap glue such as `Blizzard_CooldownBroadcaster_Bootstrap.lua`. This was later corrected on 2026-07-01 after a live-client third-party probe showed `[Bootstrap]` does not reorder TOC files.

## [2026-07-01] correction | `[Bootstrap]` preserves TOC order

Updated `systems/addon-loading.md` after live-client probes showed `[Bootstrap]` is not a separate pass and must not move files out of TOC order. `TocFile` now keeps annotated files in `files` and records a per-file bootstrap flag. Startup loads full TOCs for non-LoD addons and only annotated bootstrap files for LoD addons, preserving addon order; runtime `LoadAddOn` skips already-executed bootstrap files and a self `LoadAddOn(thisAddon)` call from bootstrap remains a benign reentrancy no-op.

## [2026-08-08] investigation | Prove remaining 12.0.0 removed runtime APIs

Classified exactly 19 removed runtime API rows as `best-effort`/`behavioral` using the committed full-LoD namespace-safe rawget batch `patch-tests/patch_12_1/strict_removals.rs::removed_remaining_runtime_apis_are_absent_after_full_lod_load` at `ec9ffbc0b`. Three obsolete simulator publications were removed (`C_CatalogShop.OpenCatalogShopInteraction`, `C_PlayerInfo.IsExpansionLandingPageUnlockedForPlayer`, and `C_StorePublic.IsDisabledByParentalControls`); the other 16 rows were already absent. Source scanning is auxiliary, no replacement behavior is inferred, and the 143/453/1/2813 totals are current.

The six retail 12.0.0 `Enum.EncounterTimelineTrackType.*` rows are best-effort/behavioral: Hidden=0, Sorted=1, Linear=2; metadata MinValue=0, MaxValue=2, NumValues=3. Focused proof is at `77fe621bb45a34b8618577e495aef3b83a7b96c1` and asserts the exact member set, Lua numeric types, and metadata. Claims are bounded to startup publication; sorting/linear behavior, ordering, consumers, persistence, and lifecycle remain unclaimed.

## [2026-08-10] investigation | Classify ExpansionLevel enums

The 16 retail 12.0.0 `Enum.ExpansionLevel.*` and `Enum.ExpansionLevelMeta.*` rows are best-effort/behavioral: `None=0, BurningCrusade=1, Northrend=2, Cataclysm=3, MistsOfPandaria=4, Draenor=5, Legion=6, BattleForAzeroth=7, Shadowlands=8, Dragonflight=9, WarWithin=10, Midnight=11, LastTitan=12`; metadata `MinValue=0`, `MaxValue=12`, `NumValues=13`. Focused proof is at `fd593d5fb9dc3ad14114da7e9fff0709a345e7ea` and asserts Lua numeric types, the exact complete 13-member set, no extras, and exact metadata. Root cause: no runtime drift; the guarded `missing_enums.lua` fallback is authoritative because no competing `ExpansionLevel` publication exists. Claims are bounded to retail 12.0.0 startup publication and exact values/types/metadata; expansion progression, availability, gameplay behavior, consumers, persistence, transitions, and lifecycle remain unclaimed.
The five retail 12.0.0 `Enum.NamePlateSimplifiedType.*` rows are best-effort/behavioral: `None=0, Minion=1, MinusMob=2, FriendlyPlayer=3, FriendlyNpc=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused proof is commit `22193d885` via `src/loader/tests/wow_api_globals/patch_12_0_0_nameplate_simplified_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; nameplate behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1692 best-effort, 982 evidence-required, 2 exception-requested, and 734 untriaged rows** (3410 total).
The eight retail 12.0.0 `Enum.NamePlateSize.*` and `Enum.NamePlateSizeMeta.*` rows are best-effort/behavioral: `Small=1, Medium=2, Large=3, ExtraLarge=4, Huge=5`; metadata `MinValue=1`, `MaxValue=5`, `NumValues=5`. Focused proof is commit `936135046` via `patch_12_0_0_nameplate_size_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit `widget.rs` member data is authoritative and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; nameplate sizing behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1700 best-effort, 982 evidence-required, 2 exception-requested, and 726 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.NeighborhoodInitiativeFlags.*` and `Enum.NeighborhoodInitiativeFlagsMeta.*` rows are best-effort/behavioral: `Disabled=1, NoAbandon=2, NoRepeat=4`; metadata `MinValue=1`, `MaxValue=4`, `NumValues=3`. Focused proof is commit `a3b138e77` via `patch_12_0_0_neighborhood_initiative_flags_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; neighborhood-initiative flag semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1706 best-effort, 982 evidence-required, 2 exception-requested, and 720 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.NeighborhoodInitiativeTaskType.*` and `Enum.NeighborhoodInitiativeTaskTypeMeta.*` rows are best-effort/behavioral: `Single=0, RepeatableFinite=1, RepeatableInfinite=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `1a6b066ec` via `patch_12_0_0_neighborhood_initiative_task_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; neighborhood-initiative task semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1712 best-effort, 982 evidence-required, 2 exception-requested, and 714 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.NeighborhoodInitiativeUpdateStatus.*` and `Enum.NeighborhoodInitiativeUpdateStatusMeta.*` rows are best-effort/behavioral: `Started=0, MilestoneCompleted=1, Completed=2, Failed=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `e0d63997b` via `patch_12_0_0_neighborhood_initiative_update_status_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; neighborhood-initiative update semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1719 best-effort, 982 evidence-required, 2 exception-requested, and 707 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.NeighborhoodInitiativesCompletionStates.*` and `Enum.NeighborhoodInitiativesCompletionStatesMeta.*` rows are best-effort/behavioral: `NiCompletionStateNotCompleted=0, NiCompletionStatePlayerCompleted=1, NiCompletionStateSystemAbandoned=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `fc49fecf8` via `patch_12_0_0_neighborhood_initiatives_completion_states_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; neighborhood completion semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1725 best-effort, 982 evidence-required, 2 exception-requested, and 701 untriaged rows** (3410 total).
The two retail 12.0.0 `Enum.NpcCraftingOrderSetFlags.*` rows are best-effort/behavioral: `AllowMultiple=1, AllowDuplicate=2`; matching metadata is `MinValue=1, MaxValue=2, NumValues=2`. Focused proof is commit `979724441` via `patch_12_0_0_npc_crafting_order_set_flags_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; crafting-order flag semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1727 best-effort, 982 evidence-required, 2 exception-requested, and 699 untriaged rows** (3410 total).
The four retail 12.0.0 `Enum.PlayerCompanionInfoFlags.*` and `Enum.PlayerCompanionInfoFlagsMeta.*` rows are best-effort/behavioral: `IgnoreSeasonInScenarios=1`; metadata `MinValue=1`, `MaxValue=1`, `NumValues=1`. Focused proof is commit `d53ae8a89` via `patch_12_0_0_player_companion_info_flags_enums.rs`; it asserts the exact member, Lua numeric type, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; player-companion flag semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1731 best-effort, 982 evidence-required, 2 exception-requested, and 695 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.PreyHuntProgressState.*` and `Enum.PreyHuntProgressStateMeta.*` rows are best-effort/behavioral: `Cold=0, Warm=1, Hot=2, Final=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `f4ddf6457` via `patch_12_0_0_prey_hunt_progress_state_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit combat-system enum data is authoritative for members and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; prey-hunt progression semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1738 best-effort, 982 evidence-required, 2 exception-requested, and 688 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.ProceduralSpawnInteractionMode.*` and `Enum.ProceduralSpawnInteractionModeMeta.*` rows are best-effort/behavioral: `None=0, Paint=1, Manipulate=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `9302f3017` via `patch_12_0_0_procedural_spawn_interaction_mode_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; procedural-spawn interaction semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1744 best-effort, 982 evidence-required, 2 exception-requested, and 682 untriaged rows** (3410 total).
The five retail 12.0.0 `Enum.ProceduralSpawnVolumeChunkFlags.*` and `Enum.ProceduralSpawnVolumeChunkFlagsMeta.*` rows are best-effort/behavioral: `None=0, AllSubChunksSet=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is commit `ae20de74a` via `patch_12_0_0_procedural_spawn_volume_chunk_flags_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; procedural-spawn chunk flag semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1749 best-effort, 982 evidence-required, 2 exception-requested, and 677 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.RaidAuraOrganizationType.*` and `Enum.RaidAuraOrganizationTypeMeta.*` rows are best-effort/behavioral: `Legacy=0, BuffsTopDebuffsBottom=1, BuffsRightDebuffsLeft=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `769c96cd6` via `patch_12_0_0_raid_aura_organization_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit edit-mode enum data is authoritative for members and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; raid-aura organization behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1755 best-effort, 982 evidence-required, 2 exception-requested, and 671 untriaged rows** (3410 total).
The six retail 12.0.0 `Enum.RaidDispelDisplayType.*` and `Enum.RaidDispelDisplayTypeMeta.*` rows are best-effort/behavioral: `Disabled=0, DispellableByMe=1, DisplayAll=2`; metadata `MinValue=0, MaxValue=2, NumValues=3`. Focused proof is commit `a1e8065e7` via `src/loader/tests/wow_api_globals/patch_12_0_0_raid_dispel_display_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit combat-system enum data is authoritative for members and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; raid-dispel display behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1775 best-effort, 982 evidence-required, 2 exception-requested, and 651 untriaged rows** (3410 total).

The fourteen retail 12.0.0 `Enum.RcoCloseReason` rename rows are best-effort/behavioral: replacement values Fulfill=0, Expire=1, Cancel=2, Reject=3, GmCancel=4, CrafterFulfill=5, Invalid=6; metadata `MinValue=0`, `MaxValue=6`, `NumValues=7`; focused proof is commit `41dc7f43e` via `patch_12_0_0_rco_close_reason_enums.rs`, asserting numeric types, no extras, and absence of every old `RcoClose*` alias. Guarded fallback is authoritative with no drift; the rename preserves numeric values. Claims are bounded to startup publication and alias absence; order-processing behavior, consumers, persistence, transitions, and lifecycle are not claimed.

The thirteen retail 12.0.0 `Enum.RenownRewardDisplayType.*` and `Enum.RenownRewardDisplayTypeMeta.*` rows are best-effort/behavioral: None=0, Item=1, Spell=2, Mount=3, Transmog=4, TransmogSet=5, TransmogIllusion=6, Title=7, GarrFollower=8, Currency=9; metadata `MinValue=0`, `MaxValue=9`, `NumValues=10`. Focused proof is commit `3e113702a` via `patch_12_0_0_renown_reward_display_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; renown reward-display behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1788 best-effort, 982 evidence-required, 2 exception-requested, and 638 untriaged rows** (3410 total).

The seven retail 12.0.0 `Enum.RenownRewardsFlags.*` and `Enum.RenownRewardsFlagsMeta.*` rows are best-effort/behavioral: `Milestone=1`, `Capstone=2`, `Hidden=4`, `AccountUnlock=8`; metadata `MinValue=1`, `MaxValue=8`, `NumValues=4`. Focused proof is commit `d773ff6e8` via `patch_12_0_0_renown_rewards_flags_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; renown-reward flag semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1811 best-effort, 982 evidence-required, 2 exception-requested, and 615 untriaged rows** (3410 total).

The eight retail 12.0.0 `Enum.SimpleOrderStatus.*` and `Enum.SimpleOrderStatusMeta.*` rows are best-effort/behavioral: `Invalid=0`, `Creating=1`, `InProgress=2`, `Success=3`, `Failed=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused proof is commit `bedbdabb1` via `patch_12_0_0_simple_order_status_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; order-status semantics, consumers, persistence, transitions, and lifecycle remain unclaimed.

The eight retail 12.0.0 `Enum.SleevesGeoRange.*` and `Enum.SleevesGeoRangeMeta.*` rows are best-effort/behavioral: `None=0`, `Default=1`, `Flared=2`, `Puffy=3`, `PandaCollar=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`. Focused proof is commit `1ed7b3764` via `patch_12_0_0_sleeves_geo_range_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; sleeves-geometry semantics, consumers, persistence, transitions, and lifecycle remain unclaimed.
The three retail 12.0.0 `Enum.NamePlateSimplifiedTypeMeta.*` rows are best-effort/behavioral: metadata `MinValue=0`, `MaxValue=4`, `NumValues=5` for the numeric family `None=0, Minion=1, MinusMob=2, FriendlyPlayer=3, FriendlyNpc=4`. Focused proof is commit `22193d885` via `patch_12_0_0_nameplate_simplified_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; nameplate behavior, consumers, persistence, transitions, and lifecycle remain unclaimed. The six retail 12.0.0 `Enum.SpellAuraVisibilityType.*` and `Enum.SpellAuraVisibilityTypeMeta.*` rows are best-effort/behavioral: `RaidInCombat=0, RaidOutOfCombat=1, EnemyTarget=2`; metadata `MinValue=0, MaxValue=2, NumValues=3`. Focused proof is commit `8f5e7bca1` via `patch_12_0_0_spell_aura_visibility_type_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit combat-system enum data is authoritative for members and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; aura-visibility behavior, consumers, persistence, transitions, and lifecycle are not claimed. The six retail 12.0.0 `Enum.SecrecyLevel.*` and `Enum.SecrecyLevelMeta.*` rows are best-effort/behavioral: `NeverSecret=0, AlwaysSecret=1, ContextuallySecret=2`; metadata `MinValue=0, MaxValue=2, NumValues=3`. Focused proof is commit `bf190ef99` via `patch_12_0_0_secrecy_level_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; secrecy semantics, consumers, persistence, transitions, and lifecycle are not claimed. The eleven retail 12.0.0 `Enum.SpellDiminishCategory.*` and `Enum.SpellDiminishCategoryMeta.*` rows are best-effort/behavioral: `Root=0, Taunt=1, Stun=2, AoEKnockback=3, Incapacitate=4, Disorient=5, Silence=6, Disarm=7`; metadata `MinValue=0, MaxValue=7, NumValues=8`. Focused proof is commit `be42e3899` via `patch_12_0_0_spell_diminish_category_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; diminishing-return behavior, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1837 best-effort, 982 evidence-required, 2 exception-requested, and 589 untriaged rows** (3410 total).

The six retail 12.0.0 `Enum.SpellDiminishRuleset.*` and `Enum.SpellDiminishRulesetMeta.*` rows are best-effort/behavioral: `None=0, PvE=1, PvP=2`; metadata `MinValue=0, MaxValue=2, NumValues=3`. Focused proof is commit `b9c450f09` via `patch_12_0_0_spell_diminish_ruleset_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; diminishing-ruleset semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1843 best-effort, 982 evidence-required, 2 exception-requested, and 583 untriaged rows** (3410 total).
The seven retail 12.0.0 `Enum.StatusBarFillStyle.*` and `Enum.StatusBarFillStyleMeta.*` rows are best-effort/behavioral: `Standard=0`, `StandardNoRangeFill=1`, `Center=2`, `Reverse=3`; metadata `MinValue=0`, `MaxValue=3`, `NumValues=4`. Focused proof is commit `2a9422810` via `patch_12_0_0_status_bar_fill_style_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; explicit combat-system enum data is authoritative for members and guarded fallback metadata matches. Claims are bounded to startup publication and exact values/types/metadata; status-bar rendering behavior, consumers, persistence, transitions, and lifecycle are not claimed. The five retail 12.0.0 `Enum.StatusBarInterpolation.*` and `Enum.StatusBarInterpolationMeta.*` rows are best-effort/behavioral: `Immediate=0`, `ExponentialEaseOut=1`; metadata `MinValue=0`, `MaxValue=1`, `NumValues=2`. Focused proof is commit `940e85e7c` via `patch_12_0_0_status_bar_interpolation_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; interpolation behavior, consumers, persistence, transitions, and lifecycle are not claimed. The six retail 12.0.0 `Enum.TableSecurityOption.*` and `Enum.TableSecurityOptionMeta.*` rows are best-effort/behavioral: `DisallowTaintedAccess=0`, `DisallowSecretKeys=1`, `SecretWrapContents=2`; metadata `MinValue=0`, `MaxValue=2`, `NumValues=3`. Focused proof is commit `4b8cd7c16` via `patch_12_0_0_table_security_option_enums.rs`; it asserts the complete exact family, Lua numeric types, no extras, and metadata. No runtime drift; guarded fallback is authoritative. Claims are bounded to startup publication and exact values/types/metadata; table-security semantics, consumers, persistence, transitions, and lifecycle are not claimed. Current totals are **1861 best-effort, 982 evidence-required, 2 exception-requested, and 565 untriaged rows** (3410 total).

The four retail 12.0.0 `Enum.TraitNodeEntryType.SpendCapstoneCircle`, `Enum.TraitNodeEntryType.SpendCapstoneSquare`, `Enum.TraitNodeEntryTypeMeta.MaxValue`, and `Enum.TraitNodeEntryTypeMeta.NumValues` rows are best-effort/behavioral: SpendCapstoneCircle=13, SpendCapstoneSquare=14; metadata MinValue=0, MaxValue=14, NumValues=15. Focused proof is commit `5ecf52224` via `patch_12_0_0_trait_node_entry_type_enums.rs`; it asserts the complete 15-member family, Lua numeric types, no extras, and metadata. No runtime drift; explicit addon_system.rs member data and guarded fallback metadata match. Claims are bounded to startup publication and exact changed values/types/metadata; trait-node entry semantics, consumers, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1865 best-effort, 982 evidence-required, 2 exception-requested, and 561 untriaged rows** (3410 total).

The three retail 12.0.0 `Enum.VasTransactionPurchaseResult` changed rows are best-effort/behavioral: `EndDbErrors=20096→20097`, metadata `MaxValue=20096→20097`, and `NumValues=144→145`. Focused proof is commit `f0d18a70b` via `patch_12_0_0_ui_enum_metadata.rs`; it asserts `DbHouseOwnerRestriction=20096`, `EndDbErrors=20097`, and the exact numeric metadata. No runtime drift; the guarded fallback is authoritative. Claims are bounded to startup publication and exact changed values/types; VAS transaction semantics, consumers, validation, persistence, transitions, and lifecycle remain unclaimed. Current totals are **1868 best-effort, 982 evidence-required, 2 exception-requested, and 558 untriaged rows** (3410 total).

The retail 12.0.0 changed `GetRaidTargetIndex` row is best-effort/behavioral: startup publishes callable `GetRaidTargetIndex(unit)` and returns nil for `player` when no raid marker is assigned. Focused proof is commit `492c4a189` via `tests/startup_api_stubs.rs`; it asserts the function is callable with a unit and returns nil without an assigned marker. Runtime publication is in `src/lua_api/globals/targeting_verbs.rs`; the current unmodeled marker state is the bounded root cause, with no drift for this contract. Claims are bounded to function publication and nil-without-marker; assigned-marker behavior and raid-target semantics remain unclaimed. Current totals are **1869 best-effort, 982 evidence-required, 2 exception-requested, and 557 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify FCT EnergyGains CVar removals

Classified exactly `removed:floatingCombatTextEnergyGains` and `removed:floatingCombatTextPeriodicEnergyGains` as best-effort/behavioral. Focused proof is commit `15a7936e6`, test `src/loader/tests/wow_api_globals/patch_12_0_0_cvar_removals.rs::test_patch_12_0_0_removed_nameplate_cvars`, asserting `GetCVar` and `GetCVarDefault` return nil for each old name. No drift: old names are absent from `src/cvars.yaml`; distinct `_v2` replacements exist without a semantic-equivalence claim. Evidence hashes: register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, defaults `e3241b1accf60051739f786962739dd7362d6f69f48ae5e134b5c4115280275c`, runtime `e4a1e46a35988dd137b95c4afeab0c95f6e65516ef8401f25397f3f2a1a13e6d`, focused test `1fb711dd4a77849a44d52f2f8f6850aeb3ae496a089ec2b936b7afb2c204598e`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Bounded claim: public getter absence only. Totals: **1921 / 1029 / 2 / 458 / 3410**.

## [2026-08-12] investigation | Classify added event producer gaps

The eighteen retail 12.0.0 added event rows PARTY_KILL, PLAYER_TARGET_DIED, REMOVE_NEIGHBORHOOD_CHARTER_SIGNATURE, SECURE_TRANSFER_CONFIRM_HOUSING_PURCHASE, SECURE_TRANSFER_HOUSING_CURRENCY_PURCHASE_CONFIRMATION, SETTINGS_PANEL_OPEN, SET_SEEN_PRODUCTS, SHOW_JOURNEYS_UI, SHOW_NEW_PRODUCT_NOTIFICATION, TOOLTIP_SHOW_ITEM_COMPARISON, TRAINING_GROUNDS_ENABLED_STATUS_UPDATED, TRANSMOG_CUSTOM_SETS_CHANGED, TRANSMOG_DISPLAYED_OUTFIT_CHANGED, TUTORIAL_COMBAT_EVENT, UNIT_DIED, UNIT_LOOT, UNIT_SPELL_DIMINISH_CATEGORY_STATE_UPDATED, and UPDATE_BULLETIN_BOARD_MEMBER_TYPE are evidence-required/unsafe. Runtime registration exists, but producer-backed firing, exact handler delivery, and payload semantics are unproven; TOOLTIP_SHOW_ITEM_COMPARISON requires callback-event probing and TUTORIAL_COMBAT_EVENT additionally requires restricted-event probing. Claims are bounded to authoritative event declarations and current registerability, with no spontaneous firing, ordering, lifecycle, or payload-semantic claims. Current totals are **1921 best-effort, 1052 evidence-required, 2 exception-requested, and 435 untriaged rows** (3410 total).


## [2026-08-12] investigation | Classify added retail CVar defaults

The thirteen retail 12.0.0 added CVars `enablePetBattleFloatingCombatText_v2`, `encounterTimelineEnabled`, `encounterTimelineHideForOtherRoles`, `encounterTimelineHideLongCountdowns`, `encounterTimelineHideQueuedCountdowns`, `encounterTimelineIconographyEnabled`, `encounterTimelineIconographyHiddenMask`, `encounterWarningsDefaultMessageDuration`, `encounterWarningsEnabled`, `encounterWarningsHideIfNotTargetingPlayer`, `encounterWarningsLevel`, `endeavorInitiativesLastPoints`, and `equipmentManager` are **best-effort/behavioral** for startup public getter/default publication only. Focused proof is commit `575893319` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts both GetCVar and GetCVarDefault exact strings (`1`, `0`, `3500`, or one-byte `string.char(1)` as applicable). No mutation, persistence, consumer, UI, initiative, equipment-manager, or replacement semantics are claimed; `enablePetBattleFloatingCombatText_v2` is not asserted equivalent to the removed old name. Evidence hashes: register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735b53e22c50b49a67`, defaults `e3241b1accf60051739f786962739dd7362d6f69f48ae5e134b5c4115280275c`, runtime `e4a1e46a35988dd137b95c4afeab0c95f6e65516ef8401f25397f3f2a1a13e6d`, test `c15ff01eb5f6beb08eeaed2a8417aef16a5255d549423adab003aa8c76c4aa8c`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Current totals are **1934 best-effort, 1088 evidence-required, 2 exception-requested, and 386 untriaged rows** (3410 total).

## [2026-08-12] investigation | Classify 15 additional added retail CVar defaults

Classified `externalDefensivesEnabled` and 14 floating-combat-text `_v2` CVar additions as best-effort/behavioral. Proof commit `a5a1a343a` and focused `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults` assert exact `GetCVar`/`GetCVarDefault` strings. Claims are bounded to startup getter/default publication; no rendering, mutation, persistence, consumer, or old-name equivalence semantics are claimed. Totals: **1984 / 1103 / 2 / 321 / 3410**.

The four retail 12.0.0 added APIs `hasanysecretvalues`, `issecrettable`, `issecretvalue`, and `mapvalues` are **evidence-required/unsafe**. The first two are not published; `issecretvalue` uses guarded local shallow-secret logic without proof of full retail semantics; `mapvalues` uses guarded shared/temporary fallbacks with only basic mapping behavior. Required state-backed/full-LoD probes cover secret API publication, zero/nil/primitives, direct/nested/mixed/tainted/protected/propagated values, non-table and empty-table inputs, boolean return types, and for mapvalues zero inputs, nils, callback arity/multiple returns/errors, result order/count, and Lua packing. Claims are bounded to current publication and model observations; no aliases or equivalence are claimed. Current totals are **1993 best-effort, 1107 evidence-required, 2 exception-requested, and 308 untriaged rows** (3410 total).

[2026-08-12] investigation | Classify twelve added nameplate CVar defaults

The twelve retail 12.0.0 nameplate CVar additions `nameplateShowFriendlyPlayerGuardians`, `nameplateShowFriendlyPlayerMinions`, `nameplateShowFriendlyPlayerPets`, `nameplateShowFriendlyPlayerTotems`, `nameplateShowFriendlyPlayers`, `nameplateShowOffscreen`, `nameplateShowOnlyNameForFriendlyPlayerUnits`, `nameplateSimplifiedTypes`, `nameplateSize`, `nameplateStackingTypes`, `nameplateStyle`, and `nameplateThreatDisplay` are **best-effort/behavioral**. Focused proof is commit `5d35250e5` via `src/loader/tests/wow_api_globals/patch_12_0_0_ui_enum_metadata.rs::test_patch_12_0_0_cvar_defaults`; it asserts both public getters return each exact startup/default string, including one-byte `0x02` values for `nameplateSimplifiedTypes`, `nameplateStackingTypes`, and `nameplateThreatDisplay`. Claims are bounded to startup getter/default publication; no rendering, UI, mutation, persistence, consumer, or later-epoch semantics are claimed. Current totals are **2005 best-effort, 1107 evidence-required, 2 exception-requested, and 296 untriaged rows** (3410 total).

The five retail 12.0.0 API additions `scrubsecretvalues`, `secretunwrap`, `secretwrap`, `securecallmethod`, and `string.concat` are **evidence-required/unsafe**. The first three are local pass-through or identity compatibility helpers with retail secret, taint, protected-value, nested-value, and variadic semantics unmodeled; `securecallmethod` is a permissive local dispatcher whose lookup, self/argument order, result packing, errors, and secure/taint boundary remain unproven; `string.concat` is absent and legacy `strconcat` is not an established alias. Required full-LoD/state-backed probes cover exact publication, zero/nil/primitives/multiple/nested/direct-secret/tainted/protected values, output count/order/types, errors, direct and `__index` method lookup, invalid receivers/names, secure boundaries, and string-concat coercion/error rules. Claims are bounded to source declarations and current local/missing behavior; no pass-through, alias, secret, taint, protected-value, or consumer semantics are claimed. Current totals are **2014 best-effort, 1115 evidence-required, 2 exception-requested, and 279 untriaged rows** (3410 total).

The ten retail 12.0.0 changed enum rows CharCustomizationTypeMeta.MaxValue/NumValues, EditModeSystemMeta.MaxValue/NumValues, EditModeUnitFrameSettingMeta.MaxValue/NumValues, HousingCatalogEntrySubtype.OwnedModifiedStack/OwnedUnmodifiedStack, and HousingCatalogEntrySubtypeMeta.MaxValue/NumValues are **best-effort/behavioral**. Existing focused startup tests prove exact post-change numeric publication and metadata; proof commits are e08b83b6a, 4d883c964, 509daf308, and c400d31be, all ancestors. Claims are bounded to startup enum/metadata values and types; no customization, Edit Mode, housing behavior, replacement semantics, or consumers are claimed. Current totals are **2024 best-effort, 1115 evidence-required, 2 exception-requested, and 269 untriaged rows** (3410 total).

The four retail 12.0.0 changed CVar rows `mountJournalGeneralFilters`, `mountJournalSourcesFilter`, `mountJournalTypeFilter`, and `nameplateGameObjectMaxDistance` are **best-effort/behavioral**. Focused proof is commit `896289b75` via `src/loader/tests/wow_api_globals/patch_12_0_0_mount_journal_nameplate_cvar_values.rs::test_patch_12_0_0_mount_journal_nameplate_cvar_values`; it passed 1 test, 0 failed, 1548 filtered and asserts both `GetCVar` and `GetCVarDefault` return exact strings: `0`, `0`, `0`, and `30.000000` respectively. Claims are bounded to startup getter/default publication; no Mount Journal/nameplate behavior, mutation, persistence, consumer, or semantic-equivalence claims are made. Evidence hashes: register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, defaults `e3241b1accf60051739f786962739dd7362d6f69f48ae5e134b5c4115280275c`, runtime `e4a1e46a35988dd137b95c4afeab0c95f6e65516ef8401f25397f3f2a1a13e6d`, focused test `271d73176d9cb53dd9f91a75ad76a49c2ec8fb9a3b834c44bffb22baa378d625`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Current totals are **2066 best-effort, 1149 evidence-required, 2 exception-requested, and 193 untriaged rows** (3410 total).

The 34 retail 12.0.0 removed housing Expert Gizmos rotation/translation CVar rows from `housingExpertGizmos_Rotation_BaseOrbScale` through `housingExpertGizmos_Translation_XRayLightAlpha` are **evidence-required/unsafe**. Checked-in `src/cvars.yaml` omission matches source removal, but public `GetCVar` and `GetCVarDefault` absence is not proven; each row requires a full-LoD nil/nil getter probe. Claims are bounded to source removal and checked-in omission only: no housing-gizmo, rendering, replacement, mutation, persistence, consumer, or lifecycle semantics are claimed. Current totals are **2119 best-effort, 1183 evidence-required, 2 exception-requested, and 106 untriaged rows** (3410 total).

The two retail 12.0.0 changed enum metadata rows `Enum.CraftingOrderResultMeta.MaxValue` and `Enum.CraftingOrderResultMeta.NumValues` are **best-effort/behavioral**: `MaxValue=46→48` and `NumValues=47→49`. Focused proof is commit `f6db33a61` via `src/loader/tests/wow_api_globals/patch_12_0_0_crafting_order_result_changed_enums.rs::test_patch_12_0_0_crafting_order_result_enum_changed_values`; it asserts both Lua numeric metadata values, the complete changed family, and `MinValue=0`, `MaxValue=48`, `NumValues=49`. Evidence hashes: source/register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, runtime `60a333e293bcc26af280e487e3b546231330e3cd9feb51c77aecb258dab3f3a6`, focused test `b5f52fb3534a7f508f8731261e7c79b3efe972b1636178ca5d1d8606c49c2f7b`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Claims are bounded to retail 12.0.0 startup Lua numeric metadata publication only; no crafting-order producer, interpretation, consumer, mutation, persistence, transition, or lifecycle semantics are claimed. Current totals are **2121 best-effort, 1183 evidence-required, 2 exception-requested, and 104 untriaged rows** (3410 total).

The three retail 12.0.0 changed globals `LE_GAME_ERR_HOUSING_RESULT_MISSING_EXPANSION_ACCESS`, `LE_GAME_ERR_HOUSING_RESULT_PERMISSION_DENIED`, and `LE_GAME_ERR_RECENT_ALLY_PIN_SERVER_ERROR` are **best-effort/behavioral**. Focused proof is commit `0df65552d` via `src/loader/tests/wow_api_globals/patch_12_0_0_housing_recent_ally_error_constants.rs::test_patch_12_0_0_housing_recent_ally_error_constants`; it asserts Lua numbers `1218`, `1219`, and `1233` respectively. Evidence hashes: source/register `6f26d194d0c3f721b3a071217cf69714f1278950512369272298735bdf44c863`, runtime `a92516cfb211c4674410ff3c7767c0421677f9a48ad31772b392df08b550deca`, focused test `1f7bc4f0300c9ee4dd61085335b5b1f8018112d41843cc8ad4b6cc9353582c4c`, discovery `4cd7ba572f01c0abf7308fe218230fe004983b0b49d4a752cdfedcf672e4a5e6`. Claims are bounded to retail 12.0.0 startup numeric global publication; no housing, guild/neighborhood, recent-ally, error production/interpretation, consumer, mutation, persistence, transition, UI, or lifecycle semantics are claimed. Current totals are **2124 best-effort, 1183 evidence-required, 2 exception-requested, and 101 untriaged rows** (3410 total).

## [2026-08-23] investigation | Resolve C_LootHistory empty-state slice

Promoted the retail/PTR `C_LootHistory` read surface from generated fallback behavior to a bounded state-backed model. Empty state returns fresh encounter/drop tables, explicit nil lookup results, and `GetLootHistoryTime() == 0.0`; the real `GroupLootHistoryFrame` can load and show its empty state with zero new Lua errors during `Show()`. Populated encounters, drops, rolls, event producers, persistence, and timer progression remain unmodeled. The generated-stubs audit now tracks five unresolved priorities.

## [2026-08-24] investigation | Restore Lua call frames after errors

Documented commit `ff01991aa`: direct `call_function_state`/`call_function_state_multi` calls now save and restore `LuaState` frame state (`top`, `base`, `ci`, and overflow status) when Lua execution fails. The focused `direct_state_call_restores_call_frame_after_lua_error` regression proves a failed call leaves `ci == 0` and a subsequent direct call returns `42`. This prevents the later `expected Lua closure in execute` cascade; remaining default-retail startup errors are not resolved by this slice.

## [2026-08-26] update | Document limited listfile canonical casing

Updated [`updating-blizzard-ui-to-a-new-patch`](../../updating-blizzard-ui-to-a-new-patch.md), [`casc-loading`](../specs/casc-loading.md), and [[casc-asset-cache]] for commit `2f88b2cab`. Ordinary community rows remain normalized lowercase; `data/listfile-overrides.csv` authoritatively replaces source display paths for normalized path and FDID resolution while preserving slash-normalized canonical casing. Generated rows sort by normalized path. Existing `index.md` catalog text remains accurate; no index change was needed.

## [2026-08-28] audit | Document timeout re-exec and current PartyFrame contract

Audited commit `e5947420e` and current PartyFrame commits `d86832096`/`87701437d`. Updated the prefork spec and system page to distinguish Linux ordinary `test_timeout!`/`with_timeout` exact-test re-exec from the prefork runner, including sibling-preserving assertion failures, process-tree timeout cleanup, visible-output forwarding, and the unchanged 120-second normal limit (1 second only in conformance). Clarified the PartyFrame investigation's hidden `PowerBarAlt` descendants after `87701437d`; its index summary remains accurate, so no index change was needed.

## [2026-08-27] update | Correct transparent-wrapper frame-level contract

Updated `investigations/transparent-wrapper-render-order.md` after current retail XML and bucket ordering showed the old opaque level-100 border regression was invalid. Documented that only regionless wrappers hoist descendants, wrappers with owned regions retain frame-level boundaries, and higher raw frame levels render after lower levels.

## [2026-08-27] update | Refresh PartyFrame tree against current retail XML

Updated `investigations/partyframe-tree.md` and its index summary. Current `EditModeSystemSelectionBaseTemplate` explicitly uses frame level 1000, so PartyFrame Selection/highlight/corner dump expectations are 1000/1001/1002 rather than the historical master LOW:3/4/5 values. Preserved the startup-hang root causes and recorded resolved sizing/coordinate follow-ups.

## [2026-08-28] update | Verify PartyFrame region draw-layer contract

Updated `investigations/partyframe-tree.md`, its index summary, and focused coverage. Portrait remains `BACKGROUND`/0; Flash and Name remain `ARTWORK`/0 through `GetDrawLayer()`. Texture and FontString raw dump strata/levels remain simulator diagnostics rather than client API assertions.

## [2026-08-28] fix | Handle nested timeout re-exec guards

Audited commit `14bfc8eb0`. Updated [[prefork-test-harness]] and the prefork spec to document that nested `with_timeout` calls execute in the guarded child with one handshake and no second re-exec boundary, and that multi-shard same-process tests use one outer timeout around the complete sequence with inner shard bodies unwrapped. The existing index summary and related timeout documentation remain accurate; no index change was needed.

## [2026-08-28] correction | Audit current PartyFrame portrait texture identity

Audited commit `640aa7fbb` against current retail portrait evidence. Updated [[partyframe-tree]], [[partyframe-portrait-composition]], and their index summaries: player and party portraits expose `GetAtlas() == nil` and numeric `GetTexture() == 237669`; `GetTextureFilePath()` resolves the authored `Interface\\TargetingFrame\\UI-Classes-Circles` path. Removed the stale legacy fallback claim; no code, test, PLAN.md, vendor, cache, or protected-file changes were made.
## [2026-09-01] investigation | Game menu Esc handlers
## [2026-09-01] investigation | Blizzard UI manifest drift

Added `investigations/blizzard-ui-manifest-drift.md`. `data/blizzard-ui-files/ptr.txt` had drifted to 310 addon directories against the 348 the Gethe `ptr` branch ships, contradicting the "full file list, no filtering" contract its own generator documents. The 39 unsynced directories include `Blizzard_ManagedFrameSystem` (a declared dependency of twelve synced TOCs), `Blizzard_StatusTrayManager` and `Blizzard_GameMenuEsc`; the last defines `RegisterGameMenuEscHandler`, which 25 Blizzard files call at file scope, so each of them lost every definition below the call — `EditModeManager.lua` before `RegisterSystemFrame`, which in turn aborted `OnSystemLoad` for every `EditModeSystemTemplate` frame. Regenerating the manifest and the bundled listfile subset, plus thirteen simulator-owned enum tables that only became reachable once the aborts cleared, drops startup errors on `client-ptr` from 659 to 26 and distinct messages from 56 to 4 (counting the `attempt to …` class; a broader count lands at 52 / 11). The managed-frame containers and the edit-mode system registration both start working as a result. Eight further defects surfaced once the aborts cleared and are recorded on the page: mask coverage decided by file name, `Blizzard_FrameXML` loading its legacy TOC, a backwards countdown on permanent auras, `C_StringUtil` rebuilt over its factory, the screenshot encoder's quality-15 WebP, the minimap mask stretched over the frame, the font `<Shadow>` element dropped by the XML loader, and a headless-only `SetHeight(836.5)` on the objective tracker that at the client's UI scale pushed it over the minimap. Then the 2x atlas art, which the render lookup never chose outside a one-name allow-list and which the client draws above 1 px per unit; a vertex-colour path that skipped the sRGB decode (solid 0.5 rendered 187, now 127), and a pixel-aligned measurement showing the simulator reproduces atlas texels 1:1 with the brightness lift off while the client capture carries a constant offset of about +45 with no pixel darker than 31 — most likely its display settings. A further round against the client capture found the atlas sampler set to nearest since April (stair-stepped icons above 1 px per unit; bilinear again, with a checker-ramp test), two more file-scope aborts on nil table keys (`LE_LFG_CATEGORY_LAIR` in Constants.lua, which took `QUEST_TAG_ATLAS` and the world map with it; `Enum.RolodexType.LegacyFriend` in RecentAlliesUtil.lua) and the legacy SharedXML TOC omitting `ListTemplates`; `lua-errors` goes 33 / 39 to 25 / 29. Two text defects followed from the same capture: `SetFontObject` took the loader's stale `__height` (small Blizzard fonts rendered at 12, not 10) and text shadows were offset upward and unscaled; both fixed with render-level tests. The page ends with the capture recipe: `wow-sim screenshot --ui-scale 1.6875` for a 1440p client at uiScale 0.9, applied before the canvas size so `ManageFramePositions` re-runs for the scaled UIParent. Two refuted leads are recorded on the page: the ptr cache was not stale, and importing the real `_retail_/WTF` layout makes the HUD worse rather than better.

## [2026-09-02] update | Global string table regenerated for 12.1.0

Extended `investigations/blizzard-ui-manifest-drift.md`. `data/global_strings.rs` was a client build behind: the 12.1.0 UI reads 16 strings it lacked (`LAIR`; the `SLASH_CAA_*` set that `TextToSpeechCommands.lua` formats in file-scope do-blocks, so the file aborted at line 652 and twenty command blocks never registered). The generator also read the CSV with `lines()` while 36 strings span lines inside a quoted field, keeping only the first line of each; it now uses `csv_util::read_csv_records` like the other generators. Regenerated from the wago.tools export of build 12.1.0.69497 (868 added, 165 changed, 33 WOWHACK/OUTFITTER strings dropped); `lua-errors` on client-ptr 25 / 29 → 22 / 26, the BattleNetFriendTags label assertion gone with the TextToSpeech abort.

## [2026-09-02] update | Blizzard UI manifest drift — second pass

Extended `investigations/blizzard-ui-manifest-drift.md` with the second pass over the client capture and the remaining startup errors: `lua-errors` on client-ptr 25 distinct → 0, `frame_positions` 15/28 → 28/28. Fixed on the branch: the stale `Blizzard_SettingsDefinitions_Frame` flavor TOC, `init_enum_globals` replacing sub-tables on the post-cleanup restore (four CooldownViewer aborts), housing and aura-button enums from the API documentation, the pet-battle queue stub and species, secure-environment replay of `Blizzard_CombatLogBase` and `Blizzard_FrameXMLUtil`, XML method handlers dispatching to the forbidden partition (TargetFrame and FocusFrame finally built), a font's `<Color>` and `FontString:SetVertexColor` reaching the renderer, `SetAtlas` no longer forcing custom button children visible, the atlas tables regenerated from 12.1.0 with override-sized `-2x` entries kept whole (micro-menu plates), glyph quads snapped to the pixel grid, single-texel tab strips sampled at their centre, the edit-mode cache parser no longer fabricating layouts, and the default chat window names.
