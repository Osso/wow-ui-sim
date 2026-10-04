
Step 1: Read handoff. Active goal: integrate only staged test/fixture/spec; classify publication observations, prove negative controls, GREEN, fmt/check, commit and ledger. No simulator/vendor/wiki/coverage edits or operational actions.

Step 2: Clean worktree at master 102d7e9be; copied staged test, fixture, spec. Inspected helper help and shared runner jobs settings.

Step 3: Shared native_build_hosts.py native_build hard-codes -j8; help exposes no jobs flag. Cargo passthrough -j4 supplied after -j8, plus CARGO_BUILD_JOBS=4. First RED command: python3 scripts/build-host.py --build-host local --test -j4 --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration patch_12_1_0_publication_sweep -- --test-threads=1; base 102d7e9be + staged files. No prior proof exists.
First run exit 1; captured once to p1210-r2-first.log.

Step 4: Cargo rejects duplicate --jobs (first.log), so passthrough cannot cap hard-coded -j8. No compilation occurred. Use an invocation-local Python wrapper that loads unmodified build-host.py and changes only native runner Cargo argv -j8 to -j4 in memory; shared files unchanged. Retirement: shared runner exposes jobs override.
First capped compile/run exit 101; p1210-r2-compile.log.

Step 5: Compiled successfully; complete first RED output contains 778 IDs, 147 non-ok. No startup infrastructure failure. Reviewing all non-ok observations before baselining.

Step 6: First RED: 631 publication-ok, 147 non-ok (91 global, 36 member, 6 CVar, 5 object-method, 4 event, 5 unprobeable). Compilation warning-free. Found probe defect: unqualified removed globals use raw lookup twice instead of ordinary lookup; correct generically before baseline.

Step 7: Generic global probe now records ordinary and raw lookup, guarding removed globals against fabricated lookup. Added full scratch-register override for negative controls. Formatted before implementation commit. Prior RED proof invalidated only for global probe/read-register changes.
Generic probe correction run exit 101; p1210-r2-probe-fix.log.

Step 8: Corrected probe RED still reports 631 ok / 147 non-ok, no compiler warnings. All rows observed, no probe-error; unsupported RadialProgress factory and CHAT_MSG_* remain explicitly unprobeable. Reviewing CVar/event details and raw-only namespace gaps.

Step 9: Scratch full register changes three previously OK rows: global and namespace wrongly expected absent, concrete event replaced with unknown event. Source register untouched; expected each becomes non-ok. Test code unchanged from 4cf15e68f, previous RED valid; this invocation verifies negative controls only.
Negative control exit 101; log p1210-r2-negative.log.

Step 10: Negative controls confirmed exactly three extra failures (150 vs 147): GetSpecializationSystem absent, C_AuraContainerUtil.ProcessAuraTooltipBackdropOptions absent, deliberately unknown event. No unexplained changed rows. Reviewed baseline contains 147 IDs: 142 direct publication/absence failures, 4 unsupported RadialProgress factory rows, 1 unexpandable CHAT_MSG_* inventory row. The wildcard is an audit limitation, not claimed as a simulator implementation defect. Wrote exact-set fixture; source inventory untouched.

Step 11: Committed exact 147-row baseline and spec with factory-kind table and negative-control evidence. GREEN invocation uses default committed register, no P1210_SWEEP_REGISTER; previous probe proof retained, fixture changes require exact-set rerun.
GREEN invocation exit 0; log p1210-r2-green.log; final per-ID output p1210-sweep-result.json.

Step 12: GREEN at bcdaf87d2: one sweep test passed, 778 results, 631 ok / 147 expected non-ok; no warnings. Scratch controls not active. Readability audit: no warning suppressions or nested Rust control flow; classify_entry (42 lines, JSON serialization) and read_register (38 lines, validation) exceed 30-line helper guideline; optional extraction deferred because task limits changes to probe correctness. Cache provenance inspected read-only.

Step 13: Final spec evidence committed at c72b7e8b332d1b9eb1ade4e0a9b50f4bd5266c4d. Test/fixture unchanged from GREEN bcdaf87d2, proof remains valid. Final gate requested: cargo fmt --check; cargo check --locked -j4 --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 with CARGO_BUILD_JOBS=4. No earlier check proof exists.
Final cargo check exit 0, output captured once to p1210-r2-check.log.

