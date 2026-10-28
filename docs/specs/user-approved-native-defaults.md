# User-approved simulator defaults

These values are simulator assumptions, not evidence of native behavior. Supplied state takes precedence where available.

| Item | Default | Profiles | Original consumer | Approved |
|---|---|---|---|---|
| `C_GameRules.IsAccountKeyBindingsEnabled` | `false` | PTR | QuickKeybindFrameMixin.OnLoad | 2026-10-28 |
| `C_AccountServices.GetPreferredLocales` | Two empty tables | PTR | CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeLocaleDropdown | 2026-10-28 |
| `C_VoiceChat.GetActiveVoiceProviderID` | `Enum.VoiceProviderID.Legacy` | PTR | VoiceTranscriptionFrame_UpdateVisibility | 2026-10-28 |
| `C_Bank.ShouldUsePlayerBagsInBank` | `false` | PTR | BankPanelMixin.OnEvent | 2026-10-28 |
| `C_ExternalEventURL` | `HasURL`, `IsNew`: false; `LaunchURL`: no-op; `CanShow`: false (PTR only) | Retail, PTR | GameMenuFrameMixin.ExternalEventClickCallback; PTR minimap | 2026-10-28 |
| `C_StableInfo.GetAvailablePetSpecInfos` | Empty table when no catalog is supplied | Retail, PTR | StableUI specialization Refresh (Retail), OnLoad and Refresh (PTR) | 2026-10-28 |
