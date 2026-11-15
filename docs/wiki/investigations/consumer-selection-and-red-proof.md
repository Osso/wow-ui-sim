# Consumer Selection and RED Proof

Lessons learned on 2026-10-23: cached Blizzard source is not proof of a runtime-selected consumer, and a passing consumer boundary is not necessarily producer acceptance. Findings below came from static reachability and test reviews; they do not claim executed RED/GREEN.

## Runtime-selected consumers

`data/blizzard-ui-files/<profile>.txt` defines synchronized cache inputs, not the implementation selected for a run. Trace the selected TOC, its filters/substitutions, and XML/Lua chain before claiming consumer applicability:

- `src/loader/mod.rs`: `find_toc_file`, `active_profile_toc_suffixes` and `select_profile_specific_toc_suffix` select TOC flavor; `discover_blizzard_addon_toc_pools_for_screen` applies screen/game-type restrictions and separates startup roots from the dependency/LoD pool.
- `src/toc/mod.rs`: `ParsedFileEntries::push_file_entry` applies `is_allowed_game_type`, substitutes `family_subdir`/`game_subdir`, then uses `selected_file_path`. `TocFile::is_game_type_restricted` handles addon-level restrictions.
- `src/loader/addon.rs`: `load_addon_files` visits parsed entries subject to phase, profile and environment checks. Selected source still needs a callable consumer path and its prerequisites.

**Forever StableUI:** `Blizzard_StableUI/Blizzard_StableUI.toc` selects `[Game]/Blizzard_StableUI.lua` and XML; `[Game]` resolves to `Camelot`. The selected Camelot implementation never calls `C_StableInfo.GetActivePetList`. The cached Standard implementation does, but is unselected. Its top-level arithmetic requires `PetConsts.STABLED_PETS_FIRST_SLOT_INDEX` and `EXTRA_PET_STABLE_SLOT`, absent from Forever's `PetConsts`; whole-file execution is therefore predicted to fail before that consumer. This was not runtime-tested. Do not force-load Standard or substitute modern constants to manufacture a Forever gap.

The upgrade/store audit found the same trap: cached Mainline upgrade calls were not selected on Mists/Era/Anniversary, and cached shared secure-store source was absent from the selected Retail/PTR/Forever TOC.

## GameTooltip minimum-width consumer

**VERIFIED (Retail 12.1.5 cache and selected TOCs):** `Blizzard_SharedXML.toc:154` selects `SharedTooltipTemplates.lua`; the original `GameTooltip_InsertFrame` is defined there at line 197. Its line 213 comparison reads `tooltipFrame:GetMinimumWidth()` as one scalar value and may then call `SetMinimumWidth(frameWidth)`. `Blizzard_GameTooltip/Blizzard_GameTooltip_Mainline.toc:5,8` selects `Mainline/GameTooltip.lua`, whose original bodies call the helper at lines 568 and 605.

**PRESERVATION:** this consumer cannot distinguish the corrected two-result getter from the old one-result getter because the comparison observes only the first result. The optional test calls the original helper and checks its width update; it is not a RED selector for getter arity. **VERIFIED (Retail AddOns cache search):** searching `AddFontStrings` found only the generated API declaration at `Blizzard_APIDocumentationGenerated/FrameAPITooltipDocumentation.lua:10`; no runtime-selected Lua caller was found. The direct API test remains producer proof for pair storage, not consumer RED.

**UNKNOWN (R88n.7):** how stored FontString pairs are used by later layout or rendering remains unproved. See the [GameTooltip method contract](../../specs/game-tooltip-methods.md).

## Casting bar secrecy predicate

Retail's `Blizzard_UIPanels_Game_Mainline.toc:6,10` selects `Shared/CastingBarFrame.lua`. Its `IsInterruptable` method calls the locally captured predicate through lines 2 and 5. The consumer test verifies source provenance and the XML-created player bar before exercising both bar types. Its Retail RED is the nil call at line 5. Because both output cases also pass with a constant-false predicate, this is missing-publication and consumer-preservation proof, not proof of a native trigger. The [contract](../../specs/secret-auras.md#casting-and-threat-predicate-surface) records the unknown behavior and validation limits.

## Combat-log processor filter staging

