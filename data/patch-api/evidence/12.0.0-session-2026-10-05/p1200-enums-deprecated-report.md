# B01/B07/B03 runtime proof — 2026-10-05

Base `fbf0d11c535ab6571c348d9b34191f8f7d400f6c`; runtime/test revision `9749c87accd27ceeb29cf8a6ed1e1e85ed12b617`. Coverage ledger unchanged.

## Outcomes

| source_id | outcome |
|---|---|
| `enumerations-Enum-CraftingOrderResult-036` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-CraftingOrderResult-037` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-CraftingOrderResult-038` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeAccountSetting-039` | superseded(12.0.5 and 12.1.0); current projection passes |
| `enumerations-Enum-EditModeAccountSetting-040` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeAccountSetting-041` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeAccountSetting-042` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeAccountSetting-043` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeSystem-044` | superseded(12.0.5 and 12.1.0); current projection passes |
| `enumerations-Enum-EditModeSystem-045` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeSystem-046` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-EditModeSystem-047` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-PlayerInteractionType-048` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-PlayerInteractionType-049` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-QuestTagType-050` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-QuestTagType-051` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-SendAddonMessageResult-052` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-SendAddonMessageResult-053` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-SendAddonMessageResult-054` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-SurveyDeliveryMoment-055` | superseded(12.0.5); current projection passes |
| `enumerations-Enum-SurveyDeliveryMoment-056` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-TooltipDataLineType-057` | superseded(12.1.0 cached surface (earlier renumbering patch not established)); current projection passes |
| `enumerations-Enum-TooltipDataLineType-058` | superseded(12.1.0 cached surface (earlier renumbering patch not established)); current projection passes |
| `enumerations-Enum-TooltipDataLineType-059` | superseded(12.1.0 cached surface (earlier renumbering patch not established)); current projection passes |
| `enumerations-Enum-TooltipDataType-060` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-TooltipDataType-061` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-TradeskillRecipeType-062` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-TradeskillRecipeType-063` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-UICursorType-064` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-UICursorType-065` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-UIWidgetVisualizationType-066` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `enumerations-Enum-UIWidgetVisualizationType-067` | proven-by-test(patch_12_0_0_enums::patch_12_0_0_enum_publication) |
| `deprecated api-GetSpellInfo-130` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetNumSpellTabs-131` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetSpellTabInfo-132` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetSpellCooldown-133` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetSpellBookItemName-134` | still-pending(C_SpellBook.GetSpellBookItemName returns one name, missing documented subName result. SpellBook producer excluded from this workstream.); retirement passes |
| `deprecated api-GetSpellTexture-135` | still-pending(C_Spell.GetSpellTexture returns path string as first result instead of documented fileID. Spell producer excluded from this workstream.); retirement passes |
| `deprecated api-GetSpellCharges-136` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetSpellDescription-137` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetSpellCount-138` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-IsUsableSpell-139` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-C_TaskQuest-GetQuestsForPlayerByMapID-142` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-GetMerchantItemInfo-143` | still-pending(C_MerchantFrame.GetItemInfo autostub returns nil for seeded merchant item. Merchant producer excluded from this workstream.); retirement passes |
| `deprecated api-C_ChallengeMode-GetCompletionInfo-144` | still-pending(C_ChallengeMode.GetChallengeCompletionInfo autostub returns nil, not non-nil ChallengeCompletionInfo. Successor producer excluded from this workstream.); retirement passes |
| `deprecated api-C_MythicPlus-IsWeeklyRewardAvailable-145` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-IsActiveQuestLegendary-148` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-C_QuestLog-IsLegendaryQuest-149` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-C_QuestLog-IsQuestRepeatableType-150` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-ConsolePrint-153` | still-pending(C_Log.LogMessage autostub returns one nil, not documented zero results. Logging producer excluded from this workstream.); retirement passes |
| `deprecated api-message-154` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-IsSpellOverlayed-157` | still-pending(C_SpellActivationOverlay.IsSpellOverlayed autostub returns nil, not bool. Overlay producer excluded from this workstream.); retirement passes |
| `deprecated api-IsArtifactRelicItem-160` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |
| `deprecated api-removal-summary-128` | proven-by-test(patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors) |

## Evidence boundary

32 current enum rows pass, including entire tables, numeric types, Meta, raw/ordinary lookup, removals and renamed-key absence. Six historical row contracts are superseded. The B07 scout conflict is falsified: `PLAYER_INTERACTION_TYPE` and `UI_WIDGET_VISUALIZATION_TYPE` constants are not in `SEQUENTIAL_ENUMS`; guarded cache-derived declarations actually publish those tables. No enum producer changes needed.

13 legacy publications initially violated retirement. Eight globals now gate registrations/bootstrap wrappers before publication; five namespaced keys are marked removed to disable autostub fabrication. Two late namespace registrations also require epoch gates because they execute after retirement marking. Earlier/non-retail registration paths remain in the source gates; Classic runtime verification was not run. Successor producers were not modified.

All 21 native and cached retirements pass; cached deprecation Lua republishes none of these old names. Exact alias identity checking remains conditional on a frozen cached assignment using the established loaded-file scanner. Fourteen listed successor fixtures pass; six remain pending in the frozen deprecated fixture. The overall deprecated test checks the exact six-gap set, not full successor completion.

## Tests and master comparison

- New enum integration test: PASS alone, 32 row projections.
- New deprecated prefork test: PASS, 21 retirements and exact six successor gaps.
- 12.1.0 enum integration: PASS alone before and after.
- 12.1.0 deprecated prefork wrappers: PASS on branch and master base.
- Existing `patch_12_0_0` lib filter: 11 PASS / 1 FAIL both before and after; TransmogSituation metadata expected by historical test differs from current epoch.
- Existing integration `deprecated` filter: 107 PASS / 1 FAIL both before and after; `publication_deprecated_alias` hits parent bytecode bypass after earlier cache use.
- Exact-12.0.0 lib and native-retirement integration controls: cannot compile (`private_aura_sounds` referenced unconditionally by `on_update.rs:61`, module gated at 12.0.5). Same compiler failure reproduced by actual detached master-base checkout in this worktree; original branch restored. Three historical unused-import warnings also reproduce at master.
- Default local debug check, cargo format check and explicit new-test rustfmt check: PASS. Existing vendor manifest deprecations unsuppressed, unchanged from master.
- Startup Lua errors: exit 0, `[]`, zero unique/occurrence errors.

[Machine outcomes and command/revision proof ledger](p1200-enums-deprecated-proof.json) records all commands, results, output hashes and superseded development evidence. Full logs are local ignored artifacts, not required to consume this report.

## INFERRED choices

- Bounded successor fixtures use existing simulator data/state; no native recordings, full logging persistence, quest-domain parity or overlay model claimed.
- Historical Tooltip values differ from current cache. Earlier renumbering patch cannot be established from the inspected 12.0.5/12.0.7 registers; assign supersession to verified current 12.1.0 rather than inventing a date. EditMode parents have explicit later additions in 12.0.5 and 12.1.0; Survey has 12.0.5 EncounterEnd.
- Historical execution is blocked, not passing. Current numeric publication and native/cache retirement evidence remains valid.