Step 14: Final artifact reconciliation complete. cargo fmt, cargo fmt --check and cargo check all exit 0; check warning-free. Worktree clean; only three authorized tracked paths changed. No push/merge/deploy/PR, no simulator/vendor/wiki/coverage edits.

## Final result
Commit SHA: `c72b7e8b332d1b9eb1ade4e0a9b50f4bd5266c4d` (branch `p1210-r2`; base `102d7e9be23221ab46880e93b016f4de38dee44f`).
Changed files:
- `docs/specs/patch-12-1-0-publication-sweep.md`
- `tests/data/patch_12_1_0_sweep_known_gaps.json`
- `tests/patch_12_1_0_publication_sweep.rs`

### Publication matrix
| Section | Direction | OK | Non-OK |
|---|---|---:|---:|
| global-api | added | 117 | 30 |
| global-api | changed | 11 | 1 |
| global-api | removed | 10 | 9 |
| framexml | added | 260 | 77 |
| framexml | changed | 0 | 0 |
| framexml | removed | 114 | 10 |
| scriptobjects | added | 7 | 1 |
| scriptobjects | changed | 0 | 0 |
| scriptobjects | removed | 0 | 0 |
| widgets | added | 37 | 8 |
| widgets | changed | 11 | 0 |
| widgets | removed | 0 | 0 |
| events | added | 40 | 3 |
| events | changed | 1 | 1 |
| events | removed | 1 | 1 |
| cvars | added | 20 | 3 |
| cvars | changed | 0 | 0 |
| cvars | removed | 2 | 3 |

**Total: 778 IDs; 631 OK; 147 non-OK.** Of non-OK: 142 direct runtime publication/absence failures, four unavailable RadialProgress factory probes, one wildcard inventory limitation. These are default Game-startup observations; optional LoD/Glue publication remains outside scope.

### Genuine startup-surface gaps grouped by probe kind