[Filter-record contract](../../specs/combat-log-event-filters.md) separates the original ApplyFilterSettings lifecycle from native event matching. The cached secure Add/Clear declarations provide restriction markers but no argument or return signatures; missing signatures do not establish validation or default values. The staged producer retains opaque requests and exposes a simulator-only snapshot, without interpreting event/mask semantics. Retail/PTR execution is pending; case032/033 remain preservation evidence, not producer acceptance.

Do not classify Forever as inapplicable merely because this TOC says `mainline`: `src/toc/mod.rs::active_game_types` admits both `camelot` and `mainline` for Forever. Its cached CombatLogProcessor TOC selects the same secure Lua file. The current Retail/PTR-only stage does not cover that selected Forever consumer. Mists/Era/Anniversary reject the mainline-only TOC; Wrath also rejects it and has no processor in its cache.

## CF-2 supplied PvP match completion

The [completion contract](../../specs/pvp-match-completion.md) uses the existing `SimState.pvp_catalog.match_completed` snapshot. GREEN's `src/c_api/c_pvp.rs` callback pushes this boolean without mutation; namespace publication checks runtime ACTIVE Retail/PTR inside the backing field's epoch gate. `IsMatchActive` and `HasMatchStarted` remain separate and unchanged. No Forever producer or native lifecycle/default behavior is inferred. The paired RED removes only this callback and registration, retaining all test and documentation bytes.

The unchanged `cf2_pvp_completion` selector invokes the normal XML-created frame's installed script. It checks original method identity and Lua-file provenance, then observes exactly one call to that original method. Incomplete/active/completed/reset snapshots assert all eight actual event registrations, hidden presentation and no new Lua errors. Constant true contradicts snapshot 1; constant false and zero-return/nil no-ops contradict snapshot 3. Missing publication can error at the original completion call. The catalog and locklist selectors are preservation only, not independent producer RED.

This is static staging, not observed RED/GREEN or runtime selection proof. Historical PTR cache evidence predates the current 12.1.7 pin. Future PTR execution must first synchronize and verify a lane-private matching cache and use it in every phase; old cache evidence cannot establish current PTR selection. See the contract for bounded native gaps and the handoff for source-bound replay.

## Retail quest tooltip consumers and host input seams

**VERIFIED (Retail 12.1.5 cache and selected TOCs):** `Blizzard_ObjectiveTracker.toc:10,35` loads `Blizzard_ObjectiveTrackerShared.lua` and `Blizzard_QuestObjectiveTracker.lua`; the original item-button `OnEnter` at `Blizzard_ObjectiveTrackerShared.lua:88-91` calls `SetQuestLogSpecialItem`, and `QuestObjectiveTrackerMixin:OnBlockHeaderEnter` at `Blizzard_QuestObjectiveTracker.lua:116-121` calls `SetQuestPartyProgress`. `Blizzard_UIPanels_Game_Mainline.toc:75` selects `Mainline/QuestMapFrame.lua`, whose original call at line 2198 also uses `SetQuestPartyProgress`. `TooltipDataHandler.lua:536,589` maps these setters to `GetQuestLogSpecialItem` and `GetQuestPartyProgress`; the pinned declarations are `TooltipInfoDocumentation.lua:817,833`.

**VERIFIED (runtime fixture):** `tests/tooltip_quest_party_progress.rs` loads the discovered `Blizzard_ObjectiveTracker` dependency closure, checks the loaded original function sources, and invokes both original hover methods and `GameTooltip` accessors. Party selectors listed one exact test each and failed against the no-value producer at test-seam commit `00ce7a683`; the host-selected item hover also failed against the no-value producer at test commit `1f08da4d0`. Red transcripts and CVar hashes are in the lane handoff evidence.

**VERIFIED (Retail 12.1.5.70688, WM94/R88n.1):** successful party-progress data uses top-level type 24 and quest ID; the optional title row has type 17 and the quest ID; supplied participant headings use type 18 with GUID; status/objective rows use type 8 with `completed`, with optional objective counts and progress percentage. Missing-source and failed-producer paths return exactly zero Lua values.

**VERIFIED (repository):** `QuestPartyProgressSnapshot` is empty by default. The host supplies title, participant headings, row content, colors and the participant marker consumed by `ignoreActivePlayer`; the simulator does not derive a party-progress tree, objective state or active-player identity. The shared tooltip builder serializes the supplied data. **UNKNOWN:** getter-specific native D56 color/instance parity and return arity for a successful zero-row composition are not separately proved. The simulator returns zero values when its supplied snapshot emits no rows.

