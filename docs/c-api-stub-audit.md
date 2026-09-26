# C_* API Stub Audit

Current state of all C_* namespace implementations in wow-ui-sim.

## Coverage Summary

The simulator registers **~260 C_* namespaces** across three layers:
1. **Hand-written Rust** (`src/lua_api/globals/c_*.rs`, `*_api.rs`) — real logic with state
2. **Hand-written stubs** (`c_stubs_api*.rs`, `c_misc_api*.rs`) — hardcoded defaults
3. **Auto-generated stubs** (`generated_stubs.rs`, ~19K lines) — catch-all from `scripts/generate_stubs.py`

Additionally, **~25 global (non-C_*) stubs** have been upgraded from auto-generated
nil returns to hand-written implementations with correct return values.

Only **16 C_* namespaces** referenced in BlizzardUI are truly missing, and all are
glue-screen (login/character creation) or legacy/removed APIs.

## Legend

- **Implemented** — Has real logic, returns meaningful data
- **Stubbed** — Returns hardcoded defaults (false, 0, nil, empty table, no-op)
- **Generated** — Auto-generated catch-all stub from generated_stubs.rs
- **Missing** — Namespace exists in WoW but not registered in the simulator

## Implemented Namespaces

### C_Timer (`timer_api.rs`)
| Function | Status |
|----------|--------|
| After | Implemented |
| NewTicker | Implemented |
| NewTimer | Implemented |

### C_XMLUtil (`c_system_api.rs`)
| Function | Status |
|----------|--------|
| GetTemplateInfo | Implemented |

### C_Reputation (`c_system_api.rs`)
| Function | Status |
|----------|--------|
| GetFactionDataByID | Implemented |
| GetFactionDataByIndex | Implemented |
| GetNumFactions | Implemented |
| GetFactionInfo | Implemented |
| GetWatchedFactionData | Implemented |
| IsFactionParagon | Stubbed |
| IsFactionParagonForCurrentPlayer | Stubbed |
| GetFactionParagonInfo | Stubbed |
| SetWatchedFactionByID | Stubbed |
| GetGuildFactionData | Stubbed |

### C_Item (`c_item_api.rs`)
| Function | Status |
|----------|--------|
| GetItemInfo | Implemented |
| GetItemInfoInstant | Implemented |
| GetItemIDForItemInfo | Implemented |
| GetItemSubClassInfo | Implemented |
| GetItemClassInfo | Implemented |
| GetItemNameByID | Implemented |
| GetItemIconByID | Stubbed |
| GetItemCount | Implemented (modeled aggregate count) |
| DoesItemExist | Implemented (modeled ItemLocation presence) |
| GetItemID | Implemented (modeled ItemLocation ID) |
| GetItemLink | Implemented (captured or catalog ItemLocation link) |
| IsItemDataCached | Implemented (modeled policy; native cache lifecycle unverified) |
| GetStackCount | Implemented (ItemLocation bag/equipment count) |
| GetItemSpecInfo | Stubbed |
| GetItemQualityColor | Stubbed |
| GetItemExpac | Stubbed |

### C_Container (`c_container_api.rs`)
| Function | Status |
|----------|--------|
| GetContainerItemID | Implemented |
| GetContainerItemLink | Implemented |
| GetContainerItemInfo | Implemented |
| HasContainerItem | Implemented |
| GetContainerItemQuestInfo | Stubbed |
| GetContainerItemCooldown | Stubbed |
| IsContainerFiltered | Stubbed |
| GetBagName | Stubbed |
| ContainerIDToInventoryID | Stubbed |
| GetBagSlotFlag | Implemented |
| SetBagSlotFlag | Implemented |
| GetBackpackAutosortDisabled | Implemented |
| SetBackpackAutosortDisabled | Stubbed |
| GetBackpackSellJunkDisabled | Implemented |
| SetBackpackSellJunkDisabled | Stubbed |
| GetContainerItemPurchaseInfo | Stubbed |
| UseContainerItem | Stubbed |
| PickupContainerItem | Stubbed |
| SplitContainerItem | Stubbed |
| IsBattlePayItem | Stubbed |
| SetBagPortraitTexture | Stubbed |