#### global (91)
| Source ID | Direction | Observation |
|---|---|---|
| `wt-framexml-AddBehavioralMessagingTrayToStatusFrames-626` | added | raw=nil; lookup=nil |
| `wt-framexml-AddGMChatStatusFrameToStatusFrames-628` | added | raw=nil; lookup=nil |
| `wt-framexml-AddWowSurveyStatusFrameToStatusFrames-630` | added | raw=nil; lookup=nil |
| `wt-framexml-AlliedRacesFrame_TryShow-631` | added | raw=nil; lookup=nil |
| `wt-framexml-ArchaeologyFrame_ToggleUI-635` | added | raw=nil; lookup=nil |
| `wt-framexml-ArcheologyDigsiteProgressBar_OnSurveyCast-636` | added | raw=nil; lookup=nil |
| `wt-framexml-ArtifactFrame_OnTraitsRefunded-638` | added | raw=nil; lookup=nil |
| `wt-framexml-AzeriteEmpoweredItemUI_LoadUI-650` | added | raw=nil; lookup=nil |
| `wt-framexml-AzeriteEssenceUI_LoadUI-651` | added | raw=nil; lookup=nil |
| `wt-framexml-BattlefieldMap_ToggleUI-652` | added | raw=nil; lookup=nil |
| `wt-framexml-BehavioralMessagingTray_OnNotification-654` | added | raw=nil; lookup=nil |
| `wt-framexml-BehavioralMessaging_LoadUI-653` | added | raw=nil; lookup=nil |
| `wt-framexml-ChallengeModeCompleteBanner_OnChallengeModeCompleted-664` | added | raw=nil; lookup=nil |
| `wt-framexml-CombatText_LoadUI-680` | added | raw=nil; lookup=nil |
| `wt-framexml-ContributionCollectionFrame_LoadUI-689` | added | raw=nil; lookup=nil |
| `wt-framexml-CovenantCallings_LoadUI-704` | added | raw=nil; lookup=nil |
| `wt-framexml-DebugTools_LoadUI-705` | added | raw=nil; lookup=nil |
| `wt-framexml-EncounterJournal_OpenToTieredEntrance-712` | added | raw=nil; lookup=nil |
| `wt-framexml-EventTrace_LoadUI-713` | added | raw=nil; lookup=nil |
| `wt-framexml-ExpansionTrial_LoadUI-714` | added | raw=nil; lookup=nil |
| `wt-framexml-GMChatFrame_OnWhisperFromGM-767` | added | raw=nil; lookup=nil |
| `wt-framexml-HideAuctionHouseFrame-785` | added | raw=nil; lookup=nil |
| `wt-framexml-HideBarberShopFrame-786` | added | raw=nil; lookup=nil |
| `wt-framexml-HideBlackMarketFrame-787` | added | raw=nil; lookup=nil |
| `wt-framexml-HideGarrisonMissionFrames-788` | added | raw=nil; lookup=nil |
| `wt-framexml-HideGarrisonShipyardFrame-789` | added | raw=nil; lookup=nil |
| `wt-framexml-HideGuildBankFrame-791` | added | raw=nil; lookup=nil |
| `wt-framexml-HideItemUpgradeFrame-794` | added | raw=nil; lookup=nil |
| `wt-framexml-HideProfessionsCustomerOrdersFrame-795` | added | raw=nil; lookup=nil |
| `wt-framexml-HouseFinderFrame_LoadUI-797` | added | raw=nil; lookup=nil |
| `wt-framexml-HousingBulletinBoardFrame_LoadUI-798` | added | raw=nil; lookup=nil |
| `wt-framexml-HousingControls_LoadUI-799` | added | raw=nil; lookup=nil |
| `wt-framexml-HybridMinimap_LoadUI-807` | added | raw=nil; lookup=nil |
| `wt-framexml-IsPlayerAtEffectiveMaxLevel-1012` | removed | raw=function; lookup=function |
| `wt-framexml-IslandsPartyPoseFrame_TryShow-819` | added | raw=nil; lookup=nil |
| `wt-framexml-KioskFrame_HandlePlayerEnteringWorld-830` | added | raw=nil; lookup=nil |
| `wt-framexml-Kiosk_LoadUI-829` | added | raw=nil; lookup=nil |
| `wt-framexml-LandingSoulbinds_LoadUI-831` | added | raw=nil; lookup=nil |
| `wt-framexml-MacroFrame_SaveMacro-1017` | removed | raw=function; lookup=function |
| `wt-framexml-MovePad_LoadUI-842` | added | raw=nil; lookup=nil |
| `wt-framexml-NPE_InitializeIfLoaded-857` | added | raw=nil; lookup=nil |
| `wt-framexml-OpenEncounterJournalToJourney-859` | added | raw=nil; lookup=nil |
| `wt-framexml-OpenEncounterJournalToTieredEntrance-860` | added | raw=nil; lookup=nil |
| `wt-framexml-OpenOrderHallTalentUI-862` | added | raw=nil; lookup=nil |
| `wt-framexml-OpenPlayerSpellsToGlyphTarget-863` | added | raw=nil; lookup=nil |
| `wt-framexml-PVPUI_LoadUI-869` | added | raw=nil; lookup=nil |
| `wt-framexml-PlayerChoiceFrame_TryShow-867` | added | raw=nil; lookup=nil |
| `wt-framexml-PlayerChoiceToggle_TryShow-868` | added | raw=nil; lookup=nil |
| `wt-framexml-RaidNotice_AddMessage-1032` | removed | raw=function; lookup=function |
| `wt-framexml-RaidNotice_Clear-1034` | removed | raw=function; lookup=function |
| `wt-framexml-RaidNotice_FadeInit-1035` | removed | raw=function; lookup=function |
| `wt-framexml-RaidNotice_UpdateSlot-1038` | removed | raw=function; lookup=function |
| `wt-framexml-RestoreGMChatFrameSession-881` | added | raw=nil; lookup=nil |
| `wt-framexml-ShouldDisplaySpellCooldown-888` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowAdventureMapFrameForFollowerType-890` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowArtifactFrame-891` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowArtifactRelicForgeFrame-892` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowAuctionHouseFrame-893` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowBarberShopFrame-894` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowBlackMarketFrame-895` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowChallengesKeystoneFrame-896` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowFlightMapFrame-897` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowGarrisonCapacitiveDisplayFrame-898` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowGarrisonMissionFrameForFollowerType-899` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowGarrisonRecruiterFrame-900` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowGarrisonShipyardFrame-901` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowGuildBankFrame-902` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowHeirloomsJournalToClosestUpgradeablePage-903` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowItemSocketingFrame-906` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowItemUpgradeFrame-907` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowMatchCelebrationPartyPoseFrame-908` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowPendingPlayerChoiceResponseUI-909` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowPerksProgramFrame-910` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowProfessionEquipmentHelpTip-911` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowProfessionsCustomerOrdersFrame-912` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowProfessionsFrame-913` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowRemixArtifactFrame-916` | added | raw=nil; lookup=nil |
| `wt-framexml-ShowRuneforgeFrame-917` | added | raw=nil; lookup=nil |
| `wt-framexml-SoulbindViewer_LoadUI-935` | added | raw=nil; lookup=nil |
| `wt-framexml-TryShowAnimaDiversionFrame-945` | added | raw=nil; lookup=nil |
| `wt-framexml-TryShowCovenantPreviewFrame-946` | added | raw=nil; lookup=nil |
| `wt-framexml-UIParent_ManageFramePositions-1070` | removed | raw=function; lookup=function |
| `wt-framexml-WarfrontsPartyPoseFrame_TryShow-958` | added | raw=nil; lookup=nil |
| `wt-framexml-WowSurveyStatusFrame_OnSurveyDelivered-959` | added | raw=nil; lookup=nil |
| `wt-framexml-getglobal-1085` | removed | raw=function; lookup=function |
| `wt-framexml-setglobal-1086` | removed | raw=function; lookup=function |
| `wt-global-api-BNGetFriendInviteInfo-563` | removed | raw=function; lookup=function |
| `wt-global-api-BNSendVerifiedBattleTagInvite-564` | removed | raw=function; lookup=function |
| `wt-global-api-CancelItemTempEnchantment-577` | removed | raw=function; lookup=function |
| `wt-global-api-GetInspectSpecialization-578` | removed | raw=function; lookup=function |
| `wt-global-api-GetWeaponEnchantInfo-580` | removed | raw=function; lookup=function |

#### member (36)
| Source ID | Direction | Observation |
|---|---|---|
| `wt-framexml-EventUtil.AreVariablesLoaded-986` | removed | raw=function; lookup=function |
| `wt-global-api-C_BattleNet.CanToggleHighResTexturesWithoutClientReload-427` | added | raw=nil; lookup=function |
| `wt-global-api-C_BattleNet.SearchFriends-432` | added | raw=nil; lookup=function |
| `wt-global-api-C_BattleNet.SendTitleFriendInviteByName-433` | added | raw=nil; lookup=function |
| `wt-global-api-C_Browser.CloseFullscreenBrowser-438` | added | parent raw-absent; raw=nil; lookup=function |
| `wt-global-api-C_Club.SendTitleFriendRequest-441` | added | raw=nil; lookup=function |
| `wt-global-api-C_DelvesUI.HasActiveLFGLair-445` | added | raw=nil; lookup=function |
| `wt-global-api-C_DelvesUI.HasActiveLair-446` | added | raw=nil; lookup=function |
| `wt-global-api-C_DelvesUI.IsInLair-447` | added | raw=nil; lookup=function |
| `wt-global-api-C_Discord.GetDiscordUserName-451` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingBlueprint.CanExportRoom-480` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingBlueprint.CanExportTypeFromCurrentLocation-481` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingBlueprint.UpdateBlueprintStringFromInput-498` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingDecor.GetAllMaxPlacementBudgets-502` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingDecor.GetAllSpentPlacementBudgets-503` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingLayout.GetHighestOccupiedFloorIndex-509` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingLayout.GetLowestOccupiedFloorIndex-510` | added | raw=nil; lookup=function |
| `wt-global-api-C_HousingLayout.GetNumFloors-568` | removed | raw=function; lookup=function |
| `wt-global-api-C_HousingLayout.RoomHasStairs-514` | added | raw=nil; lookup=function |
| `wt-global-api-C_LFGInfo.IsInMatchmadeRaidWithoutRoleRequirements-516` | added | raw=nil; lookup=function |
| `wt-global-api-C_NeighborhoodInitiative.GetInitiativeTaskRewardScaling-523` | added | raw=nil; lookup=function |
| `wt-global-api-C_PaperDollInfo.CancelTemporaryEnchantment-524` | added | raw=nil; lookup=function |
| `wt-global-api-C_PvP.JoinRandomTrainingGround-570` | removed | raw=nil; lookup=function |
| `wt-global-api-C_PvP.JoinRandomTrainingGroundArena-530` | added | raw=nil; lookup=function |
| `wt-global-api-C_PvP.JoinRandomTrainingGroundBattleground-531` | added | raw=nil; lookup=function |
| `wt-global-api-C_QuestHub.IsQuestCurrentlyRelatedToHub-602` | changed | raw=nil; lookup=function |
| `wt-global-api-C_RecentAllies.SearchRecentAllies-533` | added | raw=nil; lookup=function |
| `wt-global-api-C_Roleset.GetActiveAllowedRolesets-537` | added | raw=nil; lookup=function |
| `wt-global-api-C_Roleset.GetActiveBlockedRolesets-538` | added | raw=nil; lookup=function |
| `wt-global-api-C_Spell.GetLastCategoryCooldownSource-545` | added | raw=nil; lookup=function |
| `wt-global-api-C_Spell.GetSpellDescriptionForItemLocation-546` | added | raw=nil; lookup=function |
| `wt-global-api-C_TransmogOutfitInfo.CanPlayerTransmogSlot-548` | added | raw=nil; lookup=function |
| `wt-global-api-C_TransmogOutfitInfo.IsTransmogEnabled-549` | added | raw=nil; lookup=function |
| `wt-global-api-C_UnitAuras.AddPrivateAuraAppliedSound-573` | removed | raw=function; lookup=function |
| `wt-global-api-C_UnitAuras.CancelAuraByInstanceID-551` | added | raw=nil; lookup=function |
| `wt-global-api-C_UnitAuras.RemovePrivateAuraAppliedSound-574` | removed | raw=function; lookup=function |

