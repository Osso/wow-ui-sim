# User-approved simulator defaults

These values are simulator assumptions, not evidence of native behavior. Supplied state takes precedence where available.

| Item | Default | Profiles | Original consumer | Approved |
|---|---|---|---|---|
| `C_GameRules.IsAccountKeyBindingsEnabled` | `false` | PTR | QuickKeybindFrameMixin.OnLoad | 2026-10-28 |
| `C_AccountServices.GetPreferredLocales` | Two empty tables | PTR | CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeLocaleDropdown | 2026-10-28 |
| `C_VoiceChat.GetActiveVoiceProviderID` | `Enum.VoiceProviderID.Legacy` | PTR | VoiceTranscriptionFrame_UpdateVisibility | 2026-10-28 |
| `C_Bank.ShouldUsePlayerBagsInBank` | `false` | PTR | BankPanelMixin.OnEvent | 2026-10-28 |
| `C_ExternalEventURL` | `HasURL`, `IsNew`: false; `LaunchURL`: no-op; `CanShow`: false (PTR only) | Retail, PTR | GameMenuFrameMixin.ExternalEventClickCallback; PTR minimap | 2026-10-28 |
| `C_QuestLog.GetZoneStoryInfo` | No values for unknown map | Retail, PTR | QuestLogQuests_Update | 2026-10-28 |
| `C_StableInfo.GetAvailablePetSpecInfos` | Empty table when no catalog is supplied | Retail, PTR | StableUI specialization Refresh (Retail), OnLoad and Refresh (PTR) | 2026-10-28 |

Declaration references are relative to each profile's pinned `Blizzard_APIDocumentationGenerated`:

- PTR `GameRulesDocumentation.lua:213–219`, `BankDocumentation.lua:322–328`, `VoiceChatDocumentation.lua:116–122` and `:1476–1484`.
- `ExternalEventURLDocumentation.lua` beginning at line 11 in Retail and PTR; `StableInfoDocumentation.lua:24–30` in Retail and `:28–34` in PTR.
- `QuestLogDocumentation.lua:675–689` in Retail and `:693–707` in PTR. No declaration was found for preferred locales; PTR `Blizzard_Communities/GuildPreferredPlaySettings.lua:130` consumes two return values.