### C_Map (`c_map_api.rs`)
| Function | Status | Notes |
|----------|--------|-------|
| GetAreaInfo | Implemented | |
| GetMapInfo | Implemented | |
| GetWorldPosFromMapPos | Implemented | |
| GetBestMapForUnit | Stubbed | Returns 2274 (Dornogal) |
| GetCurrentMapID | Stubbed | Returns 2274 (Dornogal) |
| GetPlayerMapPosition | Stubbed | Returns (0.5, 0.5) |
| GetMapChildrenInfo | Stubbed | |
| GetMapWorldSize | Stubbed | |
| RequestPreloadMap | Stubbed | |
| MapHasArt | Stubbed | |

### C_QuestLog (`c_quest_api.rs`)
| Function | Status |
|----------|--------|
| GetNumQuestLogEntries | Implemented |
| GetInfo | Implemented |
| GetQuestIDForLogIndex | Implemented |
| GetLogIndexForQuestID | Implemented |
| GetQuestObjectives | Stubbed |
| GetMaxNumQuestsCanAccept | Stubbed |
| GetMaxNumQuests | Stubbed |
| SetMapForQuestPOIs | Stubbed |
| GetZoneStoryInfo | Stubbed |
| GetQuestAdditionalHighlights | Stubbed |
| IsQuestReplayable | Stubbed |
| GetQuestWatchType | Stubbed |
| HasActiveThreats | Stubbed |
| GetBountySetInfoForMapID | Stubbed |
| GetBountiesForMapID | Stubbed |
| IsUnitOnQuest | Stubbed |

### C_EditMode (`c_editmode_api.rs`)
| Function | Status |
|----------|--------|
| GetLayouts | Implemented |
| GetAccountSettings | Implemented |
| SaveLayouts | Stubbed |
| SetActiveLayout | Stubbed |
| SetAccountSetting | Stubbed |
| OnEditModeExit | Stubbed |
| OnLayoutAdded | Stubbed |
| OnLayoutDeleted | Stubbed |
| IsValidLayoutName | Stubbed |
| ConvertLayoutInfoToString | Stubbed |
| ConvertStringToLayoutInfo | Stubbed |
| ConvertLayoutInfoToHyperlink | Stubbed |

### C_DateAndTime (`c_map_api.rs` + `c_misc_api_core.rs`)
| Function | Status |
|----------|--------|
| AdjustTimeByDays | Implemented |
| AdjustTimeByMinutes | Implemented |
| CompareCalendarTime | Implemented |
| GetCurrentCalendarTime | Stubbed |
| GetServerTimeLocal | Stubbed |
| GetSecondsUntilDailyReset | Stubbed |
| GetSecondsUntilWeeklyReset | Stubbed |

### C_ColorOverrides (`c_misc_api.rs`)
| Function | Status |
|----------|--------|
| GetColorForQuality | Implemented |
| GetDefaultColorForQuality | Implemented |
| GetColorOverrideInfo | Stubbed |
| ClearColorOverrides | Stubbed |
| SetColorOverride | Stubbed |
| RemoveColorOverride | Stubbed |

### C_TransmogCollection (`c_collection_api.rs`)
| Function | Status |
|----------|--------|
| PlayerHasTransmog | Implemented |
| PlayerHasTransmogByItemInfo | Implemented |
| PlayerHasTransmogItemModifiedAppearance | Implemented |
| GetAppearanceSources | Stubbed |
| GetSourceInfo | Stubbed |
| GetAllAppearanceSources | Stubbed |
| (+ ~15 more) | Stubbed |

## Fully Stubbed Namespaces

### C_Console (`c_system_api.rs`)
- GetAllCommands → empty table
- GetColorFromType → white (1,1,1)

### C_VoiceChat (`c_system_api.rs`)
- 11 functions, all return false/nil/empty

### C_TTSSettings (`c_system_api.rs`)
- 6 functions, all return 0/100/no-op

### C_NewItems (`c_container_api.rs`)
- IsNewItem → false
- RemoveNewItem → no-op
- ClearAll → no-op

### C_ScenarioInfo (`c_misc_api_core.rs`)
- 5 functions, all return nil/false/0

### C_TradeSkillUI (`c_misc_api_core.rs`)
- 12 functions, all return 0/nil/empty/false

### C_Guild (`c_stubs_api.rs`)
- GetNumMembers → from state
- IsInGuild → from state
- GetGuildInfo → from state
- GetMemberInfo → nil