#### object-method (5)
| Source ID | Direction | Observation |
|---|---|---|
| `wt-scriptobjects-SecondsFormatter:GetRounding-1103` | added | SecondsFormatter; lookup=nil |
| `wt-widgets-FontString:SetDesaturateEmbeddedTextures-1128` | added | FontString; lookup=nil |
| `wt-widgets-Frame:IsRolesetFiltered-1150` | added | Frame; lookup=nil |
| `wt-widgets-Frame:ResizeToBoundsRect-1152` | added | Frame; lookup=nil |
| `wt-widgets-FrameScriptObject:CanBeAccessedInContext-1120` | added | FrameScriptObject; lookup=nil |

#### event (4)
| Source ID | Direction | Observation |
|---|---|---|
| `wt-events-FULLSCREEN_BROWSER_SPINNER_HIDE-1262` | added | pcall=false; return=Frame:RegisterEvent(): Frame:RegisterEvent(): Attempt to register unknown event "FULLSCREEN_BROWSER_SPINNER_HIDE"; registered=false |
| `wt-events-FULLSCREEN_BROWSER_SPINNER_SHOW-1263` | added | pcall=false; return=Frame:RegisterEvent(): Frame:RegisterEvent(): Attempt to register unknown event "FULLSCREEN_BROWSER_SPINNER_SHOW"; registered=false |
| `wt-events-HOUSING_LAYOUT_NUM_FLOORS_CHANGED-1297` | removed | pcall=true; return=true; registered=true |
| `wt-events-HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED-1283` | added | pcall=false; return=Frame:RegisterEvent(): Frame:RegisterEvent(): Attempt to register unknown event "HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED"; registered=false |

