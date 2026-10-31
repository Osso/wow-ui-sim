# Consumer Selection and RED Proof

Lessons learned on 2026-10-23: cached Blizzard source is not proof of a runtime-selected consumer, and a passing consumer boundary is not necessarily producer acceptance. Findings below came from static reachability and test reviews; they do not claim executed RED/GREEN.

## Runtime-selected consumers

`data/blizzard-ui-files/<profile>.txt` defines synchronized cache inputs, not the implementation selected for a run. Trace the selected TOC, its filters/substitutions, and XML/Lua chain before claiming consumer applicability:

- `src/loader/mod.rs`: `find_toc_file`, `active_profile_toc_suffixes` and `select_profile_specific_toc_suffix` select TOC flavor; `discover_blizzard_addon_toc_pools_for_screen` applies screen/game-type restrictions and separates startup roots from the dependency/LoD pool.
- `src/toc/mod.rs`: `ParsedFileEntries::push_file_entry` applies `is_allowed_game_type`, substitutes `family_subdir`/`game_subdir`, then uses `selected_file_path`. `TocFile::is_game_type_restricted` handles addon-level restrictions.
- `src/loader/addon.rs`: `load_addon_files` visits parsed entries subject to phase, profile and environment checks. Selected source still needs a callable consumer path and its prerequisites.

**Forever StableUI:** `Blizzard_StableUI/Blizzard_StableUI.toc` selects `[Game]/Blizzard_StableUI.lua` and XML; `[Game]` resolves to `Camelot`. The selected Camelot implementation never calls `C_StableInfo.GetActivePetList`. The cached Standard implementation does, but is unselected. Its top-level arithmetic requires `PetConsts.STABLED_PETS_FIRST_SLOT_INDEX` and `EXTRA_PET_STABLE_SLOT`, absent from Forever's `PetConsts`; whole-file execution is therefore predicted to fail before that consumer. This was not runtime-tested. Do not force-load Standard or substitute modern constants to manufacture a Forever gap.

The upgrade/store audit found the same trap: cached Mainline upgrade calls were not selected on Mists/Era/Anniversary, and cached shared secure-store source was absent from the selected Retail/PTR/Forever TOC.

## Casting bar secrecy predicate

Retail's `Blizzard_UIPanels_Game_Mainline.toc:6,10` selects `Shared/CastingBarFrame.lua`. Its `IsInterruptable` method calls the locally captured predicate through lines 2 and 5. The consumer test verifies source provenance and the XML-created player bar before exercising both bar types. Its Retail RED is the nil call at line 5. Because both output cases also pass with a constant-false predicate, this is missing-publication and consumer-preservation proof, not proof of a native trigger. The [contract](../../specs/secret-auras.md#casting-and-threat-predicate-surface) records the unknown behavior and validation limits.

## Combat-log processor filter staging

[Filter-record contract](../../specs/combat-log-event-filters.md) separates the original ApplyFilterSettings lifecycle from native event matching. The cached secure Add/Clear declarations provide restriction markers but no argument or return signatures; missing signatures do not establish validation or default values. The staged producer retains opaque requests and exposes a simulator-only snapshot, without interpreting event/mask semantics. Retail/PTR execution is pending; case032/033 remain preservation evidence, not producer acceptance.

Do not classify Forever as inapplicable merely because this TOC says `mainline`: `src/toc/mod.rs::active_game_types` admits both `camelot` and `mainline` for Forever. Its cached CombatLogProcessor TOC selects the same secure Lua file. The current Retail/PTR-only stage does not cover that selected Forever consumer. Mists/Era/Anniversary reject the mainline-only TOC; Wrath also rejects it and has no processor in its cache.

## CF-2 supplied PvP match completion

The [completion contract](../../specs/pvp-match-completion.md) uses the existing `SimState.pvp_catalog.match_completed` snapshot. GREEN's `src/c_api/c_pvp.rs` callback pushes this boolean without mutation; namespace publication checks runtime ACTIVE Retail/PTR inside the backing field's epoch gate. `IsMatchActive` and `HasMatchStarted` remain separate and unchanged. No Forever producer or native lifecycle/default behavior is inferred. The paired RED removes only this callback and registration, retaining all test and documentation bytes.