### C_GuildInfo (`c_stubs_api.rs`)
- 4 functions, all return nil/false/no-op

### C_LFGList (`c_stubs_api.rs`)
- 10 functions, all return nil/false/0/empty

### C_CurveUtil (`c_stubs_api_combat.rs`)
- CreateCurve → LuaCurveObject (fully implemented)
- CreateColorCurve → LuaColorCurveObject (fully implemented)

### C_ColorUtil (`c_stubs_api_combat.rs`)
- GenerateTextColorCode → implemented
- WrapTextInColor → implemented

### C_CombatLog (`c_stubs_api_combat.rs`)
- 14 functions, all stubbed

### C_ProfSpecs (`c_stubs_api_professions.rs`)
- ~25 functions, all return nil/false/0/empty

### C_AccountStore (`c_stubs_api_store.rs`)
- 10 functions, minimal stub data

### C_DelvesUI (`c_stubs_api_extra.rs`)
- 13 functions, all stubbed

### Other Fully Stubbed
- C_AchievementInfo, C_LossOfControl, C_Mail, C_StableInfo, C_Tutorial
- C_CampaignInfo, C_Macro, C_WoWLabsMatchmaking, C_WowLabsDataManager
- C_SpectatingUI, C_SocialRestrictions, C_LobbyMatchmakerInfo
- C_PlayerMentorship, C_RecentAllies, C_SocialQueue
- C_CinematicList, C_Login, C_SpellActivationOverlay, C_CharacterServices
- C_VideoOptions, C_IncomingSummon, C_PerksActivities, C_Log
- C_AccountServices, C_ArrowCalloutManager, C_EncounterEvents
- C_PrototypeDialog, C_Reincarnation, C_TableUtil, C_ItemSocketInfo
- C_PetInfo, C_UnitAurasPrivate, C_LevelLink, C_EventScheduler
- C_AuthChallenge, C_PingSecure, C_Ping, C_StoreSecure, C_WowTokenSecure
- C_ClassTrial, C_RecruitAFriend, C_WowTokenPublic, C_FriendList
- C_CatalogShop, C_Who, C_PrivateAuras, C_GuildBank, C_PetBattles
- C_RestrictedActions, C_TransmogOutfitInfo, C_DamageMeter
- C_CombatText, C_CombatAudioAlert, C_HousingPhotoSharing
- C_DeathRecap, C_EncounterTimeline, C_SettingsUtil
- C_Loot, C_ContentTracking, C_AchievementTelemetry
- C_PetJournal, C_MountJournal, C_ToyBox, C_Transmog, C_ZoneAbility

## Global API Stubs (Non-C_* Namespaces)

Global stubs that were upgraded from auto-generated nil returns to hand-written
stubs with correct return values.

### Player API (`player_api.rs`)
| Function | Return Value |
|----------|-------------|
| GetPlayerFacing | 0.0 |
| GetRaidDifficultyID | 14 (Normal) |
| GetResSicknessDuration | 0 |
| GetSheathState | 1 (Melee) |
| GetSpecializationNameForSpecID | Spec name lookup table |
| GetLegacyRaidDifficultyID | 1 |
| GetMaxCombatRatingBonus | 0.0 |
| GetMirrorTimerProgress | 0 |
| IsLegacyDifficulty | false |

### Event Query API (`event_query_api.rs`)
| Function | Return Value |
|----------|-------------|
| GetCurrentEventID | 0 |

### System API (`system_api.rs`)
| Function | Return Value |
|----------|-------------|
| IsKeyDown | false |

### Unit API (`unit_api.rs`)
| Function | Return Value |
|----------|-------------|
| UnitSexBase | Mapped sex enum (1/2/3) |

### Unit Combat API (`unit_combat_api.rs`)
| Function | Return Value |
|----------|-------------|
| UnitBattlePetSpeciesID | 0 |

### Unit Health/Power API (`unit_health_power_api.rs`)
| Function | Return Value |
|----------|-------------|
| UnitPercentHealthFromGUID | 100.0 |

### Misc Stubs (`c_stubs_api_extra.rs`)
| Function | Return Value |
|----------|-------------|
| CanEditGuildInfo | false |
| IsCpuBound | false |
| GetTotemCannotDismiss | false |
| GetTotemTimeLeft | 0.0 |
| GetSecondsUntilParentalControlsKick | 0 |
| ItemTextHasNextPage | false |