#### cvar (6)
| Source ID | Direction | Observation |
|---|---|---|
| `wt-cvars-auctionDisplayOnCharacter-1338` | removed | value/default queried; current='0'; default='0' |
| `wt-cvars-auctionSortByBuyoutPrice-1339` | removed | value/default queried; current='0'; default='0' |
| `wt-cvars-auctionSortByUnitPrice-1340` | removed | value/default queried; current='0'; default='0' |
| `wt-cvars-tooltipShowAuraSpellIDs-1332` | added | value/default queried; current=None; default=None |
| `wt-cvars-worldMapShowCursorCoords-1334` | added | value/default queried; current=None; default=None |
| `wt-cvars-worldMapShowPlayerCoords-1335` | added | value/default queried; current=None; default=None |

### Probe kinds that could not be checked
| Source ID | Reason |
|---|---|
| `wt-events-CHAT_MSG_*-1300` | wildcard occurrence is not a concrete event |
| `wt-widgets-RadialProgress:GetFromPercent-1142` | factory: (string):68: RadialProgress factory returned Animation |
| `wt-widgets-RadialProgress:GetToPercent-1143` | factory: (string):68: RadialProgress factory returned Animation |
| `wt-widgets-RadialProgress:SetFromPercent-1144` | factory: (string):68: RadialProgress factory returned Animation |
| `wt-widgets-RadialProgress:SetToPercent-1145` | factory: (string):68: RadialProgress factory returned Animation |

