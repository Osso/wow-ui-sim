# User-approved native-unknown defaults

These are simulator scenario defaults, approved by the user on 2026-10-28. They are not evidence of native behavior. Supplied state takes precedence where available. Replace them when the named backing systems are modeled.

## Glue screens

| Item | Value | Profiles | Consumer | Missing system |
|---|---|---|---|---|
| C_PlayerInfo.GetContentDifficultyQuestForPlayer | Quest level vs player level; user-approved, not native evidence. Green range: user-approved for Retail/PTR; values are Wrath 12340 native data, not 12.x evidence | Retail, PTR | Blizzard_UIPanels_Game/Mainline/QuestMapFrame.lua:1751 (Retail), :1835 (PTR); Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua:33–45 | Native quest trivial-level range |
| C_AccountServices.GetPreferredDatacenterLocalities | Two empty tables; user-approved, not native evidence | PTR | Blizzard_Communities/GuildPreferredPlaySettings.lua:154; Blizzard_Settings_PreferredPlaySettings.lua:75 | Account datacenter preferences |
| FORCE_DECLINE_CHARACTER | Register; do not emit | PTR | Blizzard_DeclensionFrame/Mainline/DeclensionFrameGlue.lua:12 | Declension flow |
| C_Login.GetState | Preserve modeled connection fields; waitingForRealmList, superDistrictChoicePending, keepingCharacterListOnRealmJoin, waitingForRuleSets false; hasSelectedExperiencePreset true | PTR | Blizzard_GlueParent/Mainline/GlueParent.lua:222–254 | Login policy |
| IsNameReservationOnly | false | PTR | Blizzard_GlueXML/Mainline/CharacterSelect.lua:1019 | Name-reservation realm policy |
| C_CharacterServices.HasQueuedUpgrade | false | Retail, PTR | Blizzard_GlueXML/Mainline/CharacterSelect.lua:337 (Retail), :342 (PTR) | Queued character upgrades |
| C_CharacterCreation.SetInCharacterCreate | Record supplied boolean; user-approved, not native evidence | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:194 (Retail), Mainline/Blizzard_CharacterCreate.lua:200 (PTR) | Character-create screen state consumers |
| C_CharacterCreation.SaveSeenChoices | Record offered choice IDs as seen; user-approved, not native evidence | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:744 (Retail), Mainline/Blizzard_CharacterCreate.lua:787 (PTR) | Character customization seen-choice persistence |
| C_CharacterCreation.ClearPreviewChoices | Accept clearSavedChoices; no effect or returns | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:727 (Retail), Mainline/Blizzard_CharacterCreate.lua:770 (PTR) | Character customization preview and saved choices |
| C_CharacterCreation.GetDefaultCharacterCreateFacing | 0 | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:761 (Retail), Mainline/Blizzard_CharacterCreate.lua:804 (PTR) | Default subject rotation; feeds only 3D model rotation (non-goal) |
| C_CharacterCreation.Get/SetCharacterCreateFacing | Stored number, initially the default facing 0; round-trip derived from Blizzard rotate code | Retail, PTR | Blizzard_CharacterCreate/Blizzard_CharacterCreate.lua:773-807 (Retail), Mainline/Blizzard_CharacterCreate.lua:816-850 (PTR) | 3D subject rotation (non-goal) |
| C_CharacterCreation.AreRegionalUniqueNamesEnabled | false | PTR | Blizzard_CharacterCreate/Mainline/Blizzard_CharacterCreate.lua:2079 | Regional naming policy |

## In-game

| Item | Default | Profiles | Original consumer | Approved |
|---|---|---|---|---|
| `C_GameRules.IsAccountKeyBindingsEnabled` | `false` | PTR | QuickKeybindFrameMixin.OnLoad | 2026-10-28 |
| `C_AccountServices.GetPreferredLocales` | Two empty tables | PTR | CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeLocaleDropdown | 2026-10-28 |
| `C_VoiceChat.GetActiveVoiceProviderID` | `Enum.VoiceProviderID.Legacy` | PTR | VoiceTranscriptionFrame_UpdateVisibility | 2026-10-28 |
| `C_Bank.ShouldUsePlayerBagsInBank` | `false` | PTR | BankPanelMixin.OnEvent | 2026-10-28 |
| `C_ExternalEventURL` | `HasURL`, `IsNew`: false; `LaunchURL`: no-op; `CanShow`: false (PTR only) | Retail, PTR | GameMenuFrameMixin.ExternalEventClickCallback; PTR minimap | 2026-10-28 |
| `C_QuestLog.GetZoneStoryInfo` | No values for unknown map | Retail, PTR | QuestLogQuests_Update | 2026-10-28 |
| Ordinary quest watch seed | Every seeded non-world quest is Manual; user-approved for Retail/PTR, simulator policy, not native evidence | Retail, PTR | QuestLogQuests_AddQuestButton | 2026-10-28 |
| `C_QuestLog.IsAccountQuest` | Supplied quest flag, otherwise false; user-approved, not native evidence | Retail, PTR | QuestLogQuests_AddQuestButton | 2026-10-28 |
| `C_StableInfo.GetAvailablePetSpecInfos` | Empty table when no catalog is supplied | Retail, PTR | StableUI specialization Refresh (Retail), OnLoad and Refresh (PTR) | 2026-10-28 |

Declaration references are relative to each profile's pinned `Blizzard_APIDocumentationGenerated`:

- PTR `GameRulesDocumentation.lua:213–219`, `BankDocumentation.lua:322–328`, `VoiceChatDocumentation.lua:116–122` and `:1476–1484`.
- `ExternalEventURLDocumentation.lua` beginning at line 11 in Retail and PTR; `StableInfoDocumentation.lua:24–30` in Retail and `:28–34` in PTR.
- `QuestLogDocumentation.lua:701` in Retail and `:719` in PTR declare `IsAccountQuest`; watch methods are declared in both profiles' `QuestLogDocumentation.lua`.
- `QuestLogDocumentation.lua:675–689` in Retail and `:693–707` in PTR. No declaration was found for preferred locales; PTR `Blizzard_Communities/GuildPreferredPlaySettings.lua:130` consumes two return values.