### Confirmed Correct nil Returns
These globals correctly return nil (no meaningful data in the simulator):
- `CreateFont` — no font objects in sim
- `GetNextPendingInviteConfirmation` — no invites
- `UnitPvpClassification` — no PvP
- `UnitThreatLeadSituation` — no combat
- `UnitThreatPercentageOfLead` — no combat
- `UnitTokenFromGUID` — no unit tracking

### Dead Stubs Removed from `generated_stubs.rs`
These were duplicates of existing hand-written implementations and have been
cleaned up:
- `GetFontInfo` — already in `font_api.rs`
- `UnitCastingInfo`, `UnitChannelInfo`, `UnitCastingDuration`, `UnitChannelDuration` — already in `unit_api.rs`
- `UnitIsBattlePet`, `UnitGUID` — already in `unit_api.rs`
- `C_CVar.*` (GetCVar, GetCVarBool, GetCVarDefault, GetCVarBitfield, SetCVar, SetCVarBitfield, RegisterCVar, ResetTestCVars) — already in `cvar_api.rs`
- `C_Map.GetBestMapForUnit`, `C_Map.GetPlayerMapPosition` — already in `c_map_api.rs`

## Dead Stub Cleanup

854 dead stubs were removed from `generated_stubs.rs`, reducing the file from ~22K to ~19K lines.
Dead stubs are functions that existed in both the auto-generated file AND a hand-written
`*_api.rs` file. The `is_nil()` guard meant they were functionally inert, but they added
unnecessary code.

Major namespaces cleaned:
- C_ActionBar: 46 stubs (already in action_bar_api.rs)
- C_Container/C_NewItems: 25 stubs (already in c_container_api.rs)
- C_UnitAuras: 10 stubs (already in c_misc_api_core.rs)
- C_UIWidgetManager: 6 stubs (already in c_misc_api_ui.rs)
- Plus ~767 more across 100+ namespaces
- 12 dead global function stubs (pcall, xpcall, type, etc.)

### Remaining Nil Stubs (351)

All 351 remaining nil-returning stubs have been verified as correct:
- Return types are nilable (`Type?`) where nil means "no data available"
- Systems not simulated (PvP, Auction House, Voice Chat, Housing, etc.)
- Query functions for state that doesn't exist in the simulator

## Missing Namespaces (Not Registered)

Only 16 C_* namespaces from BlizzardUI are truly missing. All are either
glue-screen APIs (login/character creation) or legacy/removed systems:

| Namespace | References | Domain |
|-----------|-----------|--------|
| C_CharacterCreation | 194 | Glue screen (character creation) |
| C_RealmList | 38 | Glue screen (server selection) |
| C_StoreGlue | 16 | Glue screen (store) |
| C_Reforge | 13 | Removed system (pre-WoD) |
| C_PaidServices | 9 | Glue screen (paid services) |
| C_WowTokenGlue | 8 | Glue screen (token) |
| C_ConfigurationWarnings | 4 | Glue screen |
| C_LiveEvent | 3 | Live event system |
| C_SocialContractGlue | 2 | Glue screen |
| C_MapInternal | 1 | Internal debug |
| C_GlyphInfo | 1 | Removed system (pre-Legion) |
| C_Barbershop | 1 | NPC interaction |
| C_ArtifactRelicForgeUI | 1 | Removed system (Legion) |

Note: All high/medium priority namespaces listed in the original audit
(C_Spell, C_SpellBook, C_CVar, C_AddOns, C_PlayerInfo, C_ClassColor,
C_UnitAuras, C_PartyInfo, C_SpecializationInfo, C_Traits, C_ChatInfo,
C_CurrencyInfo, C_GossipInfo, C_ChallengeMode, C_WeeklyRewards,
C_GameRules, C_Bank, C_SuperTrack, etc.) are ALL already implemented
in hand-written Rust or auto-generated stubs.

## Signature Mismatches (Fixed)

One signature mismatch was found and fixed:
- `C_PlayerInfo.CanPlayerUseMountEquipment()` — was returning `(bool)`,
  now correctly returns `(bool, string)` per the Blizzard API.