**VERIFIED (R88n.3/R88n.3a):** the indexed quest resolves through quest-ID request context to a cache record; the primary special-item ID and conditional candidate IDs come from that cache record, then inventory lookup resolves an existing item object. Its tooltip request carries item ID, owner GUID and item GUID. **VERIFIED (repository):** separate empty-by-default host inputs represent index-to-quest context, cache IDs/marker, and the selected inventory object. The producer checks that the selected ID is in the primary field or enabled candidate list and validates its current bag item object before shared composition. It does not model candidate predicates or upstream writers. **UNKNOWN:** cache/invalidation and inventory/object creation routes, candidate predicates, and owner-GUID effects in output. Native party/objective state writers also remain UNKNOWN.

## Retail tooltip serializer field consumers

**VERIFIED (Retail cache and selected TOCs):** `Blizzard_SharedXMLGame.toc:4,9-10` selects `TooltipDataHandler.lua` for Game/mainline. `Blizzard_UIPanels_Game_Mainline.toc:5-6,33` selects the Retail MerchantFrame source. **VERIFIED (optional test):** the fixture checks game-screen discovery and original Lua-source provenance for `TooltipDataRules` and `GameTooltip:SetItemByID`.

**VERIFIED (Retail 12.1.5.70688, R88n.4b):** the original rules read top-level `repairCost` and battle-pet fields, and read `price`/`maxPrice` from a type-11 line. **PRESERVATION (optional tests):** `tests/tooltip_serializer_fields.rs` passes distinct synthetic values through the selected original `ProcessInfo` rules; this does not establish a producer for those fields. **INFERRED (simulator policy):** the selected `SetItemByID` path maps positive `ItemInfo.sell_price` to a type-11 line's `price`; no `maxPrice` is emitted. The native eligibility predicate, line order and `maxPrice` source remain UNKNOWN.

## Retail Azerite Essence tooltip consumers

**VERIFIED (Retail 12.1.5 cache):** `data/blizzard-ui-files/retail.txt:879-882` lists the Azerite Essence UI files, `Blizzard_AzeriteEssenceUI.toc:1-3` marks the UI load-on-demand, and `retail.txt:3447` lists `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua`. Game-screen discovery includes `Blizzard_SharedXMLGame`; the Azerite test discovers the UI addon's TOC dependency closure with that addon as an explicit LOD root. **VERIFIED (pinned API docs):** `TooltipInfoDocumentation.lua:69-95` declares `GetAzeriteEssence` and `GetAzeriteEssenceSlot`. **VERIFIED (original Lua):** `Blizzard_AzeriteEssenceUI.lua:866-869` calls `SetAzeriteEssence`; its slot hover at `1256-1265` calls `SetAzeriteEssenceSlot`; `TooltipDataHandler.lua:564-565` maps both setters to those getters.

**VERIFIED (Retail 12.1.5.70688, R88n.1/R88n.2):** successful composition requires the essence, exact selected-rank and rank-one records, preserves TooltipData type 8 and essence ID, and returns zero values on the listed missing-source/failure paths. The main block uses rank-one description only in the active major slot; the passive block uses rank-one description whenever active in any slot. Otherwise, both use selected-rank actual text. Slot zero includes both blocks; other slots omit the main block.

**VERIFIED (Retail 12.1.5.70688, R88n.2):** the source-stage next rank is computed from the original requested/current rank. Next rank four disables the stage; every other next rank requires an exact power row. Next rank one selects a source heading and requires nonempty source text; higher next ranks select a next-rank heading followed by source text. Source text wraps when emitted. The slot getter omits this stage. The simulator keeps the verified heading condition but emits no source-description row because the faction-to-column mapping is UNKNOWN.

**VERIFIED (test fixture at `tests/tooltip_azerite_essence.rs:291-380`, test SHA `e7d90111473fdd9ed4fc8abeca5587d8e9486824`):** Game-screen discovery selects `Blizzard_SharedXMLGame`, the Azerite UI closure selects its load-on-demand addon, and the fixture loads the native GameTooltip/TooltipDataHandler source before it executes the original `Blizzard_AzeriteEssenceUI.lua` file. `debug.getinfo` checks both original `OnEnter` methods and both original accessor methods. The fixture does not load the full UI bootstrap or Azerite XML.

