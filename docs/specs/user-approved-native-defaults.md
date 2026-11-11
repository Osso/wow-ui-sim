# User-approved native-unknown defaults

These are simulator scenario defaults, approved by the user on 2026-10-28. They are not evidence of native behavior. Supplied state takes precedence where available. Replace them when the named backing systems are modeled. R27 defines `C_CharacterCreation.SetInCharacterCreate` in [character-create-mode](character-create-mode.md). R41 replaces the former altered-form default with the modeled API in [character-create-altered-form](character-create-altered-form.md); PTR registration remains UNKNOWN. R51 replaces the Retail `SetBlurEnabled` no-op with modeled Boolean state and adds its Forever Glue contract; PTR keeps its prior fallback because registration is UNKNOWN. R57 replaces the Retail preview stubs and registers evidence-backed camera/preview APIs on Retail and Forever Glue; PTR retains its prior preview fallbacks because registration is UNKNOWN. See [character-create blur](character-create-blur.md) and [character-create camera and preview state](character-create-visual-state.md). R30 replaced the `HasQueuedUpgrade` and `GetDefaultCharacterCreateFacing` defaults with host-backed Glue APIs; see [queued-character-upgrade](queued-character-upgrade.md) and [character-create-facing](character-create-facing.md). R34 replaces the Retail and Forever stored-number current-facing defaults with the evidenced APIs; preserve the existing PTR fallback because `SetCharacterCreateFacing` registration is UNKNOWN. The `C_QuestLog.IsAccountQuest` result contract is in [quest-account-flag](quest-account-flag.md); Retail/PTR parser parity is UNKNOWN and their existing simulator parser remains unchanged.

## Glue screens

| Item | Value | Profiles | Consumer | Missing system |
|---|---|---|---|---|
| C_PlayerInfo.GetContentDifficultyQuestForPlayer | WM89 fixed Retail/PTR bands use emulated stored quest and player levels | Retail, PTR | Blizzard_UIPanels_Game/Mainline/QuestMapFrame.lua:1751 (Retail), :1835 (PTR) | ContentTuning and effective-level inputs |
| C_AccountServices.GetPreferredDatacenterLocalities | Two empty tables; user-approved, not native evidence | PTR | Blizzard_Communities/GuildPreferredPlaySettings.lua:154; Blizzard_Settings_PreferredPlaySettings.lua:75 | Account datacenter preferences |
| FORCE_DECLINE_CHARACTER | Register; do not emit | PTR | Blizzard_DeclensionFrame/Mainline/DeclensionFrameGlue.lua:12 | Declension flow |
| C_Login.GetState | Preserve modeled connection fields; waitingForRealmList, superDistrictChoicePending, keepingCharacterListOnRealmJoin, waitingForRuleSets false; hasSelectedExperiencePreset true | PTR | Blizzard_GlueParent/Mainline/GlueParent.lua:222–254 | Login policy |
| IsNameReservationOnly | false | PTR | Blizzard_GlueXML/Mainline/CharacterSelect.lua:1019 | Name-reservation realm policy |
| C_CharacterCreation.SaveSeenChoices | Record offered choice IDs as seen; user-approved, not native evidence | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:744 (Retail), Mainline/Blizzard_CharacterCreate.lua:787 (PTR) | Character customization seen-choice persistence |
| C_CharacterCreation.ClearPreviewChoices | Accept clearSavedChoices; no effect or returns | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:727 (Retail), Mainline/Blizzard_CharacterCreate.lua:770 (PTR) | Character customization preview and saved choices |
| C_CharacterServices.IsTrialBoostEnabled | false | Retail, PTR | Blizzard_GlueXML/Mainline/CharacterServices.lua:247 (Retail, PTR); CharacterSelect.xml:429 (Retail), :440 (PTR) | Class-trial boost availability |
| C_CharacterCreation.GetTrialBoostStartingLevel | 0 | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:1454 (Retail), Mainline/Blizzard_CharacterCreate.lua:1503 (PTR) | Class-trial checkbox tooltip level |
| C_CharacterCreation.SetModelDressState, SetSelectedPreviewGearType, SetCameraZoomLevel | Existing simulator stubs retained; PTR registration UNKNOWN | PTR | Mainline/Blizzard_CharacterCreate.lua:546-561, 791, 795 | Character-create 3D presentation; see [character-create camera and preview state](character-create-visual-state.md) |
| C_CharacterCreation.SetBlurEnabled | No effect; existing simulator shim retained while PTR registration is UNKNOWN | PTR | Mainline/Blizzard_CharacterCreate.lua:546-561, 791, 795 | Permanent 3D visual gap; see [character-create blur](character-create-blur.md) |
| C_CharacterCreation.AreRegionalUniqueNamesEnabled | false | PTR | Blizzard_CharacterCreate/Mainline/Blizzard_CharacterCreate.lua:2079 | Regional naming policy |
| C_CharacterServices.GetPaidCharacterTransferSuperDistrictDestinations | {} (no destinations) | PTR | Blizzard_GlueXML/Mainline/CharacterServicesSuperDistrictSelector.lua:27 | Paid character transfer super-district dropdown |

