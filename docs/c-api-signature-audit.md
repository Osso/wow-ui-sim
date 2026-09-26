# C_* API Signature Audit

Function signatures from `Blizzard_APIDocumentationGenerated/*.lua` (Patch 12.0.1).
Used to verify stub implementations match the official WoW API.

## C_Item

| Function | Parameters | Returns |
|----------|------------|---------|
| DoesItemExist | emptiableItemLocation: EmptiableItemLocation | itemExists: bool |
| GetItemGUID | itemLocation: ItemLocation | itemGUID: WOWGUID |
| GetItemID | itemLocation: ItemLocation | itemID: number |
| GetItemLink | itemLocation: ItemLocation | itemLink: string? |
| IsItemDataCached | itemLocation: ItemLocation | isCached: bool |

## C_Spell

| Function | Parameters | Returns |
|----------|-----------|---------|
| CancelSpellByID | spellID: number | void |
| DoesSpellExist | spellIdentifier | spellExists: bool |
| GetBaseSpell | spellIdentifier, spec?: number | baseSpellID: number |
| GetDeadlyDebuffInfo | spellIdentifier | deadlyDebuffInfo? |
| GetOverrideSpell | spellIdentifier, spec?, onlyKnown?, ignoreOverrideSpellID? | overrideSpellID: number |
| GetSchoolString | schoolMask: number | result: string |
| GetSpellAutoCast | spellIdentifier | autoCastAllowed: bool, autoCastEnabled: bool |
| GetSpellCastCount | spellIdentifier | castCount: number |
| GetSpellCharges | spellIdentifier | chargeInfo? |
| GetSpellCooldown | spellIdentifier | spellCooldownInfo? |
| GetSpellDescription | spellIdentifier | description: string? |
| GetSpellIDForSpellIdentifier | spellIdentifier | spellID: number? |
| GetSpellInfo | spellIdentifier | spellInfo? |
| GetSpellLevelLearned | spellIdentifier | levelLearned: number |
| GetSpellLink | spellIdentifier, glyphID? | spellLink: string? |
| GetSpellName | spellIdentifier | name: string? |
| GetSpellPowerCost | spellIdentifier | powerCosts? |
| GetSpellQueueWindow | void | result: number |
| GetSpellSubtext | spellIdentifier | subtext: string? |
| GetSpellTexture | spellIdentifier | iconID: fileID, originalIconID: fileID |
| GetVisibilityInfo | spellID, visibilityType | hasCustom, alwaysShowMine, showForMySpec |
| IsAutoAttackSpell | spellIdentifier | isAutoAttack: bool |
| IsAutoRepeatSpell | spellIdentifier | isAutoRepeat: bool |
| IsClassTalentSpell | spellIdentifier | bool |
| IsConsumableSpell | spellIdentifier | consumable: bool |
| IsCurrentSpell | spellIdentifier | isCurrentSpell: bool |
| IsExternalDefensive | spellID | isExternalDefensive: bool |
| IsPressHoldReleaseSpell | spellIdentifier | isPressHoldRelease: bool |
| IsPriorityAura | spellID | isHighPriority: bool |
| IsPvPTalentSpell | spellIdentifier | bool |
| IsRangedAutoAttackSpell | spellIdentifier | isRangedAutoAttack: bool |
| IsSelfBuff | spellID | hasSelfEffectsOnly: bool |
| IsSpellDataCached | spellIdentifier | isCached: bool |
| IsSpellDisabled | spellIdentifier | disabled: bool |
| IsSpellHarmful | spellIdentifier | isHarmful: bool |
| IsSpellHelpful | spellIdentifier | isHelpful: bool |
| IsSpellInRange | spellIdentifier, targetUnit? | inRange: bool? |
| IsSpellPassive | spellIdentifier | isPassive: bool |
| IsSpellUsable | spellIdentifier | isUsable: bool, insufficientPower: bool |
| PickupSpell | spellIdentifier | void |
| RequestLoadSpellData | spellIdentifier | void |
| SpellHasRange | spellIdentifier | hasRange: bool |
| TargetSpellIsEnchanting | void | isEnchanting: bool |

## C_SpellBook