**VERIFIED (saved historical receipts):** the no-op producer test tree `06f389052fb109bae1e782c2050698e229c13fdb` failed the original essence and slot hover selectors against no-op base `fb8d4fe08501fe65fc9e9c24db7bc420431d2a00`. The source-stage receipt at `adf40e1152227ad2e5d4cf0aff56c8437a410033` failed at the next-rank heading assertion; it does not prove faction-source selection. These are historical test trees, not the updated test commit. **VERIFIED (test commit `9565e57ab`):** the original-consumer tests include two distinct backed essence IDs, ranks and slots plus an absent result per selector; the suite contains no faction-source or counter-value assertions. Final GREEN results belong to the later producer commit and report.

**VERIFIED (repository data):** `AzeriteEssence.csv` and `AzeriteEssencePower.csv` are exact-build 12.1.5.70391 Wago DB2 CSV exports, with hashes and row counts in `data/db2/retail-12.1.5.70391/tooltip-azerite-catalog-provenance.json`. **INFERRED:** the four native spell roles map to similarly named 70391 columns; the mapping is isolated in `src/c_api/azerite_essence_spell_selector.rs`. The generator retains the source columns as unused catalog data. **UNKNOWN:** faction-to-source-column selection, missing/empty source behavior, inline major/minor row order, the `powerState="gray"` predicate, and requirement-row expansion. The simulator emits no faction-specific source-description row, inline power row or requirement row; it does not claim native parity for those unknowns.

### R88n.3b quest-cache callback seam

**VERIFIED (Retail native):** an unready quest-cache lookup saves callback/context entries; cache removal copies and invokes them after unlinking the old record. **VERIFIED (simulator source):** `QuestSpecialItemInputs` stores only index-to-quest, cache-record and selected-object maps; it has no pending callback or removal route. The special-item seam remains UNKNOWN, so this lane adds no async behavior.

### R88n.4b LFG message seam

**VERIFIED (Retail native):** the LFG reward callback receives the enclosing decoded message, not one `JamLFGLockListSlot` element. **VERIFIED (simulator source):** the reward getter reads host-supplied `LfgTooltipRewardGroup` snapshots; this seam has no decoded message or callback argument to correct. The incoming-message route remains UNKNOWN and unchanged.

## Applicable registration and standalone RED

Register consumer tests only on evidenced applicable profiles, using the established file-level `#![cfg(any(feature = "client-…"))]` pattern or `post_wrath_ui_fixture!` when its six non-Wrath profiles match the consumer. The macro is defined in `tests/common/profile_fixtures.rs`; it is not a universal applicability predicate. Never return success at runtime for an inapplicable profile: libtest reports a vacuous PASS, even if stderr says N/A.

Each independently claimed RED selector must reject a constant/no-op producer on its own. A missing-member failure alone does not establish this. In the CF-2 reviews, separate match selectors admitted constant false and constant true respectively; combining inactive/incomplete, active/incomplete and inactive/complete observations in one selector defeated both. TTS checks needed two distinct preference pairs and reopen observations to reject constant getters/no-op setters.

Empty-catalog, empty-invitation, notification-only and already-saved-marker checks can preserve useful boundaries while allowing no-op producers. Label and count them as preservation, not producer RED; keep missing native or backing-state evidence explicit.

## CF-2 selected-unit aura cleanup

[Cleanup contract](../../specs/clear-blocked-auras.md) covers the existing normal CompactRaidFrames → RaidFrame → UnitFrame closure on Retail/PTR. The C API producer removes selected-unit entries from the existing effective `_blockedAuras` table; AddBlockedAura and enumeration still share that exact table. No shadow state, vendor changes or new default aura records are introduced. Publication uses `client_profile::ACTIVE` and does not extend other profiles.

Each of the three unchanged selectors restores party201/202 while player101 stays blocked, then restores player101 while party201/202 stay blocked. A constant return/no-op cannot restore the first selection; always-party1 fails the reverse selection; always-player fails the first selection; global clear violates opposite-unit preservation. Re-blocking rules out one-shot cleanup. The separate nil selector additionally requires a callable endpoint and rejection before mutation, under the user-authorized non-native best guess. These are static contradictions, not executed mutation or RED/GREEN proof. Initial manager errors and original pool creation/event ordering remain acceptance gates; no pool is supplied by the producer.

Static producer delivery does not establish native nil/invalid/secret/taint semantics, actual PTR70493 runtime selection or stock deltas. The slot must use the authenticated lane-private 12.1.7 PTR cache; historical 12.1.5 line receipts are not current runtime proof.