The four RadialProgress rows are an unavailable simulator factory, not a wrong lookup; the wildcard CHAT_MSG_* row is an inventory limitation, not a proven simulator defect. No per-row probe-error, unknown owner, or malformed-symbol result remained. No per-symbol exceptions were added.

### Job cap
`build-host.py --help` has no jobs option. Shared `native_build_hosts.py::native_build` inserts `-j8` and does not read a jobs override. A later passthrough `-j4` fails because Cargo forbids duplicate jobs arguments (first.log). Invocation-local `p1210-r2-capped-build.py` loads the unmodified helper and replaces only its generated Cargo `-j8` argv with `-j4` in memory; shared files unchanged. Logs print actual capped argv. Every build also used `CARGO_BUILD_JOBS=4`; every build/check used exclusive target `/home/osso-test/.cache/wow-ui-sim-target-b100`. No concurrent builds, no undefined-symbol failure, no cargo clean needed. Workaround retires when the shared helper exposes a jobs override.

### Proof ledger
| Scope / revision | Command / log | Result | Validity |
|---|---|---|
| Base + original staged test | capped helper, filter patch_12_1_0_publication_sweep; p1210-r2-compile.log | RED; 778 rows; 147 non-OK; compile 3m47s | Global-lookup probe superseded |
| 4cf15e68f | capped helper, same filter; p1210-r2-probe-fix.log | RED; 631 OK / 147 non-OK | Probe code unchanged since |
| 4cf15e68f with full scratch register | capped helper, same filter; p1210-r2-negative.log | RED; 150 non-OK, exactly three deliberately wrong rows | Negative controls valid; scratch not committed |
| bcdaf87d2 | capped helper, same filter; p1210-r2-green.log | GREEN; 1 passed, 10469 filtered out; compile 1m00s, run 8.51s | Final commit changes spec only; test/fixture proof retained |
| c72b7e8b332d1b9eb1ade4e0a9b50f4bd5266c4d | cargo fmt; cargo fmt --check | Both exit 0 | Current |
| c72b7e8b332d1b9eb1ade4e0a9b50f4bd5266c4d | cargo check --locked -j4 --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100; p1210-r2-check.log | Exit 0; 1m14s; no warnings | Current |

All commands ran with explicit worktree cwd. Test command: `python3 A/p1210-r2-capped-build.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration patch_12_1_0_publication_sweep -- --test-threads=1`, with `BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts`, `CARGO_BUILD_JOBS=4`, and `P1210_SWEEP_OUT` set to the result artifact. Only negative controls set `P1210_SWEEP_REGISTER`.

### Output ledger and provenance
Status counts: {'partial-development-green': 504, 'bounded-coverage': 127, 'audit-pending': 147}. Ledger is recommendations only; project coverage JSON untouched. Removed Lua globals/members were absent under both raw and ordinary lookup; two removed CVars were absent under both current/default API queries; one removed event was rejected as unknown. Event/CVar absence uses its own public API, not meaningless global-table lookup. No source CVar page defaults exist, so zero page-default comparisons/mismatches were available.
Cached Retail provenance (read-only): profile=retail, product=wow, version=12.1.0.69933, build_key=dcfc90fffd79ba00406ae46f5f657592, manifest_sha256=aa7dfec3fb3bc9440a8c737c2274363502cb3d22c7939b8a570a58e717202edd, source=casc-local-or-cdn, fallback=none. Metadata is not native-client authentication.

Artifacts: `A/p1210-sweep-result.json` (final complete per-ID output), `A/p1210-sweep-ledger.json` (778 recommendations), `A/p1210-r2-final-result.json` (identical final snapshot); negative-control register/results and captured logs remain under A.

### Merge risk
Low production-runtime risk: only test, fixture and spec change. Test adds one full cached Game-startup load; result depends on cached Retail UI provenance and current default startup surface. Exact gap-set comparison deliberately fails for both new regressions and newly resolved known gaps, requiring review. Baseline GREEN is publication breadth, not signature, behavior, security, event-payload or native-client parity. 147 rows remain audit-pending. No external/native validation or optional-panel coverage claimed. Minor readability debt: classify_entry has 31 body lines (one above the helper guideline), mainly result serialization; read_register has 26. No compiler warnings or suppressions.

Step 15: Artifact validation passed: 778 ledger IDs match all result IDs; every non-ok is audit-pending; final snapshot byte-identical. Readability body-line measurement corrects earlier estimate: {'classify_entry': 31, 'read_register': 26}. No code or fixture changes after GREEN/check proof.
