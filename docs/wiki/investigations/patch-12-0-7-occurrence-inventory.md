# Patch 12.0.7 Occurrence Inventory
Occurrence-level register for the committed 12.0.7 API-change source. Every named occurrence has an evidence-backed classification; crawler-omitted unnamed claims remain source metadata and are not invented.
## Content
- **Source:** `data/patch-api/sources/12.0.7-register.json`
- **Source SHA-256:** `389e3b19174bf77c3646028f764cf186ccfe1b7dddaca2a3b3fcba75e3bdec60`
- **Target:** retail build `12.0.7`
- **Rows:** 131 total — 29 implemented, 101 best-effort, 1 repository-scope-authorized impossible exception-requested, 0 untriaged
- **Directions:** 79 added, 29 changed, 23 removed

| Symbol | Status | Category | Direction | Note |
|---|---|---|---|---|
| `C_BattleNet.InviteFriend` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_Container.CalculateTotalNumberOfFreeBagSlots` | best-effort | global-api | added | Returns the simulator bag-model free-slot total; exact Blizzard container-family and hidden-bag semantics remain best-effort. |
| `C_DelvesUI.GetDelveEntranceTitleString` | best-effort | global-api | added | Returns deterministic seeded Delves UI text; exact content-data localization and area selection remain best-effort. |
| `C_DelvesUI.GetWorldTierDifficultyForActivePlayer` | best-effort | global-api | added | Returns the simulator Delves tier value; exact active-player world-tier service state remains best-effort. |
| `C_DurationUtil.CreateDurationTextBinding` | best-effort | global-api | added | Compatibility binding state and methods are directly tested; native lifetime, identity, formatter, and secret semantics remain best-effort. |
| `C_DurationUtil.CreateManualClock` | implemented | global-api | added | Manual clock construction and all documented clock mutations are directly state-tested. |
| `C_EncounterTimeline.GetEventColor` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_HousingCustomizeMode.RoomConnectionSupportsDoorType` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_HousingLayout.CanSetViewedFloor` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_MerchantFrame.GetMerchantCurrencies` | best-effort | global-api | added | Returns a deterministic empty currency list until merchant-currency state is modeled; API shape is directly tested. |
| `C_PartyInfo.ConfirmReadyCheck` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.DemoteAssistant` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.DoReadyCheck` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.IsGUIDInGroup` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.PromoteToAssistant` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.PromoteToLeader` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.SetEveryoneIsAssistant` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PartyInfo.UninviteUnit` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PingSecure.ClearPendingPingOffScreenCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_PingSecure.SetPendingPingOffScreenCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_QuestHub.GetDragonridingRacesForAreaPOI` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_UIFileAsset.GetFileID` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_UIFileAsset.IsKnownFile` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `C_UIFileAsset.IsLooseFile` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetEventCPUUsage` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetFunctionCPUUsage` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetScriptCPUUsage` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetSecurePendingButtonCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetSecurePendingPingOffScreenCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GetSecurePendingToggleRunCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `GameTooltip_AddMoneyLine` | best-effort | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `SetSecurePendingButtonCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `SetSecurePendingPingOffScreenCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `SetSecurePendingToggleRunCallback` | implemented | global-api | added | Behavior is covered by the profile-gated safe bridge test; exact Blizzard service/payload fidelity is best-effort where noted. |
| `DurationClock.GetTime` | implemented | script-object-method | added | Manual clock construction and this documented mutation/query are directly state-tested. |
| `DurationManualClock.AdvanceTime` | implemented | script-object-method | added | Manual clock construction and this documented mutation/query are directly state-tested. |
| `DurationManualClock.ResetTime` | implemented | script-object-method | added | Manual clock construction and this documented mutation/query are directly state-tested. |
| `DurationManualClock.RewindTime` | implemented | script-object-method | added | Manual clock construction and this documented mutation/query are directly state-tested. |
| `DurationManualClock.SetTime` | implemented | script-object-method | added | Manual clock construction and this documented mutation/query are directly state-tested. |
| `DurationObject.GetClock` | best-effort | script-object-method | added | Duration-object clock storage is directly tested; lifecycle queries retain explicit compatibility defaults. |
| `DurationObject.HasExpired` | best-effort | script-object-method | added | Duration-object clock storage is directly tested; lifecycle queries retain explicit compatibility defaults. |
| `DurationObject.HasStarted` | best-effort | script-object-method | added | Duration-object clock storage is directly tested; lifecycle queries retain explicit compatibility defaults. |
| `DurationObject.IsActive` | best-effort | script-object-method | added | Duration-object clock storage is directly tested; lifecycle queries retain explicit compatibility defaults. |
| `DurationObject.SetClock` | best-effort | script-object-method | added | Duration-object clock storage is directly tested; lifecycle queries retain explicit compatibility defaults. |
| `DurationTextBinding.CanFormatText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.CanUpdateFontString` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.Disable` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.Enable` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetDuration` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetExpiredText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetFontString` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetFormattedText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetTimeModifier` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetUpdateInterval` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.GetZeroDurationText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.IsEnabled` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetDuration` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetExpiredText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetFontString` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetTimeModifier` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetUpdateInterval` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextBinding.SetZeroDurationText` | best-effort | script-object-method | added | The compatibility binding method is directly exercised; native identity, formatter, interpolation, and secret semantics remain best-effort. |
| `DurationTextFormattingOptions.GetAddRemainingText` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextFormattingOptions.GetDurationType` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextFormattingOptions.SetAddRemainingText` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextFormattingOptions.SetDurationType` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextRawValue.GetMilliseconds` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextRawValue.GetSeconds` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextRawValue.SetMilliseconds` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `DurationTextRawValue.SetSeconds` | best-effort | script-object-method | added | The extracted object family has no constructor: the focused test verifies its factory remains nil, so these methods are unreachable stale-source entries. |
| `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED` | best-effort | event | added | Registration is tested; live payload/firing semantics remain best-effort. |
| `URL_TEXTURE_REQUEST_RESULT` | best-effort | event | added | Registration is tested; live payload/firing semantics remain best-effort. |
| `assistedCombatReduceHighlights` | implemented | cvar | added | Implemented profile-gated default 1; crawler-omitted CVar claims remain outside occurrence rows. |
| `developerLogFilterDebug` | implemented | cvar | added | Implemented profile-gated default 0; crawler-omitted CVar claims remain outside occurrence rows. |
| `developerLogFilterError` | implemented | cvar | added | Implemented profile-gated default 1; crawler-omitted CVar claims remain outside occurrence rows. |
| `developerLogFilterFatal` | implemented | cvar | added | Implemented profile-gated default 1; crawler-omitted CVar claims remain outside occurrence rows. |
| `developerLogFilterNormal` | implemented | cvar | added | Implemented profile-gated default 1; crawler-omitted CVar claims remain outside occurrence rows. |
| `developerLogFilterSpam` | implemented | cvar | added | Implemented profile-gated default 0; crawler-omitted CVar claims remain outside occurrence rows. |
| `Button.GetButtonState` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `Button.IsEnabled` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `Button.SetButtonState` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `Button.SetEnabled` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `EditBox.SetFont` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `Font.SetFont` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `FontString.SetFont` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `MessageFrame.SetFont` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `ModelSceneActorBase.GetModelUnitGUID` | exception-requested | widget-method | changed | Repository scope exception: `AGENTS.md#intentional-gaps` permanently excludes 3D model behavior, so this model-unit GUID method remains absent rather than guessed. |
| `ScrollFrame.GetHorizontalScroll` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `ScrollFrame.GetVerticalScroll` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `ScrollFrame.SetHorizontalScroll` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `ScrollFrame.SetVerticalScroll` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `SimpleHTML.SetFont` | best-effort | widget-method | changed | Compatibility presence is tested; exact changed secret/aspect semantics remain best-effort. |
| `CHAT_MSG_COMBAT_FACTION_CHANGE` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_COMBAT_HONOR_GAIN` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_COMBAT_MISC_INFO` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_COMBAT_XP_GAIN` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_CURRENCY` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_FILTERED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_LOOT` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_MONEY` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CHAT_MSG_RESTRICTED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CLUB_MEMBER_ADDED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CLUB_MEMBER_PRESENCE_UPDATED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CLUB_MEMBER_REMOVED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CLUB_MEMBER_ROLE_UPDATED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `CLUB_MEMBER_UPDATED` | best-effort | event | changed | Registration is tested; changed payload and secret-lockdown semantics remain best-effort. |
| `ENCOUNTER_END` | best-effort | event | changed | Explicit A_Admin.SimulateBossKill input produces the sixth ordered status-list payload, with copied records, empty omitted/nil lists, and atomic simulator validation. First five fields and subsequent BOSS_KILL stay unchanged;12.0.5 retains five event arguments. Native encounter tracking, timing, validation, security and Blizzard consumption of the new field are unproven. |
| `BNInviteFriend` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `C_ClickBindings.GetStringFromModifiers` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `C_ClickBindings.MakeModifiers` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `C_Spell.GetMawPowerBorderAtlasBySpellID` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `ConfirmReadyCheck` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `DemoteAssistant` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `DoReadyCheck` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `GetMerchantCurrencies` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `IsGUIDInGroup` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `PromoteToAssistant` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `PromoteToLeader` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `SetEveryoneIsAssistant` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `UninviteUnit` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `GetAutoCompletePresenceID` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `GetAutoCompleteResults` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `GetAutoCompleteRealms` | best-effort | global-api | removed | Patch list marks removal, but simulator retains a compatibility function to support loaded UI. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `IsRecognizedName` | best-effort | global-api | removed | Startup removal behavior is tested; exact Blizzard load-order timing remains best-effort. Runtime availability/removal is captured by the startup compatibility test; post-reset timing is not claimed. |
| `Minimap.SetBlipTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
| `Minimap.SetCorpsePOIArrowTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
| `Minimap.SetIconTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
| `Minimap.SetPOIArrowTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
| `Minimap.SetPlayerTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
| `Minimap.SetStaticPOIArrowTexture` | best-effort | widget-method | removed | Patch list marks removal, but method remains as compatibility surface; exact vendor lifecycle semantics are best-effort. |