The aura fixture's installed OnEvent pointer comparison was invalid: [direct_method.rs](../../../src/lua_api/globals/create_frame/template_chain/builder_method_family/direct_method.rs#L234) creates a delegating XML method wrapper. Historical stock diagnostics show the original parent, Update and SetUnit identities match, while the installed script is that generated wrapper. The corrected setup retains original mixin provenance and observes its exact function during installed-handler dispatch. An unrelated event must preserve blocked player101; LOSS_OF_CONTROL_UPDATE must restore it while party201 stays blocked. Target assertions remain unchanged. This is not a post-load workaround overwrite; the closure fixture does not apply post-load workarounds.
## Supplied social squelch

The [squelch contract](../../specs/social-squelch.md) adds one environment-local boolean input, not an account or voice-provider policy. `SocialRestrictionsDocumentation.lua` declares the getter on Retail/PTR at line 96 and Mists at line 62. The original selected Mainline/Classic `LFGList.lua` uses it in the local find/start restriction functions.

`tests/social_squelch.rs` loads the discovered game-screen GroupFinder closure without source overrides. It checks the loaded function's Mainline/Classic provenance and uses the original XML-created CategorySelection buttons. One selector supplies false/true/false and asserts both enabled states and both restriction tooltips. Constant false fails the middle state; constant true fails the first state; a nil/no-op producer cannot satisfy the original calls. A second environment holds true while the first changes, checking isolation. Final byte-identical selectors were executed on Retail, PTR and Mists: exact discovery lists one case; producer-only RED fails at the original `IsSquelched` call (189/176/94), and GREEN passes both control states/tooltips, exact bool and environment isolation across false/true/false.

At frozen master `9a104c448`, stock maps are Retail16/18, PTR296/410 and Mists12/18 (keys/occurrences). GREEN code `2dc0a8759` yields14/16,294/408 and10/16 respectively: only each original squelch primary/reporter pair disappears, with no added keys or changed counts. All-target checks pass: Retail zero warnings, PTR/Mists only the three permitted `fast_handler_late_bind.rs` dead-code warnings. Twenty-two payloads retain unchanged original CVar stat/hash guards. Unrelated closure errors and stock residue remain recorded; no clean-startup claim is made.

## Supplied pet specialization catalog

The [supplied catalog contract](../../specs/stable-pet-spec-infos.md) keeps unknown membership distinct from an observed empty list. PTR70493 selects Standard StableUI through its standard/camelot TOC and `[Game]` substitution; the specialization dropdown at XML373 inherits the menu template's OnLoad, which invokes original Lua1203 and queries the catalog at1205. Retail's original OnLoad1221 does not query it: use Refresh1227 for producer proof. Forever selects Camelot and never consumes this getter, despite its generated declaration.