| Function | Parameters | Returns |
|----------|-----------|---------|
| ContainsAnyDisenchantSpell | void | contains: bool |
| FindBaseSpellByID | spellID | baseSpellID? |
| FindSpellBookSlotForSpell | spellIdentifier, includeHidden?, includeFlyouts?, includeFutureSpells?, includeOffSpec? | slotIndex?, spellBank? |
| FindSpellOverrideByID | spellID | overrideSpellID? |
| GetCurrentLevelSpells | level | spellIDs? |
| GetNumSpellBookSkillLines | void | numSkillLines: number |
| GetSpellBookItemInfo | slotIndex, spellBank | spellBookItemInfo? |
| GetSpellBookItemName | slotIndex, spellBank | name, subName |
| GetSpellBookItemTexture | slotIndex, spellBank | iconID? |
| GetSpellBookItemType | slotIndex, spellBank | itemType, actionID, spellID? |
| GetSpellBookSkillLineInfo | skillLineIndex | skillLineInfo? |
| HasPetSpells | void | numPetSpells?, petNameToken? |
| IsSpellInSpellBook | spellID, spellBank?, includeOverrides? | isInSpellBook: bool |
| IsSpellKnown | spellID, spellBank? | isKnown: bool |

## C_CVar

| Function | Parameters | Returns |
|----------|-----------|---------|
| GetCVar | name: string | value: string? |
| GetCVarBitfield | name: string, index: number | value: bool? |
| GetCVarBool | name: string | value: bool? |
| GetCVarDefault | name: string | defaultValue: string? |
| GetCVarInfo | name: string | value, defaultValue, isStoredServerAccount, isStoredServerCharacter, isLockedFromUser, isSecure, isReadOnly |
| RegisterCVar | name: string, value?: string | void |
| ResetTestCVars | void | void |
| SetCVar | name: string, value?: string | success: bool |
| SetCVarBitfield | name: string, index: number, value: bool | success: bool |

## C_AddOns

| Function | Parameters | Returns |
|----------|-----------|---------|
| DisableAddOn | name, character? | void |
| DisableAllAddOns | character? | void |
| DoesAddOnExist | name | exists: bool |
| DoesAddOnHaveLoadError | name | hadError: bool |
| EnableAddOn | name, character? | void |
| EnableAllAddOns | character? | void |
| GetAddOnDependencies | name | ...deps |
| GetAddOnEnableState | name, character? | state: AddOnEnableState |
| GetAddOnInfo | name | name, title, notes, loadable, reason, security |
| GetAddOnInterfaceVersion | name | interfaceVersion: number |
| GetAddOnMetadata | name, variable | value: string |
| GetAddOnName | index | name: string |
| GetAddOnOptionalDependencies | name | ...deps |
| GetNumAddOns | void | numAddOns: number |
| IsAddOnLoadOnDemand | name | loadOnDemand: bool |
| IsAddOnLoadable | name, character?, demandLoaded? | loadable: bool, reason: string |
| IsAddOnLoaded | name | loadedOrLoading: bool, loaded: bool |
| LoadAddOn | name | loaded: bool?, value: string? |
| SaveAddOns | void | void |

## C_PlayerInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| CanPlayerEnterChromieTime | void | canEnter: bool |
| CanPlayerUseAreaLoot | void | canUseAreaLoot: bool |
| CanPlayerUseMountEquipment | void | canUse: bool, failureReason: string |
| CanUseItem | itemID: number | isUseable: bool |
| GetAlternateFormInfo | void | hasAlternateForm: bool, inAlternateForm: bool |
| GetDisplayID | void | displayID: number |
| GetGlidingInfo | void | isGliding, canGlide, forwardSpeed |
| GetNativeDisplayID | void | nativeDisplayID: number |
| GetPlayerMythicPlusRatingSummary | playerToken | ratingSummary? |
| HasAccountInventoryLock | void | hasLock: bool |
| HasVisibleInvSlot | slot | isVisible: bool |
| IsAccountBankEnabled | void | isEnabled: bool |
| IsCharacterBankEnabled | void | isEnabled: bool |
| IsDisplayRaceNative | void | isNative: bool |
| IsMirrorImage | void | isMirrorImage: bool |
| IsPlayerInChromieTime | void | inChromieTime: bool |
| IsReturningCharacter | void | isReturning: bool |
| IsSelfFoundActive | void | active: bool |
| IsTradingPostAvailable | void | isAvailable: bool |
| IsTravelersLogAvailable | void | isAvailable: bool |