## In-game

| Item | Default | Profiles | Original consumer | Approved |
|---|---|---|---|---|
| `ModelSceneActorBase:SetModelByUnitCreatureDisplayID` | `false`; permanent 3D no-op | PTR | PaperDollFrame.lua:2995 (pet model) | 2026-10-28 |
| `C_GameRules.IsAccountKeyBindingsEnabled` | `false` | PTR | QuickKeybindFrameMixin.OnLoad | 2026-10-28 |
| `C_AccountServices.GetPreferredLocales` | Two empty tables | PTR | CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeLocaleDropdown | 2026-10-28 |
| `C_VoiceChat.GetActiveVoiceProviderID` | `Enum.VoiceProviderID.Legacy` | PTR | VoiceTranscriptionFrame_UpdateVisibility | 2026-10-28 |
| `C_Bank.ShouldUsePlayerBagsInBank` | `false` | PTR | BankPanelMixin.OnEvent | 2026-10-28 |
| `C_ExternalEventURL` | No supplied promotion: `HasURL`, `IsNew` false; `LaunchURL` records nothing; `CanShow` false (PTR only). Now modeled host state; see [external event URL](external-event-url.md) | Retail, PTR, Mists, Era, Anniversary | GameMenuFrameMixin.ExternalEventClickCallback; PTR minimap | 2026-10-28 |
| `C_QuestLog.GetZoneStoryInfo` | No values for unknown map | Retail, PTR | QuestLogQuests_Update | 2026-10-28 |
| Ordinary quest watch seed | Every seeded non-world quest is Manual; user-approved for Retail/PTR, simulator policy, not native evidence | Retail, PTR | QuestLogQuests_AddQuestButton | 2026-10-28 |
| `C_StableInfo.GetAvailablePetSpecInfos` | Empty table when no catalog is supplied | Retail, PTR | StableUI specialization Refresh (Retail), OnLoad and Refresh (PTR) | 2026-10-28 |

Declaration references are relative to each profile's pinned `Blizzard_APIDocumentationGenerated`:

- PTR `GameRulesDocumentation.lua:213–219`, `BankDocumentation.lua:322–328`, `VoiceChatDocumentation.lua:116–122` and `:1476–1484`.
- `ExternalEventURLDocumentation.lua:5` in Retail, PTR, Mists, Era and Anniversary; PTR declares `CanShow` at `:11`; `StableInfoDocumentation.lua:24–30` in Retail and `:28–34` in PTR.
- `QuestLogDocumentation.lua:701` in Retail and `:719` in PTR declare `IsAccountQuest`; watch methods are declared in both profiles' `QuestLogDocumentation.lua`.
- `QuestLogDocumentation.lua:675–689` in Retail and `:693–707` in PTR. No declaration was found for preferred locales; PTR `Blizzard_Communities/GuildPreferredPlaySettings.lua:130` consumes two return values.