The unchanged `cf2_pvp_completion` selector invokes the normal XML-created frame's installed script. It checks original method identity and Lua-file provenance, then observes exactly one call to that original method. Incomplete/active/completed/reset snapshots assert all eight actual event registrations, hidden presentation and no new Lua errors. Constant true contradicts snapshot 1; constant false and zero-return/nil no-ops contradict snapshot 3. Missing publication can error at the original completion call. The catalog and locklist selectors are preservation only, not independent producer RED.

This is static staging, not observed RED/GREEN or runtime selection proof. Historical PTR cache evidence predates the current 12.1.7 pin. Future PTR execution must first synchronize and verify a lane-private matching cache and use it in every phase; old cache evidence cannot establish current PTR selection. See the contract for bounded native gaps and the handoff for source-bound replay.

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

## See Also

- [[addon-loading]] — runtime loading and dependency closure.
- [[blizzard-ui-test-lanes]] — unit versus addon-bootstrap boundaries.

## CF-2 party minimum staging

The [bounded contract](../../specs/party-min-level-scenario-availability.md) and `tests/cfactory_party_minlevel.rs` use normal dependency-closure loading with no TOC overrides. Retail selects `Blizzard_GroupFinder_Mainline.toc`; pinned PTR uses the unified `Blizzard_GroupFinder.toc` with `[Family]` and game annotations. Both parsed chains include `Shared/ScenarioFinder_Shared.lua` and `Shared/ScenarioFinder.xml`. Tests require original frame parenting, cover/label children, XML OnShow/OnEvent bindings and shared-Lua source provenance before availability. Mists is not registered in this lane's normal suite; its LFR input is a separate boundary.

Each selector independently varies member and player levels and replaces catalog level bounds while preserving IDs within an environment. Each repeats with disjoint IDs in a second normally loaded environment to reject a constant choice-ID list; the random selector also shrinks two scenario rows to one to reject a constant count. The shared Lua captures `allScenarios = GetScenariosChoiceOrder()` at file load, so fixtures seed before normal loading and later replace records under those captured IDs; do not re-execute the file to refresh that local. Threshold50→55→65 and cover transitions reject constant catalogs, while ordinary-dungeon distractors reject unfiltered projections. Mists setup diagnostics mentioning IsLFREnabled are an explicit blocker, not party-minimum RED. No Mists behavior is added.

### Static producer preparation

The explicitly authorized static GREEN stage adds `C_PartyInfo.GetMinLevel` in [the C API](../../../src/c_api/c_party_info.rs), using existing player/active-party levels. [Legacy catalog globals](../../../src/lua_api/globals/battlefield_lfg_probes.rs) project existing subtype4 scenario records: specific IDs exclude random rows; random count/info retain catalog order and read live supplied fields. Both publication/filter gates use `client_profile::ACTIVE` for Retail/PTR only. No seeded values, second model or vendor change is introduced. The info projection covers the six fields consumed here; additional native outputs and argument/category/disconnected semantics remain unproven.

The RED sibling restores only those producer files to their merged staging contents. Lane tests are byte-identical. Both Retail/PTR exact selectors independently fail at original `ScenarioFinder_Shared.lua:58` (`GetMinLevel`), after original XML bindings and Lua provenance pass, then pass GREEN under both disjoint catalog-ID sets. Each profile records 24 specific and 26 random observations. Empty-catalog preservation passes in both phases/profiles. Pinned-master producer-equivalent stock changes Retail12/14→10/12 and PTR244/312→242/310, removing only each party-minimum primary/reporter pair; no additions or surviving count changes. Retail/PTR/Era all-target checks have zero compiler warnings; full workspace formatting and changed-line readability pass. Runtime calls used 28 fresh current CVar copies with unchanged-original guards. Mists remains the separate supplied `IsLFREnabled` boundary; no Mists, native-policy or full-suite claim. Empty-catalog checks are preservation, not producer RED. Static contradictions must be read per endpoint: the random selector does not independently reject an empty specific-ID projection because its random loop can satisfy availability.