## C_ClassColor

| Function | Parameters | Returns |
|----------|-----------|---------|
| GetClassColor | className: string | classColor: colorRGB? |

## C_ChatInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| CancelEmote | void | void |
| GetChannelInfoFromIdentifier | channelIdentifier | info? |
| GetChannelRosterInfo | channelIndex, rosterIndex | name, owner, moderator, guid |
| GetChannelShortcut | channelIndex | shortcut: string |
| GetChatLineSenderGUID | chatLine | guid: WOWGUID |
| GetChatTypeName | typeID | name: string? |
| GetColorForChatType | chatType | color? |
| GetGeneralChannelID | void | channelID: number |
| GetNumActiveChannels | void | numChannels: number |
| GetNumReservedChatWindows | void | numReserved: number |
| GetRegisteredAddonMessagePrefixes | void | prefixes |
| IsAddonMessagePrefixRegistered | prefix | isRegistered: bool |
| IsRegionalServiceAvailable | void | available: bool |
| IsValidChatLine | chatLine? | isValid: bool |
| RegisterAddonMessagePrefix | prefix | result |
| ReplaceIconAndGroupExpressions | input, noIconReplacement?, noGroupReplacement? | output: string |
| SendAddonMessage | prefix, message, chatType?, target? | result |
| SendChatMessage | message, chatType?, languageID?, target? | void |

## C_CurrencyInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| ExpandCurrencyList | index, expand | void |
| GetBackpackCurrencyInfo | index | info? |
| GetBasicCurrencyInfo | currencyType, quantity? | info? |
| GetCoinTextureString | amount, fontHeight? | result: string |
| GetCurrencyDescription | type | description: string |
| GetCurrencyIDFromLink | link | currencyID: number |
| GetCurrencyInfo | type | info? |
| GetCurrencyLink | type, amount? | link: string |
| GetCurrencyListInfo | index | info? |
| GetCurrencyListSize | void | size: number |
| IsCurrencyContainer | currencyID, quantity | isCurrencyContainer: bool |
| PickupCurrency | type | void |
| SetCurrencyBackpack | index, backpack | void |

## C_SpecializationInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| CanPlayerUsePVPTalentUI | void | canUse: bool, failureReason: string |
| CanPlayerUseTalentSpecUI | void | canUse: bool, failureReason: string |
| CanPlayerUseTalentUI | void | canUse: bool, failureReason: string |
| GetActiveSpecGroup | isInspect?, isPet? | groupIndex |
| GetAllSelectedPvpTalentIDs | void | selectedPvpTalentIDs |
| GetNumSpecializationsForClassID | classID | specCount: number |
| GetPvpTalentInfo | talentID | talentInfo? |
| GetSpecialization | isInspect?, isPet?, specGroupIndex? | specializationIndex |
| GetSpecializationInfo | specIndex, isInspect?, isPet?, inspectTarget?, sex?, groupIndex?, classID? | specId, name, description, icon, role, primaryStat, pointsSpent, background, previewPointsSpent, isUnlocked |
| IsInitialized | void | isInitialized: bool |
| SetSpecialization | specIndex | success: bool |

## C_PartyInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| AllowedToDoPartyConversion | toRaid: bool | allowed: bool |
| CanFormCrossFactionParties | void | bool |
| CanInvite | void | bool |
| ConfirmInviteUnit | targetName | void |
| ConfirmLeaveParty | category? | void |
| ConvertToParty | void | void |
| ConvertToRaid | void | void |
| DoCountdown | seconds | success: bool |
| GetActiveCategories | void | categories |
| GetMinLevel | category? | minLevel: number |
| InviteUnit | targetName | void |
| IsPartyFull | category? | isFull: bool |
| LeaveParty | category? | void |

## C_Traits