The producer returns fresh arrays/records from environment-local optional observations and raises the [contract's named unknown error](../../specs/stable-pet-spec-infos.md) when unset. Five byte-identical original-consumer selectors have executed RED/GREEN: PTR installed OnLoad1205, PTR/Retail Refresh1217/1229 and unset/clear recovery through each original entry. Empty/two/one/three cardinality transitions reject fixed-length producers; distinct supplied IDs/indexes/names and two-environment observations reject constant/no-op/global state. Four supplementary input/copy cases pass.

Refresh calls the original dropdown Hide before invoking the original method directly. All getter, iteration, selected-ID, sorting and SetupMenu errors propagate. Original DropdownButton.lua237–249 stores the generator at239 and calls GenerateMenu247 only when IsShown246 is true. Hiding defers exactly that menu generation, as Blizzard's comment specifies; no assertion depends on root descriptions, radio controls/initializers, icons, rendering or selection commands. Those are not acceptance claims, and no vendor function is replaced.

Both siblings retain the inert input scaffold with None default and a stock-unused admin setter. RED's target remains unregistered; source9584d91b9 plus that scaffold is the fresh master-equivalent baseline, not literally identical source. Source668df61c6→240956a6c yields PTR stock122/138→122/138: only the original nil-member primary/reporter become named-unknown primary/reporter, count1 each. Original OnLoad1205 emits the primary; Blizzard_ScriptErrors59,71–73 emits its reporter. Retail raw stock remains byte-identical0/0. All original CVar guards remain unchanged. All seven all-target checks at source-equivalent docs revision3cb0627e2 pass with zero Cargo JSON/plain compiler warnings. All36 runtime CVar guards and27 pinned input hashes remain unchanged; formatting/readability pass. W/M/E/A/F absence tests are compiled-not-executed here; runtime coverage is deferred to I1. Native catalog/default policy and complete menu generation remain unknown.

## Supplied modern stable active list

The [supplied-list contract](../../specs/stable-active-list.md) separates Retail/PTR observations from Forever's Camelot `stable_reads`. Re-authenticated Retail declarations are `StableInfoDocumentation.lua:15–22,193–214`; PTR 70493 uses `:19–26,215–241` and adds five loyalty/happiness/experience fields. Retail's bare StableUI TOC selects the getter at `Blizzard_StableUI.lua:447`; PTR's `[Game]` entry selects `Standard/Blizzard_StableUI.lua:458`, not the old shared line447 citation.

The producer preserves supplied list order, copies declared records and ability arrays, and has no initial observation. Admin replacement, ordinal update/append and removal are simulator inputs, not native ordering or pet movement rules. An unset observation raises an explicit `native-unknown` error; an explicitly supplied empty list is not a native-default claim. Byte-identical lane selectors execute producer-free RED at original Retail:447/PTR:458, then GREEN populated/add/update/remove/empty, two-environment isolation, fresh snapshots and invalid-input recovery. Each uses original XML slots and original Refresh/SetPet identities and observes the active loop before the secondary-pet query. Producer authorship before observed RED was explicitly authorized and reported; RED execution preceded GREEN execution.

PTR first failed at Standard Lua:11, before defining Refresh. The separately authorized five-member PetConsts prerequisite follows the [complete pinned literal/alias/arithmetic chains](../../specs/stable-active-list.md#ptr-selected-consumer-prerequisites) and is identical in both siblings. PTR stock remains 134/154 but replaces exactly the top-level nil-constant primary/reporter pair with a `StablePetSpecializationMixin:OnLoad` missing `GetAvailablePetSpecInfos` pair at:1205 (count 1 each). XML:373–378 supplies that mixin on a dropdown inheriting Mainline MenuTemplates.xml:3,30 OnLoad; ScriptErrors.lua:59,71–73 emits the reporter. The old top-level error aborted the chunk before defining this callback. There are no additional record-producer stock changes. Final Retail stock remains 8/10 with no raw-key/count changes. All seven canonical all-target checks pass with zero compiler warnings at `babc8a54e`; formatting and changed-line readability pass. All 29 runtime payloads use fresh current CVar copies and retain unchanged original stat/hash guards. The final documentation update does not change compiled inputs.

The historical deferred `api_shape_stable_active_list.rs` is absent from master e98282ab8 and remains deferred by explicit decision. Its full-closure zero-errors, full Refresh completion and secondary `GetStablePetInfo` obligations are unclaimed. PTR specialization observations are covered by the separate [catalog contract](../../specs/stable-pet-spec-infos.md), not this active-list proof; unrelated closure diagnostics are retained without filtering. This is active-list proof, not a clean StableUI claim.

## Sources

- [TOC resolver](../../../src/toc/mod.rs) and [loader selection](../../../src/loader/mod.rs) — selection rules verified against source.
- [Addon execution](../../../src/loader/addon.rs) — parsed-file execution gates.
- [Forever manifest](../../../data/blizzard-ui-files/wowforever.txt) — synchronized inputs, not runtime selection.
- [Forever constants](../../../src/c_api/forever_finite_constants.rs) — StableUI prerequisite mismatch.
- [Profile fixtures](../../../tests/common/profile_fixtures.rs) — compile-time registration mechanism.
- [Retail Blizzard UI manifest](../../../data/blizzard-ui-files/retail.txt) — source-file and addon selection inputs.
- [Round 7 tooltip contract](../../specs/tooltip-r7-getters.md) — API contract and bounded evidence.
- [Azerite catalog provenance](../../../data/db2/retail-12.1.5.70391/tooltip-azerite-catalog-provenance.json) — exact-build DB2 inputs.
- [Retail CharacterCreate probe](../../../tests/glue_character_select.rs) — selected Glue TOC and original consumer preservation.
- [Texture inheritance resolver](../../../crates/sim-xml/src/template.rs) — inherited `parentArray` merge.

## See Also

- [[addon-loading]] — runtime loading and dependency closure.
- [[blizzard-ui-test-lanes]] — unit versus addon-bootstrap boundaries.
- [[lua-api]] — C API contracts and implementation boundaries.

## CF-2 party minimum staging

The [bounded contract](../../specs/party-min-level-scenario-availability.md) and `tests/cfactory_party_minlevel.rs` use normal dependency-closure loading with no TOC overrides. Retail selects `Blizzard_GroupFinder_Mainline.toc`; pinned PTR uses the unified `Blizzard_GroupFinder.toc` with `[Family]` and game annotations. Both parsed chains include `Shared/ScenarioFinder_Shared.lua` and `Shared/ScenarioFinder.xml`. Tests require original frame parenting, cover/label children, XML OnShow/OnEvent bindings and shared-Lua source provenance before availability. Mists is not registered in this lane's normal suite; its LFR input is a separate boundary.

Each selector independently varies member and player levels and replaces catalog level bounds while preserving IDs within an environment. Each repeats with disjoint IDs in a second normally loaded environment to reject a constant choice-ID list; the random selector also shrinks two scenario rows to one to reject a constant count. The shared Lua captures `allScenarios = GetScenariosChoiceOrder()` at file load, so fixtures seed before normal loading and later replace records under those captured IDs; do not re-execute the file to refresh that local. Threshold50→55→65 and cover transitions reject constant catalogs, while ordinary-dungeon distractors reject unfiltered projections. Mists setup diagnostics mentioning IsLFREnabled are an explicit blocker, not party-minimum RED. No Mists behavior is added.

### Static producer preparation

The explicitly authorized static GREEN stage adds `C_PartyInfo.GetMinLevel` in [the C API](../../../src/c_api/c_party_info.rs), using existing player/active-party levels. [Legacy catalog globals](../../../src/lua_api/globals/battlefield_lfg_probes.rs) project existing subtype4 scenario records: specific IDs exclude random rows; random count/info retain catalog order and read live supplied fields. Both publication/filter gates use `client_profile::ACTIVE` for Retail/PTR only. No seeded values, second model or vendor change is introduced. The info projection covers the six fields consumed here; additional native outputs and argument/category/disconnected semantics remain unproven.

The RED sibling restores only those producer files to their merged staging contents. Lane tests are byte-identical. Both Retail/PTR exact selectors independently fail at original `ScenarioFinder_Shared.lua:58` (`GetMinLevel`), after original XML bindings and Lua provenance pass, then pass GREEN under both disjoint catalog-ID sets. Each profile records 24 specific and 26 random observations. Empty-catalog preservation passes in both phases/profiles. Pinned-master producer-equivalent stock changes Retail12/14→10/12 and PTR244/312→242/310, removing only each party-minimum primary/reporter pair; no additions or surviving count changes. Retail/PTR/Era all-target checks have zero compiler warnings; full workspace formatting and changed-line readability pass. Runtime calls used 28 fresh current CVar copies with unchanged-original guards. Mists remains the separate supplied `IsLFREnabled` boundary; no Mists, native-policy or full-suite claim. Empty-catalog checks are preservation, not producer RED. Static contradictions must be read per endpoint: the random selector does not independently reject an empty specific-ID projection because its random loop can satisfy availability.

## Retail token tracker startup ownership

The C inventory's old TokenTracker diagnostic used `load_addon`, which does not
perform the application's per-addon startup notification. Its missing-tracker
failure was a fixture omission, not producer RED. Current application startup
registers LoadWith declarations, discovers startup addon kinds, calls
`load_startup_addon`, and restores globals after EnvironmentCleanup. Retail Full
loads dispatch ADDON_LOADED through that existing loader.

`tests/stub_c_startup_token_tracker.rs` follows that flow without a manual event,
source override, fake tracker, or post-load repair. It verifies selected Mainline
UIPanels/bare TokenUI TOCs and original manager/method source, then checks the XML
Border and OnLoad pool and calls the original capacity consumer across widths
100/250/100 (2/5/2 tokens using original XML width50). Baseline at3e5eac938 passes
one exact test with no recorded startup Lua errors. This is preservation evidence,
not RED or new native ADDON_LOADED evidence.

The Retail-only cleanup omits the stale token-tracker post-load workaround and
its compiled module. It no longer replays the handler or fabricates a bare frame
when startup is broken. Other profiles retain their existing workaround: Retail
success does not establish their lifecycle. A library guard checks the absence
of that Retail workaround registration. Final changed-source gates are recorded
separately; baseline success alone is not a final acceptance claim.

The remaining C inventory defaults are not fixed by this cleanup. TransmogUtil's
Lua-owned fallback and ContainerFrameContainer scaffold need their own selected
consumer/removal proof; legacy container tuple applicability is unproven. Native
item/spell targeting, professions, loot, merchant and transmog catalog/state
questions remain open. No native policy, vendor Lua/XML or cache bytes change.

## Retail TransmogUtil location ownership

The next C slice retains the unrelated unresolved
`GetBestItemModifiedAppearanceID` fallback but removes Retail's synthetic
`GetTransmogLocation`/`CreateTransmogLocation` and their fabricated location
methods. Other profiles retain the historical location fallback. No native slot
producer or vendor source changes; `c_transmog_outfit_info/slots.rs` still labels
its layout positions INFERRED.

The optional `stub_c_transmog_location_ownership` selector uses the application's
normal startup discovery, LoadWith registration, kind-aware load, loaded-addon
skip and cleanup restoration. It checks selected TOCs and original initializer,
helpers and mixin provenance. It does not replay the initializer or fabricate
locations: primary/secondary shoulders and both hand appearance/illusion entries
must already exist in the original initialized cache. Two passes check stable
cache identity, fresh equivalent construction, distinct equality, type/hand/
secondary predicates, and original GetCameraVariation's explicit true/false
branches (also used by original TransmogTemplates). Constants/no-op location
objects cannot satisfy all those observations.

Baseline at743765e13 passed this exact selector once with no recorded startup
Lua errors. It demonstrates existing original ownership and supports removal
preservation, not producer RED or native topology verification. Final gates use
the current merged base rather than asserting baseline source equivalence. The
Retail library boundary checks that pre-UI bootstrap does not publish either
location helper, leaves existing helpers unchanged, and separately preserves the
unresolved best-item helper. Missing native catalogs must remain blockers, not
be seeded to manufacture a passing consumer.

## Retail container parentArray ownership

The next C slice removes only the Retail bootstrap ContainerFrameContainer/
ContainerFrames table scaffold. Locale globals and legacy item-info forwarding
are unchanged; their unresolved native/legacy contracts are not settled by this
cleanup. Other profiles retain the scaffold pending independent ownership proof.

The optional `stub_c_container_frame_ownership` selector loads actual startup,
verifies selected Mainline UIPanels and original Lua enumerators, then checks the
XML owner's UIParent and all six real bag instances' parent/parentArray identities.
It calls original OnCombinedBagSettingChanged across false/true/false/true/false:
original container enumeration alternates six/seven members and original bag
enumeration alternates individual bags/combined frame. Owner and array identity
remain stable. No fake frames, membership-table mutation, native provider,
manual event or post-load repair is used. Baseline da31ffbf118 passed one exact
selector with no recorded startup diagnostics. This is original-owner/removal
PRESERVATION, not producer RED. A Retail library boundary separately proves no
bootstrap owner/array synthesis and preserves other legacy wrappers.

## Retail CharacterCreate consumer selection

**VERIFIED (Retail 12.1.5 cache):**
`Blizzard_CharacterCreate.toc:2,5-6` selects the original Lua/XML on Glue. The
`retail_character_create_transition_preserves_original_surfaces` test verifies
the TOC selected by `discover_blizzard_addons_for_screen` for
`ScreenKind::CharacterSelect`, then opens `charcreate` through
`GlueParent_SetScreen`. It confirms original Lua source for `OnLoad`,
`UpdateForwardButton`, and `SetCharacterRace`, plus four XML-created `BGTex`
textures.

**PRESERVATION:** the same consumer checks Human/Alliance and Orc/Horde as
distinct faction outputs; Pandaren/Orc and Demon Hunter/Death Knight records;
and the original 0.75/0.6/0.7/0.8 alpha results on each of the four distinct
`BGTex` children in XML order. Original `UpdateMode` reaches class/race and
customization states without calling `NavForward` or CreateCharacter: Back is
the exact `BACK` source string in both tested states, while Forward changes
from `CUSTOMIZE` to the original `NEXT` or `FINISH` string. The second
`parentArray` resolver input and explicit override are covered by a focused
resolver test; this is not runtime Azerite consumer proof. The resolved
`parentArray="BGTex"` issue and candidate dispositions are recorded in
[Lua-owned fallback retirement](lua-owned-fallback-retirement.md#retail-character-create-fallback-retirement-queue-462).