| Function | Parameters | Returns |
|----------|-----------|---------|
| CanEditConfig | configID | canEdit: bool, errorMessage: string |
| CanPurchaseRank | configID, nodeID, nodeEntryID | canPurchase: bool |
| CommitConfig | configID | success: bool |
| ConfigHasStagedChanges | configID | hasChanges: bool |
| GetConditionInfo | configID, condID | condInfo |
| GetConfigIDBySystemID | systemID | configID: number |
| GetConfigIDByTreeID | treeID | configID: number |
| GetConfigInfo | configID | configInfo |
| GetConfigsByType | configType | configIDs |
| GetDefinitionInfo | definitionID | definitionInfo |
| GetEntryInfo | configID, entryID | entryInfo |
| GetNodeCost | configID, nodeID | costs |
| GetNodeInfo | configID, nodeID | nodeInfo |
| GetStagedChangesCost | configID | costs |
| GetSubTreeInfo | configID, subTreeID | subTreeInfo |
| GetTraitCurrencyInfo | traitCurrencyID | flags, type, currencyTypesID?, icon? |
| GetTreeCurrencyInfo | configID, treeID, excludeStagedChanges | treeCurrencyInfo |
| GetTreeInfo | configID, treeID | treeInfo |
| GetTreeNodes | treeID | nodeIDs |
| IsReadyForCommit | void | bool |
| PurchaseRank | configID, nodeID | success: bool |
| RefundRank | configID, nodeID, clearEdges? | success: bool |
| RollbackConfig | configID | success: bool |
| StageConfig | configID | success: bool |

## C_GossipInfo

| Function | Parameters | Returns |
|----------|-----------|---------|
| CloseGossip | void | void |
| ForceGossip | void | forceGossip: bool |
| GetActiveQuests | void | info |
| GetAvailableQuests | void | info |
| GetFriendshipReputation | factionID | reputationInfo |
| GetNumActiveQuests | void | numQuests: number |
| GetNumAvailableQuests | void | numQuests: number |
| GetOptions | void | info |
| GetPoiForUiMapID | uiMapID | gossipPoiID? |
| GetText | void | gossipText: string |
| SelectActiveQuest | optionID | void |
| SelectAvailableQuest | optionID | void |
| SelectOption | optionID, text?, confirmed? | void |

## C_ChallengeMode

| Function | Parameters | Returns |
|----------|-----------|---------|
| GetActiveChallengeMapID | void | mapID? |
| GetActiveKeystoneInfo | void | level, affixIDs, wasCharged |
| GetAffixInfo | affixID | name, description, filedataid |
| GetDeathCount | void | numDeaths, timeLost |
| GetDungeonScoreRarityColor | score | scoreColor |
| GetMapTable | void | mapIDs |
| GetMapUIInfo | mapID | name, id, timeLimit, texture?, backgroundTexture, mapID |
| GetOverallDungeonScore | void | score: number |
| GetSlottedKeystoneInfo | void | mapID, affixIDs, keystoneLevel |
| HasSlottedKeystone | void | bool |
| IsChallengeModeActive | void | bool |

## C_WeeklyRewards

| Function | Parameters | Returns |
|----------|-----------|---------|
| AreRewardsForCurrentRewardPeriod | void | bool |
| CanClaimRewards | void | bool |
| ClaimReward | id | void |
| GetActivities | type? | activities |
| HasAvailableRewards | void | bool |
| HasGeneratedRewards | void | bool |
| HasInteraction | void | bool |
| OnUIInteract | void | void |

## C_GameRules

| Function | Parameters | Returns |
|----------|-----------|---------|
| GetActiveGameMode | void | gameMode |
| GetGameRuleAsFloat | gameRule, decimalPlaces | value: number |
| IsGameRuleActive | gameRule | isActive: bool |
| IsPlunderstorm | void | active: bool |
| IsStandard | void | active: bool |

## C_Bank

| Function | Parameters | Returns |
|----------|-----------|---------|
| CanPurchaseBankTab | bankType | bool |
| CanUseBank | bankType | bool |
| CanViewBank | bankType | bool |
| CloseBankFrame | void | void |
| FetchDepositedMoney | bankType | amount |
| FetchNumPurchasedBankTabs | bankType | number |
| FetchPurchasedBankTabIDs | bankType | bagIndexes |
| HasMaxBankTabs | bankType | bool |
