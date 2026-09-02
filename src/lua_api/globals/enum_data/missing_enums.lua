-- Auto-generated: Enum values from wowless globals.yaml
-- Source: ~/Repos/wowless/data/products/wow/globals.yaml
-- Only enums currently registered in Rust (mod.rs) are skipped here.
-- DO NOT EDIT MANUALLY - regenerate from globals.yaml.

if not Enum then Enum = {} end

if not Enum.AbbreviationDataError then
  Enum.AbbreviationDataError = {
    InvalidAbbreviation = 3,
    InvalidBreakpoint = 0,
    InvalidFractionDivisor = 2,
    InvalidSignificandDivisor = 1,
    NotMultipleOfTen = 4,
  }
end

if not Enum.AbbreviationDataErrorMeta then
  Enum.AbbreviationDataErrorMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AccountCurrencyTransferResultMeta then
  Enum.AccountCurrencyTransferResultMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.AccountData then
  Enum.AccountData = {
    Bindings = 2,
    Bindings2 = 3,
    CharacterListOrder = 16,
    ChatSettings = 7,
    ClickBindings = 12,
    Config = 0,
    Config2 = 1,
    CooldownManager = 17,
    CooldownManager2 = 18,
    FlaggedIDs = 10,
    FlaggedIDs2 = 11,
    FrontendChatSettings = 15,
    Macros = 4,
    Macros2 = 5,
    Shop2PendingOrders = 19,
    TtsSettings = 8,
    TtsSettings2 = 9,
    UIEditModeAccount = 13,
    UIEditModeChar = 14,
    UILayout = 6,
  }
end

if not Enum.AccountDataMeta then
  Enum.AccountDataMeta = {
    MaxValue = 19,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.AccountDataUpdateStatus then
  Enum.AccountDataUpdateStatus = {
    Corrupt = 2,
    Failed = 1,
    Success = 0,
    Toobig = 3,
  }
end

if not Enum.AccountDataUpdateStatusMeta then
  Enum.AccountDataUpdateStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AccountExportResult then
  Enum.AccountExportResult = {
    AlreadyInProgress = 11,
    Cancelled = 2,
    FailedToGenerateFile = 13,
    FailedToLockAccount = 12,
    FileInvalid = 8,
    FileWriteFailed = 9,
    NoAccountFound = 5,
    RequestedInvalidCharacter = 6,
    RpcError = 7,
    ShuttingDown = 3,
    Success = 0,
    TimedOut = 4,
    Unavailable = 10,
    UnknownError = 1,
  }
end

if not Enum.AccountExportResultMeta then
  Enum.AccountExportResultMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.AccountSequenceCacheType then
  Enum.AccountSequenceCacheType = {
    HouseDecor = 2,
    Invalid = 0,
    ["Local"] = 1,
  }
end

if not Enum.AccountSequenceCacheTypeMeta then
  Enum.AccountSequenceCacheTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AccountStateFlags then
  Enum.AccountStateFlags = {
    AccountUpgradeComplete = 4,
    InPetCombat = 2,
    LoadFailed = 1,
    None = 0,
    TokenEligCheckComplete = 8,
  }
end

if not Enum.AccountStateFlagsMeta then
  Enum.AccountStateFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AccountStateLoadedFlags then
  Enum.AccountStateLoadedFlags = {
    AccountCurrenciesLoaded = "0x0000000000200000",
    AccountFactionsLoaded = "0x0000000200000000",
    AccountItemsLoaded = "0x0000000400000000",
    AccountMappingLoaded = "0x0000008000000000",
    AccountNotificationsLoaded = "0x0000000008000000",
    AccountWowlabsLoaded = "0x0000000020000000",
    AchievementsLoaded = "0x0000000000000001",
    ArchivedPurchasesLoaded = "0x0000000000000200",
    AuctionableTokensLoaded = "0x0000000000002000",
    BanktabSettingsLoaded = "0x0000004000000000",
    BattleNetAccountLoaded = "0x0000000000100000",
    BitVectorsLoaded = "0x0000000100000000",
    BpayAddLicenseObjectsLoaded = "0x0000000000000800",
    BpayDistributionObjectsLoaded = "0x0000000000000100",
    BpayProductitemObjectsLoaded = "0x0000000000020000",
    CharacterItemsLoaded = "0x0000010000000000",
    CharactersLoaded = "0x0000000000000040",
    CombinedQuestLogLoaded = "0x0000000800000000",
    ConsumableTokensLoaded = "0x0000000000004000",
    CriteriaLoaded = "0x0000000000000002",
    CurrencyCapsLoaded = "0x0000000000000010",
    CurrencyTransferLogLoaded = "0x0000020000000000",
    DataElementsLoaded = "0x0000001000000000",
    DynamicCriteriaLoaded = "0x0000000001000000",
    EventRecordsLoaded = "0x0000200000000000",
    HousingDataLoaded = "0x0000080000000000",
    ItemCollectionsLoaded = "0x0000000000001000",
    LgVendorPurchaseLoaded = "0x0000040000000000",
    LoadedNone = "0x0000000000000000",
    MountsLoaded = "0x0000000000000004",
    PerksHeldItemLoaded = "0x0000000040000000",
    PerksPastRewardsLoaded = "0x0000000000008000",
    PerksPendingPurchaseLoaded = "0x0000000010000000",
    PerksPendingRewardsLoaded = "0x0000000080000000",
    PetjournalInitialized = "0x0000000000000008",
    PurchasesLoaded = "0x0000000000000080",
    QuestCriteriaLoaded = "0x0000000000080000",
    QuestLogLoaded = "0x0000000000000020",
    RafActivityLoaded = "0x0000000002000000",
    RafBalanceLoaded = "0x0000000000400000",
    RafRewardsLoaded = "0x0000000000800000",
    RevokedRafRewardsLoaded = "0x0000000004000000",
    SettingsLoaded = "0x0000000000000400",
    TransmogOutfitsLoaded = "0x0000400000000000",
    TrialBoostHistoryLoaded = "0x0000000000040000",
    VasTransactionsLoaded = "0x0000000000010000",
    WarbandScenesLoaded = "0x0000100000000000",
    WarbandsLoaded = "0x0000002000000000",
  }
end

if not Enum.AccountStateLoadedFlagsMeta then
  Enum.AccountStateLoadedFlagsMeta = {
    MaxValue = -2147483648,
    MinValue = 0,
    NumValues = 48,
  }
end

if not Enum.AccountStoreCategoryTypeMeta then
  Enum.AccountStoreCategoryTypeMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.AccountStoreFrontFlag then
  Enum.AccountStoreFrontFlag = {
    Enabled = 1,
    PurchaseEnabled = 2,
    RefundEnabled = 4,
  }
end

if not Enum.AccountStoreFrontFlagMeta then
  Enum.AccountStoreFrontFlagMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.AccountStoreItemFlag then
  Enum.AccountStoreItemFlag = {
    DisplayAsNew = 4,
    DisplayDefaultArmor = 1,
    DisplayOnly = 8,
    NotInGameReward = 2,
  }
end

if not Enum.AccountStoreItemFlagMeta then
  Enum.AccountStoreItemFlagMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.AccountStoreItemMode then
  Enum.AccountStoreItemMode = {
    Hidden = 2,
    Locked = 3,
    Normal = 1,
  }
end

if not Enum.AccountStoreItemModeMeta then
  Enum.AccountStoreItemModeMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.AccountStoreItemRewardType then
  Enum.AccountStoreItemRewardType = {
    Illusion = 7,
    Misc = 10,
    Mount = 2,
    Pet = 3,
    Tender = 9,
    Toy = 5,
    Transmog = 1,
    TransmogSet = 8,
    WarbandScene = 11,
  }
end

if not Enum.AccountStoreItemRewardTypeMeta then
  Enum.AccountStoreItemRewardTypeMeta = {
    MaxValue = 11,
    MinValue = 1,
    NumValues = 9,
  }
end

if not Enum.AccountStoreItemStatus then
  Enum.AccountStoreItemStatus = {
    Owned = 3,
    Refundable = 2,
    Unowned = 1,
  }
end

if not Enum.AccountStoreItemStatusMeta then
  Enum.AccountStoreItemStatusMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.AccountStoreSettlementAction then
  Enum.AccountStoreSettlementAction = {
    Give = 1,
    NotSet = 0,
    Remove = 2,
  }
end

if not Enum.AccountStoreSettlementActionMeta then
  Enum.AccountStoreSettlementActionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AccountStoreState then
  Enum.AccountStoreState = {
    Available = 0,
    Unavailable = 2,
    Unknown = 1,
  }
end

if not Enum.AccountStoreStateMeta then
  Enum.AccountStoreStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AccountStoreTransactionResult then
  Enum.AccountStoreTransactionResult = {
    Incomplete = 1,
    InsufficientFunds = 4,
    InvalidCurrencyType = 8,
    ItemAlreadyOwned = 6,
    ItemNotOwned = 7,
    ItemUnknown = 5,
    NotSupported = 10,
    OwnedButRefundTimeExpired = 9,
    ProxyError = 12,
    Success = 0,
    TransactionInProgress = 3,
    Unavailable = 11,
    UnknownError = 2,
  }
end

if not Enum.AccountStoreTransactionResultMeta then
  Enum.AccountStoreTransactionResultMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.AccountStoreTransactionType then
  Enum.AccountStoreTransactionType = {
    DebugRemoveItem = 4,
    DebugResetHistory = 3,
    Purchase = 1,
    Refund = 2,
    Undefined = 0,
  }
end

if not Enum.AccountStoreTransactionTypeMeta then
  Enum.AccountStoreTransactionTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AccountTransType then
  Enum.AccountTransType = {
    AccountCurrencies = 26,
    AccountNotifications = 38,
    AccountStore = 54,
    Achievements = 4,
    AddLicense = 16,
    ArchivedPurchases = 9,
    AuctionableToken = 18,
    BankTab = 48,
    BattlenetAccount = 25,
    Battlepet = 3,
    BitVectors = 50,
    CharacterDataMerge = 53,
    CharacterItems = 57,
    Characters = 7,
    CombinedQuestLog = 51,
    ConsumableToken = 19,
    CreateOrderInfo = 32,
    Criteria = 5,
    CriteriaNotif = 13,
    CurrencyCaps = 11,
    CurrencyTransferLog = 58,
    Distribution = 2,
    Distributions = 10,
    DynamicCriteria = 30,
    EventRecords = 63,
    Factions = 49,
    FixedLicense = 15,
    GetOrderStatusByPurchaseID = 46,
    HouseInitiativeFavor = 66,
    HousingItem = 64,
    ItemCollections = 17,
    Items = 47,
    LgVendorPurchase = 59,
    LoadWowlabs = 44,
    Mapping = 56,
    Mounts = 6,
    OutstandingRpc = 43,
    PerkItemHold = 39,
    PerkPastRewards = 41,
    PerkPendingRewards = 40,
    PerkTransaction = 42,
    PlayerDataElements = 52,
    Productitem = 21,
    Profile = 61,
    ProxyCreateAccountHonor = 34,
    ProxyForwarder = 0,
    ProxyGenerateBpayID = 37,
    ProxyGmSetHonor = 36,
    ProxyHonorInitialConversion = 33,
    ProxyValidateAccountHonor = 35,
    Purchase = 1,
    Purchases = 8,
    QuestCriteria = 24,
    QuestLog = 12,
    RafActivity = 31,
    RafFriendMonth = 28,
    RafRecruiterAcceptances = 27,
    RafReward = 29,
    SaveWarbandGroups = 60,
    Settings = 14,
    TransmogOutfitCollection = 65,
    TrialBoostHistories = 23,
    TrialBoostHistory = 22,
    UpgradeAccount = 45,
    VasTransaction = 20,
    WarbandGroups = 55,
    WarbandSceneCollection = 62,
  }
end

if not Enum.AccountTransTypeMeta then
  Enum.AccountTransTypeMeta = {
    MaxValue = 66,
    MinValue = 0,
    NumValues = 67,
  }
end

if not Enum.ActionBarOrientationMeta then
  Enum.ActionBarOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ActionBarVisibleSettingMeta then
  Enum.ActionBarVisibleSettingMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AddOnEnableState then
  Enum.AddOnEnableState = {
    All = 2,
    None = 0,
    Some = 1,
  }
end

if not Enum.AddOnEnableStateMeta then
  Enum.AddOnEnableStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AddOnPerformanceMessageType then
  Enum.AddOnPerformanceMessageType = {
    OverallAddOnErrorDialog = 2,
    SpecificAddOnChatWarning = 0,
    SpecificAddOnErrorDialog = 1,
  }
end

if not Enum.AddOnPerformanceMessageTypeMeta then
  Enum.AddOnPerformanceMessageTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AddOnProfilerMetric then
  Enum.AddOnProfilerMetric = {
    CountTimeOver1000Ms = 11,
    CountTimeOver100Ms = 9,
    CountTimeOver10Ms = 7,
    CountTimeOver1Ms = 5,
    CountTimeOver500Ms = 10,
    CountTimeOver50Ms = 8,
    CountTimeOver5Ms = 6,
    EncounterAverageTime = 2,
    LastTime = 3,
    PeakTime = 4,
    RecentAverageTime = 1,
    SessionAverageTime = 0,
  }
end

if not Enum.AddOnProfilerMetricMeta then
  Enum.AddOnProfilerMetricMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.AddOnRestrictionState then
  Enum.AddOnRestrictionState = {
    Activating = 1,
    Active = 2,
    Inactive = 0,
  }
end

if not Enum.AddOnRestrictionStateMeta then
  Enum.AddOnRestrictionStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AddOnRestrictionType then
  Enum.AddOnRestrictionType = {
    ChallengeMode = 2,
    Combat = 0,
    Encounter = 1,
    Map = 4,
    PvPMatch = 3,
  }
end

if not Enum.AddOnRestrictionTypeMeta then
  Enum.AddOnRestrictionTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AddOnSecurityStatus then
  Enum.AddOnSecurityStatus = {
    Banned = 2,
    Insecure = 1,
    NotAvailable = 3,
    Secure = 0,
  }
end

if not Enum.AddOnSecurityStatusMeta then
  Enum.AddOnSecurityStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AddSoulbindConduitReason then
  Enum.AddSoulbindConduitReason = {
    Cheat = 1,
    None = 0,
    SpellEffect = 2,
    Upgrade = 3,
  }
end

if not Enum.AddSoulbindConduitReasonMeta then
  Enum.AddSoulbindConduitReasonMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AnimaDiversionNodeState then
  Enum.AnimaDiversionNodeState = {
    Available = 1,
    Cooldown = 4,
    SelectedPermanent = 3,
    SelectedTemporary = 2,
    Unavailable = 0,
  }
end

if not Enum.AnimaDiversionNodeStateMeta then
  Enum.AnimaDiversionNodeStateMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ArrowCalloutDirectionMeta then
  Enum.ArrowCalloutDirectionMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ArrowCalloutType then
  Enum.ArrowCalloutType = {
    Generic = 1,
    None = 0,
    Tutorial = 3,
    WidgetContainerNoBorder = 4,
    WorldLootObject = 2,
  }
end

if not Enum.ArrowCalloutTypeMeta then
  Enum.ArrowCalloutTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AssertDomain then
  Enum.AssertDomain = {
    Art = 1,
    Design = 2,
    Engineering = 4,
    LiveOperations = 64,
    Performance = 32,
    Sound = 8,
    Tools = 16,
  }
end

if not Enum.AssertDomainMeta then
  Enum.AssertDomainMeta = {
    MaxValue = 64,
    MinValue = 1,
    NumValues = 7,
  }
end

if not Enum.AssistActionType then
  Enum.AssistActionType = {
    CapturedBuff = 6,
    GraveMarker = 2,
    LoungingPlayer = 1,
    None = 0,
    PlacedVo = 3,
    PlayerGuardian = 4,
    PlayerSlayer = 5,
  }
end

if not Enum.AssistActionTypeMeta then
  Enum.AssistActionTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.AuctionHouseCommoditySortOrder then
  Enum.AuctionHouseCommoditySortOrder = {
    Quantity = 1,
    UnitPrice = 0,
  }
end

if not Enum.AuctionHouseCommoditySortOrderMeta then
  Enum.AuctionHouseCommoditySortOrderMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.AuctionHouseError then
  Enum.AuctionHouseError = {
    BidIncrement = 2,
    BidOwn = 3,
    BoundItem = 16,
    ConjuredItem = 17,
    DatabaseError = 10,
    DoubleBid = 23,
    EquippedBag = 20,
    FavoritesMaxed = 24,
    HasRestriction = 6,
    HigherBid = 1,
    IsBag = 19,
    IsBusy = 7,
    ItemBoundToAccountUntilEquip = 26,
    ItemHasQuote = 9,
    ItemNotAvailable = 25,
    ItemNotFound = 4,
    LimitedDurationItem = 18,
    LootItem = 22,
    MinBid = 11,
    NotEnoughItems = 12,
    NotEnoughMoney = 0,
    QuestItem = 15,
    RepairItem = 13,
    RestrictedAccountTrial = 5,
    Unavailable = 8,
    UsedCharges = 14,
    WrappedItem = 21,
  }
end

if not Enum.AuctionHouseErrorMeta then
  Enum.AuctionHouseErrorMeta = {
    MaxValue = 26,
    MinValue = 0,
    NumValues = 27,
  }
end

if not Enum.AuctionHouseExtraColumn then
  Enum.AuctionHouseExtraColumn = {
    Ilvl = 1,
    Level = 3,
    None = 0,
    Skill = 4,
    Slots = 2,
  }
end

if not Enum.AuctionHouseExtraColumnMeta then
  Enum.AuctionHouseExtraColumnMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.AuctionHouseFilter then
  Enum.AuctionHouseFilter = {
    ArtifactQuality = 12,
    CommonQuality = 7,
    CurrentExpansionOnly = 3,
    EpicQuality = 10,
    ExactMatch = 5,
    LegendaryCraftedItemOnly = 13,
    LegendaryQuality = 11,
    None = 0,
    PoorQuality = 6,
    RareQuality = 9,
    UncollectedOnly = 1,
    UncommonQuality = 8,
    UpgradesOnly = 4,
    UsableOnly = 2,
  }
end

if not Enum.AuctionHouseFilterCategory then
  Enum.AuctionHouseFilterCategory = {
    Equipment = 1,
    Rarity = 2,
    Uncategorized = 0,
  }
end

if not Enum.AuctionHouseFilterCategoryMeta then
  Enum.AuctionHouseFilterCategoryMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AuctionHouseFilterMeta then
  Enum.AuctionHouseFilterMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.AuctionHouseItemSortOrder then
  Enum.AuctionHouseItemSortOrder = {
    Bid = 0,
    Buyout = 1,
  }
end

if not Enum.AuctionHouseItemSortOrderMeta then
  Enum.AuctionHouseItemSortOrderMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.AuctionHouseNotification then
  Enum.AuctionHouseNotification = {
    AuctionExpired = 5,
    AuctionOutbid = 3,
    AuctionRemoved = 1,
    AuctionSold = 4,
    AuctionWon = 2,
    BidPlaced = 0,
  }
end

if not Enum.AuctionHouseNotificationMeta then
  Enum.AuctionHouseNotificationMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.AuctionHouseSortOrder then
  Enum.AuctionHouseSortOrder = {
    Bid = 3,
    Buyout = 4,
    Level = 2,
    Name = 1,
    Price = 0,
    TimeRemaining = 5,
  }
end

if not Enum.AuctionHouseSortOrderMeta then
  Enum.AuctionHouseSortOrderMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.AuctionHouseTimeLeftBand then
  Enum.AuctionHouseTimeLeftBand = {
    Long = 2,
    Medium = 1,
    Short = 0,
    VeryLong = 3,
  }
end

if not Enum.AuctionHouseTimeLeftBandMeta then
  Enum.AuctionHouseTimeLeftBandMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AuctionStatus then
  Enum.AuctionStatus = {
    Active = 0,
    Sold = 1,
  }
end

if not Enum.AuctionStatusMeta then
  Enum.AuctionStatusMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.AuraFrameIconDirectionMeta then
  Enum.AuraFrameIconDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AuraFrameIconWrapMeta then
  Enum.AuraFrameIconWrapMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AuraFrameOrientationMeta then
  Enum.AuraFrameOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.AuraFrameVisibleSettingMeta then
  Enum.AuraFrameVisibleSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.AvgItemLevelCategories then
  Enum.AvgItemLevelCategories = {
    Base = 0,
    EquippedBase = 1,
    EquippedEffective = 2,
    EquippedEffectiveWeighted = 5,
    PvP = 3,
    PvPWeighted = 4,
  }
end

if not Enum.AvgItemLevelCategoriesMeta then
  Enum.AvgItemLevelCategoriesMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.AzeriteEssenceSlot then
  Enum.AzeriteEssenceSlot = {
    MainSlot = 0,
    PassiveOneSlot = 1,
    PassiveThreeSlot = 3,
    PassiveTwoSlot = 2,
  }
end

if not Enum.AzeriteEssenceSlotMeta then
  Enum.AzeriteEssenceSlotMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.AzeritePowerLevel then
  Enum.AzeritePowerLevel = {
    Base = 0,
    Downgraded = 2,
    Upgraded = 1,
  }
end

if not Enum.AzeritePowerLevelMeta then
  Enum.AzeritePowerLevelMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.BagFlag then
  Enum.BagFlag = {
    AllowBagsInNonBagSlots = 1024,
    AllowBuyback = 65536,
    AllowPartialStack = 16384,
    AllowSoulboundItemInAccountBank = 134217728,
    AlreadyBound = 4,
    AlreadyOwner = 2,
    AsymmetricSwap = 1048576,
    BagIsEmpty = 16,
    DontFindStack = 1,
    HasRefund = 33554432,
    IgnoreBankcheck = 512,
    IgnoreBoundItemCheck = 64,
    IgnoreExisting = 8192,
    IgnorePetBankcheck = 131072,
    IgnoreReagentBags = 8388608,
    IgnoreSoulbound = 4194304,
    LookInAccountBankOnly = 16777216,
    LookInCharacterBankOnly = 32768,
    LookInInventory = 32,
    PreferNeutralPriorityBags = 524288,
    PreferPriorityBags = 262144,
    PreferQuivers = 2048,
    PreferReagentBags = 2097152,
    RecurseQuivers = 256,
    SkipValidCountCheck = 67108864,
    StackOnly = 128,
    Swap = 8,
    SwapBags = 4096,
  }
end

if not Enum.BagFlagMeta then
  Enum.BagFlagMeta = {
    MaxValue = 134217728,
    MinValue = 1,
    NumValues = 28,
  }
end

if not Enum.BagIndexMeta then
  Enum.BagIndexMeta = {
    MaxValue = 16,
    MinValue = -3,
    NumValues = 20,
  }
end

if not Enum.BagSlotFlagsMeta then
  Enum.BagSlotFlagsMeta = {
    MaxValue = 512,
    MinValue = 1,
    NumValues = 10,
  }
end

if not Enum.BagsDirectionMeta then
  Enum.BagsDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.BagsOrientationMeta then
  Enum.BagsOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.BalanceType then
  Enum.BalanceType = {
    Eclipse = 0,
    None = -1,
  }
end

if not Enum.BalanceTypeMeta then
  Enum.BalanceTypeMeta = {
    MaxValue = 0,
    MinValue = -1,
    NumValues = 2,
  }
end

if not Enum.BankLockedReasonMeta then
  Enum.BankLockedReasonMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.BankType then
  Enum.BankType = {
    Account = 2,
    Character = 0,
    Guild = 1,
  }
end

if not Enum.BankTypeMeta then
  Enum.BankTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.Base64Variant then
  Enum.Base64Variant = {
    Standard = 0,
    StandardUrlSafe = 1,
  }
end

if not Enum.Base64VariantMeta then
  Enum.Base64VariantMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.BattlePetAbilityFlag then
  Enum.BattlePetAbilityFlag = {
    DisplayAsHostileDebuff = 1,
    HideStrongWeakHints = 2,
    Passive = 4,
    ServerOnlyAura = 8,
    ShowCast = 16,
    StartOnCooldown = 32,
  }
end

if not Enum.BattlePetAbilityFlagMeta then
  Enum.BattlePetAbilityFlagMeta = {
    MaxValue = 32,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.BattlePetAbilitySlot then
  Enum.BattlePetAbilitySlot = {
    A = 0,
    B = 1,
    C = 2,
  }
end

if not Enum.BattlePetAbilitySlotMeta then
  Enum.BattlePetAbilitySlotMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.BattlePetAbilityTargets then
  Enum.BattlePetAbilityTargets = {
    Caster = 9,
    EnemyBackPet_1 = 5,
    EnemyBackPet_2 = 6,
    EnemyFrontPet = 0,
    EnemyPad = 3,
    FriendlyBackPet_1 = 7,
    FriendlyBackPet_2 = 8,
    FriendlyFrontPet = 1,
    FriendlyPad = 4,
    Owner = 10,
    ProcTarget = 12,
    Specific = 11,
    Weather = 2,
  }
end

if not Enum.BattlePetAbilityTargetsMeta then
  Enum.BattlePetAbilityTargetsMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.BattlePetAbilityTurnFlag then
  Enum.BattlePetAbilityTurnFlag = {
    CanProcFromProc = 1,
    TriggerByAuraCaster = 32,
    TriggerByEnemy = 8,
    TriggerByFriend = 4,
    TriggerBySelf = 2,
    TriggerByWeather = 16,
  }
end

if not Enum.BattlePetAbilityTurnFlagMeta then
  Enum.BattlePetAbilityTurnFlagMeta = {
    MaxValue = 32,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.BattlePetAbilityTurnType then
  Enum.BattlePetAbilityTurnType = {
    Normal = 0,
    TriggeredEffect = 1,
  }
end

if not Enum.BattlePetAbilityTurnTypeMeta then
  Enum.BattlePetAbilityTurnTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.BattlePetAbilityType then
  Enum.BattlePetAbilityType = {
    Ability = 0,
    Aura = 1,
  }
end

if not Enum.BattlePetAbilityTypeMeta then
  Enum.BattlePetAbilityTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.BattlePetActionMeta then
  Enum.BattlePetActionMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.BattlePetBreedQuality then
  Enum.BattlePetBreedQuality = {
    Common = 1,
    Epic = 4,
    Legendary = 5,
    Poor = 0,
    Rare = 3,
    Uncommon = 2,
  }
end

if not Enum.BattlePetBreedQualityMeta then
  Enum.BattlePetBreedQualityMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.BattlePetEffectFlags then
  Enum.BattlePetEffectFlags = {
    EnableAbilityPicker = 1,
    LuaNeedsAllPets = 2,
  }
end

if not Enum.BattlePetEffectFlagsMeta then
  Enum.BattlePetEffectFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.BattlePetEffectParamType then
  Enum.BattlePetEffectParamType = {
    Ability = 1,
    Int = 0,
  }
end

if not Enum.BattlePetEffectParamTypeMeta then
  Enum.BattlePetEffectParamTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.BattlePetEvent then
  Enum.BattlePetEvent = {
    OnAbility = 9,
    OnAuraApplied = 0,
    OnAuraRemoved = 5,
    OnDamageDealt = 2,
    OnDamageTaken = 1,
    OnHealDealt = 4,
    OnHealTaken = 3,
    OnRoundEnd = 7,
    OnRoundStart = 6,
    OnSwapIn = 10,
    OnSwapOut = 11,
    OnTurn = 8,
    PostAuraTicks = 12,
  }
end

if not Enum.BattlePetEventMeta then
  Enum.BattlePetEventMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.BattlePetNpcEmote then
  Enum.BattlePetNpcEmote = {
    BattleLose = 3,
    BattleStart = 1,
    BattleUnused = 0,
    BattleWin = 2,
    PetAbility = 7,
    PetDie = 6,
    PetKill = 5,
    PetSwap = 4,
  }
end

if not Enum.BattlePetNpcEmoteMeta then
  Enum.BattlePetNpcEmoteMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.BattlePetNpcTeamFlag then
  Enum.BattlePetNpcTeamFlag = {
    MatchPlayerHighPetLevel = 1,
    NoPlayerXP = 2,
  }
end

if not Enum.BattlePetNpcTeamFlagMeta then
  Enum.BattlePetNpcTeamFlagMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.BattlePetOwner then
  Enum.BattlePetOwner = {
    Ally = 1,
    Enemy = 2,
    Weather = 0,
  }
end

if not Enum.BattlePetOwnerMeta then
  Enum.BattlePetOwnerMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.BattlePetSources then
  Enum.BattlePetSources = {
    Achievement = 5,
    Discovery = 10,
    Drop = 0,
    PetStore = 9,
    Profession = 3,
    Promotion = 7,
    Quest = 1,
    Tcg = 8,
    TradingPost = 11,
    Vendor = 2,
    WildPet = 4,
    WorldEvent = 6,
  }
end

if not Enum.BattlePetSourcesMeta then
  Enum.BattlePetSourcesMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.BattlePetSpeciesFlags then
  Enum.BattlePetSpeciesFlags = {
    AddsAllowedWithBoss = 8192,
    AllianceOnly = 512,
    Boss = 1024,
    CantBattle = 128,
    Capturable = 8,
    HideFromJournal = 32,
    HideUntilLearned = 16384,
    HordeOnly = 256,
    LegacyAccountUnique = 64,
    MatchPlayerHighPetLevel = 32768,
    NoLicenseRequired = 4096,
    NoRename = 1,
    NoWildPetAddsAllowed = 65536,
    NotAcccountwide = 4,
    NotTradable = 16,
    RandomDisplay = 2048,
    WellKnown = 2,
  }
end

if not Enum.BattlePetSpeciesFlagsMeta then
  Enum.BattlePetSpeciesFlagsMeta = {
    MaxValue = 65536,
    MinValue = 1,
    NumValues = 17,
  }
end

if not Enum.BattlePetStateFlag then
  Enum.BattlePetStateFlag = {
    Client = 8,
    DynamicScaling = 128,
    MaxHealthBonus = 16,
    None = 0,
    Power = 256,
    QualityDoesNotEffect = 64,
    ServerOnly = 2048,
    SpeedBonus = 4,
    SpeedMult = 512,
    Stamina = 32,
    SwapInLock = 1024,
    SwapOutLock = 1,
    TurnLock = 2,
  }
end

if not Enum.BattlePetStateFlagMeta then
  Enum.BattlePetStateFlagMeta = {
    MaxValue = 2048,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.BattlePetTypes then
  Enum.BattlePetTypes = {
    Aquatic = 8,
    Beast = 7,
    Critter = 4,
    Dragonkin = 1,
    Elemental = 6,
    Flying = 2,
    Humanoid = 0,
    Magic = 5,
    Mechanical = 9,
    Undead = 3,
  }
end

if not Enum.BattlePetTypesMeta then
  Enum.BattlePetTypesMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.BattlePetVisualFlag then
  Enum.BattlePetVisualFlag = {
    Test1 = 1,
    Test2 = 2,
    Test3 = 4,
  }
end

if not Enum.BattlePetVisualFlagMeta then
  Enum.BattlePetVisualFlagMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.BattlePetVisualRange then
  Enum.BattlePetVisualRange = {
    BehindMelee = 4,
    BehindRanged = 5,
    InPlace = 2,
    Melee = 0,
    PointBlank = 3,
    Ranged = 1,
  }
end

if not Enum.BattlePetVisualRangeMeta then
  Enum.BattlePetVisualRangeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.BattlepayBannerType then
  Enum.BattlepayBannerType = {
    Discount = 1,
    Featured = 2,
    New = 3,
    None = 0,
  }
end

if not Enum.BattlepayBannerTypeMeta then
  Enum.BattlepayBannerTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.BattlepayCardType then
  Enum.BattlepayCardType = {
    FullCardWithBuyButton = 4,
    FullCardWithNydusLinkButton = 8,
    LargeHorizontalCard = 2,
    LargeHorizontalCardWithBuyButton = 6,
    LargeVeritcalCard = 3,
    LargeVeritcalCardWithBuyButton = 7,
    LargeVeritcalPageableCardWithBuyButton = 9,
    MediumCard = 1,
    MediumCardWithBuyButton = 5,
    SmallCard = 0,
  }
end

if not Enum.BattlepayCardTypeMeta then
  Enum.BattlepayCardTypeMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.BattlepayDisplayFlags then
  Enum.BattlepayDisplayFlags = {
    CardAlwaysShowsTexture = 4,
    CardDoesNotShowModel = 2,
    Deprecated1 = 32,
    Deprecated2 = 64,
    Expansion = 1,
    HiddenPrice = 8,
    HideWhenOwned = 256,
    RafReward = 512,
    ShowFancyToast = 1024,
    UseHorizontalLayoutForFullCard = 16,
    UseIconBorderWithOverrideTexture = 2048,
    UseSquareIconBorder = 128,
  }
end

if not Enum.BattlepayDisplayFlagsMeta then
  Enum.BattlepayDisplayFlagsMeta = {
    MaxValue = 2048,
    MinValue = 1,
    NumValues = 12,
  }
end

if not Enum.BattlepayGroupDisplayType then
  Enum.BattlepayGroupDisplayType = {
    ActiveTimeOnly = 2,
    Everyone = 0,
    TrialAndVeteranOnly = 1,
  }
end

if not Enum.BattlepayGroupDisplayTypeMeta then
  Enum.BattlepayGroupDisplayTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.BattlepayLicenseSynthesisFlags then
  Enum.BattlepayLicenseSynthesisFlags = {
    ForceToGameAccount = 1,
  }
end

if not Enum.BattlepayLicenseSynthesisFlagsMeta then
  Enum.BattlepayLicenseSynthesisFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.BattlepayProductChoiceType then
  Enum.BattlepayProductChoiceType = {
    ChoiceNone = 0,
    ChoiceOne = 1,
    SpecAndFaction = 2,
    VasCharacter = 3,
    VasCharacterAndName = 4,
  }
end

if not Enum.BattlepayProductChoiceTypeMeta then
  Enum.BattlepayProductChoiceTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.BattlepayProductDecorator then
  Enum.BattlepayProductDecorator = {
    Boost = 0,
    Expansion = 1,
    VasService = 3,
    WoWToken = 2,
  }
end

if not Enum.BattlepayProductDecoratorMeta then
  Enum.BattlepayProductDecoratorMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.BattlepayProductGroupFlags then
  Enum.BattlepayProductGroupFlags = {
    DisableOwnedProducts = 4,
    EnabledForTrial = 2,
    EnabledForVeteran = 8,
    HideOwnedProducts = 1,
    None = 0,
  }
end

if not Enum.BattlepayProductGroupFlagsMeta then
  Enum.BattlepayProductGroupFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.BattlepayShopEntryBannerType then
  Enum.BattlepayShopEntryBannerType = {
    Discount = 1,
    Featured = 0,
    New = 2,
  }
end

if not Enum.BattlepayShopEntryBannerTypeMeta then
  Enum.BattlepayShopEntryBannerTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.BattlepayShopEntryFlags then
  Enum.BattlepayShopEntryFlags = {
    None = 0,
  }
end

if not Enum.BattlepayShopEntryFlagsMeta then
  Enum.BattlepayShopEntryFlagsMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.BattlepetDbFlags then
  Enum.BattlepetDbFlags = {
    Ability0Selection = 16,
    Ability1Selection = 32,
    Ability2Selection = 64,
    AcquiredViaLicense = 512,
    Converted = 2,
    DisplayOverridden = 256,
    FanfareNeeded = 128,
    Favorite = 1,
    LockMask = 12,
    LockedForConvert = 8,
    None = 0,
    Revoked = 4,
    TradingPost = 1024,
  }
end

if not Enum.BattlepetDbFlagsMeta then
  Enum.BattlepetDbFlagsMeta = {
    MaxValue = 1024,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.BattlepetDeletedReason then
  Enum.BattlepetDeletedReason = {
    AccountStore = 7,
    CageError = 4,
    DelJournal = 5,
    Gm = 3,
    PlayerCaged = 2,
    PlayerReleased = 1,
    TradingPost = 6,
    Unknown = 0,
  }
end

if not Enum.BattlepetDeletedReasonMeta then
  Enum.BattlepetDeletedReasonMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.BattlepetSlotLockCheat then
  Enum.BattlepetSlotLockCheat = {
    CheatOff = 0,
    Cheat_0_Locked = -1,
    Cheat_1_Locked = -2,
    Cheat_2_Locked = -3,
    UnlockAll = 1,
  }
end

if not Enum.BattlepetSlotLockCheatMeta then
  Enum.BattlepetSlotLockCheatMeta = {
    MaxValue = 1,
    MinValue = -3,
    NumValues = 5,
  }
end

if not Enum.BindingContext then
  Enum.BindingContext = {
    HousingEditor = 1,
    HousingEditorBasicAndExpertDecorMode = 7,
    HousingEditorBasicDecorMode = 2,
    HousingEditorCleanupMode = 5,
    HousingEditorCustomizeMode = 4,
    HousingEditorExpertDecorMode = 3,
    HousingEditorExteriorCustomizationMode = 8,
    HousingEditorLayoutMode = 6,
    None = 0,
    ReservedFutureFeatureBinding01 = 9,
  }
end

if not Enum.BindingContextMeta then
  Enum.BindingContextMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.BindingSet then
  Enum.BindingSet = {
    Account = 1,
    Character = 2,
    Current = 3,
    Default = 0,
  }
end

if not Enum.BindingSetMeta then
  Enum.BindingSetMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.BnetAccountFlag then
  Enum.BnetAccountFlag = {
    AccountQuestBitFixUp = 64,
    AchievementsToBi = 128,
    BattlePetTrainer = 1,
    CanBuyAhGameTimeTokens = 32768,
    CataLegendaryMountChecked = 262144,
    CataLegendaryMountObtained = 524288,
    DarkRealmLightCopy = 2048,
    Employee = 16,
    EmployeeFlagIsManual = 32,
    GdprErased = 1024,
    InvalidTransmogsFixUp = 256,
    InvalidTransmogsFixUp2 = 512,
    IsLegacy = 131072,
    LockedForExport = 16384,
    None = 0,
    PetAchievementFixUp = 65536,
    QuestLogFlagsFixUp = 4096,
    RafVeteranNotified = 2,
    TwitterHasTempSecret = 8,
    TwitterLinked = 4,
    WasSecured = 8192,
  }
end

if not Enum.BnetAccountFlagMeta then
  Enum.BnetAccountFlagMeta = {
    MaxValue = 524288,
    MinValue = 0,
    NumValues = 21,
  }
end

if not Enum.BonusStatIndex then
  Enum.BonusStatIndex = {
    Agility = 3,
    AgilityOrIntellect = 73,
    AgilityOrStrength = 72,
    AgilityOrStrengthOrIntellect = 71,
    ArcaneResistance = 56,
    AttackPower = 38,
    BlockRating = 15,
    BlockValueObsolete = 48,
    CombatRatingAvoidance = 63,
    CombatRatingLifesteal = 62,
    CombatRatingSpeed = 61,
    CombatRatingSturdiness = 64,
    CombatRatingUnused_0 = 58,
    CombatRatingUnused_10 = 68,
    CombatRatingUnused_11 = 69,
    CombatRatingUnused_12 = 70,
    CombatRatingUnused_2 = 59,
    CombatRatingUnused_27 = 66,
    CombatRatingUnused_3 = 60,
    CombatRatingUnused_7 = 65,
    CombatRatingUnused_9 = 67,
    Corruption = 22,
    CorruptionResistance = 23,
    CritMeleeRating = 19,
    CritRangedRating = 20,
    CritRating = 32,
    CritSpellRating = 21,
    CritTakenRangedRatingObsolete = 26,
    CritTakenRatingObsolete = 34,
    CritTakenSpellRatingObsolete = 27,
    DefenseSkillRating = 12,
    DodgeRating = 13,
    Endurance = 2,
    Energy = 8,
    ExpertiseRating = 37,
    ExtraArmor = 50,
    FireResistance = 51,
    Focus = 10,
    FrostResistance = 52,
    HasteMeleeRatingObsolete = 28,
    HasteRangedRatingObsolete = 29,
    HasteRating = 36,
    HasteSpellRatingObsolete = 30,
    Health = 1,
    HealthRegen = 46,
    HitMeleeRating = 16,
    HitRangedRating = 17,
    HitRating = 31,
    HitSpellRating = 18,
    HitTakenRatingObsolete = 33,
    HolyResistance = 53,
    Intellect = 5,
    Mana = 0,
    ManaRegenerationObsolete = 43,
    MasteryRating = 49,
    ModifiedCraftingStat_1 = 24,
    ModifiedCraftingStat_2 = 25,
    NatureResistance = 55,
    ParryRating = 14,
    ProfessionCraftingSpeed = 80,
    ProfessionDeftness = 78,
    ProfessionFinesse = 77,
    ProfessionIngenuity = 82,
    ProfessionInspiration = 75,
    ProfessionMulticraft = 81,
    ProfessionPerception = 79,
    ProfessionResourcefulness = 76,
    PvPPower = 57,
    Rage = 9,
    RangedAttackPower = 39,
    ResilienceRating = 35,
    ShadowResistance = 54,
    SpellDamageDone = 42,
    SpellHealingDone = 41,
    SpellPenetration = 47,
    SpellPower = 45,
    Spirit = 6,
    Stamina = 7,
    Strength = 4,
    StrengthOrIntellect = 74,
    Unused = 44,
    Versatility = 40,
    WeaponSkillRatingObsolete = 11,
  }
end

if not Enum.BonusStatIndexMeta then
  Enum.BonusStatIndexMeta = {
    MaxValue = 82,
    MinValue = 0,
    NumValues = 83,
  }
end

if not Enum.BrawlType then
  Enum.BrawlType = {
    Arena = 2,
    Battleground = 1,
    LFG = 3,
    None = 0,
    SoloRbg = 5,
    SoloShuffle = 4,
  }
end

if not Enum.BrawlTypeMeta then
  Enum.BrawlTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.BulkPurchaseResult then
  Enum.BulkPurchaseResult = {
    ResultFailed = 3,
    ResultInProgress = 1,
    ResultInsufficientFunds = 6,
    ResultOk = 0,
    ResultPartialSuccess = 2,
    ResultPurchaseTimeout = 7,
    ResultSystemDisabled = 5,
    ResultTooManyProducts = 4,
  }
end

if not Enum.BulkPurchaseResultMeta then
  Enum.BulkPurchaseResultMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.BulkPurchaseStatus then
  Enum.BulkPurchaseStatus = {
    Done = 3,
    Failed = 4,
    FetchEntitlements = 2,
    PlacingOrders = 0,
    PurchasesPending = 1,
  }
end

if not Enum.BulkPurchaseStatusMeta then
  Enum.BulkPurchaseStatusMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.BulkRefundResult then
  Enum.BulkRefundResult = {
    ResultFailed = 1,
    ResultInvalidRequest = 2,
    ResultOk = 0,
    ResultRefundWindowExpired = 3,
    ResultSystemDisabled = 4,
    ResultTimeout = 5,
  }
end

if not Enum.BulkRefundResultMeta then
  Enum.BulkRefundResultMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CachedRewardType then
  Enum.CachedRewardType = {
    Currency = 2,
    Item = 1,
    None = 0,
    Quest = 3,
  }
end

if not Enum.CachedRewardTypeMeta then
  Enum.CachedRewardTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CalendarCommandType then
  Enum.CalendarCommandType = {
    Complain = 10,
    Create = 0,
    GetCalendar = 7,
    GetEvent = 8,
    Invite = 1,
    ModeratorStatus = 6,
    Notes = 11,
    RemoveEvent = 4,
    RemoveInvite = 3,
    Rsvp = 2,
    Status = 5,
    UpdateEvent = 9,
  }
end

if not Enum.CalendarCommandTypeMeta then
  Enum.CalendarCommandTypeMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.CalendarErrorType then
  Enum.CalendarErrorType = {
    ArenaEventsExceeded = 27,
    CalendarDisabled = 25,
    CommunityEventsExceeded = 1,
    ComplaintAdded = 50,
    ComplaintDisabled = 31,
    ComplaintGm = 34,
    ComplaintLimit = 35,
    ComplaintNotFound = 36,
    ComplaintSameGuild = 33,
    ComplaintSelf = 32,
    CreatorNotFound = 46,
    DataAlreadySet = 24,
    DeleteCreatorFailed = 23,
    EventInvalid = 6,
    EventLocked = 22,
    EventPassed = 21,
    EventThrottled = 47,
    EventWrongServer = 37,
    EventsExceeded = 2,
    Ignored = 14,
    Internal = 49,
    InvalidClub = 45,
    InvalidDate = 17,
    InvalidDescription = 44,
    InvalidMaxSize = 16,
    InvalidNotes = 42,
    InvalidSignup = 39,
    InvalidTime = 18,
    InvalidTitle = 43,
    InviteThrottled = 48,
    InvitesExceeded = 15,
    ModeratorRestricted = 41,
    NameNotFound = 12,
    NeedsTitle = 20,
    NoCommunityInvites = 38,
    NoInvite = 30,
    NoInvites = 19,
    NoModerator = 40,
    NoPermission = 5,
    NotInCommunity = 10,
    NotInGuild = 9,
    NotInvited = 7,
    OtherInvitesExceeded = 4,
    RestrictedAccount = 26,
    RestrictedLevel = 28,
    SelfInvitesExceeded = 3,
    Squelched = 29,
    Success = 0,
    TargetAlreadyInvited = 11,
    UnknownError = 8,
    WrongFaction = 13,
  }
end

if not Enum.CalendarErrorTypeMeta then
  Enum.CalendarErrorTypeMeta = {
    MaxValue = 50,
    MinValue = 0,
    NumValues = 51,
  }
end

if not Enum.CalendarEventBits then
  Enum.CalendarEventBits = {
    ArenaDeprecated = 256,
    AutoApprove = 32,
    CantComplain = 3788,
    CommunityAnnouncement = 64,
    CommunitySignup = 1024,
    CommunityWide = 3136,
    GuildDeprecated = 2,
    GuildSignup = 2048,
    Holiday = 8,
    Locked = 16,
    Player = 1,
    PlayerCreated = 3395,
    RaidLockout = 128,
    RaidResetDeprecated = 512,
    System = 4,
  }
end

if not Enum.CalendarEventBitsMeta then
  Enum.CalendarEventBitsMeta = {
    MaxValue = 3788,
    MinValue = 1,
    NumValues = 15,
  }
end

if not Enum.CalendarEventRepeatOptions then
  Enum.CalendarEventRepeatOptions = {
    Biweekly = 2,
    Monthly = 3,
    Never = 0,
    Weekly = 1,
  }
end

if not Enum.CalendarEventRepeatOptionsMeta then
  Enum.CalendarEventRepeatOptionsMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CalendarEventTypeMeta then
  Enum.CalendarEventTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CalendarFilterFlags then
  Enum.CalendarFilterFlags = {
    Battleground = 4,
    Darkmoon = 2,
    RaidLockout = 8,
    RaidReset = 16,
    WeeklyHoliday = 1,
  }
end

if not Enum.CalendarFilterFlagsMeta then
  Enum.CalendarFilterFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.CalendarGetEventType then
  Enum.CalendarGetEventType = {
    Add = 1,
    Copy = 2,
    Get = 0,
  }
end

if not Enum.CalendarGetEventTypeMeta then
  Enum.CalendarGetEventTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CalendarHolidayFilterType then
  Enum.CalendarHolidayFilterType = {
    Battleground = 2,
    Darkmoon = 1,
    Weekly = 0,
  }
end

if not Enum.CalendarHolidayFilterTypeMeta then
  Enum.CalendarHolidayFilterTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CalendarInviteBits then
  Enum.CalendarInviteBits = {
    Creator = 4,
    Moderator = 2,
    None = 0,
    PendingInvite = 1,
    Signup = 8,
  }
end

if not Enum.CalendarInviteBitsMeta then
  Enum.CalendarInviteBitsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CalendarInviteSortType then
  Enum.CalendarInviteSortType = {
    Class = 2,
    Level = 1,
    Name = 0,
    Notes = 5,
    Party = 4,
    Status = 3,
  }
end

if not Enum.CalendarInviteSortTypeMeta then
  Enum.CalendarInviteSortTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CalendarInviteType then
  Enum.CalendarInviteType = {
    Normal = 0,
    Signup = 1,
  }
end

if not Enum.CalendarInviteTypeMeta then
  Enum.CalendarInviteTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CalendarModeratorStatus then
  Enum.CalendarModeratorStatus = {
    Creator = 2,
    Moderator = 1,
    None = 0,
  }
end

if not Enum.CalendarModeratorStatusMeta then
  Enum.CalendarModeratorStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CalendarStatusMeta then
  Enum.CalendarStatusMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.CalendarTexturesType then
  Enum.CalendarTexturesType = {
    Dungeons = 0,
    Raid = 1,
  }
end

if not Enum.CalendarTexturesTypeMeta then
  Enum.CalendarTexturesTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CalendarType then
  Enum.CalendarType = {
    Community = 1,
    Holiday = 4,
    HolidayBattleground = 7,
    HolidayDarkmoon = 6,
    HolidayWeekly = 5,
    Player = 0,
    RaidLockout = 2,
    RaidResetDeprecated = 3,
  }
end

if not Enum.CalendarTypeMeta then
  Enum.CalendarTypeMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.CalendarWebActionType then
  Enum.CalendarWebActionType = {
    Accept = 0,
    Decline = 1,
    Remove = 2,
    ReportSpam = 3,
    Signup = 4,
    Tentative = 5,
    TentativeSignup = 6,
  }
end

if not Enum.CalendarWebActionTypeMeta then
  Enum.CalendarWebActionTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.CallingStates then
  Enum.CallingStates = {
    QuestActive = 1,
    QuestCompleted = 2,
    QuestOffer = 0,
  }
end

if not Enum.CallingStatesMeta then
  Enum.CallingStatesMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CameraModeAspectRatioMeta then
  Enum.CameraModeAspectRatioMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CampaignState then
  Enum.CampaignState = {
    Complete = 1,
    InProgress = 2,
    Invalid = 0,
    Stalled = 3,
  }
end

if not Enum.CampaignStateMeta then
  Enum.CampaignStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CanRedeemTokenForBalanceResult then
  Enum.CanRedeemTokenForBalanceResult = {
    FailureCap = 1,
    Ok = 0,
  }
end

if not Enum.CanRedeemTokenForBalanceResultMeta then
  Enum.CanRedeemTokenForBalanceResultMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CaptureBarWidgetFillDirectionType then
  Enum.CaptureBarWidgetFillDirectionType = {
    LeftToRight = 1,
    RightToLeft = 0,
  }
end

if not Enum.CaptureBarWidgetFillDirectionTypeMeta then
  Enum.CaptureBarWidgetFillDirectionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.Causeofdeath then
  Enum.Causeofdeath = {
    Creature = 3,
    Drowning = 5,
    Falling = 4,
    Fatigue = 6,
    Fire = 9,
    Lava = 8,
    None = 0,
    PlayerDuel = 2,
    PlayerPvP = 1,
    Slime = 7,
  }
end

if not Enum.CauseofdeathFlags then
  Enum.CauseofdeathFlags = {
    CreatureNameNeeded = 2,
    NoneNeeded = 0,
    PlayerNameNeeded = 1,
    ZoneNameNeeded = 4,
  }
end

if not Enum.CauseofdeathFlagsMeta then
  Enum.CauseofdeathFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CauseofdeathMeta then
  Enum.CauseofdeathMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.ChallengeModeHistoryFlags then
  Enum.ChallengeModeHistoryFlags = {
    ConfirmedLeaver = 1,
    None = 0,
  }
end

if not Enum.ChallengeModeHistoryFlagsMeta then
  Enum.ChallengeModeHistoryFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ChallengeModeHistoryResult then
  Enum.ChallengeModeHistoryResult = {
    Leaver = 1,
    Successful = 0,
  }
end

if not Enum.ChallengeModeHistoryResultMeta then
  Enum.ChallengeModeHistoryResultMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ChallengeModeHistoryStatus then
  Enum.ChallengeModeHistoryStatus = {
    Leaver = 1,
    Normal = 0,
  }
end

if not Enum.ChallengeModeHistoryStatusMeta then
  Enum.ChallengeModeHistoryStatusMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ChannelPlayerFlags then
  Enum.ChannelPlayerFlags = {
    ChannelPlayerHidden = 8,
    ChannelPlayerModerator = 2,
    ChannelPlayerNone = 0,
    ChannelPlayerOwner = 1,
    ChannelPlayerTextAllow = 4,
  }
end

if not Enum.ChannelPlayerFlagsMeta then
  Enum.ChannelPlayerFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CharCustomizationType then
  Enum.CharCustomizationType = {
    CustomOptionFacewear = 7,
    CustomOptionHorn = 6,
    CustomOptionTattoo = 5,
    CustomOptionTattooColor = 8,
    Face = 1,
    FacialHair = 4,
    Hair = 2,
    HairColor = 3,
    Skin = 0,
  }
end

if not Enum.CharCustomizationTypeMeta then
  Enum.CharCustomizationTypeMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.CharacterServiceInfoFlag then
  Enum.CharacterServiceInfoFlag = {
    AllowMaxLevelBoost = 2,
    RestrictToRecommendedSpecs = 1,
  }
end

if not Enum.CharacterServiceInfoFlagMeta then
  Enum.CharacterServiceInfoFlagMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.ChatChannelRuleset then
  Enum.ChatChannelRuleset = {
    ChromieTimeBuringCrusade = 4,
    ChromieTimeCataclysm = 3,
    ChromieTimeLegion = 8,
    ChromieTimeMists = 6,
    ChromieTimeWoD = 7,
    ChromieTimeWrath = 5,
    Disabled = 2,
    Mentor = 1,
    None = 0,
  }
end

if not Enum.ChatChannelRulesetMeta then
  Enum.ChatChannelRulesetMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.ChatChannelTypeMeta then
  Enum.ChatChannelTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ChatMessagingLockdownReason then
  Enum.ChatMessagingLockdownReason = {
    ActiveEncounter = 0,
    ActiveMythicKeystoneOrChallengeMode = 1,
    ActivePvPMatch = 2,
  }
end

if not Enum.ChatMessagingLockdownReasonMeta then
  Enum.ChatMessagingLockdownReasonMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ChatToxityFilterOptOut then
  Enum.ChatToxityFilterOptOut = {
    ExcludeFilterAll = 4294967295,
    ExcludeFilterFriend = 1,
    ExcludeFilterGuild = 2,
    FilterAll = 0,
  }
end

if not Enum.ChatToxityFilterOptOutMeta then
  Enum.ChatToxityFilterOptOutMeta = {
    MaxValue = -1,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ChatWhisperTargetStatus then
  Enum.ChatWhisperTargetStatus = {
    CanWhisper = 0,
    CanWhisperGuild = 1,
    Offline = 2,
    WrongFaction = 3,
  }
end

if not Enum.ChatWhisperTargetStatusMeta then
  Enum.ChatWhisperTargetStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ChrCustomizationCategoryFlag then
  Enum.ChrCustomizationCategoryFlag = {
    Subcategory = 2,
    UndressModel = 1,
  }
end

if not Enum.ChrCustomizationCategoryFlagMeta then
  Enum.ChrCustomizationCategoryFlagMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.ChrCustomizationOptionType then
  Enum.ChrCustomizationOptionType = {
    Checkbox = 1,
    Dropdown = 0,
    Slider = 2,
  }
end

if not Enum.ChrCustomizationOptionTypeMeta then
  Enum.ChrCustomizationOptionTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ChrModelFeatureFlags then
  Enum.ChrModelFeatureFlags = {
    Deprecated0 = 8,
    Forms = 2,
    HunterPets = 32,
    Identity = 4,
    Mounts = 16,
    None = 0,
    Players = 64,
    Summons = 1,
  }
end

if not Enum.ChrModelFeatureFlagsMeta then
  Enum.ChrModelFeatureFlagsMeta = {
    MaxValue = 64,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.ChrRacesAllianceType then
  Enum.ChrRacesAllianceType = {
    Alliance = 0,
    Horde = 1,
    NeutralOrNpc = 2,
  }
end

if not Enum.ChrRacesAllianceTypeMeta then
  Enum.ChrRacesAllianceTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CinematicType then
  Enum.CinematicType = {
    GameCinematicSequence = 3,
    GameClientScene = 2,
    GameMovie = 1,
    GlueMovie = 0,
  }
end

if not Enum.CinematicTypeMeta then
  Enum.CinematicTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ClickBindingInteraction then
  Enum.ClickBindingInteraction = {
    OpenContextMenu = 2,
    Target = 1,
  }
end

if not Enum.ClickBindingInteractionMeta then
  Enum.ClickBindingInteractionMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.ClickBindingTypeMeta then
  Enum.ClickBindingTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ClientPlatformType then
  Enum.ClientPlatformType = {
    Macintosh = 1,
    Windows = 0,
  }
end

if not Enum.ClientPlatformTypeMeta then
  Enum.ClientPlatformTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ClientDebugAISpellReadyStatus then
  Enum.ClientDebugAISpellReadyStatus = {
    CasterAuraEffectRestrictions = 29,
    CasterAuraSpellRestrictions = 21,
    CasterAuraStateRestrictions = 19,
    Charmed = 5,
    CheckArea = 26,
    CombatCondition = 27,
    Confused = 11,
    Cooldown = 1,
    DungeonEncounter = 23,
    DungeonEncounterID = 25,
    Enabled = 3,
    EquippedItems = 13,
    ExcludeCasterAuraEffect = 30,
    ExcludeCasterAuraStateRestrictions = 20,
    ExcludeCasterSpell = 22,
    FactionReaction = 24,
    Fleeing = 10,
    GeneralAuraRestrictions = 31,
    HasTargets = 28,
    InitialDelay = 2,
    NoActions = 9,
    Pacified = 8,
    Paused = 4,
    Peaceful = 18,
    PowerCost = 15,
    Ready = 0,
    ReadyToCast = 14,
    ShapeShiftRules = 16,
    Silenced = 7,
    StealthRules = 17,
    Stunned = 6,
    Tokenless = 12,
  }
end

if not Enum.ClientDebugAISpellReadyStatusMeta then
  Enum.ClientDebugAISpellReadyStatusMeta = {
    MaxValue = 31,
    MinValue = 0,
    NumValues = 32,
  }
end

if not Enum.ClientSceneType then
  Enum.ClientSceneType = {
    DefaultSceneType = 0,
    MinigameSceneType = 1,
  }
end

if not Enum.ClientSceneTypeMeta then
  Enum.ClientSceneTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ClientSettingsConfigFlag then
  Enum.ClientSettingsConfigFlag = {
    ClientSettingsConfigBeta = 64,
    ClientSettingsConfigBetaRetail = 128,
    ClientSettingsConfigDebug = 1,
    ClientSettingsConfigGm = 8,
    ClientSettingsConfigInternal = 2,
    ClientSettingsConfigPerf = 4,
    ClientSettingsConfigRetail = 256,
    ClientSettingsConfigTest = 16,
    ClientSettingsConfigTestRetail = 32,
  }
end

if not Enum.ClientSettingsConfigFlagMeta then
  Enum.ClientSettingsConfigFlagMeta = {
    MaxValue = 256,
    MinValue = 1,
    NumValues = 9,
  }
end

if not Enum.ClubActionTypeMeta then
  Enum.ClubActionTypeMeta = {
    MaxValue = 26,
    MinValue = 0,
    NumValues = 27,
  }
end

if not Enum.ClubErrorType then
  Enum.ClubErrorType = {
    ErrorClubAlreadyMember = 19,
    ErrorClubBanAlreadyExists = 35,
    ErrorClubBanCountAtMax = 36,
    ErrorClubDoesntAllowCrossFaction = 40,
    ErrorClubEditHasCrossFactionMembers = 41,
    ErrorClubFull = 16,
    ErrorClubInsufficientPrivileges = 24,
    ErrorClubInvalidRoleID = 23,
    ErrorClubInvitationAlreadyExists = 22,
    ErrorClubMemberHasRequiredRole = 31,
    ErrorClubNoClub = 17,
    ErrorClubNoSuchInvitation = 21,
    ErrorClubNoSuchMember = 20,
    ErrorClubNotMember = 18,
    ErrorClubReceivedInvitationCountAtMax = 33,
    ErrorClubSentInvitationCountAtMax = 32,
    ErrorClubStreamCountAtMax = 30,
    ErrorClubStreamCountAtMin = 29,
    ErrorClubStreamInvalidName = 28,
    ErrorClubStreamNoStream = 27,
    ErrorClubTargetIsBanned = 34,
    ErrorClubTicketCountAtMax = 37,
    ErrorClubTicketHasConsumedAllowedRedeemCount = 39,
    ErrorClubTicketNoSuchTicket = 38,
    ErrorClubTooManyClubsJoined = 25,
    ErrorClubVoiceFull = 26,
    ErrorCommunitiesBadTarget = 4,
    ErrorCommunitiesChatMute = 15,
    ErrorCommunitiesGuild = 8,
    ErrorCommunitiesIgnored = 7,
    ErrorCommunitiesMissingShortName = 11,
    ErrorCommunitiesNeutralFaction = 2,
    ErrorCommunitiesNone = 0,
    ErrorCommunitiesProfanity = 12,
    ErrorCommunitiesRestricted = 6,
    ErrorCommunitiesTrial = 13,
    ErrorCommunitiesUnknown = 1,
    ErrorCommunitiesUnknownRealm = 3,
    ErrorCommunitiesUnknownTicket = 10,
    ErrorCommunitiesVeteranTrial = 14,
    ErrorCommunitiesWrongFaction = 5,
    ErrorCommunitiesWrongRegion = 9,
  }
end

if not Enum.ClubErrorTypeMeta then
  Enum.ClubErrorTypeMeta = {
    MaxValue = 41,
    MinValue = 0,
    NumValues = 42,
  }
end

if not Enum.ClubFieldType then
  Enum.ClubFieldType = {
    ClubBroadcast = 3,
    ClubDescription = 2,
    ClubName = 0,
    ClubShortName = 1,
    ClubStreamName = 4,
    ClubStreamSubject = 5,
    NumTypes = 6,
  }
end

if not Enum.ClubFieldTypeMeta then
  Enum.ClubFieldTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.ClubFinderApplicationUpdateType then
  Enum.ClubFinderApplicationUpdateType = {
    AcceptInvite = 1,
    Cancel = 3,
    DeclineInvite = 2,
    None = 0,
  }
end

if not Enum.ClubFinderApplicationUpdateTypeMeta then
  Enum.ClubFinderApplicationUpdateTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ClubFinderClubPostingStatusFlags then
  Enum.ClubFinderClubPostingStatusFlags = {
    Banned = 5,
    FakePost = 6,
    ForceDescriptionChange = 2,
    ForceNameChange = 3,
    NeedsCacheUpdate = 1,
    None = 0,
    PendingDelete = 7,
    PostDelisted = 8,
    UnderReview = 4,
  }
end

if not Enum.ClubFinderClubPostingStatusFlagsMeta then
  Enum.ClubFinderClubPostingStatusFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.ClubFinderDisableReason then
  Enum.ClubFinderDisableReason = {
    Muted = 0,
    Silenced = 1,
    VeteranTrial = 2,
  }
end

if not Enum.ClubFinderDisableReasonMeta then
  Enum.ClubFinderDisableReasonMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ClubFinderPostingReportType then
  Enum.ClubFinderPostingReportType = {
    ApplicantsName = 3,
    ClubName = 1,
    JoinNote = 4,
    PostersName = 0,
    PostingDescription = 2,
  }
end

if not Enum.ClubFinderPostingReportTypeMeta then
  Enum.ClubFinderPostingReportTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ClubFinderRequestTypeMeta then
  Enum.ClubFinderRequestTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ClubFinderSettingFlagsMeta then
  Enum.ClubFinderSettingFlagsMeta = {
    MaxValue = 25,
    MinValue = 0,
    NumValues = 26,
  }
end

if not Enum.ClubInvitationCandidateStatus then
  Enum.ClubInvitationCandidateStatus = {
    AlreadyMember = 2,
    Available = 0,
    InvitePending = 1,
  }
end

if not Enum.ClubInvitationCandidateStatusMeta then
  Enum.ClubInvitationCandidateStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ClubMemberPresenceMeta then
  Enum.ClubMemberPresenceMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.ClubRemovedReasonMeta then
  Enum.ClubRemovedReasonMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ClubRestrictionReason then
  Enum.ClubRestrictionReason = {
    None = 0,
    Unavailable = 1,
  }
end

if not Enum.ClubRestrictionReasonMeta then
  Enum.ClubRestrictionReasonMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ClubRoleIdentifierMeta then
  Enum.ClubRoleIdentifierMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.ClubStreamNotificationFilter then
  Enum.ClubStreamNotificationFilter = {
    All = 2,
    Mention = 1,
    None = 0,
  }
end

if not Enum.ClubStreamNotificationFilterMeta then
  Enum.ClubStreamNotificationFilterMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ClubStreamTypeMeta then
  Enum.ClubStreamTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ClubTypeMeta then
  Enum.ClubTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ColorOverrideMeta then
  Enum.ColorOverrideMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.CombatAudioAlertCastState then
  Enum.CombatAudioAlertCastState = {
    Off = 0,
    OnCastEnd = 2,
    OnCastStart = 1,
  }
end

if not Enum.CombatAudioAlertCastStateMeta then
  Enum.CombatAudioAlertCastStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CombatAudioAlertCategory then
  Enum.CombatAudioAlertCategory = {
    General = 0,
    PartyHealth = 7,
    PlayerCast = 3,
    PlayerDebuffs = 8,
    PlayerHealth = 1,
    PlayerResource1 = 5,
    PlayerResource2 = 6,
    TargetCast = 4,
    TargetHealth = 2,
  }
end

if not Enum.CombatAudioAlertCategoryMeta then
  Enum.CombatAudioAlertCategoryMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.CombatAudioAlertDebuffSelfAlertValues then
  Enum.CombatAudioAlertDebuffSelfAlertValues = {
    DispelType = 1,
    Off = 0,
  }
end

if not Enum.CombatAudioAlertPartyPercentValues then
  Enum.CombatAudioAlertPartyPercentValues = {
    Off = 0,
    Under100Percent = 1,
    Under10Percent = 10,
    Under20Percent = 9,
    Under30Percent = 8,
    Under40Percent = 7,
    Under50Percent = 6,
    Under60Percent = 5,
    Under70Percent = 4,
    Under80Percent = 3,
    Under90Percent = 2,
  }
end

if not Enum.CombatAudioAlertPercentValues then
  Enum.CombatAudioAlertPercentValues = {
    Every10Percent = 1,
    Every20Percent = 2,
    Every30Percent = 3,
    Every40Percent = 4,
    Every50Percent = 5,
    Off = 0,
  }
end

if not Enum.CombatAudioAlertPercentValuesMeta then
  Enum.CombatAudioAlertPercentValuesMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CombatAudioAlertPlayerCastFormatValues then
  Enum.CombatAudioAlertPlayerCastFormatValues = {
    Cast = 3,
    CastSpellname = 1,
    Casting = 2,
    CastingSpellname = 0,
    Spellname = 4,
  }
end

if not Enum.CombatAudioAlertPlayerDebuffFormatValues then
  Enum.CombatAudioAlertPlayerDebuffFormatValues = {
    Full = 0,
    NoSpellname = 1,
  }
end

if not Enum.CombatAudioAlertPlayerHealthFormatValues then
  Enum.CombatAudioAlertPlayerHealthFormatValues = {
    HealthFull = 0,
    HealthNoPercent = 1,
    HealthNoPercentDiv10 = 2,
    NoHealthFull = 3,
    NoHealthNoPercent = 4,
    NoHealthNoPercentDiv10 = 5,
  }
end

if not Enum.CombatAudioAlertPlayerResourceFormatValues then
  Enum.CombatAudioAlertPlayerResourceFormatValues = {
    NoResourceFull = 3,
    NoResourceNoPercent = 4,
    NoResourceNoPercentDiv10 = 5,
    ResourceFull = 0,
    ResourceNoPercent = 1,
    ResourceNoPercentDiv10 = 2,
  }
end

if not Enum.CombatAudioAlertPlayerResourceFormatValuesMeta then
  Enum.CombatAudioAlertPlayerResourceFormatValuesMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CombatAudioAlertSayIfTargetedType then
  Enum.CombatAudioAlertSayIfTargetedType = {
    Aggro = 1,
    None = 0,
    Targeted = 2,
    TargetedBy = 3,
  }
end

if not Enum.CombatAudioAlertSpecSetting then
  Enum.CombatAudioAlertSpecSetting = {
    Resource1Format = 1,
    Resource1Percent = 0,
    Resource2Format = 3,
    Resource2Percent = 2,
    SayIfTargeted = 4,
  }
end

if not Enum.CombatAudioAlertSpecSettingMeta then
  Enum.CombatAudioAlertSpecSettingMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CombatAudioAlertTargetCastFormatValues then
  Enum.CombatAudioAlertTargetCastFormatValues = {
    Cast = 5,
    CastSpellname = 3,
    Casting = 4,
    CastingSpellname = 2,
    Spellname = 6,
    TargetCastSpellname = 1,
    TargetCastingSpellname = 0,
  }
end

if not Enum.CombatAudioAlertTargetDeathBehavior then
  Enum.CombatAudioAlertTargetDeathBehavior = {
    Default = 0,
    SayTargetDead = 1,
  }
end

if not Enum.CombatAudioAlertTargetHealthFormatValues then
  Enum.CombatAudioAlertTargetHealthFormatValues = {
    HealthFull = 3,
    HealthNoPercent = 4,
    HealthNoPercentDiv10 = 5,
    NoHealthFull = 0,
    NoHealthNoPercent = 1,
    NoHealthNoPercentDiv10 = 2,
    TargetFull = 6,
    TargetNoPercent = 7,
    TargetNoPercentDiv10 = 8,
  }
end

if not Enum.CombatAudioAlertThrottleMeta then
  Enum.CombatAudioAlertThrottleMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.CombatAudioAlertTypeMeta then
  Enum.CombatAudioAlertTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CombatAudioAlertUnitMeta then
  Enum.CombatAudioAlertUnitMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CombatLogMessageOrder then
  Enum.CombatLogMessageOrder = {
    Newest = 0,
    Oldest = 1,
  }
end

if not Enum.CombatLogMessageOrderMeta then
  Enum.CombatLogMessageOrderMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CombatLogObject then
  Enum.CombatLogObject = {
    AffiliationMine = 1,
    AffiliationOutsider = 8,
    AffiliationParty = 2,
    AffiliationRaid = 4,
    ControlNpc = 512,
    ControlPlayer = 256,
    Empty = 0,
    Focus = 131072,
    Mainassist = 524288,
    Maintank = 262144,
    None = 2147483648,
    ReactionFriendly = 16,
    ReactionHostile = 64,
    ReactionNeutral = 32,
    Target = 65536,
    TypeGuardian = 8192,
    TypeNpc = 2048,
    TypeObject = 16384,
    TypePet = 4096,
    TypePlayer = 1024,
  }
end

if not Enum.CombatLogObjectMeta then
  Enum.CombatLogObjectMeta = {
    MaxValue = -2147483648,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.CombatLogObjectTarget then
  Enum.CombatLogObjectTarget = {
    RaidNone = 2147483648,
    Raidtarget1 = 1,
    Raidtarget2 = 2,
    Raidtarget3 = 4,
    Raidtarget4 = 8,
    Raidtarget5 = 16,
    Raidtarget6 = 32,
    Raidtarget7 = 64,
    Raidtarget8 = 128,
  }
end

if not Enum.CombatLogObjectTargetMeta then
  Enum.CombatLogObjectTargetMeta = {
    MaxValue = -2147483648,
    MinValue = 1,
    NumValues = 9,
  }
end

if not Enum.CombinedQuestLogStatus then
  Enum.CombinedQuestLogStatus = {
    Available = 0,
    Complete = 1,
    CompleteDaily = 2,
    CompleteGameReset = 6,
    CompleteMonthly = 4,
    CompleteWeekly = 3,
    CompleteYearly = 5,
    Reset = 7,
  }
end

if not Enum.CombinedQuestLogStatusMeta then
  Enum.CombinedQuestLogStatusMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.CombinedQuestStatus then
  Enum.CombinedQuestStatus = {
    Completed = 1,
    Invalid = 0,
    NotCompleted = 2,
  }
end

if not Enum.CombinedQuestStatusMeta then
  Enum.CombinedQuestStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CommunicationMode then
  Enum.CommunicationMode = {
    OpenMic = 1,
    PushToTalk = 0,
  }
end

if not Enum.CommunicationModeMeta then
  Enum.CommunicationModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CompanionConfigSlotTypesMeta then
  Enum.CompanionConfigSlotTypesMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CompanionRoleType then
  Enum.CompanionRoleType = {
    Dps = 0,
    Heal = 1,
    Tank = 2,
  }
end

if not Enum.CompanionRoleTypeMeta then
  Enum.CompanionRoleTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CompressionLevel then
  Enum.CompressionLevel = {
    Default = 0,
    OptimizeForSize = 2,
    OptimizeForSpeed = 1,
  }
end

if not Enum.CompressionLevelMeta then
  Enum.CompressionLevelMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CompressionMethod then
  Enum.CompressionMethod = {
    Deflate = 0,
    Gzip = 2,
    Zlib = 1,
  }
end

if not Enum.CompressionMethodMeta then
  Enum.CompressionMethodMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ConfirmationPromptUIType then
  Enum.ConfirmationPromptUIType = {
    BonusRoll = 1,
    SimpleWarning = 2,
    SimpleWarningAlert = 4,
    StaticText = 0,
    StaticTextAlert = 3,
  }
end

if not Enum.ConfirmationPromptUITypeMeta then
  Enum.ConfirmationPromptUITypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ConquestProgressBarDisplayType then
  Enum.ConquestProgressBarDisplayType = {
    AdditionalChest = 1,
    FirstChest = 0,
    Seasonal = 2,
  }
end

if not Enum.ConquestProgressBarDisplayTypeMeta then
  Enum.ConquestProgressBarDisplayTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ConsoleCategory then
  Enum.ConsoleCategory = {
    Combat = 3,
    Console = 2,
    Debug = 0,
    Default = 5,
    Game = 4,
    Gm = 8,
    Graphics = 1,
    Net = 6,
    None = 10,
    Reveal = 9,
    Sound = 7,
  }
end

if not Enum.ConsoleCategoryMeta then
  Enum.ConsoleCategoryMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.ConsoleColorType then
  Enum.ConsoleColorType = {
    AdminColor = 6,
    BackgroundColor = 8,
    ClickbufferColor = 9,
    DefaultColor = 0,
    DefaultGreen = 11,
    EchoColor = 2,
    ErrorColor = 3,
    GlobalColor = 5,
    HighlightColor = 7,
    InputColor = 1,
    PrivateColor = 10,
    WarningColor = 4,
  }
end

if not Enum.ConsoleColorTypeMeta then
  Enum.ConsoleColorTypeMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.ConsoleCommandType then
  Enum.ConsoleCommandType = {
    Command = 1,
    Cvar = 0,
    Macro = 2,
    Script = 3,
  }
end

if not Enum.ConsoleCommandTypeMeta then
  Enum.ConsoleCommandTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ContentTrackingError then
  Enum.ContentTrackingError = {
    AlreadyTracked = 2,
    MaxTracked = 1,
    Untrackable = 0,
  }
end

if not Enum.ContentTrackingErrorMeta then
  Enum.ContentTrackingErrorMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ContentTrackingResult then
  Enum.ContentTrackingResult = {
    DataPending = 1,
    Failure = 2,
    Success = 0,
  }
end

if not Enum.ContentTrackingResultMeta then
  Enum.ContentTrackingResultMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ContentTrackingStopType then
  Enum.ContentTrackingStopType = {
    Collected = 1,
    Invalidated = 0,
    Manual = 2,
  }
end

if not Enum.ContentTrackingStopTypeMeta then
  Enum.ContentTrackingStopTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ContentTrackingTargetType then
  Enum.ContentTrackingTargetType = {
    Achievement = 2,
    JournalEncounter = 0,
    Profession = 3,
    Quest = 4,
    Vendor = 1,
  }
end

if not Enum.ContentTrackingTargetTypeMeta then
  Enum.ContentTrackingTargetTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ContentTrackingTypeMeta then
  Enum.ContentTrackingTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ContributionAppearanceFlags then
  Enum.ContributionAppearanceFlags = {
    TooltipUseTimeRemaining = 0,
  }
end

if not Enum.ContributionAppearanceFlagsMeta then
  Enum.ContributionAppearanceFlagsMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.ContributionResult then
  Enum.ContributionResult = {
    FailedConditionCheck = 5,
    IncorrectState = 2,
    InternalError = 7,
    InvalidID = 3,
    MustBeNearNpc = 1,
    QuestDataMissing = 4,
    Success = 0,
    UnableToCompleteTurnIn = 6,
  }
end

if not Enum.ContributionResultMeta then
  Enum.ContributionResultMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.ContributionState then
  Enum.ContributionState = {
    Active = 2,
    Building = 1,
    Destroyed = 4,
    None = 0,
    UnderAttack = 3,
  }
end

if not Enum.ContributionStateMeta then
  Enum.ContributionStateMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CooldownSetLinkedSpellFlags then
  Enum.CooldownSetLinkedSpellFlags = {
    UseAsTooltip = 1,
  }
end

if not Enum.CooldownSetLinkedSpellFlagsMeta then
  Enum.CooldownSetLinkedSpellFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.CooldownSetSpellFlags then
  Enum.CooldownSetSpellFlags = {
    HideAura = 1,
    HideByDefault = 2,
  }
end

if not Enum.CooldownSetSpellFlagsMeta then
  Enum.CooldownSetSpellFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.CooldownViewerAddAlertStatus then
  Enum.CooldownViewerAddAlertStatus = {
    DuplicateAlert = 3,
    InvalidAlertType = 1,
    InvalidEventType = 2,
    Success = 0,
  }
end

if not Enum.CooldownViewerAddAlertStatusMeta then
  Enum.CooldownViewerAddAlertStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CooldownViewerAlertEventType then
  Enum.CooldownViewerAlertEventType = {
    Available = 1,
    ChargeGained = 4,
    OnAuraApplied = 5,
    OnAuraRemoved = 6,
    OnCooldown = 3,
    PandemicTime = 2,
  }
end

if not Enum.CooldownViewerAlertEventTypeMeta then
  Enum.CooldownViewerAlertEventTypeMeta = {
    MaxValue = 6,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.CooldownViewerAlertType then
  Enum.CooldownViewerAlertType = {
    Sound = 1,
    Visual = 2,
  }
end

if not Enum.CooldownViewerAlertTypeMeta then
  Enum.CooldownViewerAlertTypeMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.CooldownViewerBarContentMeta then
  Enum.CooldownViewerBarContentMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CooldownViewerCategoryMeta then
  Enum.CooldownViewerCategoryMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CooldownViewerIconDirectionMeta then
  Enum.CooldownViewerIconDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CooldownViewerOrientationMeta then
  Enum.CooldownViewerOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CooldownViewerVisibleSettingMeta then
  Enum.CooldownViewerVisibleSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CornerstonePurchaseMode then
  Enum.CornerstonePurchaseMode = {
    Basic = 0,
    Import = 1,
    Move = 2,
  }
end

if not Enum.CornerstonePurchaseModeMeta then
  Enum.CornerstonePurchaseModeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CovenantAbilityType then
  Enum.CovenantAbilityType = {
    Class = 0,
    Signature = 1,
    Soulbind = 2,
  }
end

if not Enum.CovenantAbilityTypeMeta then
  Enum.CovenantAbilityTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CovenantSkill then
  Enum.CovenantSkill = {
    Kyrian = 2730,
    Necrolord = 2733,
    NightFae = 2732,
    Venthyr = 2731,
  }
end

if not Enum.CovenantSkillMeta then
  Enum.CovenantSkillMeta = {
    MaxValue = 2733,
    MinValue = 2730,
    NumValues = 4,
  }
end

if not Enum.CovenantType then
  Enum.CovenantType = {
    Kyrian = 1,
    Necrolord = 4,
    NightFae = 3,
    None = 0,
    Venthyr = 2,
  }
end

if not Enum.CovenantTypeMeta then
  Enum.CovenantTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CraftingOrderCustomerCategoryType then
  Enum.CraftingOrderCustomerCategoryType = {
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
  }
end

if not Enum.CraftingOrderCustomerCategoryTypeMeta then
  Enum.CraftingOrderCustomerCategoryTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CraftingOrderDuration then
  Enum.CraftingOrderDuration = {
    Long = 2,
    Medium = 1,
    Short = 0,
  }
end

if not Enum.CraftingOrderDurationMeta then
  Enum.CraftingOrderDurationMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CraftingOrderFlags then
  Enum.CraftingOrderFlags = {
    HasAllReagents = 8,
    HasNoneReagents = 2,
    HasSomeReagents = 4,
    IsFulfillable = 16,
    IsRecraft = 1,
    None = 0,
  }
end

if not Enum.CraftingOrderFlagsMeta then
  Enum.CraftingOrderFlagsMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CraftingOrderItemFlags then
  Enum.CraftingOrderItemFlags = {
    HasEnchantmentData = 2,
    None = 0,
    NpcProvided = 1,
  }
end

if not Enum.CraftingOrderItemFlagsMeta then
  Enum.CraftingOrderItemFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CraftingOrderItemType then
  Enum.CraftingOrderItemType = {
    CraftedResult = 2,
    Currency = 5,
    Deprecated = 4,
    Item = 0,
    Recraft = 1,
    RemoveReagent = 3,
  }
end

if not Enum.CraftingOrderItemTypeMeta then
  Enum.CraftingOrderItemTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.CraftingOrderReagentSource then
  Enum.CraftingOrderReagentSource = {
    Any = 0,
    Crafter = 2,
    Customer = 1,
    None = 3,
  }
end

if not Enum.CraftingOrderReagentSourceMeta then
  Enum.CraftingOrderReagentSourceMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CraftingOrderReagentsType then
  Enum.CraftingOrderReagentsType = {
    All = 0,
    None = 2,
    Some = 1,
  }
end

if not Enum.CraftingOrderReagentsTypeMeta then
  Enum.CraftingOrderReagentsTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CraftingOrderResult then
  Enum.CraftingOrderResult = {
    Aborted = 1,
    AlreadyClaimed = 2,
    AlreadyCrafted = 3,
    CannotBeOrdered = 4,
    CannotCancel = 5,
    CannotClaim = 6,
    CannotClaimOwnOrder = 7,
    CannotCraft = 8,
    CannotCreate = 9,
    CannotFulfill = 10,
    CannotRecraft = 11,
    CannotReject = 12,
    CannotRelease = 13,
    CrafterIsIgnored = 14,
    DatabaseError = 15,
    Expired = 16,
    InvalidDuration = 18,
    InvalidMinQuality = 19,
    InvalidNotes = 20,
    InvalidReagent = 21,
    InvalidRealm = 22,
    InvalidRecipe = 23,
    InvalidRecraftItem = 24,
    InvalidSort = 25,
    InvalidTarget = 26,
    InvalidType = 27,
    Locked = 17,
    MaxOrdersReached = 28,
    MissingCraftingTable = 29,
    MissingCurrency = 30,
    MissingItem = 31,
    MissingNpc = 32,
    MissingOrder = 33,
    MissingRecraftItem = 34,
    NoAccountItems = 35,
    NotClaimed = 36,
    NotCrafted = 37,
    NotInGuild = 38,
    NotYetImplemented = 39,
    Ok = 0,
    OutOfPublicOrderCapacity = 40,
    ServerIsNotAvailable = 41,
    TargetCannotCraft = 43,
    TargetLocked = 44,
    ThrottleViolation = 42,
    Timeout = 45,
    TooManyCurrencies = 46,
    TooManyItems = 47,
    WrongVersion = 48,
  }
end

if not Enum.CraftingOrderResultMeta then
  Enum.CraftingOrderResultMeta = {
    MaxValue = 48,
    MinValue = 0,
    NumValues = 49,
  }
end

if not Enum.CraftingOrderSortType then
  Enum.CraftingOrderSortType = {
    AveTip = 1,
    ItemName = 0,
    MaxTip = 2,
    Quantity = 3,
    Reagents = 4,
    Status = 7,
    TimeRemaining = 6,
    Tip = 5,
  }
end

if not Enum.CraftingOrderSortTypeMeta then
  Enum.CraftingOrderSortTypeMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.CraftingOrderState then
  Enum.CraftingOrderState = {
    Canceled = 13,
    Canceling = 12,
    Claimed = 4,
    Claiming = 3,
    Crafting = 8,
    Created = 2,
    Creating = 1,
    Expired = 15,
    Expiring = 14,
    Fulfilled = 11,
    Fulfilling = 10,
    None = 0,
    Recrafting = 9,
    Rejected = 6,
    Rejecting = 5,
    Releasing = 7,
  }
end

if not Enum.CraftingOrderStateMeta then
  Enum.CraftingOrderStateMeta = {
    MaxValue = 15,
    MinValue = 0,
    NumValues = 16,
  }
end

if not Enum.CraftingOrderType then
  Enum.CraftingOrderType = {
    Guild = 1,
    Npc = 3,
    Personal = 2,
    Public = 0,
  }
end

if not Enum.CraftingOrderTypeMeta then
  Enum.CraftingOrderTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CraftingReagentItemFlag then
  Enum.CraftingReagentItemFlag = {
    TooltipShowsAsStatModifications = 1,
  }
end

if not Enum.CraftingReagentItemFlagMeta then
  Enum.CraftingReagentItemFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.CraftingReagentType then
  Enum.CraftingReagentType = {
    Automatic = 3,
    Basic = 1,
    Finishing = 2,
    Modifying = 0,
  }
end

if not Enum.CraftingReagentTypeMeta then
  Enum.CraftingReagentTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CreateAllAccountData then
  Enum.CreateAllAccountData = {
    AccountCurrenciesDone = "0x0000000000200000",
    AccountDynamicCriteriaDone = "0x0000000001000000",
    AccountFactionsDone = "0x0000000200000000",
    AccountItemsDone = "0x0000000400000000",
    AccountMappingDone = "0x0000008000000000",
    AccountNotificationsDone = "0x0000000008000000",
    AccountStateHousingData = "0x0000080000000000",
    AchievementsDone = "0x0000000000000001",
    ArchivedPurchasesDone = "0x0000000000000200",
    AuctionableTokensDone = "0x0000000000002000",
    BanktabSettingsDone = "0x0000004000000000",
    BattlepetsDone = "0x0000000000000008",
    BitVectorsDone = "0x0000000100000000",
    BpayAddLicenseObjectsDone = "0x0000000000000800",
    BpayDistributionObjectsDone = "0x0000000000000100",
    BpayProductitemObjectsDone = "0x0000000000020000",
    CharacterItemsDone = "0x0000010000000000",
    CharactersDone = "0x0000000000000040",
    CombinedQuestLogEntriesDone = "0x0000000800000000",
    ConsumableTokensDone = "0x0000000000004000",
    CriteriaDone = "0x0000000000000002",
    CurrencyTransferLogDone = "0x0000020000000000",
    CurrencycapsDone = "0x0000000000000010",
    DataElementsDone = "0x0000001000000000",
    EventRecordsDone = "0x0000200000000000",
    ItemCollectionItemsDone = "0x0000000000001000",
    LgVendorPurchaseDone = "0x0000040000000000",
    MountsDone = "0x0000000000000004",
    None = "0x0000000000000000",
    Object = "0x0000000000100000",
    PerkHeldItemsDone = "0x0000000040000000",
    PerkPastRewardsDone = "0x0000000000008000",
    PerkPendingPurchasesDone = "0x0000000010000000",
    PerkPendingRewardsDone = "0x0000000080000000",
    PurchasesDone = "0x0000000000000080",
    QuestCriteriaDone = "0x0000000000080000",
    QuestLogDone = "0x0000000000000020",
    RafActivitiesDone = "0x0000000002000000",
    RafBalanceDone = "0x0000000000400000",
    RafRewardsDone = "0x0000000000800000",
    RevokedRafRewardsDone = "0x0000000004000000",
    SettingsDone = "0x0000000000000400",
    TransmogOutfitsLoadedDone = "0x0000400000000000",
    TrialBoostHistoryDone = "0x0000000000040000",
    VasTransactionsDone = "0x0000000000010000",
    WarbandGroupsDone = "0x0000002000000000",
    WarbandScenesLoadedDone = "0x0000100000000000",
    WowlabsDataDone = "0x0000000020000000",
  }
end

if not Enum.CreateAllAccountDataMeta then
  Enum.CreateAllAccountDataMeta = {
    MaxValue = -2147483648,
    MinValue = 0,
    NumValues = 48,
  }
end

if not Enum.CreateNeighborhoodErrorType then
  Enum.CreateNeighborhoodErrorType = {
    None = 0,
    OversizedGuild = 3,
    Profanity = 1,
    UndersizedGuild = 2,
  }
end

if not Enum.CreateNeighborhoodErrorTypeMeta then
  Enum.CreateNeighborhoodErrorTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.CurioRarityMeta then
  Enum.CurioRarityMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.CurioType then
  Enum.CurioType = {
    Combat = 0,
    Utility = 1,
  }
end

if not Enum.CurioTypeMeta then
  Enum.CurioTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.CurrencyDestroyReason then
  Enum.CurrencyDestroyReason = {
    AccountTransfer = 14,
    BonusRoll = 9,
    Capped = 6,
    Cheat = 0,
    ConcentrationCast = 13,
    CraftingOrderReagent = 16,
    DroppedToCorpse = 8,
    FactionConversion = 10,
    FulfillCraftingOrder = 11,
    Garrison = 7,
    HonorLoss = 15,
    QuestTurnin = 3,
    Script = 12,
    Spell = 1,
    Trade = 5,
    Vendor = 4,
    VersionUpdate = 2,
  }
end

if not Enum.CurrencyDestroyReasonMeta then
  Enum.CurrencyDestroyReasonMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 17,
  }
end

if not Enum.CurrencyFilterTypeMeta then
  Enum.CurrencyFilterTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CurrencyFlags then
  Enum.CurrencyFlags = {
    CurrencyAccountWide = 16777216,
    CurrencyAllowOverflowMailer = 33554432,
    CurrencyAppearsInLootWindow = 2,
    CurrencyComputedWeeklyMaximum = 4,
    CurrencyDeprecated = 131072,
    CurrencyDestroyExtraOnLoot = 2097152,
    CurrencyDoNotCompressChat = 8192,
    CurrencyDoNotLogAcquisitionToBi = 16384,
    CurrencyDoNotToast = 1048576,
    CurrencyDontCoalesceInLootWindow = 8388608,
    CurrencyDontShowTotalInTooltip = 4194304,
    CurrencyDynamicMaximum = 262144,
    CurrencyHasWarmodeBonus = 134217728,
    CurrencyHasWeeklyCatchup = 4096,
    CurrencyHideAsReward = 67108864,
    CurrencyIgnoreMaxQtyOnLoad = 32,
    CurrencyIsAllianceOnly = 268435456,
    CurrencyIsHordeOnly = 536870912,
    CurrencyLimitWarmodeBonusOncePerTooltip = 1073741824,
    CurrencyLogOnWorldChange = 64,
    CurrencyNoLowLevelDrop = 16,
    CurrencyNoRaidDrop = 32768,
    CurrencyNotPersistent = 65536,
    CurrencyResetTrackedQuantity = 256,
    CurrencySingleDropInLoot = 2048,
    CurrencySuppressChatMessageOnVersionChange = 1024,
    CurrencySuppressChatMessages = 524288,
    CurrencyTrackQuantity = 128,
    CurrencyTradable = 1,
    CurrencyUpdateVersionIgnoreMax = 512,
    CurrencyUsesLedgerBalance = 2147483648,
    Currency_100_Scaler = 8,
  }
end

if not Enum.CurrencyFlagsB then
  Enum.CurrencyFlagsB = {
    CurrencyBAllowReductionByResourcefulness = 1024,
    CurrencyBBattlenetVirtualCurrency = 8,
    CurrencyBDontDisplayIfZero = 32,
    CurrencyBForceMaxQuantityOnConversion = 256,
    CurrencyBNoNotificationMailOnOfflineProgress = 4,
    CurrencyBScaleMaxQuantityBySeasonWeeks = 64,
    CurrencyBScaleMaxQuantityByWeeksSinceStart = 128,
    CurrencyBShowQuestXPGainInTooltip = 2,
    CurrencyBUnearnableBeforeMaxQuantityStart = 512,
    CurrencyBUseTotalEarnedForEarned = 1,
    FutureCurrencyFlag = 16,
  }
end

if not Enum.CurrencyFlagsBMeta then
  Enum.CurrencyFlagsBMeta = {
    MaxValue = 1024,
    MinValue = 1,
    NumValues = 11,
  }
end

if not Enum.CurrencyFlagsMeta then
  Enum.CurrencyFlagsMeta = {
    MaxValue = -2147483648,
    MinValue = 1,
    NumValues = 32,
  }
end

if not Enum.CurrencyGainFlags then
  Enum.CurrencyGainFlags = {
    Autotracking = 8,
    BonusAward = 1,
    DroppedFromDeath = 2,
    FromAccountServer = 4,
    None = 0,
  }
end

if not Enum.CurrencyGainFlagsMeta then
  Enum.CurrencyGainFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.CurrencySource then
  Enum.CurrencySource = {
    AccountCopy = 42,
    AccountHwmUpdate = 61,
    AccountTransfer = 65,
    AddConduitToCollection = 46,
    Arena = 17,
    AuctionDeposit = 51,
    AzeriteRespec = 34,
    Barbershop = 47,
    BonusRoll = 33,
    CatalystBalancing = 57,
    CatalystCraft = 58,
    Cheat = 4,
    ConvertItemsToCurrencyAndReputation = 62,
    ConvertItemsToCurrencyValue = 48,
    ConvertOldItem = 0,
    ConvertOldPvPCurrency = 1,
    CraftingOrder = 56,
    CurrencyWalletClaim = 29,
    DailyQuestReward = 38,
    DailyQuestWarModeReward = 39,
    DailyReset = 45,
    ExceededMaxQty = 18,
    FactionConversion = 37,
    GarrisonBuilding = 23,
    GarrisonBuildingRefund = 26,
    GarrisonFollowerActivation = 25,
    GarrisonMissionReward = 27,
    GarrisonResourceOverTime = 28,
    GarrisonTalent = 30,
    GarrisonTalentTreeReset = 44,
    GarrisonWorldQuestBonus = 31,
    GuildBankWithdrawal = 21,
    InitiativeReward = 67,
    ItemDeletion = 14,
    ItemRefund = 2,
    LFGReward = 11,
    Loot = 9,
    PhBuffer_53 = 53,
    PhBuffer_54 = 54,
    PhBuffer_63 = 63,
    PlayerTrait = 52,
    PlayerTraitRefund = 60,
    ProfessionInitialAward = 59,
    Pushloot = 22,
    PvPCompletionBonus = 19,
    PvPDrop = 24,
    PvPHonorReward = 32,
    PvPKillCredit = 6,
    PvPMetaCredit = 7,
    PvPScriptedAward = 8,
    PvPTeamContribution = 49,
    QuestReward = 3,
    RandomBattleground = 16,
    RatedBattleground = 15,
    RenownRepGain = 55,
    RenownRepGainInitialVisibility = 66,
    Script = 20,
    Spell = 13,
    SpellSkipLinkedCurrency = 64,
    Trade = 12,
    Transmogrify = 50,
    UpdatingVersion = 10,
    Vendor = 5,
    WeeklyQuestReward = 40,
    WeeklyQuestWarModeReward = 41,
    WeeklyRewardChest = 43,
    WorldQuestReward = 35,
    WorldQuestRewardIgnoreCapsDeprecated = 36,
  }
end

if not Enum.CurrencySourceMeta then
  Enum.CurrencySourceMeta = {
    MaxValue = 67,
    MinValue = 0,
    NumValues = 68,
  }
end

if not Enum.CurrencyTokenCategoryFlags then
  Enum.CurrencyTokenCategoryFlags = {
    FlagPlayerItemAssignment = 2,
    FlagSortLast = 1,
    Hidden = 4,
    StartsCollapsed = 16,
    Virtual = 8,
  }
end

if not Enum.CurrencyTokenCategoryFlagsMeta then
  Enum.CurrencyTokenCategoryFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.CurrencyType then
  Enum.CurrencyType = {
    Copper = 0,
    Silver = 1,
    Gold = 2,
  }
end

if not Enum.CurrencyTypeMeta then
  Enum.CurrencyTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.CursorStyle then
  Enum.CursorStyle = {
    Crosshair = 1,
    Mouse = 0,
  }
end

if not Enum.CursorStyleMeta then
  Enum.CursorStyleMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.Cursormode then
  Enum.Cursormode = {
    AttackCursor = 4,
    AttackErrorCursor = 50,
    BroomCursor = 42,
    BroomErrorCursor = 88,
    BuyCursor = 3,
    BuyErrorCursor = 49,
    CampaignQuestCursor = 23,
    CampaignQuestErrorCursor = 69,
    CampaignQuestTurninCursor = 24,
    CampaignQuestTurninErrorCursor = 70,
    CastCursor = 2,
    CastErrorCursor = 48,
    CustomCursor = 91,
    EnchantCursor = 39,
    EnchantErrorCursor = 85,
    GatherCursor = 13,
    GatherErrorCursor = 59,
    GrabbingHandCursor = 44,
    GrabbingHandErrorCursor = 90,
    HoldingHandCursor = 43,
    HoldingHandErrorCursor = 89,
    InnkeeperCursor = 22,
    InnkeeperErrorCursor = 68,
    InspectCursor = 7,
    InspectErrorCursor = 53,
    InteractCursor = 5,
    InteractErrorCursor = 51,
    ItemCursor = 19,
    ItemErrorCursor = 65,
    LockCursor = 14,
    LockErrorCursor = 60,
    LootAllCursor = 16,
    LootAllErrorCursor = 62,
    MailCursor = 15,
    MailErrorCursor = 61,
    MapPinCursor = 37,
    MapPinErrorCursor = 83,
    MineCursor = 11,
    MineErrorCursor = 57,
    NoCursor = 0,
    PaletteCursor = 41,
    PaletteErrorCursor = 87,
    PickupCursor = 8,
    PickupErrorCursor = 54,
    PingCursor = 38,
    PingErrorCursor = 84,
    PointCursor = 1,
    PointErrorCursor = 47,
    QuestCursor = 25,
    QuestErrorCursor = 71,
    QuestImportantCursor = 30,
    QuestImportantErrorCursor = 76,
    QuestImportantTurninCursor = 31,
    QuestImportantTurninErrorCursor = 77,
    QuestLegendaryCursor = 28,
    QuestLegendaryErrorCursor = 74,
    QuestLegendaryTurninCursor = 29,
    QuestLegendaryTurninErrorCursor = 75,
    QuestMetaCursor = 32,
    QuestMetaErrorCursor = 78,
    QuestMetaTurninCursor = 33,
    QuestMetaTurninErrorCursor = 79,
    QuestRecurringCursor = 34,
    QuestRecurringErrorCursor = 80,
    QuestRecurringTurninCursor = 35,
    QuestRecurringTurninErrorCursor = 81,
    QuestRepeatableCursor = 26,
    QuestRepeatableErrorCursor = 72,
    QuestTurninCursor = 27,
    QuestTurninErrorCursor = 73,
    RepairCursor = 17,
    RepairErrorCursor = 63,
    RepairnpcCursor = 18,
    RepairnpcErrorCursor = 64,
    SkinAllianceCursor = 21,
    SkinAllianceErrorCursor = 67,
    SkinCursor = 12,
    SkinErrorCursor = 58,
    SkinHordeCursor = 20,
    SkinHordeErrorCursor = 66,
    SpeakCursor = 6,
    SpeakErrorCursor = 52,
    StablemasterCursor = 40,
    StablemasterErrorCursor = 86,
    TaxiCursor = 9,
    TaxiErrorCursor = 55,
    TrainerCursor = 10,
    TrainerErrorCursor = 56,
    UIMoveCursor = 45,
    UIResizeCursor = 46,
    VehicleCursor = 36,
    VehicleErrorCursor = 82,
  }
end

if not Enum.CursormodeMeta then
  Enum.CursormodeMeta = {
    MaxValue = 91,
    MinValue = 0,
    NumValues = 92,
  }
end

if not Enum.CustomBindingTypeMeta then
  Enum.CustomBindingTypeMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.DamageMeterCombineSessionType then
  Enum.DamageMeterCombineSessionType = {
    Arena = 2,
    ChallengeMode = 1,
    None = 0,
  }
end

if not Enum.DamageMeterCombineSessionTypeMeta then
  Enum.DamageMeterCombineSessionTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.DamageMeterNumbersMeta then
  Enum.DamageMeterNumbersMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.DamageMeterOverrideType then
  Enum.DamageMeterOverrideType = {
    AllowFriendlyFire = 1,
    Ignore = 0,
    IgnoreForAbsorbSpell = 4,
    RedirectSourceToAuraCaster = 3,
    RedirectSourceToOwner = 2,
  }
end

if not Enum.DamageMeterOverrideTypeMeta then
  Enum.DamageMeterOverrideTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.DamageMeterSessionTypeMeta then
  Enum.DamageMeterSessionTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.DamageMeterSpellDetailsDisplayType then
  Enum.DamageMeterSpellDetailsDisplayType = {
    Deaths = 3,
    EnemyDamageTaken = 4,
    SpellAffected = 2,
    SpellCasted = 0,
    UnitSpecificSpellCasted = 1,
  }
end

if not Enum.DamageMeterSpellDetailsDisplayTypeMeta then
  Enum.DamageMeterSpellDetailsDisplayTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.DamageMeterSourceDisplayTypeMeta then
  Enum.DamageMeterSourceDisplayTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.DamageMeterStorageType then
  Enum.DamageMeterStorageType = {
    Absorbs = 2,
    AvoidableDamageTaken = 6,
    Damage = 0,
    DamageTaken = 5,
    Deaths = 7,
    Dispels = 4,
    EnemyDamageTaken = 8,
    HealingAndAbsorbs = 1,
    Interrupts = 3,
  }
end

if not Enum.DamageMeterStorageTypeMeta then
  Enum.DamageMeterStorageTypeMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.DamageMeterStyleMeta then
  Enum.DamageMeterStyleMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.DamageMeterTypeMeta then
  Enum.DamageMeterTypeMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.DamageMeterVisibilityMeta then
  Enum.DamageMeterVisibilityMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.Damageclass then
  Enum.Damageclass = {
    All = 127,
    AllMagical = 126,
    AllPhysical = 1,
    Arcane = 6,
    Fire = 2,
    FirstResist = 2,
    Frost = 4,
    Holy = 1,
    LastResist = 6,
    MaskArcane = 64,
    MaskChaos = 124,
    MaskChromatic = 62,
    MaskCosmic = 106,
    MaskDivine = 66,
    MaskElemental = 28,
    MaskFire = 4,
    MaskFirestorm = 12,
    MaskFlamestrike = 5,
    MaskFrost = 16,
    MaskFrostfire = 20,
    MaskFroststorm = 24,
    MaskFroststrike = 17,
    MaskHoly = 2,
    MaskHolyfire = 6,
    MaskHolyfrost = 18,
    MaskHolystorm = 10,
    MaskHolystrike = 3,
    MaskMagical = 126,
    MaskNature = 8,
    MaskNone = 0,
    MaskPhysical = 1,
    MaskShadow = 32,
    MaskShadowflame = 36,
    MaskShadowfrost = 48,
    MaskShadowstorm = 40,
    MaskShadowstrike = 33,
    MaskSpellfire = 68,
    MaskSpellfrost = 80,
    MaskSpellshadow = 96,
    MaskSpellstorm = 72,
    MaskSpellstrike = 65,
    MaskStormstrike = 9,
    MaskTwilight = 34,
    Nature = 3,
    Physical = 0,
    Shadow = 5,
  }
end

if not Enum.DamageclassMeta then
  Enum.DamageclassMeta = {
    MaxValue = 127,
    MinValue = 0,
    NumValues = 46,
  }
end

if not Enum.DamageclassType then
  Enum.DamageclassType = {
    Magical = 1,
    Physical = 0,
  }
end

if not Enum.DamageclassTypeMeta then
  Enum.DamageclassTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.DisableAccountProfilesFlags then
  Enum.DisableAccountProfilesFlags = {
    DecorsCollections = 32,
    Document = 1,
    ItemsCollections = 16,
    MountsCollections = 4,
    None = 0,
    PetsCollections = 8,
    SharedCollections = 2,
  }
end

if not Enum.DisableAccountProfilesFlagsMeta then
  Enum.DisableAccountProfilesFlagsMeta = {
    MaxValue = 32,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.DungeonEncounterFlags then
  Enum.DungeonEncounterFlags = {
    AutoEnd = 8,
    Cosmetic = 16,
    DisableEncounterEvents = 512,
    GuildNews = 2,
    HideUntilCompleted = 64,
    IgnoreSpawnLimit = 256,
    NoAutoStart = 128,
    RaidLockPlayers = 4,
    StickyNews = 1,
    Unused = 32,
  }
end

if not Enum.DungeonEncounterFlagsMeta then
  Enum.DungeonEncounterFlagsMeta = {
    MaxValue = 512,
    MinValue = 1,
    NumValues = 10,
  }
end

if not Enum.DungeonEncounterTriggerType then
  Enum.DungeonEncounterTriggerType = {
    Invalid = 0,
    OnComplete = 2,
    OnEnd = 3,
    OnStart = 1,
    PreviouslyCompleted = 4,
  }
end

if not Enum.DungeonEncounterTriggerTypeMeta then
  Enum.DungeonEncounterTriggerTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.DungeonEncounterXCreatureFlags then
  Enum.DungeonEncounterXCreatureFlags = {
    BossCreature = 1,
    DoNotDespawnOnSuccess = 4,
    DropLootImmediately = 2,
  }
end

if not Enum.DungeonEncounterXCreatureFlagsMeta then
  Enum.DungeonEncounterXCreatureFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.DurationTimeModifier then
  Enum.DurationTimeModifier = {
    BaseTime = 1,
    RealTime = 0,
  }
end

if not Enum.DurationTimeModifierMeta then
  Enum.DurationTimeModifierMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EditModeAccountSettingMeta then
  Enum.EditModeAccountSettingMeta = {
    MaxValue = 35,
    MinValue = 0,
    NumValues = 36,
  }
end

if not Enum.EditModeActionBarSettingMeta then
  Enum.EditModeActionBarSettingMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.EditModeActionBarSystemIndicesMeta then
  Enum.EditModeActionBarSystemIndicesMeta = {
    MaxValue = 13,
    MinValue = 1,
    NumValues = 11,
  }
end

if not Enum.EditModeArchaeologyBarSettingMeta then
  Enum.EditModeArchaeologyBarSettingMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.EditModeAuraFrameSettingMeta then
  Enum.EditModeAuraFrameSettingMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.EditModeAuraFrameSystemIndicesMeta then
  Enum.EditModeAuraFrameSystemIndicesMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.EditModeBagsSettingMeta then
  Enum.EditModeBagsSettingMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EditModeCastBarSettingMeta then
  Enum.EditModeCastBarSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EditModeChatFrameSettingMeta then
  Enum.EditModeChatFrameSettingMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EditModeCooldownViewerSettingMeta then
  Enum.EditModeCooldownViewerSettingMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.EditModeCooldownViewerSystemIndicesMeta then
  Enum.EditModeCooldownViewerSystemIndicesMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.EditModeDamageMeterSettingMeta then
  Enum.EditModeDamageMeterSettingMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.EditModeDurabilityFrameSettingMeta then
  Enum.EditModeDurabilityFrameSettingMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.EditModeEncounterEventsSettingMeta then
  Enum.EditModeEncounterEventsSettingMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.EditModeEncounterEventsSystemIndicesMeta then
  Enum.EditModeEncounterEventsSystemIndicesMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.EditModeLayoutTypeMeta then
  Enum.EditModeLayoutTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EditModeLossOfControlSettingMeta then
  Enum.EditModeLossOfControlSettingMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.EditModeMicroMenuSettingMeta then
  Enum.EditModeMicroMenuSettingMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EditModeMinimapSettingMeta then
  Enum.EditModeMinimapSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EditModeObjectiveTrackerSettingMeta then
  Enum.EditModeObjectiveTrackerSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EditModePersonalResourceDisplaySettingMeta then
  Enum.EditModePersonalResourceDisplaySettingMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.PersonalResourceDisplayVisibleSettingMeta then
  Enum.PersonalResourceDisplayVisibleSettingMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WorldTierDifficultyMeta then
  Enum.WorldTierDifficultyMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.EditModePresetLayoutsMeta then
  Enum.EditModePresetLayoutsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EditModeSettingDisplayTypeMeta then
  Enum.EditModeSettingDisplayTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EditModeStatusTrackingBarSettingMeta then
  Enum.EditModeStatusTrackingBarSettingMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EditModeStatusTrackingBarSystemIndicesMeta then
  Enum.EditModeStatusTrackingBarSystemIndicesMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.EditModeSystemMeta then
  Enum.EditModeSystemMeta = {
    MaxValue = 26,
    MinValue = 0,
    NumValues = 27,
  }
end

if not Enum.EditModeTimerBarsSettingMeta then
  Enum.EditModeTimerBarsSettingMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.EditModeUnitFrameSettingMeta then
  Enum.EditModeUnitFrameSettingMeta = {
    MaxValue = 21,
    MinValue = 0,
    NumValues = 22,
  }
end

if not Enum.EditModeUnitFrameSystemIndicesMeta then
  Enum.EditModeUnitFrameSystemIndicesMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 8,
  }
end

if not Enum.EditModeVehicleSeatIndicatorSettingMeta then
  Enum.EditModeVehicleSeatIndicatorSettingMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.EncounterEventCastState then
  Enum.EncounterEventCastState = {
    Casting = 1,
    Expired = 3,
    NotCasting = 2,
  }
end

if not Enum.EncounterEventCastStateMeta then
  Enum.EncounterEventCastStateMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.EncounterEventFlags then
  Enum.EncounterEventFlags = {
    Disabled = 1,
    IgnoreCastConsume = 2,
  }
end

if not Enum.EncounterEventFlagsMeta then
  Enum.EncounterEventFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.EncounterEventIconmask then
  Enum.EncounterEventIconmask = {
    BleedEffect = 4,
    CurseEffect = 32,
    DeadlyEffect = 1,
    DiseaseEffect = 16,
    DpsRole = 512,
    EnrageEffect = 2,
    HealerRole = 256,
    MagicEffect = 8,
    PoisonEffect = 64,
    TankRole = 128,
  }
end

if not Enum.EncounterEventIconmaskMeta then
  Enum.EncounterEventIconmaskMeta = {
    MaxValue = 512,
    MinValue = 1,
    NumValues = 10,
  }
end

if not Enum.EncounterEventSeverityMeta then
  Enum.EncounterEventSeverityMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterEventSoundTrigger then
  Enum.EncounterEventSoundTrigger = {
    OnTextWarningShown = 0,
    OnTimelineEventFinished = 1,
    OnTimelineEventHighlight = 2,
  }
end

if not Enum.EncounterEventSoundTriggerMeta then
  Enum.EncounterEventSoundTriggerMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterEventsIconDirection then
  Enum.EncounterEventsIconDirection = {
    Bottom = 1,
    Left = 0,
    Right = 1,
    Top = 0,
  }
end

if not Enum.EncounterEventsIconDirectionMeta then
  Enum.EncounterEventsIconDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EncounterEventsOrientationMeta then
  Enum.EncounterEventsOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EncounterEventsTooltipAnchorMeta then
  Enum.EncounterEventsTooltipAnchorMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterEventsViewTypeMeta then
  Enum.EncounterEventsViewTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EncounterEventsVisibilityMeta then
  Enum.EncounterEventsVisibilityMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterLootDropRollState then
  Enum.EncounterLootDropRollState = {
    Greed = 3,
    NeedMainSpec = 0,
    NeedOffSpec = 1,
    NoRoll = 4,
    Pass = 5,
    Transmog = 2,
  }
end

if not Enum.EncounterLootDropRollStateMeta then
  Enum.EncounterLootDropRollStateMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.EncounterTimelineEventSortDirection then
  Enum.EncounterTimelineEventSortDirection = {
    Ascending = 1,
    Descending = 0,
  }
end

if not Enum.EncounterTimelineEventSortDirectionMeta then
  Enum.EncounterTimelineEventSortDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EncounterTimelineEventSource then
  Enum.EncounterTimelineEventSource = {
    EditMode = 2,
    Encounter = 0,
    Script = 1,
  }
end

if not Enum.EncounterTimelineEventSourceMeta then
  Enum.EncounterTimelineEventSourceMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterTimelineEventStateMeta then
  Enum.EncounterTimelineEventStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.EncounterTimelineIconSet then
  Enum.EncounterTimelineIconSet = {
    DamageAlert = 3,
    Deadly = 4,
    Dispel = 5,
    Enrage = 6,
    HealerAlert = 2,
    TankAlert = 1,
  }
end

if not Enum.EncounterTimelineIconSetMeta then
  Enum.EncounterTimelineIconSetMeta = {
    MaxValue = 6,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.EncounterTimelineTrackMeta then
  Enum.EncounterTimelineTrackMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.EncounterTimelineTrackType then
  Enum.EncounterTimelineTrackType = {
    Hidden = 0,
    Linear = 2,
    Sorted = 1,
  }
end

if not Enum.EncounterTimelineTrackTypeMeta then
  Enum.EncounterTimelineTrackTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EncounterTimelineViewType then
  Enum.EncounterTimelineViewType = {
    Bars = 2,
    None = 0,
    Timeline = 1,
  }
end

if not Enum.EncounterTimelineViewTypeMeta then
  Enum.EncounterTimelineViewTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EndOfMatchTypeMeta then
  Enum.EndOfMatchTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.EnvironmentalDamageFlags then
  Enum.EnvironmentalDamageFlags = {
    DmgIsPct = 2,
    OneTime = 1,
  }
end

if not Enum.EnvironmentalDamageFlagsMeta then
  Enum.EnvironmentalDamageFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.Environmentaldamagetype then
  Enum.Environmentaldamagetype = {
    Drowning = 1,
    Falling = 2,
    Fatigue = 0,
    Fire = 5,
    Lava = 3,
    Slime = 4,
  }
end

if not Enum.EnvironmentaldamagetypeMeta then
  Enum.EnvironmentaldamagetypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.ErrorDomain then
  Enum.ErrorDomain = {
    Assert = 0,
    AssertData = 1,
    Frame = 2,
  }
end

if not Enum.ErrorDomainMeta then
  Enum.ErrorDomainMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.EventRealmQueuesMeta then
  Enum.EventRealmQueuesMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.EventToastDisplayTypeMeta then
  Enum.EventToastDisplayTypeMeta = {
    MaxValue = 15,
    MinValue = 0,
    NumValues = 16,
  }
end

if not Enum.EventToastEventType then
  Enum.EventToastEventType = {
    BattlePetLevelChanged = 8,
    BattlePetLevelUpAbility = 9,
    CriteriaUpdated = 19,
    FlightpointDiscovered = 25,
    HouseUpgradeAvailable = 26,
    LevelUp = 0,
    LevelUpDungeon = 2,
    LevelUpOther = 15,
    LevelUpPvP = 4,
    LevelUpRaid = 3,
    LevelUpSpell = 1,
    MythicPlusWeeklyRecord = 11,
    PetBattleCapture = 7,
    PetBattleFinalRound = 6,
    PetBattleNewAbility = 5,
    PlayerAuraAdded = 16,
    PlayerAuraRemoved = 17,
    PvPTierUpdate = 20,
    QuestBossEmote = 10,
    QuestTurnedIn = 12,
    Scenario = 14,
    SpellLearned = 21,
    SpellScript = 18,
    TreasureItem = 22,
    WeeklyRewardUnlock = 23,
    WeeklyRewardUpgrade = 24,
    WorldStateChange = 13,
  }
end

if not Enum.EventToastEventTypeMeta then
  Enum.EventToastEventTypeMeta = {
    MaxValue = 26,
    MinValue = 0,
    NumValues = 27,
  }
end

if not Enum.EventToastFlags then
  Enum.EventToastFlags = {
    DisableRightClickDismiss = 1,
  }
end

if not Enum.EventToastFlagsMeta then
  Enum.EventToastFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.ExcludedCensorSources then
  Enum.ExcludedCensorSources = {
    All = 255,
    Friends = 1,
    Guild = 2,
    None = 0,
    Reserve1 = 4,
    Reserve2 = 8,
    Reserve3 = 16,
    Reserve4 = 32,
    Reserve5 = 64,
    Reserve6 = 128,
  }
end

if not Enum.ExcludedCensorSourcesMeta then
  Enum.ExcludedCensorSourcesMeta = {
    MaxValue = 255,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.ExpansionLevel then
  Enum.ExpansionLevel = {
    BattleForAzeroth = 7,
    BurningCrusade = 1,
    Cataclysm = 3,
    Draenor = 5,
    Dragonflight = 9,
    LastTitan = 12,
    Legion = 6,
    Midnight = 11,
    MistsOfPandaria = 4,
    None = 0,
    Northrend = 2,
    Shadowlands = 8,
    WarWithin = 10,
  }
end

if not Enum.ExpansionLevelMeta then
  Enum.ExpansionLevelMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.FlightPathFaction then
  Enum.FlightPathFaction = {
    Alliance = 2,
    Horde = 1,
    Neutral = 0,
  }
end

if not Enum.FlightPathFactionMeta then
  Enum.FlightPathFactionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.FlightPathState then
  Enum.FlightPathState = {
    Current = 0,
    Reachable = 1,
    Unreachable = 2,
  }
end

if not Enum.FlightPathStateMeta then
  Enum.FlightPathStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.FollowerAbilityCastResult then
  Enum.FollowerAbilityCastResult = {
    AlreadyAtMaxDurability = 13,
    CannotTargetLimitedUseFollower = 11,
    CannotTargetNonAutoMissionFollower = 14,
    Failure = 1,
    InvalidFollowerSpell = 4,
    InvalidFollowerType = 9,
    InvalidTarget = 3,
    MustBeUnique = 10,
    MustTargetFollower = 7,
    MustTargetLimitedUseFollower = 12,
    MustTargetTrait = 8,
    NoPendingCast = 2,
    RerollNotAllowed = 5,
    SingleMissionDuration = 6,
    Success = 0,
  }
end

if not Enum.FollowerAbilityCastResultMeta then
  Enum.FollowerAbilityCastResultMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.FontStringScaleAnimationMode then
  Enum.FontStringScaleAnimationMode = {
    FontSize = 0,
    Vertex = 1,
  }
end

if not Enum.FontStringScaleAnimationModeMeta then
  Enum.FontStringScaleAnimationModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.FragmentID then
  Enum.FragmentID = {
    Actor = 15,
    CgObject = 2,
    ClientObservablesData = 6,
    DeferredMessages = 9,
    EntityPosition = 1,
    FHousingDecor = 20,
    FHousingDecorActor = 28,
    FHousingDecorProxy = 26,
    FHousingFixture = 34,
    FHousingHouseDecorSet = 24,
    FHousingHouseFixtureSet = 35,
    FHousingHouseRoomSet = 25,
    FHousingPlayerHouse = 23,
    FHousingPlotAreaTrigger = 29,
    FHousingRoom = 21,
    FHousingRoomComponentMesh = 22,
    FHousingStorageMirrorData = 33,
    FJamHousingCornerstone = 27,
    FLootObjectList = 12,
    FMeshObjectData = 19,
    FMirroredPositionData = 31,
    FNeighborhoodMirrorData = 30,
    FNeighborhoodStateData = 38,
    FPersistableJamServers = 10,
    FPhaseShiftData = 16,
    FPlayerHouseInfo = 32,
    FPlayerInitiativeInfo = 37,
    FPlayerJamServers = 11,
    FPlayerOwnershipLink = 13,
    FTransportLink = 5,
    FUnitAIGroupLink = 39,
    FUnitAreaTriggerLink = 14,
    FVendor = 17,
    HeartbeatData = 4,
    JamDispatcher = 3,
    MirrorState = 0,
    MirroredObjectC = 18,
    ReservedArchetypeSeparator = 255,
    TagAIGroup = 212,
    TagActiveClientS = 216,
    TagActiveObjectC = 217,
    TagActivePlayer = 215,
    TagAreaTrigger = 209,
    TagAzeriteEmpoweredItem = 202,
    TagAzeriteItem = 203,
    TagContainer = 201,
    TagConversation = 211,
    TagCorpse = 208,
    TagDynamicObject = 207,
    TagGameObject = 206,
    TagHouseExteriorPiece = 224,
    TagHouseExteriorRoot = 225,
    TagHousingPoolObject = 223,
    TagHousingRoom = 220,
    TagHousingSubdivisionObject = 222,
    TagItem = 200,
    TagLootObject = 214,
    TagMeshObject = 221,
    TagPlayer = 205,
    TagScenario = 213,
    TagSceneObject = 210,
    TagUnit = 204,
    TagUnitVehicle = 219,
    TagVisibleObjectC = 218,
    TestFragment_1 = 250,
    TestFragment_2 = 251,
    TestFragment_3 = 252,
    TestFragment_4 = 253,
    TestFragment_5 = 254,
    Timer = 36,
    TimerQueues = 7,
    TransferSuspensionData = 8,
  }
end

if not Enum.FragmentIDMeta then
  Enum.FragmentIDMeta = {
    MaxValue = 255,
    MinValue = 0,
    NumValues = 72,
  }
end

if not Enum.FrameTutorialAccount then
  Enum.FrameTutorialAccount = {
    AccountWideReputation = 9,
    AssistedCombatRotationActionButton = 22,
    AssistedCombatRotationDragSpell = 21,
    BindToAccountUntilEquip = 11,
    CompletedQuestsFilter = 12,
    CompletedQuestsFilterSeen = 13,
    ConcentrationCurrency = 14,
    EditModeManager = 3,
    EnconterJournalTutorialsTabSeen = 33,
    EventSchedulerTabSeen = 20,
    HeirloomJournalLevel = 7,
    HousingCleanupMode = 40,
    HousingDecorCleanup = 23,
    HousingDecorClippingGrid = 25,
    HousingDecorCustomization = 26,
    HousingDecorLayout = 27,
    HousingDecorPlace = 24,
    HousingExpertMode = 39,
    HousingHouseFinderMap = 28,
    HousingHouseFinderVisitHouse = 29,
    HousingInvalidCollision = 37,
    HousingItemAcquisition = 30,
    HousingMarketTab = 34,
    HousingModesUnlocked = 38,
    HousingNewPip = 31,
    HousingTeleportButton = 35,
    HudRevampBagChanges = 1,
    LFGList = 6,
    LocalStoriesFilterSeen = 19,
    MapLegendOpened = 15,
    MountCollectionDragonriding = 5,
    NpcCraftingOrderCreateButton = 17,
    NpcCraftingOrderTabNew = 18,
    NpcCraftingOrders = 16,
    PerksProgramActivitiesIntro = 2,
    PerksProgramActivitiesOpen = 32,
    RPETalentStarterBuild = 36,
    TimerunnersAdvantage = 8,
    TransferableCurrencies = 10,
    TransmogCustomSets = 43,
    TransmogCustomSetsMigration = 47,
    TransmogOutfits = 41,
    TransmogSets = 42,
    TransmogSetsTab = 4,
    TransmogSituations = 44,
    TransmogTrialOfStyle = 46,
    TransmogWeaponOptions = 45,
  }
end

if not Enum.FrameTutorialAccountMeta then
  Enum.FrameTutorialAccountMeta = {
    MaxValue = 47,
    MinValue = 1,
    NumValues = 47,
  }
end

if not Enum.GameModeMeta then
  Enum.GameModeMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.GamePadPowerLevel then
  Enum.GamePadPowerLevel = {
    Critical = 0,
    High = 3,
    Low = 1,
    Medium = 2,
    Unknown = 5,
    Wired = 4,
  }
end

if not Enum.GamePadPowerLevelMeta then
  Enum.GamePadPowerLevelMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.GameRule then
  Enum.GameRule = {
    AchievementsPanelDisabled = 40,
    ActionButtonTypeOverlayStrategy = 165,
    ActionCombatTargetLockEnabled = 87,
    ActionbarIconIntroDisabled = 60,
    AfterDeathSpectatingUI = 49,
    AllPlayersAreFastMovers = 53,
    AlwaysAllowAlliedRaces = 59,
    AutoAttacksDisabled = 103,
    BagSpaceOverride = 172,
    BagsUIDisabled = 64,
    BlockWhileSheathedAllowed = 88,
    CharNameReservationEnabled = 2,
    CharReservationsPerRealmReopenThreshold = 8,
    CharacterCreateUseFixedBackgroundModel = 55,
    CharacterPanelDisabled = 37,
    CharacterlessLogin = 14,
    ChatLinkLevelToastsDisabled = 63,
    ClearMailOnRealmTransfer = 92,
    CollectionsPanelDisabled = 36,
    CommunitiesPanelDisabled = 41,
    CompactRaidFrameManagerDisabled = 81,
    CustomActionbarOverlayHeightOffset = 33,
    DeleteItemConfirmationDisabled = 62,
    DisableCampsites = 114,
    DisableHonorDecay = 23,
    DisablePct = 9,
    DisableQuickJoin = 162,
    DisableRaidGroups = 163,
    DisableRealmSelection = 113,
    DisableVas = 119,
    DoesNotCountTowardAccountCharacterMax = 142,
    EditModeDisabled = 82,
    EjDungeonsDisabled = 149,
    EjItemSetsDisabled = 151,
    EjJourneysDisabled = 156,
    EjRaidsDisabled = 150,
    EjSuggestedContentDisabled = 148,
    EncounterJournalDisabled = 42,
    EtaRealmLaunchTime = 5,
    ExperienceBarDisabled = 159,
    FastAreaTriggerTick = 52,
    FinderPanelDisabled = 43,
    ForceAlteredFormsOn = 56,
    ForcedChatLanguage = 34,
    ForcedMultiActionBarSetting = 133,
    ForcedPartyFrameScale = 32,
    FrontEndChat = 50,
    FullCharacterCreateDisabled = 84,
    GameMode = 13,
    GdapiCharacterProfileDisabled = 153,
    GroupFinderCapabilities = 98,
    GuildsDisabled = 46,
    HardcoreRuleset = 10,
    HelpPanelDisabled = 45,
    HideAllMultiActionBars = 135,
    HideFaction = 116,
    HideTransmogZeroCost = 161,
    HideUnavailableTransmogSlots = 160,
    HousingDashboardDisabled = 102,
    HousingEnabled = 154,
    IgnoreChrclassDisabledFlag = 54,
    IngameCalendarDisabled = 75,
    IngameFriendsListDisabled = 79,
    IngameMailNotificationDisabled = 74,
    IngameTrackingDisabled = 76,
    IngameWhoListDisabled = 77,
    InstanceDifficultyBannerDisabled = 83,
    LandingPageFactionID = 35,
    LootMethodStyle = 157,
    MacrosDisabled = 80,
    MailGameRule = 132,
    MapPlunderstormCircle = 48,
    MaxAccountCharReservationsPerContentset = 4,
    MaxCharReservationsPerRealm = 3,
    MaxCharactersPerContentSet = 139,
    MaxLootDropLevel = 25,
    MaxNameplateDistance = 28,
    MaxUnitNameDistance = 27,
    MaximizeWorldMapDisabled = 67,
    MerchantFilterDisabled = 101,
    MicrobarScale = 26,
    MinUndeleteLevelRequired = 140,
    MinimapDisabled = 146,
    NameplateCastBarDisabled = 107,
    NoDebuffLimit = 1,
    NoMultiboxing = 15,
    NonPlayerNameplateScale = 31,
    ObjectiveTrackerDisabled = 104,
    PerksProgramActivityTrackingDisabled = 66,
    PersonalResourceDisplayDisabled = 129,
    PetBattlesDisabled = 65,
    PlayerCastBarDisabled = 105,
    PlayerFrameDisabled = 131,
    PlayerNameplateAlternateHealthColor = 58,
    PlayerNameplateDifficultyIcon = 57,
    PlunderstormAreaSelection = 94,
    PremadeGroupFinderStyle = 93,
    PvPInitialRatingOverride = 190,
    QuestLogMicrobuttonDisabled = 47,
    QuestLogPanelDisabled = 71,
    QuestLogSuperTrackingDisabled = 72,
    RaceAlteredFormsDisabled = 78,
    RecommendLeastPopulatedRealm = 169,
    ReleaseSpiritGhostDisabled = 61,
    RepairArmorDisabled = 147,
    ReplaceAbsentGmSeconds = 11,
    ReplaceGmRankLastOnlineSeconds = 12,
    RestrictedAchievementCategoryID = 155,
    Runecarving = 17,
    SelfFoundAllowed = 22,
    SpellbookPanelDisabled = 38,
    StoreDisabled = 44,
    SummoningStones = 108,
    TalentRespecCostMax = 19,
    TalentRespecCostMin = 18,
    TalentRespecCostStep = 20,
    TalentsPanelDisabled = 39,
    TargetCastBarDisabled = 106,
    TargetFrameBuffsDisabled = 85,
    TargetFrameDisabled = 130,
    TimerunningAllowed = 137,
    TransmogEnabled = 109,
    TrivialGroupXPPercent = 7,
    TutorialFrameDisabled = 73,
    UnflaggedPlayersCanAttackPvPFlaggedPlayers = 173,
    UnitFramePvPContextualDisabled = 86,
    UniversalNameplateOcclusion = 51,
    UseGameTableVariation = 164,
    UseSimpleCharacterSelectList = 115,
    UserAddonsDisabled = 29,
    UserScriptsDisabled = 30,
    VanillaAccountMailInstant = 91,
    VanillaNpcKnockback = 16,
    VanillaRageGenerationModifier = 21,
    WorldMapDisabled = 145,
    WorldMapFrameStrata = 100,
    WorldMapHelpPlateDisabled = 70,
    WorldMapLegendDisabled = 99,
    WorldMapTrackingOptionsDisabled = 68,
    WorldMapTrackingPinDisabled = 69,
  }
end

if not Enum.GameRuleFlags then
  Enum.GameRuleFlags = {
    AllowClient = 1,
    None = 0,
    RequiresDefault = 2,
  }
end

if not Enum.GameRuleFlagsMeta then
  Enum.GameRuleFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.GameRuleMeta then
  Enum.GameRuleMeta = {
    MaxValue = 190,
    MinValue = 1,
    NumValues = 140,
  }
end

if not Enum.GameRuleType then
  Enum.GameRuleType = {
    Bool = 2,
    Float = 1,
    Int = 0,
  }
end

if not Enum.GameRuleTypeMeta then
  Enum.GameRuleTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.GarrAutoBoardIndex then
  Enum.GarrAutoBoardIndex = {
    AllyCenterFront = 3,
    AllyLeftBack = 0,
    AllyLeftFront = 2,
    AllyRightBack = 1,
    AllyRightFront = 4,
    EnemyCenterLeftBack = 10,
    EnemyCenterLeftFront = 6,
    EnemyCenterRightBack = 11,
    EnemyCenterRightFront = 7,
    EnemyLeftBack = 9,
    EnemyLeftFront = 5,
    EnemyRightBack = 12,
    EnemyRightFront = 8,
    None = -1,
  }
end

if not Enum.GarrAutoBoardIndexMeta then
  Enum.GarrAutoBoardIndexMeta = {
    MaxValue = 12,
    MinValue = -1,
    NumValues = 14,
  }
end

if not Enum.GarrAutoCombatSpellTutorialFlag then
  Enum.GarrAutoCombatSpellTutorialFlag = {
    All = 4,
    Column = 2,
    None = 0,
    Row = 3,
    Single = 1,
  }
end

if not Enum.GarrAutoCombatSpellTutorialFlagMeta then
  Enum.GarrAutoCombatSpellTutorialFlagMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.GarrAutoCombatTutorial then
  Enum.GarrAutoCombatTutorial = {
    AttackAll = 256,
    AttackColumn = 64,
    AttackRow = 128,
    AttackSingle = 32,
    BeneficialEffect = 16,
    EnvironmentalEffect = 1024,
    HealCompanion = 4,
    LevelHeal = 8,
    PlaceCompanion = 2,
    SelectMission = 1,
    TroopTutorial = 512,
  }
end

if not Enum.GarrAutoCombatTutorialMeta then
  Enum.GarrAutoCombatTutorialMeta = {
    MaxValue = 1024,
    MinValue = 1,
    NumValues = 11,
  }
end

if not Enum.GarrAutoCombatantRole then
  Enum.GarrAutoCombatantRole = {
    HealSupport = 4,
    Melee = 1,
    None = 0,
    RangedMagic = 3,
    RangedPhysical = 2,
    Tank = 5,
  }
end

if not Enum.GarrAutoCombatantRoleMeta then
  Enum.GarrAutoCombatantRoleMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.GarrAutoEventFlags then
  Enum.GarrAutoEventFlags = {
    AutoAttack = 1,
    Environment = 4,
    None = 0,
    Passive = 2,
  }
end

if not Enum.GarrAutoEventFlagsMeta then
  Enum.GarrAutoEventFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GarrAutoMissionEventTypeMeta then
  Enum.GarrAutoMissionEventTypeMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.GarrAutoPreviewTargetType then
  Enum.GarrAutoPreviewTargetType = {
    Buff = 4,
    Damage = 1,
    Debuff = 8,
    Heal = 2,
    None = 0,
  }
end

if not Enum.GarrAutoPreviewTargetTypeMeta then
  Enum.GarrAutoPreviewTargetTypeMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.GarrFollowerMissionCompleteState then
  Enum.GarrFollowerMissionCompleteState = {
    Alive = 0,
    KilledByMissionFailure = 1,
    OutOfDurability = 3,
    SavedByPreventDeath = 2,
  }
end

if not Enum.GarrFollowerMissionCompleteStateMeta then
  Enum.GarrFollowerMissionCompleteStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GarrFollowerQuality then
  Enum.GarrFollowerQuality = {
    Common = 1,
    Epic = 4,
    Legendary = 5,
    None = 0,
    Rare = 3,
    Title = 6,
    Uncommon = 2,
  }
end

if not Enum.GarrFollowerQualityMeta then
  Enum.GarrFollowerQualityMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.GarrTalentCostType then
  Enum.GarrTalentCostType = {
    Initial = 0,
    MakePermanent = 2,
    Respec = 1,
    TreeReset = 3,
  }
end

if not Enum.GarrTalentCostTypeMeta then
  Enum.GarrTalentCostTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GarrTalentFeatureSubtype then
  Enum.GarrTalentFeatureSubtype = {
    Ardenweald = 3,
    Bastion = 1,
    Generic = 0,
    Maldraxxus = 4,
    Revendreth = 2,
  }
end

if not Enum.GarrTalentFeatureSubtypeMeta then
  Enum.GarrTalentFeatureSubtypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.GarrTalentFeatureType then
  Enum.GarrTalentFeatureType = {
    Adventures = 3,
    AnimaDiversion = 1,
    AnimaDiversionMap = 7,
    Cyphers = 8,
    Generic = 0,
    ReservoirUpgrades = 4,
    SanctumUnique = 5,
    SoulBinds = 6,
    TravelPortals = 2,
  }
end

if not Enum.GarrTalentFeatureTypeMeta then
  Enum.GarrTalentFeatureTypeMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.GarrTalentResearchCostSource then
  Enum.GarrTalentResearchCostSource = {
    Talent = 0,
    Tree = 1,
  }
end

if not Enum.GarrTalentResearchCostSourceMeta then
  Enum.GarrTalentResearchCostSourceMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.GarrTalentSocketType then
  Enum.GarrTalentSocketType = {
    Conduit = 2,
    None = 0,
    Spell = 1,
  }
end

if not Enum.GarrTalentSocketTypeMeta then
  Enum.GarrTalentSocketTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.GarrTalentTreeType then
  Enum.GarrTalentTreeType = {
    Classic = 1,
    Tiers = 0,
  }
end

if not Enum.GarrTalentTreeTypeMeta then
  Enum.GarrTalentTreeTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.GarrTalentType then
  Enum.GarrTalentType = {
    Major = 2,
    Minor = 1,
    Socket = 3,
    Standard = 0,
  }
end

if not Enum.GarrTalentTypeMeta then
  Enum.GarrTalentTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GarrTalentUI then
  Enum.GarrTalentUI = {
    AnimaDiversionMap = 3,
    CovenantSanctum = 1,
    Generic = 0,
    SoulBinds = 2,
  }
end

if not Enum.GarrTalentUIMeta then
  Enum.GarrTalentUIMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GarrisonFollowerTypeMeta then
  Enum.GarrisonFollowerTypeMeta = {
    MaxValue = 123,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.GarrisonTalentAvailability then
  Enum.GarrisonTalentAvailability = {
    Available = 0,
    Unavailable = 1,
    UnavailableAlreadyHave = 7,
    UnavailableAnotherIsResearching = 2,
    UnavailableNotEnoughGold = 4,
    UnavailableNotEnoughResources = 3,
    UnavailablePlayerCondition = 6,
    UnavailableRequiresPrerequisiteTalent = 8,
    UnavailableTierUnavailable = 5,
  }
end

if not Enum.GarrisonTalentAvailabilityMeta then
  Enum.GarrisonTalentAvailabilityMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.GarrisonTypeMeta then
  Enum.GarrisonTypeMeta = {
    MaxValue = 111,
    MinValue = 2,
    NumValues = 4,
  }
end

if not Enum.GossipNpcOption then
  Enum.GossipNpcOption = {
    AccountBanker = 57,
    AdventureMap = 31,
    ArtifactRespec = 21,
    Auctioneer = 10,
    AzeriteRespec = 35,
    Banker = 6,
    BarbersChoice = 52,
    Battlemaster = 9,
    Binder = 5,
    BlackMarketAuctionHouse = 46,
    CemeterySelect = 22,
    CharacterBanker = 56,
    ChromieTimeNpc = 40,
    ContributionCollector = 33,
    CovenantPreviewNpc = 41,
    CovenantRenownNpc = 45,
    DisableXPGain = 16,
    EnableXPGain = 17,
    ForgeMaster = 55,
    GarrisonArchitect = 26,
    GarrisonMissionNpc = 27,
    GarrisonRecruitment = 30,
    GarrisonTalent = 32,
    GarrisonTradeskillNpc = 29,
    GlyphMaster = 24,
    GuildBanker = 14,
    GuildRename = 62,
    GuildTabardVendor = 8,
    HouseFinder = 65,
    HousingCreateGuildNeighborhood = 60,
    HousingGetNeighborhoodCharter = 61,
    HousingOpenCharterConfirmation = 63,
    IslandsMissionNpc = 36,
    ItemUpgrade = 64,
    LFGDungeon = 20,
    Mailbox = 18,
    MajorFactionRenown = 53,
    NewPlayerGuide = 43,
    None = 0,
    PerksProgramVendor = 47,
    PersonalTabardVendor = 54,
    PetSpecializationMaster = 13,
    PetitionVendor = 7,
    ProfessionRespec = 58,
    ProfessionsCraftingOrder = 48,
    ProfessionsCustomerOrder = 50,
    ProfessionsOpen = 49,
    QueueScenario = 25,
    RenameNeighborhood = 59,
    RuneforgeLegendaryCrafting = 42,
    RuneforgeLegendaryUpgrade = 44,
    ShipmentCrafter = 28,
    Soulbind = 39,
    SpecializationMaster = 23,
    Spellclick = 15,
    SpiritHealer = 4,
    Stablemaster = 12,
    TalentMaster = 11,
    Taxinode = 2,
    TieredEntrance = 66,
    Trainer = 3,
    TraitSystem = 51,
    Transmogrify = 34,
    UIItemInteraction = 37,
    Vendor = 1,
    WorldMap = 38,
    WorldPvPQueue = 19,
  }
end

if not Enum.GossipNpcOptionDisplayFlags then
  Enum.GossipNpcOptionDisplayFlags = {
    ForceInteractionOnSingleChoice = 1,
  }
end

if not Enum.GossipNpcOptionDisplayFlagsMeta then
  Enum.GossipNpcOptionDisplayFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.GossipNpcOptionMeta then
  Enum.GossipNpcOptionMeta = {
    MaxValue = 66,
    MinValue = 0,
    NumValues = 67,
  }
end

if not Enum.GossipOptionRecFlags then
  Enum.GossipOptionRecFlags = {
    HideOptionIDFromClient = 2,
    PlayMovieLabelPrepend = 4,
    QuestLabelPrepend = 1,
  }
end

if not Enum.GossipOptionRecFlagsMeta then
  Enum.GossipOptionRecFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.GossipOptionRewardType then
  Enum.GossipOptionRewardType = {
    Currency = 1,
    Item = 0,
  }
end

if not Enum.GossipOptionRewardTypeMeta then
  Enum.GossipOptionRewardTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.GossipOptionStatus then
  Enum.GossipOptionStatus = {
    AlreadyComplete = 3,
    Available = 0,
    Locked = 2,
    Unavailable = 1,
  }
end

if not Enum.GossipOptionStatusMeta then
  Enum.GossipOptionStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.GossipOptionUIWidgetSetTypes then
  Enum.GossipOptionUIWidgetSetTypes = {
    Background = 1,
    Modifiers = 0,
  }
end

if not Enum.GossipOptionUIWidgetSetTypesMeta then
  Enum.GossipOptionUIWidgetSetTypesMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.GraphicsValidationResult then
  Enum.GraphicsValidationResult = {
    AmdGpuUnsupported = 36,
    AppleGpuUnsupported = 35,
    CompatMode = 41,
    CpuMem_2 = 6,
    CpuMem_4 = 7,
    CpuMem_8 = 8,
    DualCore = 4,
    Dx11Unsupported = 30,
    Dx12Win7Unsupported = 31,
    GpuDriver = 40,
    Graphics = 3,
    Illegal = 1,
    IntelGpuUnsupported = 37,
    LegacyUnsupported = 29,
    MacOsUnsupported = 27,
    NeedsAmdGpu = 15,
    NeedsAppleGpu = 14,
    NeedsDx12 = 12,
    NeedsDx12Vrs2 = 13,
    NeedsIntelGpu = 16,
    NeedsMacOs_10_13 = 19,
    NeedsMacOs_10_14 = 20,
    NeedsMacOs_10_15 = 21,
    NeedsMacOs_11_0 = 22,
    NeedsMacOs_12_0 = 23,
    NeedsMacOs_13_0 = 24,
    NeedsNvidiaGpu = 17,
    NeedsQualcommGpu = 18,
    NeedsRt = 11,
    NeedsWindows_10 = 25,
    NeedsWindows_11 = 26,
    Needs_5_0 = 9,
    Needs_6_0 = 10,
    NvapiWineUnsupported = 34,
    NvidiaGpuUnsupported = 38,
    QuadCore = 5,
    QualcommGpuUnsupported = 39,
    RemoteDesktopUnsupported = 32,
    Supported = 0,
    Unknown = 42,
    Unsupported = 2,
    WindowsUnsupported = 28,
    WineUnsupported = 33,
  }
end

if not Enum.GraphicsValidationResultMeta then
  Enum.GraphicsValidationResultMeta = {
    MaxValue = 42,
    MinValue = 0,
    NumValues = 43,
  }
end

if not Enum.GuildErrorTypeMeta then
  Enum.GuildErrorTypeMeta = {
    MaxValue = 52,
    MinValue = 0,
    NumValues = 53,
  }
end

if not Enum.HolidayCalendarFlags then
  Enum.HolidayCalendarFlags = {
    Alliance = 1,
    Horde = 2,
  }
end

if not Enum.HolidayCalendarFlagsMeta then
  Enum.HolidayCalendarFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.HolidayFlags then
  Enum.HolidayFlags = {
    BeginEventOnlyOnStageChange = 64,
    DontDisplayBanner = 8,
    DontDisplayEnd = 4,
    DontShowInCalendar = 2,
    DurationUseMinutes = 32,
    IsRegionwide = 1,
    NotAvailableClientSide = 16,
  }
end

if not Enum.HolidayFlagsMeta then
  Enum.HolidayFlagsMeta = {
    MaxValue = 64,
    MinValue = 1,
    NumValues = 7,
  }
end

if not Enum.HouseEditingContext then
  Enum.HouseEditingContext = {
    Decor = 1,
    Fixture = 3,
    None = 0,
    Room = 2,
  }
end

if not Enum.HouseEditingContextMeta then
  Enum.HouseEditingContextMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HouseEditorMode then
  Enum.HouseEditorMode = {
    BasicDecor = 1,
    Cleanup = 5,
    Customize = 4,
    ExpertDecor = 2,
    ExteriorCustomization = 6,
    Layout = 3,
    None = 0,
  }
end

if not Enum.HouseEditorModeMeta then
  Enum.HouseEditorModeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.HouseExteriorWMODataFlags then
  Enum.HouseExteriorWMODataFlags = {
    AllowedInAllianceNeighborhoods = 4,
    AllowedInHordeNeighborhoods = 2,
    None = 0,
    UnlockedByDefault = 1,
  }
end

if not Enum.HouseExteriorWMODataFlagsMeta then
  Enum.HouseExteriorWMODataFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HouseFinderSuggestionReason then
  Enum.HouseFinderSuggestionReason = {
    BNetFriends = 8,
    CharterInvite = 2,
    Guild = 4,
    None = 0,
    Owner = 1,
    PartySync = 16,
    Random = 32,
  }
end

if not Enum.HouseFinderSuggestionReasonMeta then
  Enum.HouseFinderSuggestionReasonMeta = {
    MaxValue = 32,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.HouseLevelRewardType then
  Enum.HouseLevelRewardType = {
    Object = 1,
    Value = 0,
  }
end

if not Enum.HouseLevelRewardTypeMeta then
  Enum.HouseLevelRewardTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HouseLevelRewardValueType then
  Enum.HouseLevelRewardValueType = {
    ExteriorDecor = 0,
    Fixtures = 3,
    InteriorDecor = 1,
    Rooms = 2,
  }
end

if not Enum.HouseLevelRewardValueTypeMeta then
  Enum.HouseLevelRewardValueTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HouseOwnerError then
  Enum.HouseOwnerError = {
    Faction = 1,
    GenericPermission = 3,
    Guild = 2,
    None = 0,
  }
end

if not Enum.HouseOwnerErrorMeta then
  Enum.HouseOwnerErrorMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HouseSettingFlags then
  Enum.HouseSettingFlags = {
    HouseAccessAnyone = 1,
    HouseAccessFriends = 8,
    HouseAccessGuild = 4,
    HouseAccessNeighbors = 2,
    HouseAccessParty = 16,
    None = 0,
    PlotAccessAnyone = 32,
    PlotAccessFriends = 256,
    PlotAccessGuild = 128,
    PlotAccessNeighbors = 64,
    PlotAccessParty = 512,
  }
end

if not Enum.HouseSettingFlagsMeta then
  Enum.HouseSettingFlagsMeta = {
    MaxValue = 512,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.HouseVisitType then
  Enum.HouseVisitType = {
    Friend = 1,
    Guild = 2,
    Party = 3,
    Unknown = 0,
  }
end

if not Enum.HouseVisitTypeMeta then
  Enum.HouseVisitTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HousingBasicModeTargetType then
  Enum.HousingBasicModeTargetType = {
    Decor = 1,
    House = 2,
    None = 0,
  }
end

if not Enum.HousingBasicModeTargetTypeMeta then
  Enum.HousingBasicModeTargetTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingCatalogEntryModelScenePresets then
  Enum.HousingCatalogEntryModelScenePresets = {
    DecorCeiling = 1338,
    DecorDefault = 1317,
    DecorFlat = 1318,
    DecorHorizontalSurface = 1452,
    DecorHuge = 1337,
    DecorLarge = 1336,
    DecorMedium = 1335,
    DecorSmall = 1334,
    DecorTiny = 1333,
    DecorWall = 1339,
  }
end

if not Enum.HousingCatalogEntryModelScenePresetsMeta then
  Enum.HousingCatalogEntryModelScenePresetsMeta = {
    MaxValue = 1452,
    MinValue = 1317,
    NumValues = 10,
  }
end

if not Enum.HousingCatalogEntrySize then
  Enum.HousingCatalogEntrySize = {
    Huge = 69,
    Large = 68,
    Medium = 67,
    None = 0,
    Small = 66,
    Tiny = 65,
  }
end

if not Enum.HousingCatalogEntrySizeMeta then
  Enum.HousingCatalogEntrySizeMeta = {
    MaxValue = 69,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.HousingCatalogEntrySubtype then
  Enum.HousingCatalogEntrySubtype = {
    Invalid = 0,
    OwnedModifiedStack = 2,
    OwnedUnmodifiedStack = 3,
    Unowned = 1,
  }
end

if not Enum.HousingCatalogEntrySubtypeMeta then
  Enum.HousingCatalogEntrySubtypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HousingCatalogEntryType then
  Enum.HousingCatalogEntryType = {
    Decor = 1,
    Invalid = 0,
    Room = 2,
  }
end

if not Enum.HousingCatalogEntryTypeMeta then
  Enum.HousingCatalogEntryTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingCatalogSortType then
  Enum.HousingCatalogSortType = {
    Alphabetical = 1,
    DateAdded = 0,
  }
end

if not Enum.HousingCatalogSortTypeMeta then
  Enum.HousingCatalogSortTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingCleanupModeTargetType then
  Enum.HousingCleanupModeTargetType = {
    Decor = 1,
    HouseExterior = 2,
    None = 0,
  }
end

if not Enum.HousingCleanupModeTargetTypeMeta then
  Enum.HousingCleanupModeTargetTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingCustomizeModeTargetType then
  Enum.HousingCustomizeModeTargetType = {
    Decor = 1,
    ExteriorHouse = 3,
    None = 0,
    RoomComponent = 2,
  }
end

if not Enum.HousingCustomizeModeTargetTypeMeta then
  Enum.HousingCustomizeModeTargetTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HousingDecorActionFlags then
  Enum.HousingDecorActionFlags = {
    Add = 1,
    ClickTarget = 16,
    DragMove = 4,
    HoverTarget = 32,
    IncludeTargetChildren = 512,
    MaintainLastTarget = 256,
    None = 0,
    PrecisionMove = 8,
    PreviewDecor = 2048,
    Remove = 2,
    TargetHouseExterior = 128,
    TargetRoomComponents = 64,
    UsePlacedDecorList = 1024,
  }
end

if not Enum.HousingDecorActionFlagsMeta then
  Enum.HousingDecorActionFlagsMeta = {
    MaxValue = 2048,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.HousingDecorModelType then
  Enum.HousingDecorModelType = {
    M2 = 1,
    None = 0,
    WMO = 2,
  }
end

if not Enum.HousingDecorModelTypeMeta then
  Enum.HousingDecorModelTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingDecorPlacementRestriction then
  Enum.HousingDecorPlacementRestriction = {
    ChildOutsideBounds = 8,
    InvalidCollision = 32,
    InvalidTarget = 16,
    OutsidePlotBounds = 4,
    OutsideRoomBounds = 2,
    TooFarAway = 1,
  }
end

if not Enum.HousingDecorPlacementRestrictionMeta then
  Enum.HousingDecorPlacementRestrictionMeta = {
    MaxValue = 32,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.HousingDecorTheme then
  Enum.HousingDecorTheme = {
    BloodElf = 5,
    Folk = 1,
    Generic = 3,
    NightElf = 4,
    None = 0,
    Rugged = 2,
  }
end

if not Enum.HousingDecorThemeMeta then
  Enum.HousingDecorThemeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.HousingDecorType then
  Enum.HousingDecorType = {
    Ceiling = 3,
    Floor = 1,
    Flooring = 4,
    None = 0,
    Wall = 2,
  }
end

if not Enum.HousingDecorTypeMeta then
  Enum.HousingDecorTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingExpertModeTargetType then
  Enum.HousingExpertModeTargetType = {
    Decor = 1,
    House = 2,
    None = 0,
  }
end

if not Enum.HousingExpertModeTargetTypeMeta then
  Enum.HousingExpertModeTargetTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingExpertSubmodeRestrictionMeta then
  Enum.HousingExpertSubmodeRestrictionMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HousingFavorUpdateSource then
  Enum.HousingFavorUpdateSource = {
    DecorCollection = 1,
    DeferredRewards = 2,
    InitiativeChest = 6,
    InitiativeTask = 5,
    NewHouseDecorFavor = 4,
    Quest = 7,
    RetroactiveDecor = 3,
    Unknown = 0,
  }
end

if not Enum.HousingFavorUpdateSourceMeta then
  Enum.HousingFavorUpdateSourceMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.HousingFavorUpdateType then
  Enum.HousingFavorUpdateType = {
    Add = 0,
    InitiativeAdd = 1,
    Set = 2,
  }
end

if not Enum.HousingFavorUpdateTypeMeta then
  Enum.HousingFavorUpdateTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingFixtureFlags then
  Enum.HousingFixtureFlags = {
    IsDefaultFixture = 1,
    None = 0,
    UnlockedByDefault = 2,
  }
end

if not Enum.HousingFixtureFlagsMeta then
  Enum.HousingFixtureFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingFixtureSize then
  Enum.HousingFixtureSize = {
    Any = 1,
    Large = 4,
    Medium = 3,
    None = 0,
    Small = 2,
  }
end

if not Enum.HousingFixtureSizeMeta then
  Enum.HousingFixtureSizeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingFixtureType then
  Enum.HousingFixtureType = {
    Base = 9,
    Chimney = 16,
    Door = 11,
    None = 0,
    Roof = 10,
    RoofDetail = 13,
    RoofWindow = 14,
    Tower = 15,
    Window = 12,
  }
end

if not Enum.HousingFixtureTypeMeta then
  Enum.HousingFixtureTypeMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.HousingIncrementType then
  Enum.HousingIncrementType = {
    Back = 8,
    Down = 32,
    Forward = 4,
    Left = 1,
    Right = 2,
    RotateLeft = 64,
    RotateRight = 128,
    ScaleDown = 512,
    ScaleUp = 256,
    Up = 16,
  }
end

if not Enum.HousingIncrementTypeMeta then
  Enum.HousingIncrementTypeMeta = {
    MaxValue = 512,
    MinValue = 1,
    NumValues = 10,
  }
end

if not Enum.HousingItemToastType then
  Enum.HousingItemToastType = {
    Customization = 2,
    Decor = 3,
    Fixture = 1,
    House = 4,
    Room = 0,
  }
end

if not Enum.HousingItemToastTypeMeta then
  Enum.HousingItemToastTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingLayoutCameraDirection then
  Enum.HousingLayoutCameraDirection = {
    Down = 2,
    Left = 4,
    None = 0,
    Right = 8,
    Up = 1,
  }
end

if not Enum.HousingLayoutCameraDirectionMeta then
  Enum.HousingLayoutCameraDirectionMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingLayoutPinType then
  Enum.HousingLayoutPinType = {
    Door = 0,
    Room = 1,
  }
end

if not Enum.HousingLayoutPinTypeMeta then
  Enum.HousingLayoutPinTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingLayoutRestrictionMeta then
  Enum.HousingLayoutRestrictionMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.HousingLayoutStairDirection then
  Enum.HousingLayoutStairDirection = {
    Down = 1,
    Up = 0,
  }
end

if not Enum.HousingLayoutStairDirectionMeta then
  Enum.HousingLayoutStairDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingPlotOwnerTypeMeta then
  Enum.HousingPlotOwnerTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.HousingPrecisionSubmode then
  Enum.HousingPrecisionSubmode = {
    Rotate = 1,
    Scale = 2,
    Translate = 0,
  }
end

if not Enum.HousingPrecisionSubmodeMeta then
  Enum.HousingPrecisionSubmodeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingResult then
  Enum.HousingResult = {
    AccountBanned = 1,
    ActionLockedByCombat = 2,
    BlueprintCodeInvalid = 3,
    BlueprintDyeFailed = 4,
    BlueprintGenericExportError = 5,
    BlueprintGenericImportError = 6,
    BlueprintLocationInvalid = 7,
    BlueprintNameInvalid = 8,
    BlueprintNotFound = 9,
    BlueprintRequirementsUnmet = 10,
    BlueprintRoomPlacementRequired = 11,
    BlueprintStorageLimit = 14,
    BlueprintTypeInvalid = 12,
    BlueprintTypeLocationInvalid = 13,
    BlueprintVersionInvalid = 15,
    BoundToStartingArea = 19,
    BoundsFailureChildren = 16,
    BoundsFailurePlot = 17,
    BoundsFailureRoom = 18,
    CannotAfford = 20,
    CharterComplete = 21,
    CollisionInvalid = 22,
    DbError = 23,
    DecorCannotBeRedeemed = 24,
    DecorItemNotDestroyable = 25,
    DecorNotFound = 26,
    DecorNotFoundInStorage = 27,
    DuplicateCharterSignature = 28,
    FilterRejected = 29,
    FixtureCantDeleteDoor = 30,
    FixtureHookEmpty = 31,
    FixtureHookOccupied = 32,
    FixtureHouseTypeMismatch = 33,
    FixtureNotFound = 34,
    FixtureSizeMismatch = 35,
    FixtureTypeMismatch = 36,
    GenericFailure = 37,
    GuildMoreAccountsNeeded = 38,
    GuildMoreActivePlayersNeeded = 39,
    GuildNotLoaded = 40,
    HookNotChildOfFixture = 49,
    HouseEditLockFailed = 41,
    HouseExteriorAlreadyThatSize = 42,
    HouseExteriorAlreadyThatType = 43,
    HouseExteriorRootNotFound = 44,
    HouseExteriorSizeNotAvailable = 48,
    HouseExteriorTypeNeighborhoodMismatch = 45,
    HouseExteriorTypeNotFound = 46,
    HouseExteriorTypeSizeMismatch = 47,
    HouseNotFound = 50,
    IncorrectFaction = 51,
    InsufficientRoomBudget = 64,
    InvalidDecorItem = 52,
    InvalidDistance = 53,
    InvalidExteriorDocument = 54,
    InvalidGuild = 55,
    InvalidHouse = 56,
    InvalidInstance = 57,
    InvalidInteraction = 58,
    InvalidInteriorDocument = 59,
    InvalidLightOverlap = 60,
    InvalidMap = 61,
    InvalidNeighborhoodName = 62,
    InvalidRoomLayout = 63,
    LockOperationFailed = 66,
    LockedByOtherPlayer = 65,
    MaxPetDecorReached = 68,
    MaxPlacedDecorReached = 67,
    MaxPreviewDecorReached = 69,
    MaxStorageDecorReached = 70,
    MissingCoreFixture = 71,
    MissingDye = 72,
    MissingExpansionAccess = 73,
    MissingFactionMap = 74,
    MissingPrivateNeighborhoodInvite = 75,
    MoreHouseSlotsNeeded = 76,
    MoreSignaturesNeeded = 77,
    NeighborhoodNotFound = 78,
    NoNeighborhoodOwnershipRequests = 79,
    NotInDecorEditMode = 80,
    NotInFixtureEditMode = 81,
    NotInLayoutEditMode = 82,
    NotInsideHouse = 83,
    NotOnOwnedPlot = 84,
    OperationAborted = 85,
    OwnerNotInGuild = 86,
    PermissionDenied = 87,
    PlacementTargetInvalid = 88,
    PlayerNotFound = 89,
    PlayerNotInInstance = 90,
    PlotNotFound = 91,
    PlotNotVacant = 92,
    PlotReservationCooldown = 93,
    PlotReserved = 94,
    RoomNotFound = 95,
    RoomPlacementOutOfBounds = 96,
    RoomUpdateFailed = 97,
    RpcFailure = 98,
    ServiceNotAvailable = 99,
    StaticDataNotFound = 100,
    Success = 0,
    TimeoutLimit = 101,
    TimerunningNotAllowed = 102,
    TokenRequired = 103,
    TooManyRequests = 104,
    TransactionFailure = 105,
    UncollectedExteriorFixture = 106,
    UncollectedHouseType = 107,
    UncollectedRoom = 108,
    UncollectedRoomMaterial = 109,
    UncollectedRoomTheme = 110,
    UnlockOperationFailed = 111,
  }
end

if not Enum.HousingResultMeta then
  Enum.HousingResultMeta = {
    MaxValue = 111,
    MinValue = 0,
    NumValues = 112,
  }
end

if not Enum.HousingRoomComponentCeilingType then
  Enum.HousingRoomComponentCeilingType = {
    Flat = 0,
    Vaulted = 1,
  }
end

if not Enum.HousingRoomComponentCeilingTypeMeta then
  Enum.HousingRoomComponentCeilingTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingRoomComponentDoorType then
  Enum.HousingRoomComponentDoorType = {
    Doorway = 1,
    None = 0,
    Threshold = 2,
  }
end

if not Enum.HousingRoomComponentDoorTypeMeta then
  Enum.HousingRoomComponentDoorTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingRoomComponentFlags then
  Enum.HousingRoomComponentFlags = {
    HiddenInLayoutMode = 1,
    None = 0,
  }
end

if not Enum.HousingRoomComponentFlagsMeta then
  Enum.HousingRoomComponentFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingRoomComponentFloorType then
  Enum.HousingRoomComponentFloorType = {
    Floor = 0,
  }
end

if not Enum.HousingRoomComponentFloorTypeMeta then
  Enum.HousingRoomComponentFloorTypeMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.HousingRoomComponentOptionFlags then
  Enum.HousingRoomComponentOptionFlags = {
    IsDefault = 1,
    None = 0,
  }
end

if not Enum.HousingRoomComponentOptionFlagsMeta then
  Enum.HousingRoomComponentOptionFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingRoomComponentOptionType then
  Enum.HousingRoomComponentOptionType = {
    Cosmetic = 0,
    Doorway = 2,
    DoorwayWall = 1,
  }
end

if not Enum.HousingRoomComponentOptionTypeMeta then
  Enum.HousingRoomComponentOptionTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingRoomComponentStairType then
  Enum.HousingRoomComponentStairType = {
    MiddleToEnd = 4,
    MiddleToMiddle = 3,
    None = 0,
    StartToEnd = 1,
    StartToMiddle = 2,
  }
end

if not Enum.HousingRoomComponentStairTypeMeta then
  Enum.HousingRoomComponentStairTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingRoomComponentTextureFlags then
  Enum.HousingRoomComponentTextureFlags = {
    None = 0,
    UnlockedByDefault = 1,
  }
end

if not Enum.HousingRoomComponentTextureFlagsMeta then
  Enum.HousingRoomComponentTextureFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.HousingRoomComponentType then
  Enum.HousingRoomComponentType = {
    Ceiling = 3,
    Doorway = 7,
    DoorwayWall = 6,
    Floor = 2,
    None = 0,
    Pillar = 5,
    Stairs = 4,
    Wall = 1,
  }
end

if not Enum.HousingRoomComponentTypeMeta then
  Enum.HousingRoomComponentTypeMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.HousingRoomFlags then
  Enum.HousingRoomFlags = {
    BaseRoom = 1,
    HasCustomGeometry = 8,
    HasStairs = 2,
    None = 0,
    UnlockedByDefault = 4,
  }
end

if not Enum.HousingRoomFlagsMeta then
  Enum.HousingRoomFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.HousingTeleportReason then
  Enum.HousingTeleportReason = {
    Booted = 3,
    Cheat = 1,
    ExitingHouse = 9,
    Friend = 6,
    GuildMember = 7,
    Homestone = 4,
    None = 0,
    PartyMember = 8,
    Portal = 10,
    Tutorial = 11,
    UnspecifiedSpellcast = 2,
    Visit = 5,
  }
end

if not Enum.HousingTeleportReasonMeta then
  Enum.HousingTeleportReasonMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.HousingThemeFlags then
  Enum.HousingThemeFlags = {
    None = 0,
    ShowInStyleSelector = 2,
    UnlockedByDefault = 1,
  }
end

if not Enum.HousingThemeFlagsMeta then
  Enum.HousingThemeFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.HousingThrottleType then
  Enum.HousingThrottleType = {
    Decoration = 1,
    General = 0,
  }
end

if not Enum.HousingThrottleTypeMeta then
  Enum.HousingThrottleTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.IconAndTextShiftTextType then
  Enum.IconAndTextShiftTextType = {
    None = 0,
    ShiftText = 1,
  }
end

if not Enum.IconAndTextShiftTextTypeMeta then
  Enum.IconAndTextShiftTextTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.IconAndTextWidgetStateMeta then
  Enum.IconAndTextWidgetStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.IconState then
  Enum.IconState = {
    Hidden = 0,
    ShowState1 = 1,
    ShowState2 = 2,
  }
end

if not Enum.IconStateMeta then
  Enum.IconStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ImageSharingResult then
  Enum.ImageSharingResult = {
    AccountDataRequestFailure = 6,
    EncryptionFailure = 11,
    EncryptionSecretNotSet = 14,
    FailedToCreateHeader = 10,
    Failure = 0,
    GetAccountDataRequestFailure = 7,
    InvalidOAuthExpiration = 12,
    InvalidRefreshExpiration = 13,
    MissingFieldApiError = 5,
    NotConfigured = 2,
    PlatformApiError = 4,
    RefreshTokenExpired = 9,
    RetrievedExisting = 3,
    SetAccountDataRequestFailure = 8,
    Success = 1,
  }
end

if not Enum.ImageSharingResultMeta then
  Enum.ImageSharingResultMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.InitiativeMilestoneFlags then
  Enum.InitiativeMilestoneFlags = {
    FinalMilestone = 1,
  }
end

if not Enum.InitiativeMilestoneFlagsMeta then
  Enum.InitiativeMilestoneFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.InitiativeRewardFlags then
  Enum.InitiativeRewardFlags = {
    PermanentWorldState = 1,
  }
end

if not Enum.InitiativeRewardFlagsMeta then
  Enum.InitiativeRewardFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.InputContext then
  Enum.InputContext = {
    GamePad = 3,
    Keyboard = 1,
    Mouse = 2,
    None = 0,
  }
end

if not Enum.InputContextMeta then
  Enum.InputContextMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.InvalidPlotScreenshotReasonMeta then
  Enum.InvalidPlotScreenshotReasonMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.InventoryType then
  Enum.InventoryType = {
    Index2HweaponType = 17,
    IndexAmmoType = 24,
    IndexBagType = 18,
    IndexBodyType = 4,
    IndexChestType = 5,
    IndexCloakType = 16,
    IndexEquipablespellDefensiveType = 33,
    IndexEquipablespellOffensiveType = 31,
    IndexEquipablespellUtilityType = 32,
    IndexEquipablespellWeaponType = 34,
    IndexFeetType = 8,
    IndexFingerType = 11,
    IndexHandType = 10,
    IndexHeadType = 1,
    IndexHoldableType = 23,
    IndexLegsType = 7,
    IndexNeckType = 2,
    IndexNonEquipType = 0,
    IndexProfessionGearType = 30,
    IndexProfessionToolType = 29,
    IndexQuiverType = 27,
    IndexRangedType = 15,
    IndexRangedrightType = 26,
    IndexRelicType = 28,
    IndexRobeType = 20,
    IndexShieldType = 14,
    IndexShoulderType = 3,
    IndexTabardType = 19,
    IndexThrownType = 25,
    IndexTrinketType = 12,
    IndexWaistType = 6,
    IndexWeaponType = 13,
    IndexWeaponmainhandType = 21,
    IndexWeaponoffhandType = 22,
    IndexWristType = 9,
  }
end

if not Enum.InventoryTypeMeta then
  Enum.InventoryTypeMeta = {
    MaxValue = 34,
    MinValue = 0,
    NumValues = 35,
  }
end

if not Enum.ItemArmorSubclass then
  Enum.ItemArmorSubclass = {
    Cloth = 1,
    Cosmetic = 5,
    Generic = 0,
    Idol = 8,
    Leather = 2,
    Libram = 7,
    Mail = 3,
    Plate = 4,
    Relic = 11,
    Shield = 6,
    Sigil = 10,
    Totem = 9,
  }
end

if not Enum.ItemArmorSubclassMeta then
  Enum.ItemArmorSubclassMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.ItemBind then
  Enum.ItemBind = {
    None = 0,
    OnAcquire = 1,
    OnEquip = 2,
    OnUse = 3,
    Quest = 4,
    ToBnetAccount = 8,
    ToBnetAccountUntilEquipped = 9,
    ToWoWAccount = 7,
    Unused1 = 5,
    Unused2 = 6,
  }
end

if not Enum.ItemBindMeta then
  Enum.ItemBindMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.ItemClass then
  Enum.ItemClass = {
    Armor = 4,
    Battlepet = 17,
    Consumable = 0,
    Container = 1,
    CurrencyTokenObsolete = 10,
    Gem = 3,
    Glyph = 16,
    Housing = 20,
    ItemEnhancement = 8,
    Key = 13,
    Miscellaneous = 15,
    PermanentObsolete = 14,
    Profession = 19,
    Projectile = 6,
    Questitem = 12,
    Quiver = 11,
    Reagent = 5,
    Recipe = 9,
    Tradegoods = 7,
    Weapon = 2,
    WoWToken = 18,
  }
end

if not Enum.ItemClassMeta then
  Enum.ItemClassMeta = {
    MaxValue = 20,
    MinValue = 0,
    NumValues = 21,
  }
end

if not Enum.ItemCollectionType then
  Enum.ItemCollectionType = {
    ExteriorFixture = 9,
    Heirloom = 2,
    HouseType = 13,
    None = 0,
    Room = 8,
    RoomMaterial = 11,
    RoomTheme = 10,
    RuneforgeLegendaryAbility = 5,
    Toy = 1,
    Transmog = 3,
    TransmogIllusion = 6,
    TransmogOutfit = 12,
    TransmogSetFavorite = 4,
    WarbandScene = 7,
  }
end

if not Enum.ItemCollectionTypeMeta then
  Enum.ItemCollectionTypeMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.ItemCommodityStatus then
  Enum.ItemCommodityStatus = {
    Commodity = 2,
    Item = 1,
    Unknown = 0,
  }
end

if not Enum.ItemCommodityStatusMeta then
  Enum.ItemCommodityStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ItemConsumableSubclass then
  Enum.ItemConsumableSubclass = {
    Bandage = 7,
    CombatCurio = 11,
    Elixir = 2,
    Flasksphials = 3,
    Fooddrink = 5,
    Generic = 0,
    Itemenhancement = 6,
    Other = 8,
    Potion = 1,
    Relic = 12,
    Scroll = 4,
    UtilityCurio = 10,
    VantusRune = 9,
  }
end

if not Enum.ItemConsumableSubclassMeta then
  Enum.ItemConsumableSubclassMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.ItemCreationContext then
  Enum.ItemCreationContext = {
    BlackMarket = 15,
    ChallengeModeJackpot = 35,
    ChallengeMode_1 = 16,
    ChallengeMode_2 = 33,
    ChallengeMode_3 = 34,
    ChallengeMode_4 = 87,
    CharacterBoost_1 = 61,
    CharacterBoost_2 = 62,
    CharacterBoost_3 = 86,
    CorpseRecovery = 80,
    DelvesBonus_1 = 129,
    DelvesBonus_10 = 138,
    DelvesBonus_2 = 130,
    DelvesBonus_3 = 131,
    DelvesBonus_4 = 132,
    DelvesBonus_5 = 133,
    DelvesBonus_6 = 134,
    DelvesBonus_7 = 135,
    DelvesBonus_8 = 136,
    DelvesBonus_9 = 137,
    DelvesBounty_1 = 117,
    DelvesBounty_2 = 118,
    DelvesBounty_3 = 119,
    DelvesBounty_4 = 120,
    DelvesBounty_5 = 121,
    DelvesBounty_6 = 122,
    DelvesBounty_7 = 123,
    DelvesBounty_8 = 124,
    DelvesJackpot = 108,
    DelvesKey_1 = 109,
    DelvesKey_2 = 110,
    DelvesKey_3 = 111,
    DelvesKey_4 = 112,
    DelvesKey_5 = 113,
    DelvesKey_6 = 114,
    DelvesKey_7 = 115,
    DelvesKey_8 = 116,
    DelvesLevelUp_1 = 125,
    DelvesLevelUp_2 = 126,
    DelvesLevelUp_3 = 127,
    DelvesLevelUp_4 = 128,
    Delves_1 = 104,
    Delves_2 = 106,
    Delves_3 = 107,
    DungeonBonus_1 = 139,
    DungeonBonus_10 = 148,
    DungeonBonus_2 = 140,
    DungeonBonus_3 = 141,
    DungeonBonus_4 = 142,
    DungeonBonus_5 = 143,
    DungeonBonus_6 = 144,
    DungeonBonus_7 = 145,
    DungeonBonus_8 = 146,
    DungeonBonus_9 = 147,
    DungeonHardMode_1 = 159,
    DungeonHardMode_2 = 160,
    DungeonHardMode_3 = 161,
    DungeonHeroic = 2,
    DungeonHeroicJackpot = 102,
    DungeonLevelUp_1 = 17,
    DungeonLevelUp_2 = 18,
    DungeonLevelUp_3 = 19,
    DungeonLevelUp_4 = 20,
    DungeonMythic = 23,
    DungeonMythicJackpot = 103,
    DungeonNormal = 1,
    DungeonNormalJackpot = 101,
    Endeavors = 185,
    ForceNone = 21,
    LegendaryCrafting_1 = 63,
    LegendaryCrafting_2 = 64,
    LegendaryCrafting_3 = 65,
    LegendaryCrafting_4 = 66,
    LegendaryCrafting_5 = 67,
    LegendaryCrafting_6 = 68,
    LegendaryCrafting_7 = 69,
    LegendaryCrafting_8 = 70,
    LegendaryCrafting_9 = 71,
    LegendaryForge = 59,
    MissionReward_1 = 31,
    MissionReward_2 = 32,
    NewCharacter = 75,
    None = 0,
    PvPBrawl_1 = 77,
    PvPBrawl_2 = 78,
    PvPHonorReward = 24,
    PvPRankedJackpot = 56,
    PvPRanked_1 = 8,
    PvPRanked_2 = 38,
    PvPRanked_3 = 39,
    PvPRanked_4 = 40,
    PvPRanked_5 = 44,
    PvPRanked_6 = 45,
    PvPRanked_7 = 46,
    PvPRanked_8 = 52,
    PvPRanked_9 = 88,
    PvPUnranked_1 = 7,
    PvPUnranked_2 = 41,
    PvPUnranked_3 = 47,
    PvPUnranked_4 = 48,
    PvPUnranked_5 = 49,
    PvPUnranked_6 = 50,
    PvPUnranked_7 = 51,
    QuestBonusLoot = 60,
    QuestReward = 11,
    RaidBonus_1 = 149,
    RaidBonus_10 = 158,
    RaidBonus_2 = 150,
    RaidBonus_3 = 151,
    RaidBonus_4 = 152,
    RaidBonus_5 = 153,
    RaidBonus_6 = 154,
    RaidBonus_7 = 155,
    RaidBonus_8 = 156,
    RaidBonus_9 = 157,
    RaidFinder = 4,
    RaidFinderExtended = 83,
    RaidFinderExtended_2 = 90,
    RaidFinderExtended_3 = 94,
    RaidHeroic = 5,
    RaidHeroicExtended = 84,
    RaidHeroicExtended_2 = 91,
    RaidHeroicExtended_3 = 95,
    RaidMythic = 6,
    RaidMythicExtended = 85,
    RaidMythicExtended_2 = 92,
    RaidMythicExtended_3 = 96,
    RaidNormal = 3,
    RaidNormalExtended = 82,
    RaidNormalExtended_2 = 89,
    RaidNormalExtended_3 = 93,
    Relinquished = 58,
    ScenarioHeroic = 10,
    ScenarioNormal = 9,
    Store = 12,
    TemplateCharacter_1 = 97,
    TemplateCharacter_2 = 98,
    TemplateCharacter_3 = 99,
    TemplateCharacter_4 = 100,
    Timerunning = 105,
    TimewalkerLevelUp = 22,
    TimewalkerMaxLevel = 186,
    Torghast = 79,
    TournamentRealm_1 = 57,
    TournamentRealm_2 = 162,
    TournamentRealm_3 = 163,
    TournamentRealm_4 = 164,
    TradeSkill = 13,
    Vendor = 14,
    WarMode = 76,
    Warbound_1 = 165,
    Warbound_10 = 174,
    Warbound_11 = 175,
    Warbound_12 = 176,
    Warbound_13 = 177,
    Warbound_14 = 178,
    Warbound_15 = 179,
    Warbound_16 = 180,
    Warbound_17 = 181,
    Warbound_18 = 182,
    Warbound_19 = 183,
    Warbound_2 = 166,
    Warbound_20 = 184,
    Warbound_3 = 167,
    Warbound_4 = 168,
    Warbound_5 = 169,
    Warbound_6 = 170,
    Warbound_7 = 171,
    Warbound_8 = 172,
    Warbound_9 = 173,
    WeeklyRewardsAdditional = 72,
    WeeklyRewardsConcession = 73,
    WorldBoss = 81,
    WorldQuestJackpot = 74,
    WorldQuest_1 = 25,
    WorldQuest_10 = 43,
    WorldQuest_11 = 53,
    WorldQuest_12 = 54,
    WorldQuest_13 = 55,
    WorldQuest_2 = 26,
    WorldQuest_3 = 27,
    WorldQuest_4 = 28,
    WorldQuest_5 = 29,
    WorldQuest_6 = 30,
    WorldQuest_7 = 36,
    WorldQuest_8 = 37,
    WorldQuest_9 = 42,
  }
end

if not Enum.ItemCreationContextMeta then
  Enum.ItemCreationContextMeta = {
    MaxValue = 186,
    MinValue = 0,
    NumValues = 187,
  }
end

if not Enum.ItemDisplayTextDisplayStyle then
  Enum.ItemDisplayTextDisplayStyle = {
    ItemNameAndInfoText = 1,
    ItemNameOnlyCentered = 2,
    PlayerChoiceReward = 3,
    WorldQuestReward = 0,
  }
end

if not Enum.ItemDisplayTextDisplayStyleMeta then
  Enum.ItemDisplayTextDisplayStyleMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ItemDisplayTooltipEnabledType then
  Enum.ItemDisplayTooltipEnabledType = {
    Disabled = 1,
    Enabled = 0,
  }
end

if not Enum.ItemDisplayTooltipEnabledTypeMeta then
  Enum.ItemDisplayTooltipEnabledTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ItemGemColor then
  Enum.ItemGemColor = {
    Arcane = 1024,
    Blood = 128,
    Blue = 8,
    Cogwheel = 32,
    Cypher = 8388608,
    DominationBlood = 1048576,
    DominationFrost = 2097152,
    DominationUnholy = 4194304,
    Fel = 512,
    Fiber = 1073741824,
    Fire = 4096,
    Fragrance = 67108864,
    Frost = 2048,
    Holy = 65536,
    Hydraulic = 16,
    Iron = 64,
    Life = 16384,
    Meta = 1,
    Primordial = 33554432,
    PunchcardBlue = 524288,
    PunchcardRed = 131072,
    PunchcardYellow = 262144,
    Red = 2,
    Shadow = 256,
    SingingSea = 268435456,
    SingingThunder = 134217728,
    SingingWind = 536870912,
    Tinker = 16777216,
    Water = 8192,
    Wind = 32768,
    Yellow = 4,
  }
end

if not Enum.ItemGemColorMeta then
  Enum.ItemGemColorMeta = {
    MaxValue = 1073741824,
    MinValue = 1,
    NumValues = 31,
  }
end

if not Enum.ItemGemSubclass then
  Enum.ItemGemSubclass = {
    Agility = 1,
    Artifactrelic = 11,
    Criticalstrike = 5,
    Haste = 7,
    Intellect = 0,
    Mastery = 6,
    Multiplestats = 10,
    Other = 9,
    Spirit = 4,
    Stamina = 3,
    Strength = 2,
    Versatility = 8,
  }
end

if not Enum.ItemGemSubclassMeta then
  Enum.ItemGemSubclassMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.ItemHousingSubclass then
  Enum.ItemHousingSubclass = {
    Decor = 0,
    Dye = 1,
    ExteriorCustomization = 4,
    Room = 2,
    RoomCustomization = 3,
    ServiceItem = 5,
  }
end

if not Enum.ItemHousingSubclassMeta then
  Enum.ItemHousingSubclassMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.ItemMiscellaneousSubclass then
  Enum.ItemMiscellaneousSubclass = {
    CompanionPet = 2,
    Holiday = 3,
    Junk = 0,
    Mount = 5,
    MountEquipment = 6,
    Other = 4,
    Reagent = 1,
  }
end

if not Enum.ItemMiscellaneousSubclassMeta then
  Enum.ItemMiscellaneousSubclassMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.ItemModification then
  Enum.ItemModification = {
    ArtifactAppearanceID = 8,
    ArtifactTier = 24,
    BattlePetBreed = 4,
    BattlePetCreaturedisplayid = 6,
    BattlePetLevel = 5,
    BattlePetSpecies = 3,
    ChangeModifiedCraftingStat_1 = 29,
    ChangeModifiedCraftingStat_2 = 30,
    ContentTuningID = 28,
    CraftingDataID = 40,
    CraftingQualityID = 38,
    CraftingReagentSlot_0 = 43,
    CraftingReagentSlot_1 = 44,
    CraftingReagentSlot_10 = 53,
    CraftingReagentSlot_11 = 54,
    CraftingReagentSlot_12 = 55,
    CraftingReagentSlot_13 = 56,
    CraftingReagentSlot_14 = 57,
    CraftingReagentSlot_2 = 45,
    CraftingReagentSlot_3 = 46,
    CraftingReagentSlot_4 = 47,
    CraftingReagentSlot_5 = 48,
    CraftingReagentSlot_6 = 49,
    CraftingReagentSlot_7 = 50,
    CraftingReagentSlot_8 = 51,
    CraftingReagentSlot_9 = 52,
    CraftingSkillLineAbilityID = 39,
    CraftingSkillReagents = 41,
    CraftingSkillWatermark = 42,
    CurrencyWalletID = 61,
    CurrencyWalletQuantity = 62,
    CurrencyWalletVersion = 63,
    DbidHigh = 59,
    DbidLow = 60,
    IncrementLevelObsolete = 2,
    KeystoneAffix0 = 19,
    KeystoneAffix01 = 20,
    KeystoneAffix02 = 21,
    KeystoneAffix03 = 22,
    KeystoneMapChallengeModeID = 17,
    KeystonePowerLevel = 18,
    LegionArtifactKnowledgeObsolete = 23,
    PvPRating = 26,
    Reforge = 58,
    SoulbindConduitRank = 37,
    TimewalkerLevel = 9,
    TransmogrifyItemModifiedAppearanceIDSpecAll = 0,
    TransmogrifyItemModifiedAppearanceIDSpec_0 = 1,
    TransmogrifyItemModifiedAppearanceIDSpec_1 = 11,
    TransmogrifyItemModifiedAppearanceIDSpec_2 = 13,
    TransmogrifyItemModifiedAppearanceIDSpec_3 = 15,
    TransmogrifyItemModifiedAppearanceIDSpec_4 = 25,
    TransmogrifyOverrideEnchantVisualIDSpecAll = 7,
    TransmogrifyOverrideEnchantVisualIDSpec_0 = 10,
    TransmogrifyOverrideEnchantVisualIDSpec_1 = 12,
    TransmogrifyOverrideEnchantVisualIDSpec_2 = 14,
    TransmogrifyOverrideEnchantVisualIDSpec_3 = 16,
    TransmogrifyOverrideEnchantVisualIDSpec_4 = 27,
    TransmogrifySecondaryItemModifiedAppearanceIDSpecAll = 31,
    TransmogrifySecondaryItemModifiedAppearanceIDSpec_0 = 32,
    TransmogrifySecondaryItemModifiedAppearanceIDSpec_1 = 33,
    TransmogrifySecondaryItemModifiedAppearanceIDSpec_2 = 34,
    TransmogrifySecondaryItemModifiedAppearanceIDSpec_3 = 35,
    TransmogrifySecondaryItemModifiedAppearanceIDSpec_4 = 36,
  }
end

if not Enum.ItemModificationMeta then
  Enum.ItemModificationMeta = {
    MaxValue = 63,
    MinValue = 0,
    NumValues = 64,
  }
end

if not Enum.ItemProfessionSubclass then
  Enum.ItemProfessionSubclass = {
    Alchemy = 2,
    Archaeology = 13,
    Blacksmithing = 0,
    Cooking = 4,
    Enchanting = 8,
    Engineering = 7,
    Fishing = 9,
    Herbalism = 3,
    Inscription = 12,
    Jewelcrafting = 11,
    Leatherworking = 1,
    Mining = 5,
    Skinning = 10,
    Tailoring = 6,
  }
end

if not Enum.ItemProfessionSubclassMeta then
  Enum.ItemProfessionSubclassMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.ItemQuality then
  Enum.ItemQuality = {
    Artifact = 6,
    Common = 1,
    Epic = 4,
    Heirloom = 7,
    Legendary = 5,
    Poor = 0,
    Rare = 3,
    Uncommon = 2,
    WoWToken = 8,
  }
end

if not Enum.ItemQualityMeta then
  Enum.ItemQualityMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.ItemReagentSubclass then
  Enum.ItemReagentSubclass = {
    ContextToken = 2,
    Keystone = 1,
    Reagent = 0,
  }
end

if not Enum.ItemReagentSubclassMeta then
  Enum.ItemReagentSubclassMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ItemRecipeSubclass then
  Enum.ItemRecipeSubclass = {
    Alchemy = 6,
    Blacksmithing = 4,
    Book = 0,
    Cooking = 5,
    Enchanting = 8,
    Engineering = 3,
    FirstAid = 7,
    Fishing = 9,
    Inscription = 11,
    Jewelcrafting = 10,
    Leatherworking = 1,
    Tailoring = 2,
  }
end

if not Enum.ItemRecipeSubclassMeta then
  Enum.ItemRecipeSubclassMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.ItemRecraftFlags then
  Enum.ItemRecraftFlags = {
    Invalid = 1,
  }
end

if not Enum.ItemRecraftFlagsMeta then
  Enum.ItemRecraftFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.ItemRedundancySlot then
  Enum.ItemRedundancySlot = {
    Chest = 3,
    Cloak = 11,
    Feet = 6,
    Finger = 9,
    Hand = 8,
    Head = 0,
    Legs = 5,
    MainhandWeapon = 13,
    Neck = 1,
    Offhand = 16,
    OnehandWeapon = 14,
    OnehandWeaponSecond = 15,
    Shoulder = 2,
    Trinket = 10,
    Twohand = 12,
    Waist = 4,
    Wrist = 7,
  }
end

if not Enum.ItemRedundancySlotMeta then
  Enum.ItemRedundancySlotMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 17,
  }
end

if not Enum.ItemSlotFilterType then
  Enum.ItemSlotFilterType = {
    Chest = 4,
    Cloak = 3,
    Feet = 9,
    Finger = 12,
    Hand = 6,
    Head = 0,
    Legs = 8,
    MainHand = 10,
    Neck = 1,
    NoFilter = 15,
    OffHand = 11,
    Other = 14,
    Shoulder = 2,
    Trinket = 13,
    Waist = 7,
    Wrist = 5,
  }
end

if not Enum.ItemSlotFilterTypeMeta then
  Enum.ItemSlotFilterTypeMeta = {
    MaxValue = 15,
    MinValue = 0,
    NumValues = 16,
  }
end

if not Enum.ItemSocketInfoUIType then
  Enum.ItemSocketInfoUIType = {
    Default = 0,
  }
end

if not Enum.ItemSocketInfoUITypeMeta then
  Enum.ItemSocketInfoUITypeMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.ItemSocketType then
  Enum.ItemSocketType = {
    Arcane = 12,
    Blood = 9,
    Blue = 4,
    Cogwheel = 6,
    Cypher = 23,
    Domination = 22,
    Fel = 11,
    Fiber = 30,
    Fire = 14,
    Fragrance = 26,
    Frost = 13,
    Holy = 18,
    Hydraulic = 5,
    Iron = 8,
    Life = 16,
    Meta = 1,
    None = 0,
    Primordial = 25,
    Prismatic = 7,
    PunchcardBlue = 21,
    PunchcardRed = 19,
    PunchcardYellow = 20,
    Red = 2,
    Shadow = 10,
    SingingSea = 28,
    SingingThunder = 27,
    SingingWind = 29,
    Tinker = 24,
    Water = 15,
    Wind = 17,
    Yellow = 3,
  }
end

if not Enum.ItemSocketTypeMeta then
  Enum.ItemSocketTypeMeta = {
    MaxValue = 30,
    MinValue = 0,
    NumValues = 31,
  }
end

if not Enum.ItemSoundType then
  Enum.ItemSoundType = {
    Close = 3,
    Drop = 1,
    Pickup = 0,
    Use = 2,
  }
end

if not Enum.ItemSoundTypeMeta then
  Enum.ItemSoundTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ItemSubclassDisplay then
  Enum.ItemSubclassDisplay = {
    HideSubclassInAuction = 2,
    HideSubclassInTooltips = 1,
    ShowItemCount = 4,
  }
end

if not Enum.ItemSubclassDisplayMeta then
  Enum.ItemSubclassDisplayMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.ItemSubclassFlag then
  Enum.ItemSubclassFlag = {
    ArmorsubclassLfgscalingarmor = 1024,
    ItemsubclassQuivernotrequired = 64,
    ItemsubclassUsesInvtype = 512,
    WeaponsubclassCanparry = 1,
    WeaponsubclassDeprecatedReuseMe = 256,
    WeaponsubclassIsrifle = 8,
    WeaponsubclassIsthrown = 16,
    WeaponsubclassIsunarmed = 4,
    WeaponsubclassRanged = 128,
    WeaponsubclassRighthandRanged = 32,
    WeaponsubclassSetfingerseq = 2,
  }
end

if not Enum.ItemSubclassFlagMeta then
  Enum.ItemSubclassFlagMeta = {
    MaxValue = 1024,
    MinValue = 1,
    NumValues = 11,
  }
end

if not Enum.ItemTryOnReason then
  Enum.ItemTryOnReason = {
    DataPending = 3,
    NotEquippable = 2,
    Success = 0,
    WrongRace = 1,
  }
end

if not Enum.ItemTryOnReasonMeta then
  Enum.ItemTryOnReasonMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ItemWeaponSubclass then
  Enum.ItemWeaponSubclass = {
    Axe1H = 0,
    Axe2H = 1,
    Bearclaw = 11,
    Bows = 2,
    Catclaw = 12,
    Crossbow = 18,
    Dagger = 15,
    Fishingpole = 20,
    Generic = 14,
    Guns = 3,
    Mace1H = 4,
    Mace2H = 5,
    Obsolete3 = 17,
    Polearm = 6,
    Staff = 10,
    Sword1H = 7,
    Sword2H = 8,
    Thrown = 16,
    Unarmed = 13,
    Wand = 19,
    Warglaive = 9,
  }
end

if not Enum.ItemWeaponSubclassMeta then
  Enum.ItemWeaponSubclassMeta = {
    MaxValue = 20,
    MinValue = 0,
    NumValues = 21,
  }
end

if not Enum.Itemclassfilterflags then
  Enum.Itemclassfilterflags = {
    Armor = 16,
    Battlepet = 131072,
    Consumable = 1,
    Container = 2,
    CurrencyTokenObsolete = 1024,
    Gem = 8,
    Glyph = 65536,
    ItemEnhancement = 256,
    Key = 8192,
    Miscellaneous = 32768,
    PermanentObsolete = 16384,
    Projectile = 64,
    Questitemclassfilterflags = 4096,
    Quiver = 2048,
    Reagent = 32,
    Recipe = 512,
    Tradegoods = 128,
    Weapon = 4,
  }
end

if not Enum.ItemclassfilterflagsMeta then
  Enum.ItemclassfilterflagsMeta = {
    MaxValue = 131072,
    MinValue = 1,
    NumValues = 18,
  }
end

if not Enum.Itemsetflags then
  Enum.Itemsetflags = {
    Legacy = 1,
    RequiresPvPTalentsActive = 4,
    UseItemHistorySetSlots = 2,
  }
end

if not Enum.ItemsetflagsMeta then
  Enum.ItemsetflagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.JailersTowerType then
  Enum.JailersTowerType = {
    AdamantVaults = 11,
    ArkobanHall = 7,
    BossRush = 14,
    Coldheart = 4,
    ForgottenCatacombs = 12,
    FractureChambers = 2,
    Mortregar = 5,
    Ossuary = 13,
    SkoldusHalls = 1,
    Soulforges = 3,
    TormentChamberAnduin = 10,
    TormentChamberJaina = 8,
    TormentChamberThrall = 9,
    TwistingCorridors = 0,
    UpperReaches = 6,
  }
end

if not Enum.JailersTowerTypeMeta then
  Enum.JailersTowerTypeMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.JournalEncounterFlags then
  Enum.JournalEncounterFlags = {
    AllianceOnly = 4,
    DoNotDisplayEncounter = 64,
    HordeOnly = 8,
    InternalOnly = 32,
    LimitDifficulties = 2,
    NoMap = 16,
    Obsolete = 1,
  }
end

if not Enum.JournalEncounterFlagsMeta then
  Enum.JournalEncounterFlagsMeta = {
    MaxValue = 64,
    MinValue = 1,
    NumValues = 7,
  }
end

if not Enum.JournalEncounterIconFlags then
  Enum.JournalEncounterIconFlags = {
    Bleed = 8192,
    Curse = 256,
    Deadly = 16,
    Disease = 1024,
    Dps = 2,
    Enrage = 2048,
    Healer = 4,
    Heroic = 8,
    Important = 32,
    Interruptible = 64,
    Magic = 128,
    Mythic = 4096,
    Poison = 512,
    Tank = 1,
  }
end

if not Enum.JournalEncounterIconFlagsMeta then
  Enum.JournalEncounterIconFlagsMeta = {
    MaxValue = 8192,
    MinValue = 1,
    NumValues = 14,
  }
end

if not Enum.JournalEncounterItemFlags then
  Enum.JournalEncounterItemFlags = {
    DisplayAsExtremelyRare = 16,
    DisplayAsPerPlayerLoot = 4,
    DisplayAsVeryRare = 8,
    LimitDifficulties = 2,
    Obsolete = 1,
  }
end

if not Enum.JournalEncounterItemFlagsMeta then
  Enum.JournalEncounterItemFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.JournalEncounterLocFlags then
  Enum.JournalEncounterLocFlags = {
    Primary = 1,
  }
end

if not Enum.JournalEncounterLocFlagsMeta then
  Enum.JournalEncounterLocFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.JournalEncounterSecTypes then
  Enum.JournalEncounterSecTypes = {
    Ability = 2,
    Creature = 1,
    Generic = 0,
    Overview = 3,
  }
end

if not Enum.JournalEncounterSecTypesMeta then
  Enum.JournalEncounterSecTypesMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.JournalEncounterSectionFlags then
  Enum.JournalEncounterSectionFlags = {
    LimitDifficulties = 2,
    StartExpanded = 1,
  }
end

if not Enum.JournalEncounterSectionFlagsMeta then
  Enum.JournalEncounterSectionFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.JournalInstanceFlags then
  Enum.JournalInstanceFlags = {
    DoNotDisplayInstance = 4,
    HideUserSelectableDifficulty = 2,
    Timewalker = 1,
  }
end

if not Enum.JournalInstanceFlagsMeta then
  Enum.JournalInstanceFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.JournalLinkTypes then
  Enum.JournalLinkTypes = {
    Encounter = 1,
    Instance = 0,
    Section = 2,
    Tier = 3,
  }
end

if not Enum.JournalLinkTypesMeta then
  Enum.JournalLinkTypesMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LFGEntryGeneralPlaystyle then
  Enum.LFGEntryGeneralPlaystyle = {
    Expert = 4,
    FunRelaxed = 2,
    FunSerious = 3,
    Learning = 1,
    None = 0,
  }
end

if not Enum.LFGEntryGeneralPlaystyleMeta then
  Enum.LFGEntryGeneralPlaystyleMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.LFGEntryPlaystyle then
  Enum.LFGEntryPlaystyle = {
    Casual = 2,
    Hardcore = 3,
    None = 0,
    Standard = 1,
  }
end

if not Enum.LFGEntryPlaystyleMeta then
  Enum.LFGEntryPlaystyleMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LFGListDisplayType then
  Enum.LFGListDisplayType = {
    ClassEnumerate = 2,
    Comment = 5,
    HideAll = 3,
    PlayerCount = 4,
    RoleCount = 0,
    RoleEnumerate = 1,
  }
end

if not Enum.LFGListDisplayTypeMeta then
  Enum.LFGListDisplayTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.LFGListFilter then
  Enum.LFGListFilter = {
    CurrentExpansion = 32,
    CurrentSeason = 64,
    NotCurrentSeason = 128,
    NotRecommended = 2,
    PvE = 4,
    PvP = 8,
    Recommended = 1,
    Timerunning = 16,
  }
end

if not Enum.LFGListFilterMeta then
  Enum.LFGListFilterMeta = {
    MaxValue = 128,
    MinValue = 1,
    NumValues = 8,
  }
end

if not Enum.LFGRole then
  Enum.LFGRole = {
    Damage = 2,
    Healer = 1,
    Tank = 0,
  }
end

if not Enum.LFGRoleMeta then
  Enum.LFGRoleMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.LFGSlotInvalidReason then
  Enum.LFGSlotInvalidReason = {
    AreaNotExplored = 9,
    CannotRunAnyChildDungeon = 14,
    ChromieTime = 16,
    EngagedInPvP = 12,
    ExpansionTooLow = 1,
    GearTooHigh = 5,
    GearTooLow = 4,
    LevelTargetTooHigh = 8,
    LevelTargetTooLow = 7,
    LevelTooHigh = 3,
    LevelTooLow = 2,
    NoSpec = 13,
    NoValidRoles = 11,
    None = 0,
    Npe = 17,
    PlayerConditionFailed = 19,
    RaidLocked = 6,
    Restricted = 15,
    Timerunning = 18,
    WrongFaction = 10,
  }
end

if not Enum.LFGSlotInvalidReasonMeta then
  Enum.LFGSlotInvalidReasonMeta = {
    MaxValue = 19,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.LanguageFlag then
  Enum.LanguageFlag = {
    HiddenFromPlayer = 2,
    HideLanguageNameInChat = 4,
    IsExotic = 1,
  }
end

if not Enum.LanguageFlagMeta then
  Enum.LanguageFlagMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.LeavePartyConfirmReason then
  Enum.LeavePartyConfirmReason = {
    QuestSync = 0,
    RestrictedChallengeMode = 1,
  }
end

if not Enum.LeavePartyConfirmReasonMeta then
  Enum.LeavePartyConfirmReasonMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.LgVendorPurchaseSettlementState then
  Enum.LgVendorPurchaseSettlementState = {
    NotSettled = 1,
    Settled = 0,
  }
end

if not Enum.LgVendorPurchaseSettlementStateMeta then
  Enum.LgVendorPurchaseSettlementStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.LgVendorPurchaseSqlResults then
  Enum.LgVendorPurchaseSqlResults = {
    AlreadyOwned = 102,
    Failed = 0,
    InsufficientFunds = 101,
    InsufficientSpent = 103,
    InvalidState = 107,
    NoRecord = 106,
    NotOwned = 104,
    RefundExpired = 105,
    Success = 100,
  }
end

if not Enum.LgVendorPurchaseSqlResultsMeta then
  Enum.LgVendorPurchaseSqlResultsMeta = {
    MaxValue = 107,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.LgVendorPurchaseState then
  Enum.LgVendorPurchaseState = {
    NotOwned = 0,
    Owned = 1,
  }
end

if not Enum.LgVendorPurchaseStateMeta then
  Enum.LgVendorPurchaseStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.LimitedInputType then
  Enum.LimitedInputType = {
    MouseDown = 1,
    MouseMove = 0,
    MouseUp = 2,
    MouseWheel = 3,
  }
end

if not Enum.LimitedInputTypeMeta then
  Enum.LimitedInputTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LinkedCurrencyFlags then
  Enum.LinkedCurrencyFlags = {
    AddIgnoresMax = 8,
    IgnoreAdd = 1,
    IgnoreSubtract = 2,
    SuppressChatLog = 4,
  }
end

if not Enum.LinkedCurrencyFlagsMeta then
  Enum.LinkedCurrencyFlagsMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.LoadConfigResult then
  Enum.LoadConfigResult = {
    Error = 0,
    LoadInProgress = 2,
    NoChangesNecessary = 1,
    Ready = 3,
  }
end

if not Enum.LoadConfigResultMeta then
  Enum.LoadConfigResultMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LogPriority then
  Enum.LogPriority = {
    Debug = 30,
    Error = 2,
    Fatal = 1,
    Normal = 10,
    Spam = 40,
    Warning = 3,
  }
end

if not Enum.LogPriorityMeta then
  Enum.LogPriorityMeta = {
    MaxValue = 40,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.LogicLogicop then
  Enum.LogicLogicop = {
    ["And"] = 1,
    None = 0,
    ["Or"] = 2,
    Xor = 3,
  }
end

if not Enum.LogicLogicopMeta then
  Enum.LogicLogicopMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LogicMathop then
  Enum.LogicMathop = {
    Div = 4,
    Minus = 2,
    Mod = 5,
    None = 0,
    Plus = 1,
    Times = 3,
  }
end

if not Enum.LogicMathopMeta then
  Enum.LogicMathopMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.LogicRelop then
  Enum.LogicRelop = {
    Equal = 1,
    Gt = 5,
    Gteq = 6,
    Lt = 3,
    Lteq = 4,
    None = 0,
    Notequal = 2,
  }
end

if not Enum.LogicRelopMeta then
  Enum.LogicRelopMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.LootMethod then
  Enum.LootMethod = {
    Freeforall = 0,
    Group = 3,
    Masterlooter = 2,
    Needbeforegreed = 4,
    Personal = 5,
    Roundrobin = 1,
  }
end

if not Enum.LootMethodMeta then
  Enum.LootMethodMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.LootMethodStyles then
  Enum.LootMethodStyles = {
    PersonalOnly = 0,
    Vanilla = 1,
  }
end

if not Enum.LootMethodStylesMeta then
  Enum.LootMethodStylesMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.LootSlotType then
  Enum.LootSlotType = {
    Currency = 3,
    Item = 1,
    Money = 2,
    None = 0,
  }
end

if not Enum.LootSlotTypeMeta then
  Enum.LootSlotTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.LuaCurveType then
  Enum.LuaCurveType = {
    Cosine = 2,
    Cubic = 3,
    Linear = 0,
    Step = 1,
  }
end

if not Enum.LuaCurveTypeMeta then
  Enum.LuaCurveTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.MajorFactionFeatureAbility then
  Enum.MajorFactionFeatureAbility = {
    Fishing = 1,
    Generic = 0,
    Hunts = 2,
  }
end

if not Enum.MajorFactionFeatureAbilityMeta then
  Enum.MajorFactionFeatureAbilityMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.MajorFactionType then
  Enum.MajorFactionType = {
    DragonscaleExpedition = 1,
    IskaaraTuskarr = 3,
    MaruukCentaur = 2,
    None = 0,
    ValdrakkenAccord = 4,
  }
end

if not Enum.MajorFactionTypeMeta then
  Enum.MajorFactionTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ManipulatorEventType then
  Enum.ManipulatorEventType = {
    Complete = 2,
    Delete = 3,
    Move = 1,
    Start = 0,
  }
end

if not Enum.ManipulatorEventTypeMeta then
  Enum.ManipulatorEventTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.MapCanvasPositionMeta then
  Enum.MapCanvasPositionMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.MapIconUIWidgetSetType then
  Enum.MapIconUIWidgetSetType = {
    AdventureMapDetails = 2,
    BehindIcon = 1,
    Tooltip = 0,
  }
end

if not Enum.MapIconUIWidgetSetTypeMeta then
  Enum.MapIconUIWidgetSetTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.MapOverlayDisplayLocation then
  Enum.MapOverlayDisplayLocation = {
    BottomLeft = 1,
    BottomRight = 3,
    Default = 0,
    Hidden = 5,
    TopLeft = 2,
    TopRight = 4,
  }
end

if not Enum.MapOverlayDisplayLocationMeta then
  Enum.MapOverlayDisplayLocationMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.MapPinAnimationTypeMeta then
  Enum.MapPinAnimationTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.MapobjEventTypes then
  Enum.MapobjEventTypes = {
    PlayAnim = 0,
    SetAnimSpeed = 1,
  }
end

if not Enum.MapobjEventTypesMeta then
  Enum.MapobjEventTypesMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.MatchDetailTypeMeta then
  Enum.MatchDetailTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.MicroMenuOrderMeta then
  Enum.MicroMenuOrderMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.MicroMenuOrientationMeta then
  Enum.MicroMenuOrientationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.MinimapTrackingFilterMeta then
  Enum.MinimapTrackingFilterMeta = {
    MaxValue = 4194304,
    MinValue = 0,
    NumValues = 24,
  }
end

if not Enum.ModelBlendOperation then
  Enum.ModelBlendOperation = {
    Anim = 1,
    None = 0,
  }
end

if not Enum.ModelBlendOperationMeta then
  Enum.ModelBlendOperationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ModelLightType then
  Enum.ModelLightType = {
    Directional = 0,
    Point = 1,
  }
end

if not Enum.ModelLightTypeMeta then
  Enum.ModelLightTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ModelSceneSetting then
  Enum.ModelSceneSetting = {
    AlignLightToOrbitDelta = 1,
  }
end

if not Enum.ModelSceneSettingMeta then
  Enum.ModelSceneSettingMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.ModelSceneType then
  Enum.ModelSceneType = {
    ArtifactRelicTalentEffect = 9,
    ArtifactTier2 = 5,
    ArtifactTier2ForgingScene = 6,
    ArtifactTier2SlamEffect = 7,
    AzeriteItemLevelUpToast = 13,
    AzeritePowers = 14,
    AzeriteRewardGlow = 15,
    CommentatorVictoryFanfare = 8,
    EncounterJournal = 3,
    HeartOfAzeroth = 16,
    JailersTowerAnimaGlow = 19,
    MountJournal = 0,
    PartyPose = 12,
    PetJournalCard = 1,
    PetJournalLoadout = 4,
    PvPWarModeFire = 11,
    PvPWarModeOrb = 10,
    ShopCard = 2,
    Soulbinds = 18,
    WorldMapThreat = 17,
  }
end

if not Enum.ModelSceneTypeMeta then
  Enum.ModelSceneTypeMeta = {
    MaxValue = 19,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.ModelSoundOverrideType then
  Enum.ModelSoundOverrideType = {
    Mute = 2,
    UseOverride = 1,
    UseParent = 0,
  }
end

if not Enum.ModelSoundOverrideTypeMeta then
  Enum.ModelSoundOverrideTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ModelSoundTagType then
  Enum.ModelSoundTagType = {
    Looping = 2,
    Oneshot = 1,
  }
end

if not Enum.ModelSoundTagTypeMeta then
  Enum.ModelSoundTagTypeMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.MountType then
  Enum.MountType = {
    Aquatic = 2,
    Dragonriding = 3,
    Flying = 1,
    Ground = 0,
    RideAlong = 4,
  }
end

if not Enum.MountTypeFlag then
  Enum.MountTypeFlag = {
    IsAquaticMount = 2,
    IsDragonRidingMount = 4,
    IsFlyingMount = 1,
    IsRideAlongMount = 8,
  }
end

if not Enum.MountTypeFlagMeta then
  Enum.MountTypeFlagMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.MountTypeMeta then
  Enum.MountTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.NamePlateCastBarDisplay then
  Enum.NamePlateCastBarDisplay = {
    HighlightImportantCasts = 4,
    HighlightWhenCastTarget = 5,
    None = 0,
    SpellIcon = 2,
    SpellName = 1,
    SpellTarget = 3,
  }
end

if not Enum.NamePlateCastBarDisplayMeta then
  Enum.NamePlateCastBarDisplayMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.NamePlateEnemyNpcAuraDisplay then
  Enum.NamePlateEnemyNpcAuraDisplay = {
    Buffs = 1,
    CrowdControl = 3,
    Debuffs = 2,
    None = 0,
  }
end

if not Enum.NamePlateEnemyNpcAuraDisplayMeta then
  Enum.NamePlateEnemyNpcAuraDisplayMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NamePlateEnemyPlayerAuraDisplay then
  Enum.NamePlateEnemyPlayerAuraDisplay = {
    Buffs = 1,
    Debuffs = 2,
    LossOfControl = 3,
    None = 0,
  }
end

if not Enum.NamePlateEnemyPlayerAuraDisplayMeta then
  Enum.NamePlateEnemyPlayerAuraDisplayMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NamePlateFriendlyPlayerAuraDisplay then
  Enum.NamePlateFriendlyPlayerAuraDisplay = {
    Buffs = 1,
    Debuffs = 2,
    LossOfControl = 3,
    None = 0,
  }
end

if not Enum.NamePlateFriendlyPlayerAuraDisplayMeta then
  Enum.NamePlateFriendlyPlayerAuraDisplayMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NamePlateInfoDisplay then
  Enum.NamePlateInfoDisplay = {
    CurrentHealthPercent = 1,
    CurrentHealthValue = 2,
    None = 0,
    RarityIcon = 3,
  }
end

if not Enum.NamePlateInfoDisplayMeta then
  Enum.NamePlateInfoDisplayMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NamePlateSimplifiedType then
  Enum.NamePlateSimplifiedType = {
    FriendlyNpc = 4,
    FriendlyPlayer = 3,
    Minion = 1,
    MinusMob = 2,
    None = 0,
  }
end

if not Enum.NamePlateSimplifiedTypeMeta then
  Enum.NamePlateSimplifiedTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.NamePlateSizeMeta then
  Enum.NamePlateSizeMeta = {
    MaxValue = 5,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.NamePlateStackType then
  Enum.NamePlateStackType = {
    Enemy = 1,
    Friendly = 2,
    None = 0,
  }
end

if not Enum.NamePlateStackTypeMeta then
  Enum.NamePlateStackTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NamePlateStyleMeta then
  Enum.NamePlateStyleMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.NamePlateThreatDisplay then
  Enum.NamePlateThreatDisplay = {
    Flash = 2,
    HealthBarColor = 3,
    None = 0,
    Progressive = 1,
  }
end

if not Enum.NamePlateThreatDisplayMeta then
  Enum.NamePlateThreatDisplayMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NamePlateType then
  Enum.NamePlateType = {
    Enemy = 1,
    Friendly = 0,
  }
end

if not Enum.NamePlateTypeMeta then
  Enum.NamePlateTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.NavigationStateMeta then
  Enum.NavigationStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NeighborhoodFlags then
  Enum.NeighborhoodFlags = {
    None = 0,
    OpenToPublic = 2,
    PoolParent = 1,
  }
end

if not Enum.NeighborhoodFlagsMeta then
  Enum.NeighborhoodFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NeighborhoodInitiativeChestResult then
  Enum.NeighborhoodInitiativeChestResult = {
    NiNoHouseFound = 2,
    NiNoRewards = 3,
    NiServiceDisabled = 5,
    NiSuccess = 0,
    NiThrottled = 4,
    NiUnspecifiedFailure = 1,
  }
end

if not Enum.NeighborhoodInitiativeChestResultMeta then
  Enum.NeighborhoodInitiativeChestResultMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.NeighborhoodInitiativeFlags then
  Enum.NeighborhoodInitiativeFlags = {
    Disabled = 1,
    NoAbandon = 2,
    NoRepeat = 4,
  }
end

if not Enum.NeighborhoodInitiativeFlagsMeta then
  Enum.NeighborhoodInitiativeFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.NeighborhoodInitiativeNeighborhoodTypes then
  Enum.NeighborhoodInitiativeNeighborhoodTypes = {
    NiNeighborhoodTypePool = 1,
    NiNeighborhoodTypeSingleton = 0,
  }
end

if not Enum.NeighborhoodInitiativeNeighborhoodTypesMeta then
  Enum.NeighborhoodInitiativeNeighborhoodTypesMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.NeighborhoodInitiativeTaskType then
  Enum.NeighborhoodInitiativeTaskType = {
    RepeatableFinite = 1,
    RepeatableInfinite = 2,
    Single = 0,
  }
end

if not Enum.NeighborhoodInitiativeTaskTypeMeta then
  Enum.NeighborhoodInitiativeTaskTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NeighborhoodInitiativeUpdateStatus then
  Enum.NeighborhoodInitiativeUpdateStatus = {
    Completed = 2,
    Failed = 3,
    MilestoneCompleted = 1,
    Started = 0,
  }
end

if not Enum.NeighborhoodInitiativeUpdateStatusMeta then
  Enum.NeighborhoodInitiativeUpdateStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NeighborhoodInitiativesCompletionStates then
  Enum.NeighborhoodInitiativesCompletionStates = {
    NiCompletionStateNotCompleted = 0,
    NiCompletionStatePlayerCompleted = 1,
    NiCompletionStateSystemAbandoned = 2,
  }
end

if not Enum.NeighborhoodInitiativesCompletionStatesMeta then
  Enum.NeighborhoodInitiativesCompletionStatesMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NeighborhoodInviteResult then
  Enum.NeighborhoodInviteResult = {
    AlreadyInNeighborhood = 11,
    DbError = 1,
    Faction = 5,
    GenericFailure = 3,
    InviteLimit = 7,
    NotEnoughPlots = 8,
    NotFound = 9,
    PendingInvitation = 6,
    Permission = 4,
    RpcFailure = 2,
    Success = 0,
    TooManyRequests = 10,
  }
end

if not Enum.NeighborhoodInviteResultMeta then
  Enum.NeighborhoodInviteResultMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.NeighborhoodMapFlags then
  Enum.NeighborhoodMapFlags = {
    AlliancePurchasable = 1,
    CanSystemGenerate = 4,
    HordePurchasable = 2,
    None = 0,
  }
end

if not Enum.NeighborhoodMapFlagsMeta then
  Enum.NeighborhoodMapFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.NeighborhoodOwnerTypeMeta then
  Enum.NeighborhoodOwnerTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NeighborhoodType then
  Enum.NeighborhoodType = {
    Open = 0,
    Private = 1,
    Public = 2,
  }
end

if not Enum.NeighborhoodTypeMeta then
  Enum.NeighborhoodTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.NewCharGear then
  Enum.NewCharGear = {
    Preview = 1,
    Start = 0,
  }
end

if not Enum.NewCharGearMeta then
  Enum.NewCharGearMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.NodeOpFailureReason then
  Enum.NodeOpFailureReason = {
    AccountDataNoMatch = 25,
    DataError = 13,
    EntryNotFound = 19,
    HasMutuallyExclusiveEdge = 4,
    LevelTooLow = 22,
    MaxRank = 12,
    MissingAchievement = 8,
    MissingEdgeConnection = 1,
    MissingQuest = 9,
    MissingRequiredEdge = 3,
    NodeNeverPurchasable = 24,
    NodeNotFound = 18,
    None = 0,
    NotEnoughCurrency = 15,
    NotEnoughCurrencySpent = 6,
    NotEnoughGold = 16,
    NotEnoughGoldSpent = 7,
    NotEnoughRanksInEntry = 26,
    NotEnoughSourcedCurrency = 14,
    NotEnoughSourcedCurrencySpent = 5,
    RequiredForCondition = 20,
    RequiredForEdge = 2,
    SameSelection = 17,
    TreeFlaggedNoRefund = 23,
    WrongSelection = 11,
    WrongSpec = 10,
    WrongTreeID = 21,
  }
end

if not Enum.NodeOpFailureReasonMeta then
  Enum.NodeOpFailureReasonMeta = {
    MaxValue = 26,
    MinValue = 0,
    NumValues = 27,
  }
end

if not Enum.NpcCraftingOrderSetFlags then
  Enum.NpcCraftingOrderSetFlags = {
    AllowDuplicate = 2,
    AllowMultiple = 1,
  }
end

if not Enum.NpcCraftingOrderSetFlagsMeta then
  Enum.NpcCraftingOrderSetFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.PartyPlaylistEntryMeta then
  Enum.PartyPlaylistEntryMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PartyPoseFlags then
  Enum.PartyPoseFlags = {
    HideLeaveInstanceButton = 1,
  }
end

if not Enum.PartyPoseFlagsMeta then
  Enum.PartyPoseFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.PartyRequestJoinRelation then
  Enum.PartyRequestJoinRelation = {
    Club = 3,
    Friend = 1,
    Guild = 2,
    None = 0,
    RecentAllies = 4,
  }
end

if not Enum.PartyRequestJoinRelationMeta then
  Enum.PartyRequestJoinRelationMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PerksVendorCategoryType then
  Enum.PerksVendorCategoryType = {
    Achievement = 23,
    Activity = 21,
    GmAdjustment = 22,
    Illusion = 7,
    Mount = 2,
    Pet = 3,
    RefundUnused = 24,
    Stipend = 20,
    Toy = 5,
    Transmog = 1,
    Transmogset = 8,
    WarbandScene = 9,
  }
end

if not Enum.PerksVendorCategoryTypeMeta then
  Enum.PerksVendorCategoryTypeMeta = {
    MaxValue = 24,
    MinValue = 1,
    NumValues = 12,
  }
end

if not Enum.PermanentChatChannelType then
  Enum.PermanentChatChannelType = {
    Communities = 2,
    Custom = 3,
    None = 0,
    Zone = 1,
  }
end

if not Enum.PermanentChatChannelTypeMeta then
  Enum.PermanentChatChannelTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PetActionFeedback then
  Enum.PetActionFeedback = {
    Dead = 1,
    FriendlyTarget = 3,
    InvalidTarget = 2,
    NoPath = 4,
    Success = 0,
  }
end

if not Enum.PetActionFeedbackMeta then
  Enum.PetActionFeedbackMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PetActionbuttonType then
  Enum.PetActionbuttonType = {
    Max = 18,
    Mode = 6,
    None = 0,
    Orders = 7,
    Slot1 = 8,
    Slot10 = 17,
    Slot1Obsolete = 2,
    Slot2 = 9,
    Slot2Obsolete = 3,
    Slot3 = 10,
    Slot3Obsolete = 4,
    Slot4 = 11,
    Slot4Obsolete = 5,
    Slot5 = 12,
    Slot6 = 13,
    Slot7 = 14,
    Slot8 = 15,
    Slot9 = 16,
    Spell = 1,
    VehicleAction = 19,
  }
end

if not Enum.PetActionbuttonTypeMeta then
  Enum.PetActionbuttonTypeMeta = {
    MaxValue = 19,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.PetBattleQueueStatus then
  Enum.PetBattleQueueStatus = {
    AlreadyQueued = 3,
    InBattle = 20,
    JoinFailed = 4,
    JoinFailedJournalLock = 6,
    JoinFailedNeutral = 7,
    JoinFailedSlots = 5,
    LostConnection = 16,
    MatchAccepted = 8,
    MatchDeclined = 9,
    MatchOpponentDeclined = 10,
    Matchmaking = 15,
    NoBattlingHere = 21,
    None = 0,
    ProposalTimedOut = 11,
    Queued = 1,
    QueuedUpdate = 2,
    Removed = 12,
    RequeuedAfterInternalError = 13,
    RequeuedAfterOpponentRemoved = 14,
    Shutdown = 17,
    Suspended = 18,
    Unsuspended = 19,
  }
end

if not Enum.PetBattleQueueStatusMeta then
  Enum.PetBattleQueueStatusMeta = {
    MaxValue = 21,
    MinValue = 0,
    NumValues = 22,
  }
end

if not Enum.PetJournalError then
  Enum.PetJournalError = {
    InvalidFaction = 3,
    JournalIsLocked = 2,
    NoFavoritesToSummon = 4,
    NoValidRandomSummon = 5,
    None = 0,
    PetIsDead = 1,
  }
end

if not Enum.PetJournalErrorMeta then
  Enum.PetJournalErrorMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.PetMode then
  Enum.PetMode = {
    Aggressive = 2,
    Assist = 3,
    Defensive = 1,
    Passive = 0,
  }
end

if not Enum.PetModeMeta then
  Enum.PetModeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PetOrders then
  Enum.PetOrders = {
    Attack = 2,
    Dismiss = 3,
    Follow = 1,
    MoveTo = 4,
    Wait = 0,
  }
end

if not Enum.PetOrdersMeta then
  Enum.PetOrdersMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PetOverride then
  Enum.PetOverride = {
    AICombatControl = 1,
    AICombatPassive = 2,
    None = 0,
    OwnerMounted = 4,
  }
end

if not Enum.PetOverrideMeta then
  Enum.PetOverrideMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PetbattleAuraStateFlags then
  Enum.PetbattleAuraStateFlags = {
    Canceled = 2,
    CountdownFirstRound = 8,
    Infinite = 1,
    InitDisabled = 4,
    JustApplied = 16,
    None = 0,
    RemoveEventHandled = 32,
  }
end

if not Enum.PetbattleAuraStateFlagsMeta then
  Enum.PetbattleAuraStateFlagsMeta = {
    MaxValue = 32,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.PetbattleCheatFlags then
  Enum.PetbattleCheatFlags = {
    AutoPlay = 1,
    None = 0,
  }
end

if not Enum.PetbattleCheatFlagsMeta then
  Enum.PetbattleCheatFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PetbattleEffectFlags then
  Enum.PetbattleEffectFlags = {
    Absorb = 256,
    AuraReapply = 8192,
    Blocked = 8,
    Crit = 4,
    Dodge = 16,
    Heal = 32,
    Immune = 512,
    InvalidTarget = 1,
    Miss = 2,
    None = 0,
    Reflect = 128,
    Strong = 1024,
    SuccessChain = 4096,
    Unkillable = 64,
    Weak = 2048,
  }
end

if not Enum.PetbattleEffectFlagsMeta then
  Enum.PetbattleEffectFlagsMeta = {
    MaxValue = 8192,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.PetbattleEffectType then
  Enum.PetbattleEffectType = {
    AbilityChange = 11,
    AuraApply = 1,
    AuraCancel = 2,
    AuraChange = 3,
    AuraProcessingBegin = 13,
    AuraProcessingEnd = 14,
    NpcEmote = 12,
    OverrideAbility = 16,
    PetSwap = 4,
    ReplacePet = 15,
    SetHealth = 0,
    SetMaxHealth = 7,
    SetPower = 9,
    SetSpeed = 8,
    SetState = 6,
    StatusChange = 5,
    TriggerAbility = 10,
    WorldStateUpdate = 17,
  }
end

if not Enum.PetbattleEffectTypeMeta then
  Enum.PetbattleEffectTypeMeta = {
    MaxValue = 17,
    MinValue = 0,
    NumValues = 18,
  }
end

if not Enum.PetbattleEnviros then
  Enum.PetbattleEnviros = {
    Pad0 = 0,
    Pad1 = 1,
    Weather = 2,
  }
end

if not Enum.PetbattleEnvirosMeta then
  Enum.PetbattleEnvirosMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PetbattleInputMoveMsgDebugFlag then
  Enum.PetbattleInputMoveMsgDebugFlag = {
    DontValidate = 1,
    EnemyCast = 2,
    None = 0,
  }
end

if not Enum.PetbattleInputMoveMsgDebugFlagMeta then
  Enum.PetbattleInputMoveMsgDebugFlagMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PetbattleMoveType then
  Enum.PetbattleMoveType = {
    Ability = 1,
    FinalRoundOk = 4,
    Pass = 5,
    Quit = 0,
    Swap = 2,
    Trap = 3,
  }
end

if not Enum.PetbattleMoveTypeMeta then
  Enum.PetbattleMoveTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.PetbattlePboid then
  Enum.PetbattlePboid = {
    EnvPad_0 = 6,
    EnvPad_1 = 7,
    EnvWeather = 8,
    P0Pet_0 = 0,
    P0Pet_1 = 1,
    P0Pet_2 = 2,
    P1Pet_0 = 3,
    P1Pet_1 = 4,
    P1Pet_2 = 5,
  }
end

if not Enum.PetbattlePboidMeta then
  Enum.PetbattlePboidMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.PetbattlePetStatus then
  Enum.PetbattlePetStatus = {
    FlagNone = 0,
    FlagTrapped = 1,
    Stunned = 2,
    SwapInLocked = 8,
    SwapOutLocked = 4,
  }
end

if not Enum.PetbattlePetStatusMeta then
  Enum.PetbattlePetStatusMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PetbattlePlayer then
  Enum.PetbattlePlayer = {
    Player_0 = 0,
    Player_1 = 1,
  }
end

if not Enum.PetbattlePlayerInputFlags then
  Enum.PetbattlePlayerInputFlags = {
    AbilityLocked = 2,
    None = 0,
    SwapLocked = 4,
    TurnInProgress = 1,
    WaitingForPet = 8,
  }
end

if not Enum.PetbattlePlayerInputFlagsMeta then
  Enum.PetbattlePlayerInputFlagsMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PetbattlePlayerMeta then
  Enum.PetbattlePlayerMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PetbattleResult then
  Enum.PetbattleResult = {
    FailDeclined = 12,
    FailDisconnect = 22,
    FailInBattle = 13,
    FailInvalidLoadout = 14,
    FailInvalidLoadoutAllDead = 15,
    FailInvalidLoadoutNoneSlotted = 16,
    FailLogout = 21,
    FailNoJournalLock = 17,
    FailNotATrainer = 11,
    FailNotHere = 1,
    FailNotHereObstructed = 4,
    FailNotHereOnTransport = 2,
    FailNotHereUnevenGround = 3,
    FailNotWhileDead = 6,
    FailNotWhileFlying = 7,
    FailNotWhileInCombat = 5,
    FailOpponentNotAvailable = 20,
    FailRestrictedAccount = 19,
    FailTargetInvalid = 8,
    FailTargetNotCapturable = 10,
    FailTargetOutOfRange = 9,
    FailUnknown = 0,
    FailWildPetTapped = 18,
    Success = 23,
  }
end

if not Enum.PetbattleResultMeta then
  Enum.PetbattleResultMeta = {
    MaxValue = 23,
    MinValue = 0,
    NumValues = 24,
  }
end

if not Enum.PetbattleSlot then
  Enum.PetbattleSlot = {
    Slot_0 = 0,
    Slot_1 = 1,
    Slot_2 = 2,
  }
end

if not Enum.PetbattleSlotAbility then
  Enum.PetbattleSlotAbility = {
    Ability_0 = 0,
    Ability_1 = 1,
    Ability_2 = 2,
  }
end

if not Enum.PetbattleSlotAbilityMeta then
  Enum.PetbattleSlotAbilityMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PetbattleSlotMeta then
  Enum.PetbattleSlotMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PetbattleSlotResult then
  Enum.PetbattleSlotResult = {
    CantBattle = 5,
    Dead = 7,
    NoPet = 8,
    NoSpeciesRec = 4,
    NoTracker = 3,
    Revoked = 6,
    SlotEmpty = 2,
    SlotLocked = 1,
    Success = 0,
  }
end

if not Enum.PetbattleSlotResultMeta then
  Enum.PetbattleSlotResultMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.PetbattleState then
  Enum.PetbattleState = {
    Created = 0,
    CreatedFailed = 4,
    FinalRound = 5,
    Finished = 6,
    RoundInProgress = 2,
    WaitingForFrontPets = 3,
    WaitingPreBattle = 1,
  }
end

if not Enum.PetbattleStateMeta then
  Enum.PetbattleStateMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.PetbattleTrapstatus then
  Enum.PetbattleTrapstatus = {
    CanTrap = 1,
    CantTrapNewbie = 2,
    CantTrapNoRoomInJournal = 5,
    CantTrapPetDead = 3,
    CantTrapPetHealth = 4,
    CantTrapPetNotCapturable = 6,
    CantTrapTrainerBattle = 7,
    CantTrapTwice = 8,
    Invalid = 0,
  }
end

if not Enum.PetbattleTrapstatusMeta then
  Enum.PetbattleTrapstatusMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.PetbattleType then
  Enum.PetbattleType = {
    Lfpb = 2,
    Npc = 3,
    PvE = 0,
    PvP = 1,
  }
end

if not Enum.PetbattleTypeMeta then
  Enum.PetbattleTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.Pettameresult then
  Enum.Pettameresult = {
    Anothersummonactive = 5,
    Cantcontrolexotic = 12,
    Creaturealreadyowned = 3,
    Dead = 10,
    EliteToohighlevel = 14,
    Internalerror = 8,
    Invalidcreature = 1,
    Invalidslot = 13,
    Nopetavailable = 7,
    Notdead = 11,
    Nottameable = 4,
    Numresults = 15,
    Ok = 0,
    Toohighlevel = 9,
    Toomany = 2,
    Unitscanttame = 6,
  }
end

if not Enum.PettameresultMeta then
  Enum.PettameresultMeta = {
    MaxValue = 15,
    MinValue = 0,
    NumValues = 16,
  }
end

if not Enum.PhaseReason then
  Enum.PhaseReason = {
    ChromieTime = 3,
    Phasing = 0,
    Sharding = 1,
    TimerunningHwt = 4,
    WarMode = 2,
  }
end

if not Enum.PhaseReasonMeta then
  Enum.PhaseReasonMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.PhotoSharingStatus then
  Enum.PhotoSharingStatus = {
    AADCRestricted = 6,
    Configured = 2,
    Disabled = 0,
    Error = 7,
    Expired = 3,
    FailedToClear = 8,
    NotConfigured = 1,
    ParentalControl = 4,
    TrialOrVeteran = 5,
  }
end

if not Enum.PhotoSharingStatusMeta then
  Enum.PhotoSharingStatusMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.PhotoSharingUploadStatus then
  Enum.PhotoSharingUploadStatus = {
    CreatePageApiCallFailed = 12,
    CreatePageBadRequest = 13,
    CreatePageForbidden = 15,
    CreatePageGenericFailure = 18,
    CreatePageNoBoardIDFound = 19,
    CreatePageNotFound = 16,
    CreatePageTooManyRequests = 17,
    CreatePageUnauthorized = 14,
    CreatePostApiCallFailed = 20,
    CreatePostBadRequest = 21,
    CreatePostForbidden = 23,
    CreatePostGenericFailure = 26,
    CreatePostNoIDFound = 27,
    CreatePostNotFound = 24,
    CreatePostThrottled = 28,
    CreatePostTooManyRequests = 25,
    CreatePostUnauthorized = 22,
    Failed = 0,
    ListPagesApiCallFailed = 3,
    ListPagesBadRequest = 4,
    ListPagesEmptyBoardID = 10,
    ListPagesForbidden = 6,
    ListPagesGenericFailure = 9,
    ListPagesInvalidBookmark = 11,
    ListPagesNotFound = 7,
    ListPagesTooManyRequests = 8,
    ListPagesUnauthorized = 5,
    Locked = 2,
    Success = 1,
  }
end

if not Enum.PhotoSharingUploadStatusMeta then
  Enum.PhotoSharingUploadStatusMeta = {
    MaxValue = 28,
    MinValue = 0,
    NumValues = 29,
  }
end

if not Enum.PingMode then
  Enum.PingMode = {
    ClickDrag = 1,
    KeyDown = 0,
  }
end

if not Enum.PingModeMeta then
  Enum.PingModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PingResultMeta then
  Enum.PingResultMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.PingSubjectType then
  Enum.PingSubjectType = {
    AlertNotThreat = 5,
    AlertThreat = 4,
    Assist = 2,
    Attack = 0,
    OnMyWay = 3,
    Warning = 1,
  }
end

if not Enum.PingSubjectTypeMeta then
  Enum.PingSubjectTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.PingTextureType then
  Enum.PingTextureType = {
    Center = 0,
    Expand = 1,
    Rotation = 2,
  }
end

if not Enum.PingTextureTypeMeta then
  Enum.PingTextureTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PingTypeFlags then
  Enum.PingTypeFlags = {
    DefaultPing = 1,
  }
end

if not Enum.PingTypeFlagsMeta then
  Enum.PingTypeFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.PlayerChoiceRarity then
  Enum.PlayerChoiceRarity = {
    Common = 0,
    Epic = 3,
    Rare = 2,
    Uncommon = 1,
  }
end

if not Enum.PlayerChoiceRarityMeta then
  Enum.PlayerChoiceRarityMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PlayerClubRequestStatus then
  Enum.PlayerClubRequestStatus = {
    Approved = 4,
    AutoApproved = 2,
    Canceled = 7,
    Declined = 3,
    Joined = 5,
    JoinedAnother = 6,
    None = 0,
    Pending = 1,
  }
end

if not Enum.PlayerClubRequestStatusMeta then
  Enum.PlayerClubRequestStatusMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.PlayerCompanionInfoFlags then
  Enum.PlayerCompanionInfoFlags = {
    IgnoreSeasonInScenarios = 1,
  }
end

if not Enum.PlayerCompanionInfoFlagsMeta then
  Enum.PlayerCompanionInfoFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.PlayerCurrencyFlags then
  Enum.PlayerCurrencyFlags = {
    Incremented = 1,
    Loading = 2,
  }
end

if not Enum.PlayerCurrencyFlagsDbFlags then
  Enum.PlayerCurrencyFlagsDbFlags = {
    IgnoreMaxQtyOnload = 1,
    InBackpack = 4,
    Reuse1 = 2,
    Reuse2 = 16,
    UnusedInUI = 8,
  }
end

if not Enum.PlayerCurrencyFlagsDbFlagsMeta then
  Enum.PlayerCurrencyFlagsDbFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.PlayerCurrencyFlagsMeta then
  Enum.PlayerCurrencyFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.PlayerDataElementType then
  Enum.PlayerDataElementType = {
    Float = 1,
    Int = 0,
  }
end

if not Enum.PlayerDataElementTypeMeta then
  Enum.PlayerDataElementTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PlayerInteractionType then
  Enum.PlayerInteractionType = {
    AccountBanker = 68,
    AdventureJournal = 54,
    AdventureMap = 28,
    AlliedRaceDetailsGiver = 9,
    AnimaDiversion = 47,
    AreaSpiritHealer = 19,
    ArtifactForge = 38,
    Auctioneer = 21,
    AzeriteForge = 56,
    AzeriteRespec = 42,
    Banker = 8,
    BarbersChoice = 62,
    BattleMaster = 23,
    Binder = 20,
    BlackMarketAuctioneer = 27,
    CharacterBanker = 67,
    ChromieTime = 45,
    ContributionCollector = 41,
    CornerstoneInteraction = 70,
    CovenantPreview = 46,
    CovenantSanctum = 51,
    CreateGuildNeighborhood = 74,
    ForgeMaster = 66,
    GarrArchitect = 30,
    GarrMission = 32,
    GarrRecruitment = 34,
    GarrTalent = 35,
    GarrTradeskill = 31,
    Gossip = 3,
    GuildBanker = 10,
    GuildRename = 76,
    GuildTabardVendor = 14,
    HousingBulletinBoard = 72,
    HousingPedestal = 73,
    IslandQueue = 43,
    Item = 2,
    ItemInteraction = 44,
    ItemUpgrade = 53,
    JailersTowerBuffs = 63,
    LFGDungeon = 25,
    LegendaryCrafting = 48,
    MailInfo = 17,
    MajorFactionRenown = 64,
    Merchant = 5,
    NeighborhoodCharter = 75,
    NewPlayerGuide = 52,
    None = 0,
    ObliterumForge = 39,
    OpenHouseFinder = 78,
    OpenNeighborhoodCharterConfirmation = 77,
    PerksProgramVendor = 57,
    PersonalTabardVendor = 65,
    PetitionVendor = 13,
    PlayerChoice = 37,
    ProfessionRespec = 69,
    Professions = 59,
    ProfessionsCraftingOrder = 58,
    ProfessionsCustomerOrder = 60,
    QuestGiver = 4,
    Registrar = 11,
    RenameNeighborhood = 71,
    Renown = 55,
    ScrappingMachine = 40,
    ShipmentCrafter = 33,
    Soulbind = 50,
    SpecializationMaster = 16,
    SpiritHealer = 18,
    StableMaster = 22,
    TalentMaster = 15,
    TaxiNode = 6,
    TieredEntrance = 79,
    TradePartner = 1,
    Trainer = 7,
    TraitSystem = 61,
    Transmogrifier = 24,
    Trophy = 36,
    Vendor = 12,
    VoidStorageBanker = 26,
    WeeklyRewards = 49,
    WorldMap = 29,
  }
end

if not Enum.PlayerInteractionTypeMeta then
  Enum.PlayerInteractionTypeMeta = {
    MaxValue = 79,
    MinValue = 0,
    NumValues = 80,
  }
end

if not Enum.PlayerMentorshipApplicationResult then
  Enum.PlayerMentorshipApplicationResult = {
    AlreadyMentor = 1,
    Ineligible = 2,
    Success = 0,
  }
end

if not Enum.PlayerMentorshipApplicationResultMeta then
  Enum.PlayerMentorshipApplicationResultMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PlayerMentorshipStatusMeta then
  Enum.PlayerMentorshipStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PlunderstormQueueState then
  Enum.PlunderstormQueueState = {
    None = 0,
    Proposed = 2,
    Queued = 1,
    Suspended = 3,
  }
end

if not Enum.PlunderstormQueueStateMeta then
  Enum.PlunderstormQueueStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.PowerType then
  Enum.PowerType = {
    Alternate = 10,
    AlternateEncounter = 24,
    AlternateMount = 25,
    AlternateQuest = 23,
    ArcaneCharges = 16,
    Balance = 26,
    BurningEmbers = 14,
    Chi = 12,
    ComboPoints = 4,
    DemonicFury = 15,
    Energy = 3,
    Essence = 19,
    Focus = 2,
    Fury = 17,
    Happiness = 27,
    HolyPower = 9,
    Insanity = 13,
    LunarPower = 8,
    Maelstrom = 11,
    Mana = 0,
    Pain = 18,
    Rage = 1,
    RuneBlood = 20,
    RuneChromatic = 29,
    RuneFrost = 21,
    RuneUnholy = 22,
    Runes = 5,
    RunicPower = 6,
    ShadowOrbs = 28,
    SoulShards = 7,
  }
end

if not Enum.PowerTypeMeta then
  Enum.PowerTypeMeta = {
    MaxValue = 29,
    MinValue = 0,
    NumValues = 30,
  }
end

if not Enum.PowerTypeSign then
  Enum.PowerTypeSign = {
    Negative = 1,
    None = -1,
    Positive = 0,
  }
end

if not Enum.PowerTypeSignMeta then
  Enum.PowerTypeSignMeta = {
    MaxValue = 1,
    MinValue = -1,
    NumValues = 3,
  }
end

if not Enum.PowerTypeSlot then
  Enum.PowerTypeSlot = {
    Slot_0 = 0,
    Slot_1 = 1,
    Slot_2 = 2,
    Slot_3 = 3,
    Slot_4 = 4,
    Slot_5 = 5,
    Slot_6 = 6,
    Slot_7 = 7,
    Slot_8 = 8,
    Slot_9 = 9,
  }
end

if not Enum.PowerTypeSlotMeta then
  Enum.PowerTypeSlotMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.PremadeGroupFinderStyle then
  Enum.PremadeGroupFinderStyle = {
    Disabled = 0,
    Mainline = 1,
    Vanilla = 2,
  }
end

if not Enum.PremadeGroupFinderStyleMeta then
  Enum.PremadeGroupFinderStyleMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PreyHuntProgressStateMeta then
  Enum.PreyHuntProgressStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ProceduralSpawnInteractionMode then
  Enum.ProceduralSpawnInteractionMode = {
    Manipulate = 2,
    None = 0,
    Paint = 1,
  }
end

if not Enum.ProceduralSpawnInteractionModeMeta then
  Enum.ProceduralSpawnInteractionModeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ProceduralSpawnVolumeChunkFlags then
  Enum.ProceduralSpawnVolumeChunkFlags = {
    AllSubChunksSet = 1,
    None = 0,
  }
end

if not Enum.ProceduralSpawnVolumeChunkFlagsMeta then
  Enum.ProceduralSpawnVolumeChunkFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ProfTraitPerkNodeFlags then
  Enum.ProfTraitPerkNodeFlags = {
    IsMajorBonus = 2,
    UnlocksSubpath = 1,
  }
end

if not Enum.ProfTraitPerkNodeFlagsMeta then
  Enum.ProfTraitPerkNodeFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.Profession then
  Enum.Profession = {
    Alchemy = 3,
    Archaeology = 14,
    Blacksmithing = 1,
    Cooking = 5,
    Enchanting = 9,
    Engineering = 8,
    FirstAid = 0,
    Fishing = 10,
    Herbalism = 4,
    Inscription = 13,
    Jewelcrafting = 12,
    Leatherworking = 2,
    Mining = 6,
    Skinning = 11,
    Tailoring = 7,
  }
end

if not Enum.ProfessionActionType then
  Enum.ProfessionActionType = {
    Craft = 0,
    Gather = 1,
  }
end

if not Enum.ProfessionActionTypeMeta then
  Enum.ProfessionActionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ProfessionEffect then
  Enum.ProfessionEffect = {
    AccumulateRanksByLabel = 25,
    ConcentrationRefund = 30,
    DecreaseDifficulty = 22,
    IncreaseDifficulty = 23,
    ModConcentration = 27,
    ModCraftCritSize = 20,
    ModCraftExtraQuantity = 18,
    ModCraftReductionQuantity = 21,
    ModCraftingSpeed = 14,
    ModDeftness = 12,
    ModFinesse = 11,
    ModGatherExtraQuantity = 19,
    ModIngenuity = 29,
    ModInspiration = 9,
    ModMulticraft = 15,
    ModPerception = 13,
    ModResourcefulness = 10,
    ModSkillGain = 24,
    ModUnused_1 = 16,
    ModUnused_2 = 17,
    Skill = 0,
    StatCraftingSpeed = 6,
    StatDeftness = 4,
    StatFinesse = 3,
    StatIngenuity = 26,
    StatInspiration = 1,
    StatMulticraft = 7,
    StatPerception = 5,
    StatResourcefulness = 2,
    Tokenizer = 28,
    UnlockReagentSlot = 8,
  }
end

if not Enum.ProfessionEffectMeta then
  Enum.ProfessionEffectMeta = {
    MaxValue = 30,
    MinValue = 0,
    NumValues = 31,
  }
end

if not Enum.ProfessionMeta then
  Enum.ProfessionMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.ProfessionRating then
  Enum.ProfessionRating = {
    CraftingSpeed = 5,
    Deftness = 3,
    Finesse = 2,
    Ingenuity = 7,
    Inspiration = 0,
    Multicraft = 6,
    Perception = 4,
    Resourcefulness = 1,
    Unused_2 = 8,
  }
end

if not Enum.ProfessionRatingMeta then
  Enum.ProfessionRatingMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.ProfessionRatingType then
  Enum.ProfessionRatingType = {
    Craft = 0,
    Gather = 1,
  }
end

if not Enum.ProfessionRatingTypeMeta then
  Enum.ProfessionRatingTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ProfessionsSpecPathState then
  Enum.ProfessionsSpecPathState = {
    Completed = 2,
    Locked = 0,
    Progressing = 1,
  }
end

if not Enum.ProfessionsSpecPathStateMeta then
  Enum.ProfessionsSpecPathStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ProfessionsSpecPerkState then
  Enum.ProfessionsSpecPerkState = {
    Earned = 2,
    Pending = 1,
    Unearned = 0,
  }
end

if not Enum.ProfessionsSpecPerkStateMeta then
  Enum.ProfessionsSpecPerkStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ProfessionsSpecTabState then
  Enum.ProfessionsSpecTabState = {
    Locked = 0,
    Unlockable = 2,
    Unlocked = 1,
  }
end

if not Enum.ProfessionsSpecTabStateMeta then
  Enum.ProfessionsSpecTabStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.PurchaseEligibility then
  Enum.PurchaseEligibility = {
    ExpansionTooHigh = 4,
    ExpansionTooLow = 3,
    MissingRequiredDeliverable = 5,
    Ok = 0,
    Owned = 2,
    PartiallyOwned = 1,
  }
end

if not Enum.PurchaseEligibilityMeta then
  Enum.PurchaseEligibilityMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.PurchaseHouseDisabledReason then
  Enum.PurchaseHouseDisabledReason = {
    CharterLockout = 7,
    GuildLockout = 6,
    MaxHouses = 8,
    NoExpansion = 4,
    NoGameTimeRemaining = 9,
    None = 0,
    NotInvited = 3,
    Reserved = 5,
    WrongFaction = 1,
    WrongGuild = 2,
  }
end

if not Enum.PurchaseHouseDisabledReasonMeta then
  Enum.PurchaseHouseDisabledReasonMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.PurchaseResult then
  Enum.PurchaseResult = {
    Ok = 0,
    ErrorRiskDenied = 1,
    ErrorOrderFailed = 2,
    ErrorClientTimeout = 3,
    ErrorClientDeclined = 4,
    ErrorFailedToCreatePurchaseRecord = 5,
    ErrorFailedToGetPurchaseID = 6,
    ErrorFailedToSaveMopAndRiskStatus = 7,
    ErrorFailedToSaveMopAndRiskResponse = 8,
    ErrorClientGone = 9,
    ErrorFailedToSavePlaceOrderStatus = 10,
    ErrorBattlepayTimeoutMopAndRisk = 11,
    ErrorCurrencyInvalidInRegion = 12,
    ErrorBattlepayTemporarilyUnavailable = 13,
    ErrorBattlepayDisabled = 14,
    ErrorServerRestarted = 15,
    ErrorInvalidProduct = 16,
    ErrorInvalidRegionGroupMask = 17,
    ErrorProductInvalidInRegion = 18,
    ErrorDistributionObjectNotFound = 19,
    ErrorDistributionObjectAlreadyAssigned = 20,
    ErrorRiskResponseMissingScore = 21,
    ErrorMopAndRiskAmqpRpcFailed = 22,
    ErrorMopAndRiskRpcErrorFromBattlepay = 23,
    ErrorMopAndRiskUnexpectedResponseFromBattlepay = 24,
    ErrorMopAndRiskNoValidPaymentMethods = 25,
    ErrorOrderCanceledPriceChange = 26,
    ErrorBuyingProductNotAllowedInGlueScreen = 27,
    ErrorMopAndRiskNotEnoughBalance = 28,
    ErrorPlaceOrderNotEnoughBalance = 29,
    ErrorProgrammerManualFail = 30,
    ErrorThrottledByUserServer = 31,
    ErrorBuyingProductNotAllowedInWorld = 32,
    ErrorBlockedByParentalControls = 33,
    ErrorNoStorePurchaseFlagIsSet = 34,
    ErrorDistributionObjectInvalidChoice = 35,
    ErrorInvalidCreditCardExpiryDate = 36,
    ErrorAuthorizingPayment = 37,
    ErrorPaymentProviderDeniedPayment = 38,
    ErrorOverSpendingLimit = 39,
    ErrorClientTimeoutChallenge = 40,
    ErrorClientDeclinedChallenge = 41,
    ErrorDistributionObjectInvalidTarget = 42,
    ErrorFixedLicenseProductAlreadyExists = 43,
    ErrorDistributionObjectTrialBlocked = 44,
    ErrorDistributionObjectExpansionBlocked = 45,
    ErrorOwnsConsumableToken = 46,
    ErrorTooManyTokens = 47,
    ErrorCommerceServerDisabled = 48,
    ErrorFailedToWriteVasPurchaseID = 49,
    ErrorFailedToWriteVasLicenseID = 50,
    ErrorCharacterUpgradeOperationInProgress = 51,
    ErrorInvalidPlatform = 52,
    ErrorInvalidDeviceID = 53,
    ErrorFailedToSaveValidatePurchaseStatus = 54,
    ErrorFailedToSaveValidatePurchaseResponse = 55,
    ErrorBattlepayTimeoutValidatePurchase = 56,
    ErrorProductNotPurchasable = 57,
    ErrorFailedToSaveClientCheckoutStatus = 58,
    ErrorFailedOldPurchaseFlow = 59,
    ErrorClientCheckoutTimeout = 60,
    ErrorFailedToSaveValidationHashStatus = 61,
    ErrorBattlepayTimeoutValidationHash = 62,
    ErrorClientCheckoutCanceledOrFailed = 63,
    ErrorCouldNotGetValidationHash = 64,
    ErrorDistributionObjectWrongGameAccount = 65,
    ErrorClientRestricted = 66,
    ErrorFailedToSaveRedeemTokenStatus = 67,
    ErrorInvalidPurchaseID = 68,
    ErrorTokenRedemptionOrderFailed = 69,
    ErrorVasTransactionCreateFailed = 70,
    ErrorInManualReview = 71,
  }
end

if not Enum.PvPFaction then
  Enum.PvPFaction = {
    Alliance = 1,
    Horde = 0,
  }
end

if not Enum.PvPFactionMeta then
  Enum.PvPFactionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PvPMatchState then
  Enum.PvPMatchState = {
    Complete = 5,
    Engaged = 3,
    Inactive = 0,
    PostRound = 4,
    StartUp = 2,
    Waiting = 1,
  }
end

if not Enum.PvPMatchStateMeta then
  Enum.PvPMatchStateMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.PvPMatchmakingType then
  Enum.PvPMatchmakingType = {
    Arena = 1,
    Battleground = 0,
  }
end

if not Enum.PvPMatchmakingTypeMeta then
  Enum.PvPMatchmakingTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.PvPRanks then
  Enum.PvPRanks = {
    RankDishonored = 4,
    RankExiled = 3,
    RankNone = 0,
    RankOutlaw = 2,
    RankPariah = 1,
    Rank_1 = 5,
    Rank_10 = 14,
    Rank_11 = 15,
    Rank_12 = 16,
    Rank_13 = 17,
    Rank_14 = 18,
    Rank_2 = 6,
    Rank_3 = 7,
    Rank_4 = 8,
    Rank_5 = 9,
    Rank_6 = 10,
    Rank_7 = 11,
    Rank_8 = 12,
    Rank_9 = 13,
  }
end

if not Enum.PvPRanksMeta then
  Enum.PvPRanksMeta = {
    MaxValue = 18,
    MinValue = 0,
    NumValues = 19,
  }
end

if not Enum.PvPUnitClassification then
  Enum.PvPUnitClassification = {
    AssassinAlliance = 6,
    AssassinHorde = 5,
    CartRunnerAlliance = 4,
    CartRunnerHorde = 3,
    FlagCarrierAlliance = 1,
    FlagCarrierHorde = 0,
    FlagCarrierNeutral = 2,
    OrbCarrierBlue = 7,
    OrbCarrierGreen = 8,
    OrbCarrierOrange = 9,
    OrbCarrierPurple = 10,
  }
end

if not Enum.PvPUnitClassificationMeta then
  Enum.PvPUnitClassificationMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.QuestClassification then
  Enum.QuestClassification = {
    BonusObjective = 8,
    Calling = 3,
    Campaign = 2,
    Important = 0,
    Legendary = 1,
    Meta = 4,
    Normal = 7,
    Questline = 6,
    Recurring = 5,
    Threat = 9,
    WorldQuest = 10,
  }
end

if not Enum.QuestClassificationMeta then
  Enum.QuestClassificationMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.QuestCompleteSpellTypeMeta then
  Enum.QuestCompleteSpellTypeMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.QuestFrequency then
  Enum.QuestFrequency = {
    Daily = 1,
    Default = 0,
    ResetByScheduler = 3,
    Weekly = 2,
  }
end

if not Enum.QuestFrequencyMeta then
  Enum.QuestFrequencyMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.QuestLineFloorLocation then
  Enum.QuestLineFloorLocation = {
    Above = 0,
    Below = 1,
    Same = 2,
  }
end

if not Enum.QuestLineFloorLocationMeta then
  Enum.QuestLineFloorLocationMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.QuestRepeatability then
  Enum.QuestRepeatability = {
    Daily = 1,
    None = 0,
    Turnin = 3,
    Weekly = 2,
    World = 4,
  }
end

if not Enum.QuestRepeatabilityMeta then
  Enum.QuestRepeatabilityMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.QuestRewardContextFlags then
  Enum.QuestRewardContextFlags = {
    FirstCompletionBonus = 1,
    None = 0,
    RepeatCompletionBonus = 2,
  }
end

if not Enum.QuestRewardContextFlagsMeta then
  Enum.QuestRewardContextFlagsMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.QuestSessionCommandMeta then
  Enum.QuestSessionCommandMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.QuestSessionResultMeta then
  Enum.QuestSessionResultMeta = {
    MaxValue = 34,
    MinValue = 0,
    NumValues = 35,
  }
end

if not Enum.QuestTag then
  Enum.QuestTag = {
    Account = 102,
    CombatAlly = 266,
    Delve = 288,
    Dungeon = 81,
    Group = 1,
    Heroic = 85,
    Legendary = 83,
    PvP = 41,
    Raid = 62,
    Raid10 = 88,
    Raid25 = 89,
    Scenario = 98,
  }
end

if not Enum.QuestTagMeta then
  Enum.QuestTagMeta = {
    MaxValue = 288,
    MinValue = 1,
    NumValues = 12,
  }
end

if not Enum.QuestTagType then
  Enum.QuestTagType = {
    Bounty = 5,
    Capstone = 17,
    Contribution = 9,
    CovenantCalling = 15,
    DragonRiderRacing = 16,
    Dungeon = 6,
    FactionAssault = 12,
    Invasion = 7,
    InvasionWrapper = 11,
    Islands = 13,
    Normal = 2,
    PetBattle = 4,
    Prey = 19,
    Profession = 1,
    PvP = 3,
    Raid = 8,
    RatedReward = 10,
    Tag = 0,
    Threat = 14,
    WorldBoss = 18,
  }
end

if not Enum.QuestTagTypeMeta then
  Enum.QuestTagTypeMeta = {
    MaxValue = 19,
    MinValue = 0,
    NumValues = 20,
  }
end

if not Enum.QuestTreasurePickerType then
  Enum.QuestTreasurePickerType = {
    Hidden = 1,
    Select = 2,
    Visible = 0,
  }
end

if not Enum.QuestTreasurePickerTypeMeta then
  Enum.QuestTreasurePickerTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.QuestWatchType then
  Enum.QuestWatchType = {
    Automatic = 0,
    Manual = 1,
  }
end

if not Enum.QuestWatchTypeMeta then
  Enum.QuestWatchTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.RafLinkType then
  Enum.RafLinkType = {
    Both = 3,
    Friend = 2,
    None = 0,
    Recruit = 1,
  }
end

if not Enum.RafLinkTypeMeta then
  Enum.RafLinkTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RafRecruitActivityState then
  Enum.RafRecruitActivityState = {
    Complete = 1,
    Incomplete = 0,
    RewardClaimed = 2,
  }
end

if not Enum.RafRecruitActivityStateMeta then
  Enum.RafRecruitActivityStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RafRecruitSubStatus then
  Enum.RafRecruitSubStatus = {
    Active = 1,
    Inactive = 2,
    Trial = 0,
  }
end

if not Enum.RafRecruitSubStatusMeta then
  Enum.RafRecruitSubStatusMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RafRewardType then
  Enum.RafRewardType = {
    Appearance = 2,
    AppearanceSet = 5,
    GameTime = 4,
    Illusion = 6,
    Invalid = 7,
    Mount = 1,
    Pet = 0,
    Title = 3,
  }
end

if not Enum.RafRewardTypeMeta then
  Enum.RafRewardTypeMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.RaidAuraOrganizationTypeMeta then
  Enum.RaidAuraOrganizationTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RaidDispelDisplayTypeMeta then
  Enum.RaidDispelDisplayTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RaidGroupDisplayTypeMeta then
  Enum.RaidGroupDisplayTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RcoCloseReason then
  Enum.RcoCloseReason = {
    Cancel = 2,
    CrafterFulfill = 5,
    Expire = 1,
    Fulfill = 0,
    GmCancel = 4,
    Invalid = 6,
    Reject = 3,
  }
end

if not Enum.RcoCloseReasonMeta then
  Enum.RcoCloseReasonMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.RecentAllyPinResult then
  Enum.RecentAllyPinResult = {
    ServerError = 1,
    Success = 0,
  }
end

if not Enum.RecentAllyPinResultMeta then
  Enum.RecentAllyPinResultMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.RecipeRequirementType then
  Enum.RecipeRequirementType = {
    Area = 2,
    SpellFocus = 0,
    Totem = 1,
  }
end

if not Enum.RecipeRequirementTypeMeta then
  Enum.RecipeRequirementTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RecruitAFriendRewardsVersionMeta then
  Enum.RecruitAFriendRewardsVersionMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RegisterAddonMessagePrefixResult then
  Enum.RegisterAddonMessagePrefixResult = {
    DuplicatePrefix = 1,
    InvalidPrefix = 2,
    MaxPrefixes = 3,
    Success = 0,
  }
end

if not Enum.RegisterAddonMessagePrefixResultMeta then
  Enum.RegisterAddonMessagePrefixResultMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RelativeContentDifficultyMeta then
  Enum.RelativeContentDifficultyMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ReleaseType then
  Enum.ReleaseType = {
    Classic = 2,
    Original = 1,
  }
end

if not Enum.ReleaseTypeMeta then
  Enum.ReleaseTypeMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.RenownRewardDisplayType then
  Enum.RenownRewardDisplayType = {
    Currency = 9,
    GarrFollower = 8,
    Item = 1,
    Mount = 3,
    None = 0,
    Spell = 2,
    Title = 7,
    Transmog = 4,
    TransmogIllusion = 6,
    TransmogSet = 5,
  }
end

if not Enum.RenownRewardDisplayTypeMeta then
  Enum.RenownRewardDisplayTypeMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.RenownRewardsFlags then
  Enum.RenownRewardsFlags = {
    AccountUnlock = 8,
    Capstone = 2,
    Hidden = 4,
    Milestone = 1,
  }
end

if not Enum.RenownRewardsFlagsMeta then
  Enum.RenownRewardsFlagsMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.ReportEvidenceType then
  Enum.ReportEvidenceType = {
    HouseLayoutJson = 2,
    HouseScreenshot = 1,
    Invalid = 0,
  }
end

if not Enum.ReportEvidenceTypeMeta then
  Enum.ReportEvidenceTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ReportMajorCategory then
  Enum.ReportMajorCategory = {
    Cheating = 2,
    GameplaySabotage = 1,
    InappropriateCommunication = 0,
    InappropriateDecor = 4,
    InappropriateName = 3,
  }
end

if not Enum.ReportMajorCategoryMeta then
  Enum.ReportMajorCategoryMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ReportMinorCategory then
  Enum.ReportMinorCategory = {
    Advertisement = 256,
    Afk = 8,
    BTag = 512,
    BlockingProgress = 32,
    Boosting = 2,
    Botting = 128,
    CharacterName = 2048,
    ChildSexualExploitationAndAbuse = 262144,
    Description = 8192,
    Disruption = 65536,
    GroupName = 1024,
    GuildName = 4096,
    Hacking = 64,
    HarmfulToMinors = 32768,
    IntentionallyFeeding = 16,
    Name = 16384,
    NeighborhoodName = 524288,
    Spam = 4,
    TerroristAndViolentExtremistContent = 131072,
    TextChat = 1,
  }
end

if not Enum.ReportMinorCategoryMeta then
  Enum.ReportMinorCategoryMeta = {
    MaxValue = 524288,
    MinValue = 1,
    NumValues = 20,
  }
end

if not Enum.ReportSubComplaintTypes then
  Enum.ReportSubComplaintTypes = {
    Advertising = 1,
    Inappropriate = 0,
  }
end

if not Enum.ReportSubComplaintTypesMeta then
  Enum.ReportSubComplaintTypesMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ReportThrottleType then
  Enum.ReportThrottleType = {
    Expensive = 2,
    General = 1,
    Invalid = 0,
  }
end

if not Enum.ReportThrottleTypeMeta then
  Enum.ReportThrottleTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ReportType then
  Enum.ReportType = {
    BattlePet = 10,
    Calendar = 11,
    Chat = 0,
    ClubFinderApplicant = 3,
    ClubFinderPosting = 2,
    ClubMember = 6,
    CraftingOrder = 16,
    Friend = 8,
    GroupFinderApplicant = 5,
    GroupFinderPosting = 4,
    GroupMember = 7,
    HousingDecor = 18,
    InWorld = 1,
    Mail = 12,
    Neighborhood = 19,
    NeighborhoodRoster = 20,
    Pet = 9,
    PvP = 13,
    PvPGroupMember = 15,
    PvPScoreboard = 14,
    RecentAlly = 17,
  }
end

if not Enum.ReportTypeMeta then
  Enum.ReportTypeMeta = {
    MaxValue = 20,
    MinValue = 0,
    NumValues = 21,
  }
end

if not Enum.ReputationSortTypeMeta then
  Enum.ReputationSortTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ReservationFlags then
  Enum.ReservationFlags = {
    Canceled = 2,
    None = 0,
    Plotless = 4,
    Relinquish = 1,
  }
end

if not Enum.ReservationFlagsMeta then
  Enum.ReservationFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ResidentType then
  Enum.ResidentType = {
    Manager = 1,
    Owner = 2,
    Resident = 0,
  }
end

if not Enum.ResidentTypeMeta then
  Enum.ResidentTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.RestrictPingsTo then
  Enum.RestrictPingsTo = {
    Assist = 2,
    Lead = 1,
    None = 0,
    TankHealer = 3,
  }
end

if not Enum.RestrictPingsToMeta then
  Enum.RestrictPingsToMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RetroactiveDecorRewardFlags then
  Enum.RetroactiveDecorRewardFlags = {
    AllCriteriaRequired = 1,
    None = 0,
  }
end

if not Enum.RetroactiveDecorRewardFlagsMeta then
  Enum.RetroactiveDecorRewardFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.RolodexContextIDType then
  Enum.RolodexContextIDType = {
    AreaID = 2,
    ItemID = 1,
    MapID = 3,
    None = 0,
  }
end

if not Enum.RolodexContextIDTypeMeta then
  Enum.RolodexContextIDTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RolodexContextLevelType then
  Enum.RolodexContextLevelType = {
    DelveTier = 3,
    Difficulty = 1,
    KeystoneLevel = 2,
    None = 0,
  }
end

if not Enum.RolodexContextLevelTypeMeta then
  Enum.RolodexContextLevelTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RolodexTypeFlags then
  Enum.RolodexTypeFlags = {
    HiddenFromHistory = 1,
    None = 0,
  }
end

if not Enum.RolodexTypeFlagsMeta then
  Enum.RolodexTypeFlagsMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.RolodexTypeMeta then
  Enum.RolodexTypeMeta = {
    MaxValue = 20,
    MinValue = 0,
    NumValues = 21,
  }
end

if not Enum.RoomConnectionType then
  Enum.RoomConnectionType = {
    All = 1,
    None = 0,
  }
end

if not Enum.RoomConnectionTypeMeta then
  Enum.RoomConnectionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.RuneforgePowerFilter then
  Enum.RuneforgePowerFilter = {
    All = 0,
    Available = 2,
    Relevant = 1,
    Unavailable = 3,
  }
end

if not Enum.RuneforgePowerFilterMeta then
  Enum.RuneforgePowerFilterMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.RuneforgePowerState then
  Enum.RuneforgePowerState = {
    Available = 0,
    Invalid = 2,
    Unavailable = 1,
  }
end

if not Enum.RuneforgePowerStateMeta then
  Enum.RuneforgePowerStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ScreenLocationTypeMeta then
  Enum.ScreenLocationTypeMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.ScreenshotSource then
  Enum.ScreenshotSource = {
    Automatic = 0,
    Cheat = 2,
    PlayerReport = 1,
  }
end

if not Enum.ScreenshotSourceMeta then
  Enum.ScreenshotSourceMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ScriptedAnimationBehavior then
  Enum.ScriptedAnimationBehavior = {
    None = 0,
    SourceCollideWithTarget = 4,
    SourceRecoil = 3,
    TargetKnockBack = 2,
    TargetShake = 1,
    UIParentShake = 5,
  }
end

if not Enum.ScriptedAnimationBehaviorMeta then
  Enum.ScriptedAnimationBehaviorMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.ScriptedAnimationFlags then
  Enum.ScriptedAnimationFlags = {
    UseTargetAsSource = 1,
  }
end

if not Enum.ScriptedAnimationFlagsMeta then
  Enum.ScriptedAnimationFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.ScriptedAnimationTrajectoryMeta then
  Enum.ScriptedAnimationTrajectoryMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.ScrubStringFlags then
  Enum.ScrubStringFlags = {
    AllowBarCodes = 2,
    None = 0,
    StripControlCodes = 4,
    TruncateNewLines = 1,
  }
end

if not Enum.ScrubStringFlagsMeta then
  Enum.ScrubStringFlagsMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SeasonID then
  Enum.SeasonID = {
    Fresh = 11,
    FreshHardcore = 12,
    Hardcore = 3,
    NoSeason = 0,
    SeasonOfDiscovery = 2,
    SeasonOfMastery = 1,
  }
end

if not Enum.SeasonIDMeta then
  Enum.SeasonIDMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.SecrecyLevel then
  Enum.SecrecyLevel = {
    AlwaysSecret = 1,
    ContextuallySecret = 2,
    NeverSecret = 0,
  }
end

if not Enum.SecrecyLevelMeta then
  Enum.SecrecyLevelMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SecretAspect then
  Enum.SecretAspect = {
    Alpha = 128,
    Attributes = 1,
    BarValue = 16384,
    Cooldown = 32768,
    CooldownStyle = 524288,
    Cursor = 1024,
    Desaturation = 4096,
    FrameLevel = 256,
    Hierarchy = 1,
    ID = 2,
    MinimumWidth = 131072,
    ObjectDebug = 1,
    ObjectName = 1,
    ObjectSecrets = 1,
    ObjectSecurity = 1,
    ObjectType = 1,
    Padding = 262144,
    Rotation = 65536,
    Scale = 64,
    ScrollRange = 512,
    SecureText = 16,
    Shown = 32,
    TexCoords = 8192,
    Text = 8,
    Toplevel = 4,
    VertexColor = 2048,
  }
end

if not Enum.SecretAspectMeta then
  Enum.SecretAspectMeta = {
    MaxValue = 524288,
    MinValue = 1,
    NumValues = 26,
  }
end

if not Enum.SelfResurrectOptionType then
  Enum.SelfResurrectOptionType = {
    Item = 1,
    Spell = 0,
  }
end

if not Enum.SelfResurrectOptionTypeMeta then
  Enum.SelfResurrectOptionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SendAddonMessageResult then
  Enum.SendAddonMessageResult = {
    AddOnMessageLockdown = 11,
    AddonMessageThrottle = 3,
    ChannelThrottle = 8,
    GeneralError = 9,
    InvalidChannel = 7,
    InvalidChatType = 4,
    InvalidMessage = 2,
    InvalidPrefix = 1,
    NotInGroup = 5,
    NotInGuild = 10,
    Success = 0,
    TargetOffline = 12,
    TargetRequired = 6,
  }
end

if not Enum.SendAddonMessageResultMeta then
  Enum.SendAddonMessageResultMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.SendReportResultMeta then
  Enum.SendReportResultMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.SharedStringFlag then
  Enum.SharedStringFlag = {
    InternalOnly = 1,
  }
end

if not Enum.SharedStringFlagMeta then
  Enum.SharedStringFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.ShutdownSessionReason then
  Enum.ShutdownSessionReason = {
    AccountLoginCommand = 4,
    Error = 6,
    ForceLogoutCommand = 2,
    LuaScript = 5,
    RealmSelectCommand = 3,
    RestartSessionCommand = 1,
    SessionEnded = 0,
  }
end

if not Enum.ShutdownSessionReasonMeta then
  Enum.ShutdownSessionReasonMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.Siflag then
  Enum.Siflag = {
    Affectedbyaltitude = 16,
    Affectedbysanity = 2048,
    AutocreatedByBroadcastText = 4,
    CasterOwnsTargetSound = 128,
    Disablepositionallpf = 1,
    Dontplayindoors = 256,
    Dontplayunderwater = 1024,
    Dontstopondeath = 4096,
    Ignoresuppressors = 64,
    Ignorevopriority = 16384,
    Looping = 512,
    Noduplicates = 32,
    None = 0,
    Onlyplayindoors = 32768,
    Onlyplayunderwater = 65536,
    Playonlyforowner = 8192,
    Playsequential = 8,
    UseModCastSpeed = 2,
  }
end

if not Enum.SiflagMeta then
  Enum.SiflagMeta = {
    MaxValue = 65536,
    MinValue = 0,
    NumValues = 18,
  }
end

if not Enum.SimpleOrderStatus then
  Enum.SimpleOrderStatus = {
    Creating = 1,
    Failed = 4,
    InProgress = 2,
    Invalid = 0,
    Success = 3,
  }
end

if not Enum.SimpleOrderStatusMeta then
  Enum.SimpleOrderStatusMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.SkinningState then
  Enum.SkinningState = {
    Looting = 3,
    None = 0,
    Reserved = 1,
    Skinned = 4,
    Skinning = 2,
  }
end

if not Enum.SkinningStateMeta then
  Enum.SkinningStateMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.SleevesGeoRange then
  Enum.SleevesGeoRange = {
    Default = 1,
    Flared = 2,
    None = 0,
    PandaCollar = 4,
    Puffy = 3,
  }
end

if not Enum.SleevesGeoRangeMeta then
  Enum.SleevesGeoRangeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.SlotRegion then
  Enum.SlotRegion = {
    AccountBank = 5,
    CharacterBank = 4,
    Invalid = 0,
    PlayerBags = 2,
    PlayerEquip = 1,
    PlayerInv = 3,
  }
end

if not Enum.SlotRegionMask then
  Enum.SlotRegionMask = {
    AccountBank = 64,
    CharacterBank = 16,
    Invalid = 1,
    PlayerBags = 4,
    PlayerEquip = 2,
    PlayerInv = 8,
  }
end

if not Enum.SlotRegionMaskMeta then
  Enum.SlotRegionMaskMeta = {
    MaxValue = 64,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.SlotRegionMeta then
  Enum.SlotRegionMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.SocialWhoOrigin then
  Enum.SocialWhoOrigin = {
    Chat = 2,
    Item = 3,
    Social = 1,
    Unknown = 0,
  }
end

if not Enum.SocialWhoOriginMeta then
  Enum.SocialWhoOriginMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SoftTargetEnableFlags then
  Enum.SoftTargetEnableFlags = {
    Any = 3,
    Gamepad = 1,
    Kbm = 2,
    None = 0,
  }
end

if not Enum.SoftTargetEnableFlagsMeta then
  Enum.SoftTargetEnableFlagsMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SortPlayersByMeta then
  Enum.SortPlayersByMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SoulbindConduitFlags then
  Enum.SoulbindConduitFlags = {
    VisibleToGetallsoulbindconduitScript = 1,
  }
end

if not Enum.SoulbindConduitFlagsMeta then
  Enum.SoulbindConduitFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.SoulbindConduitInstallResult then
  Enum.SoulbindConduitInstallResult = {
    DuplicateConduit = 4,
    ForgeNotInProximity = 5,
    InvalidConduit = 2,
    InvalidItem = 1,
    InvalidTalent = 3,
    SocketNotEmpty = 6,
    Success = 0,
  }
end

if not Enum.SoulbindConduitInstallResultMeta then
  Enum.SoulbindConduitInstallResultMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.SoulbindConduitTransactionType then
  Enum.SoulbindConduitTransactionType = {
    Install = 0,
    Uninstall = 1,
  }
end

if not Enum.SoulbindConduitTransactionTypeMeta then
  Enum.SoulbindConduitTransactionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SoulbindConduitType then
  Enum.SoulbindConduitType = {
    Endurance = 2,
    Finesse = 0,
    Flex = 3,
    Potency = 1,
  }
end

if not Enum.SoulbindConduitTypeMeta then
  Enum.SoulbindConduitTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SoulbindNodeState then
  Enum.SoulbindNodeState = {
    Selectable = 2,
    Selected = 3,
    Unavailable = 0,
    Unselected = 1,
  }
end

if not Enum.SoulbindNodeStateMeta then
  Enum.SoulbindNodeStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SoundBusFlag then
  Enum.SoundBusFlag = {
    Disablepositionallpf = 1,
  }
end

if not Enum.SoundBusFlagMeta then
  Enum.SoundBusFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.SpecializationSystem then
  Enum.SpecializationSystem = {
    ChrSpecialization = 1,
    TalentTab = 0,
  }
end

if not Enum.SpecializationSystemMeta then
  Enum.SpecializationSystemMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SpellAuraVisibilityTypeMeta then
  Enum.SpellAuraVisibilityTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SpellBookItemTypeMeta then
  Enum.SpellBookItemTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.SpellBookSkillLineIndexMeta then
  Enum.SpellBookSkillLineIndexMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.SpellBookSpellBankMeta then
  Enum.SpellBookSpellBankMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SpellDiminishCategory then
  Enum.SpellDiminishCategory = {
    AoEKnockback = 3,
    Disarm = 7,
    Disorient = 5,
    Incapacitate = 4,
    Root = 0,
    Silence = 6,
    Stun = 2,
    Taunt = 1,
  }
end

if not Enum.SpellDiminishCategoryMeta then
  Enum.SpellDiminishCategoryMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.SpellDiminishRuleset then
  Enum.SpellDiminishRuleset = {
    None = 0,
    PvE = 1,
    PvP = 2,
  }
end

if not Enum.SpellDiminishRulesetMeta then
  Enum.SpellDiminishRulesetMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SpellDisplayBorderColorMeta then
  Enum.SpellDisplayBorderColorMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.SpellDisplayIconDisplayType then
  Enum.SpellDisplayIconDisplayType = {
    Buff = 0,
    Circular = 2,
    Debuff = 1,
    NoBorder = 3,
  }
end

if not Enum.SpellDisplayIconDisplayTypeMeta then
  Enum.SpellDisplayIconDisplayTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SpellDisplayTextShownStateType then
  Enum.SpellDisplayTextShownStateType = {
    Hidden = 1,
    Shown = 0,
  }
end

if not Enum.SpellDisplayTextShownStateTypeMeta then
  Enum.SpellDisplayTextShownStateTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SpellDisplayTint then
  Enum.SpellDisplayTint = {
    None = 0,
    Red = 1,
  }
end

if not Enum.SpellDisplayTintMeta then
  Enum.SpellDisplayTintMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SplashScreenType then
  Enum.SplashScreenType = {
    SeasonRollOver = 1,
    WhatsNew = 0,
  }
end

if not Enum.SplashScreenTypeMeta then
  Enum.SplashScreenTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.StableResult then
  Enum.StableResult = {
    AlreadyStabled = 5,
    AlreadySummoned = 6,
    BuySlotSuccess = 14,
    CantControlExotic = 11,
    CheckForLuaHack = 13,
    FavoriteToggle = 15,
    InsufficientFunds = 1,
    InternalError = 12,
    InvalidSlot = 3,
    MaxSlots = 0,
    NoPet = 4,
    NotFound = 7,
    NotStableMaster = 2,
    PetRenamed = 16,
    ReviveSuccess = 10,
    StableSuccess = 8,
    UnstableSuccess = 9,
  }
end

if not Enum.StableResultMeta then
  Enum.StableResultMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 17,
  }
end

if not Enum.StartTimerTypeMeta then
  Enum.StartTimerTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.StatusBarColorTintValueMeta then
  Enum.StatusBarColorTintValueMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.StatusBarFillStyleMeta then
  Enum.StatusBarFillStyleMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.StatusBarInterpolation then
  Enum.StatusBarInterpolation = {
    ExponentialEaseOut = 1,
    Immediate = 0,
  }
end

if not Enum.StatusBarInterpolationMeta then
  Enum.StatusBarInterpolationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.StatusBarOverrideBarTextShownType then
  Enum.StatusBarOverrideBarTextShownType = {
    Always = 1,
    Never = 0,
    OnlyNotOnMouseover = 3,
    OnlyOnMouseover = 2,
  }
end

if not Enum.StatusBarOverrideBarTextShownTypeMeta then
  Enum.StatusBarOverrideBarTextShownTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.StatusBarTimerDirection then
  Enum.StatusBarTimerDirection = {
    ElapsedTime = 0,
    RemainingTime = 1,
  }
end

if not Enum.StatusBarTimerDirectionMeta then
  Enum.StatusBarTimerDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.StatusBarValueTextTypeMeta then
  Enum.StatusBarValueTextTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.StoreError then
  Enum.StoreError = {
    AlreadyOwned = 7,
    BattlepayDisabled = 4,
    ClientRestricted = 13,
    ConsumableTokenOwned = 10,
    InsufficientBalance = 5,
    InvalidPaymentMethod = 1,
    ItemUnavailable = 12,
    Other = 6,
    ParentalControlsNoPurchase = 8,
    PaymentFailed = 2,
    PurchaseDenied = 9,
    Success = 0,
    TooManyTokens = 11,
    WrongCurrency = 3,
  }
end

if not Enum.StoreErrorMeta then
  Enum.StoreErrorMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.SubcontainerType then
  Enum.SubcontainerType = {
    AccountBankTabs = 36,
    Auction = 5,
    Bag = 0,
    Bankbag = 3,
    Bankgeneric = 2,
    BuybackSlots = 26,
    CachedReward = 27,
    CharacterBankTabs = 38,
    Childequipmentstorage = 23,
    CraftingOrder = 34,
    CraftingOrderReagents = 35,
    CreatedImmediately = 25,
    CurrencyTransfer = 37,
    CurrencytokenOboslete = 15,
    Equipablespells = 14,
    Equipped = 1,
    EquippedBags = 28,
    EquippedCooking = 31,
    EquippedFishing = 32,
    EquippedProfession1 = 29,
    EquippedProfession2 = 30,
    EquippedReagentbag = 33,
    GuildBank0 = 7,
    GuildBank1 = 8,
    GuildBank10 = 20,
    GuildBank11 = 21,
    GuildBank2 = 9,
    GuildBank3 = 10,
    GuildBank4 = 11,
    GuildBank5 = 12,
    GuildBank6 = 16,
    GuildBank7 = 17,
    GuildBank8 = 18,
    GuildBank9 = 19,
    GuildOverflow = 13,
    HousingDecorConversion = 39,
    Keyring = 6,
    Mail = 4,
    Quarantine = 24,
    Reagentbank = 22,
  }
end

if not Enum.SubcontainerTypeMeta then
  Enum.SubcontainerTypeMeta = {
    MaxValue = 39,
    MinValue = 0,
    NumValues = 40,
  }
end

if not Enum.SubscriptionInterstitialResponseType then
  Enum.SubscriptionInterstitialResponseType = {
    Clicked = 0,
    Closed = 1,
    WebRedirect = 2,
  }
end

if not Enum.SubscriptionInterstitialResponseTypeMeta then
  Enum.SubscriptionInterstitialResponseTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SubscriptionInterstitialType then
  Enum.SubscriptionInterstitialType = {
    LeftNpeArea = 1,
    MaxLevel = 2,
    Standard = 0,
  }
end

if not Enum.SubscriptionInterstitialTypeMeta then
  Enum.SubscriptionInterstitialTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.SummonReason then
  Enum.SummonReason = {
    Scenario = 1,
    Spell = 0,
  }
end

if not Enum.SummonReasonMeta then
  Enum.SummonReasonMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SummonStatus then
  Enum.SummonStatus = {
    Accepted = 2,
    Declined = 3,
    None = 0,
    Pending = 1,
  }
end

if not Enum.SummonStatusMeta then
  Enum.SummonStatusMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.SuperTrackingMapPinTypeMeta then
  Enum.SuperTrackingMapPinTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.SuperTrackingTypeMeta then
  Enum.SuperTrackingTypeMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.SurveyDeliveryMoment then
  Enum.SurveyDeliveryMoment = {
    ChestLooted = 3,
    Login = 0,
    MythicPlusCompleted = 4,
    ProfessionTable = 1,
    QuestTurnIn = 2,
  }
end

if not Enum.SurveyDeliveryMomentMeta then
  Enum.SurveyDeliveryMomentMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TableSecurityOption then
  Enum.TableSecurityOption = {
    DisallowSecretKeys = 1,
    DisallowTaintedAccess = 0,
    SecretWrapContents = 2,
  }
end

if not Enum.TableSecurityOptionMeta then
  Enum.TableSecurityOptionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TimeEventFlag then
  Enum.TimeEventFlag = {
    GlobalLaunch = 4,
    GlueScreenShortcut = 1,
    WeeklyReset = 2,
  }
end

if not Enum.TimeEventFlagMeta then
  Enum.TimeEventFlagMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TitleIconVersion then
  Enum.TitleIconVersion = {
    Large = 2,
    Medium = 1,
    Small = 0,
  }
end

if not Enum.TitleIconVersionMeta then
  Enum.TitleIconVersionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TooltipComparisonMethod then
  Enum.TooltipComparisonMethod = {
    Single = 0,
    WithBagMainHandItem = 2,
    WithBagOffHandItem = 3,
    WithBothHands = 1,
  }
end

if not Enum.TooltipComparisonMethodMeta then
  Enum.TooltipComparisonMethodMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TooltipDataItemBinding then
  Enum.TooltipDataItemBinding = {
    Account = 1,
    AccountUntilEquipped = 9,
    BindOnEquip = 7,
    BindOnPickup = 6,
    BindOnUse = 8,
    BindToAccount = 4,
    BindToAccountUntilEquipped = 10,
    BindToBnetAccount = 5,
    BnetAccount = 2,
    Quest = 0,
    Soulbound = 3,
  }
end

if not Enum.TooltipDataItemBindingMeta then
  Enum.TooltipDataItemBindingMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.TooltipDataLineType then
  Enum.TooltipDataLineType = {
    AzeriteEssencePower = 5,
    AzeriteEssenceSlot = 4,
    AzeriteItemPowerDescription = 9,
    Blank = 1,
    CurrencyTotal = 14,
    DisabledLine = 42,
    EquipSlot = 21,
    ErrorLine = 41,
    FlavorText = 37,
    GemSocket = 3,
    GemSocketEnchantment = 30,
    ItemBinding = 20,
    ItemEnchantmentPermanent = 15,
    ItemLevel = 31,
    ItemName = 22,
    ItemQuality = 35,
    ItemSpellTriggerLearn = 38,
    ItemUpgradeLevel = 32,
    LearnTransmogIllusion = 40,
    LearnTransmogSet = 39,
    LearnableSpell = 6,
    NestedBlock = 19,
    None = 0,
    ProfessionCraftingQuality = 12,
    QuestObjective = 8,
    QuestPlayer = 18,
    QuestTitle = 17,
    RuneforgeLegendaryPowerDescription = 10,
    SellPrice = 11,
    Separator = 23,
    SpellDescription = 34,
    SpellName = 13,
    SpellPassive = 33,
    ToyDescription = 28,
    ToyDuration = 27,
    ToyEffect = 26,
    ToyName = 24,
    ToySource = 29,
    ToyText = 25,
    TradeTimeRemaining = 36,
    UnitName = 2,
    UnitOwner = 16,
    UnitThreat = 7,
    UsageRequirement = 43,
  }
end

if not Enum.TooltipDataLineTypeMeta then
  Enum.TooltipDataLineTypeMeta = {
    MaxValue = 43,
    MinValue = 0,
    NumValues = 44,
  }
end

if not Enum.TooltipDataType then
  Enum.TooltipDataType = {
    Achievement = 12,
    AzeriteEssence = 8,
    BattlePet = 6,
    CompanionPet = 9,
    Corpse = 3,
    CorruptionCleanser = 20,
    Currency = 5,
    Debug = 26,
    EnhancedConduit = 13,
    EquipmentSet = 14,
    Flyout = 22,
    InstanceLock = 15,
    Item = 0,
    Macro = 25,
    MinimapMouseover = 21,
    Mount = 10,
    Object = 4,
    Outfit = 27,
    PetAction = 11,
    PvPBrawl = 16,
    Quest = 23,
    QuestPartyProgress = 24,
    RecipeRankInfo = 17,
    Spell = 1,
    Totem = 18,
    Toy = 19,
    Unit = 2,
    UnitAura = 7,
  }
end

if not Enum.TooltipDataTypeMeta then
  Enum.TooltipDataTypeMeta = {
    MaxValue = 27,
    MinValue = 0,
    NumValues = 28,
  }
end

if not Enum.TooltipDataUsageRequirementType then
  Enum.TooltipDataUsageRequirementType = {
    Achievement = 7,
    ArenaRating = 11,
    EarnCurrency = 12,
    EquippedItem = 9,
    Faction = 1,
    Guild = 6,
    Level = 5,
    NotAlreadyKnown = 14,
    NotInArena = 15,
    NotInRatedBg = 16,
    PvPMedal = 3,
    PvPRank = 8,
    RaceClass = 0,
    Reputation = 4,
    ShapeshiftForm = 10,
    Skill = 2,
    Specialization = 13,
  }
end

if not Enum.TooltipDataUsageRequirementTypeMeta then
  Enum.TooltipDataUsageRequirementTypeMeta = {
    MaxValue = 16,
    MinValue = 0,
    NumValues = 17,
  }
end

if not Enum.TooltipSide then
  Enum.TooltipSide = {
    Bottom = 3,
    Left = 0,
    Right = 1,
    Top = 2,
  }
end

if not Enum.TooltipSideMeta then
  Enum.TooltipSideMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TooltipTextureAnchor then
  Enum.TooltipTextureAnchor = {
    All = 6,
    LeftBottom = 2,
    LeftCenter = 1,
    LeftTop = 0,
    RightBottom = 5,
    RightCenter = 4,
    RightTop = 3,
  }
end

if not Enum.TooltipTextureAnchorMeta then
  Enum.TooltipTextureAnchorMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.TooltipTextureRelativeRegion then
  Enum.TooltipTextureRelativeRegion = {
    LeftLine = 0,
    RightLine = 1,
  }
end

if not Enum.TooltipTextureRelativeRegionMeta then
  Enum.TooltipTextureRelativeRegionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TrackedSpellCategory then
  Enum.TrackedSpellCategory = {
    Debuff = 3,
    Defensive = 2,
    None = 0,
    Offensive = 1,
    RacialAbility = 4,
  }
end

if not Enum.TrackedSpellCategoryMeta then
  Enum.TrackedSpellCategoryMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TradeskillOrderDuration then
  Enum.TradeskillOrderDuration = {
    Long = 3,
    Medium = 2,
    Short = 1,
  }
end

if not Enum.TradeskillOrderDurationMeta then
  Enum.TradeskillOrderDurationMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TradeskillOrderRecipient then
  Enum.TradeskillOrderRecipient = {
    Guild = 2,
    Private = 3,
    Public = 1,
  }
end

if not Enum.TradeskillOrderRecipientMeta then
  Enum.TradeskillOrderRecipientMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TradeskillOrderStatus then
  Enum.TradeskillOrderStatus = {
    Completed = 3,
    Expired = 4,
    Started = 2,
    Unclaimed = 1,
  }
end

if not Enum.TradeskillOrderStatusMeta then
  Enum.TradeskillOrderStatusMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.TradeskillRecipeType then
  Enum.TradeskillRecipeType = {
    Enchant = 3,
    Gathering = 4,
    Item = 1,
    Salvage = 2,
  }
end

if not Enum.TradeskillRecipeTypeMeta then
  Enum.TradeskillRecipeTypeMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.TradeskillRelativeDifficulty then
  Enum.TradeskillRelativeDifficulty = {
    Easy = 2,
    Medium = 1,
    Optimal = 0,
    Trivial = 3,
  }
end

if not Enum.TradeskillRelativeDifficultyMeta then
  Enum.TradeskillRelativeDifficultyMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TradeskillSlotDataType then
  Enum.TradeskillSlotDataType = {
    Currency = 3,
    ModifiedReagent = 2,
    Reagent = 1,
  }
end

if not Enum.TradeskillSlotDataTypeMeta then
  Enum.TradeskillSlotDataTypeMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TraitCombatConfigFlags then
  Enum.TraitCombatConfigFlags = {
    ActiveForSpec = 1,
    SharedActionBars = 4,
    StarterBuild = 2,
  }
end

if not Enum.TraitCombatConfigFlagsMeta then
  Enum.TraitCombatConfigFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TraitCondFlag then
  Enum.TraitCondFlag = {
    IsAlwaysMet = 2,
    IsGate = 1,
    IsSufficient = 4,
  }
end

if not Enum.TraitCondFlagMeta then
  Enum.TraitCondFlagMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TraitConditionType then
  Enum.TraitConditionType = {
    Available = 0,
    DisplayError = 4,
    Granted = 2,
    Increased = 3,
    RanksAllowed = 5,
    Visible = 1,
  }
end

if not Enum.TraitConditionTypeMeta then
  Enum.TraitConditionTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.TraitConfigDbState then
  Enum.TraitConfigDbState = {
    Created = 1,
    Deleted = 3,
    Ready = 0,
    Removed = 2,
  }
end

if not Enum.TraitConfigDbStateMeta then
  Enum.TraitConfigDbStateMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TraitConfigType then
  Enum.TraitConfigType = {
    Combat = 1,
    Generic = 3,
    Invalid = 0,
    Profession = 2,
  }
end

if not Enum.TraitConfigTypeMeta then
  Enum.TraitConfigTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TraitCurrencyFlag then
  Enum.TraitCurrencyFlag = {
    ShowQuantityAsSpent = 1,
    TraitSourcedShowMax = 2,
    UseClassIcon = 4,
    UseSpecIcon = 8,
  }
end

if not Enum.TraitCurrencyFlagMeta then
  Enum.TraitCurrencyFlagMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.TraitCurrencyType then
  Enum.TraitCurrencyType = {
    CurrencyTypesBased = 1,
    Gold = 0,
    TraitSourced = 2,
    TraitSourcedPlayerDataElement = 3,
  }
end

if not Enum.TraitCurrencyTypeMeta then
  Enum.TraitCurrencyTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TraitDefinitionSubTypeMeta then
  Enum.TraitDefinitionSubTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TraitEdgeType then
  Enum.TraitEdgeType = {
    DeprecatedRankConnection = 1,
    DeprecatedSelectionOption = 5,
    MutuallyExclusive = 4,
    RequiredForAvailability = 3,
    SufficientForAvailability = 2,
    VisualOnly = 0,
  }
end

if not Enum.TraitEdgeTypeMeta then
  Enum.TraitEdgeTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.TraitEdgeVisualStyleMeta then
  Enum.TraitEdgeVisualStyleMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TraitNodeEntryTypeMeta then
  Enum.TraitNodeEntryTypeMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.TraitNodeFlag then
  Enum.TraitNodeFlag = {
    ActiveAtFirstRank = 16,
    HideMaxRank = 64,
    HighestChosenRank = 128,
    NeverPurchasable = 2,
    ShowExpandedSelection = 32,
    ShowMultipleIcons = 1,
    ShowTierTrack = 256,
    TestGridPositioned = 8,
    TestPositionLocked = 4,
  }
end

if not Enum.TraitNodeFlagMeta then
  Enum.TraitNodeFlagMeta = {
    MaxValue = 256,
    MinValue = 1,
    NumValues = 9,
  }
end

if not Enum.TraitNodeGroupFlag then
  Enum.TraitNodeGroupFlag = {
    AvailableByDefault = 1,
  }
end

if not Enum.TraitNodeGroupFlagMeta then
  Enum.TraitNodeGroupFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.TraitNodeTypeMeta then
  Enum.TraitNodeTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TraitPointsOperationType then
  Enum.TraitPointsOperationType = {
    Multiply = 1,
    None = -1,
    Set = 0,
  }
end

if not Enum.TraitPointsOperationTypeMeta then
  Enum.TraitPointsOperationTypeMeta = {
    MaxValue = 1,
    MinValue = -1,
    NumValues = 3,
  }
end

if not Enum.TraitSystemFlag then
  Enum.TraitSystemFlag = {
    AllowEditInChallengeMode = 8,
    AllowEditInCombat = 4,
    AllowMultipleLoadoutsPerTree = 1,
    ShowSpendConfirmation = 2,
  }
end

if not Enum.TraitSystemFlagMeta then
  Enum.TraitSystemFlagMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.TraitSystemVariationType then
  Enum.TraitSystemVariationType = {
    None = 0,
    Spec = 1,
  }
end

if not Enum.TraitSystemVariationTypeMeta then
  Enum.TraitSystemVariationTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TraitTreeFlag then
  Enum.TraitTreeFlag = {
    CannotRefund = 1,
    HideSingleRankNumbers = 2,
  }
end

if not Enum.TraitTreeFlagMeta then
  Enum.TraitTreeFlagMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.TransformManipulatorAxis then
  Enum.TransformManipulatorAxis = {
    X = 1,
    Y = 2,
    Z = 4,
  }
end

if not Enum.TransformManipulatorAxisMeta then
  Enum.TransformManipulatorAxisMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TransformManipulatorControlState then
  Enum.TransformManipulatorControlState = {
    Default = 0,
    Dimmed = 2,
    ExternallyHighlighted = 3,
    Hidden = 1,
    Hovered = 4,
    Moving = 6,
    Selected = 5,
  }
end

if not Enum.TransformManipulatorControlStateMeta then
  Enum.TransformManipulatorControlStateMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.TransformManipulatorDirection then
  Enum.TransformManipulatorDirection = {
    Negative = 1,
    Positive = 0,
  }
end

if not Enum.TransformManipulatorDirectionMeta then
  Enum.TransformManipulatorDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TransformManipulatorEvent then
  Enum.TransformManipulatorEvent = {
    Cancel = 3,
    Change = 1,
    Complete = 2,
    Hover = 4,
    MouseDown = 5,
    MouseUp = 6,
    Start = 0,
  }
end

if not Enum.TransformManipulatorEventMeta then
  Enum.TransformManipulatorEventMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.TransformManipulatorMode then
  Enum.TransformManipulatorMode = {
    Rotate = 1,
    Scale = 2,
    Translate = 0,
  }
end

if not Enum.TransformManipulatorModeMeta then
  Enum.TransformManipulatorModeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TransmogCameraVariation then
  Enum.TransmogCameraVariation = {
    CloakBackpack = 1,
    None = 0,
    RightShoulder = 1,
  }
end

if not Enum.TransmogCameraVariationMeta then
  Enum.TransmogCameraVariationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TransmogCollectionType then
  Enum.TransmogCollectionType = {
    Back = 3,
    Bow = 25,
    Chest = 4,
    Crossbow = 27,
    Dagger = 16,
    Feet = 11,
    Fist = 17,
    Gun = 26,
    Hands = 8,
    Head = 1,
    Holdable = 19,
    Legs = 10,
    None = 0,
    OneHAxe = 13,
    OneHMace = 15,
    OneHSword = 14,
    Paired = 29,
    Polearm = 24,
    Shield = 18,
    Shirt = 5,
    Shoulder = 2,
    Staff = 23,
    Tabard = 6,
    TwoHAxe = 20,
    TwoHMace = 22,
    TwoHSword = 21,
    Waist = 9,
    Wand = 12,
    Warglaives = 28,
    Wrist = 7,
  }
end

if not Enum.TransmogCollectionTypeMeta then
  Enum.TransmogCollectionTypeMeta = {
    MaxValue = 29,
    MinValue = 0,
    NumValues = 30,
  }
end

if not Enum.TransmogIllusionFlags then
  Enum.TransmogIllusionFlags = {
    HideUntilCollected = 1,
    PlayerConditionGrantsOnLogin = 2,
  }
end

if not Enum.TransmogIllusionFlagsMeta then
  Enum.TransmogIllusionFlagsMeta = {
    MaxValue = 2,
    MinValue = 1,
    NumValues = 2,
  }
end

if not Enum.TransmogModification then
  Enum.TransmogModification = {
    Main = 0,
    Secondary = 1,
  }
end

if not Enum.TransmogModificationMeta then
  Enum.TransmogModificationMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TransmogOutfitCostModifiersApplied then
  Enum.TransmogOutfitCostModifiersApplied = {
    AuraDiscountApplied = 8,
    DebugOnlyFreeDiscountApplied = 1,
    OutfitCostModifierApplied = 4,
    VoidRacialDiscountApplied = 2,
  }
end

if not Enum.TransmogOutfitCostModifiersAppliedMeta then
  Enum.TransmogOutfitCostModifiersAppliedMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.TransmogOutfitDataFlags then
  Enum.TransmogOutfitDataFlags = {
    IsCachedLocally = 1,
  }
end

if not Enum.TransmogOutfitDataFlagsMeta then
  Enum.TransmogOutfitDataFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.TransmogOutfitDisplayType then
  Enum.TransmogOutfitDisplayType = {
    Assigned = 1,
    Disabled = 4,
    Equipped = 2,
    Hidden = 3,
    Unassigned = 0,
  }
end

if not Enum.TransmogOutfitDisplayTypeMeta then
  Enum.TransmogOutfitDisplayTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TransmogOutfitEntryFlags then
  Enum.TransmogOutfitEntryFlags = {
    AutomaticallyAwardedOnLogin = 1,
    IsDefaultEquipped = 32,
    OnlyAvailableDuringEvent = 4,
    SortedToTopOfList = 8,
    UseOverrideCostModifier = 16,
    UseOverrideName = 2,
  }
end

if not Enum.TransmogOutfitEntryFlagsMeta then
  Enum.TransmogOutfitEntryFlagsMeta = {
    MaxValue = 32,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.TransmogOutfitEntrySource then
  Enum.TransmogOutfitEntrySource = {
    AutomaticallyAwarded = 1,
    PlayerPurchased = 2,
    StampedSource = 0,
  }
end

if not Enum.TransmogOutfitEntrySourceMeta then
  Enum.TransmogOutfitEntrySourceMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TransmogOutfitEquipAction then
  Enum.TransmogOutfitEquipAction = {
    Equip = 0,
    EquipAndLock = 1,
    Lock = 5,
    Remove = 2,
    RemoveAndLock = 3,
    Unlock = 4,
  }
end

if not Enum.TransmogOutfitEquipActionMeta then
  Enum.TransmogOutfitEquipActionMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.TransmogOutfitSetType then
  Enum.TransmogOutfitSetType = {
    CustomSet = 2,
    Equipped = 0,
    Outfit = 1,
  }
end

if not Enum.TransmogOutfitSetTypeMeta then
  Enum.TransmogOutfitSetTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TransmogOutfitSlot then
  Enum.TransmogOutfitSlot = {
    Back = 3,
    Body = 6,
    Chest = 4,
    Feet = 11,
    Hand = 8,
    Head = 0,
    Legs = 10,
    ShoulderLeft = 2,
    ShoulderRight = 1,
    Tabard = 5,
    Waist = 9,
    WeaponMainHand = 12,
    WeaponOffHand = 13,
    WeaponRanged = 14,
    Wrist = 7,
  }
end

if not Enum.TransmogOutfitSlotError then
  Enum.TransmogOutfitSlotError = {
    CannotUseItem = 10,
    IncompatibleWithMainHand = 14,
    InvalidDestination = 5,
    InvalidItemType = 4,
    InvalidSlotForForm = 13,
    InvalidSlotForRace = 11,
    InvalidSource = 8,
    InvalidSourceQuality = 9,
    Legendary = 3,
    Mismatch = 6,
    NoIllusion = 12,
    NoItem = 1,
    NotSoulbound = 2,
    Ok = 0,
    SameItem = 7,
  }
end

if not Enum.TransmogOutfitSlotErrorMeta then
  Enum.TransmogOutfitSlotErrorMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.TransmogOutfitSlotFlags then
  Enum.TransmogOutfitSlotFlags = {
    CanHaveIllusions = 2,
    CannotBeHidden = 1,
    IsSecondarySlot = 4,
  }
end

if not Enum.TransmogOutfitSlotFlagsMeta then
  Enum.TransmogOutfitSlotFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TransmogOutfitSlotMeta then
  Enum.TransmogOutfitSlotMeta = {
    MaxValue = 14,
    MinValue = 0,
    NumValues = 15,
  }
end

if not Enum.TransmogOutfitSlotOption then
  Enum.TransmogOutfitSlotOption = {
    ArtifactSpecFour = 11,
    ArtifactSpecOne = 8,
    ArtifactSpecThree = 10,
    ArtifactSpecTwo = 9,
    DeprecatedReuseMe = 6,
    FuryTwoHandedWeapon = 7,
    None = 0,
    OffHand = 4,
    OneHandedWeapon = 1,
    RangedWeapon = 3,
    Shield = 5,
    TwoHandedWeapon = 2,
  }
end

if not Enum.TransmogOutfitSlotOptionFlags then
  Enum.TransmogOutfitSlotOptionFlags = {
    DisablesOffhandSlot = 4,
    DynamicOptionName = 2,
    IllusionNotAllowed = 1,
  }
end

if not Enum.TransmogOutfitSlotOptionFlagsMeta then
  Enum.TransmogOutfitSlotOptionFlagsMeta = {
    MaxValue = 4,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TransmogOutfitSlotOptionMeta then
  Enum.TransmogOutfitSlotOptionMeta = {
    MaxValue = 11,
    MinValue = 0,
    NumValues = 12,
  }
end

if not Enum.TransmogOutfitSlotPosition then
  Enum.TransmogOutfitSlotPosition = {
    Bottom = 2,
    Left = 0,
    Right = 1,
  }
end

if not Enum.TransmogOutfitSlotPositionMeta then
  Enum.TransmogOutfitSlotPositionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TransmogOutfitSlotSaveFlags then
  Enum.TransmogOutfitSlotSaveFlags = {
    AppearanceIsNotValid = 1,
  }
end

if not Enum.TransmogOutfitSlotSaveFlagsMeta then
  Enum.TransmogOutfitSlotSaveFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.TransmogOutfitSlotWarning then
  Enum.TransmogOutfitSlotWarning = {
    InvalidEquippedDestinationItem = 1,
    NothingEquipped = 5,
    Ok = 0,
    PendingWeaponChanges = 3,
    WeaponDoesNotSupportIllusions = 4,
    WrongWeaponCategoryEquipped = 2,
  }
end

if not Enum.TransmogOutfitSlotWarningMeta then
  Enum.TransmogOutfitSlotWarningMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.TransmogOutfitTransactionFlags then
  Enum.TransmogOutfitTransactionFlags = {
    AddNewOutfitMask = 20,
    AddOutfitAndUpdateSlots = 28,
    CreateAndUpdateOutfitInfoMask = 6,
    CreateOutfitInfo = 4,
    FullOutfitUpdateMask = 27,
    UpdateMetadata = 1,
    UpdateOutfitInfo = 2,
    UpdateSituations = 16,
    UpdateSituationsMask = 18,
    UpdateSlots = 8,
  }
end

if not Enum.TransmogOutfitTransactionFlagsMeta then
  Enum.TransmogOutfitTransactionFlagsMeta = {
    MaxValue = 28,
    MinValue = 1,
    NumValues = 10,
  }
end

if not Enum.TransmogOutfitTransactionType then
  Enum.TransmogOutfitTransactionType = {
    CreateOutfitInfo = 2,
    UpdateMetadata = 0,
    UpdateOutfitInfo = 1,
    UpdateSituations = 4,
    UpdateSlots = 3,
  }
end

if not Enum.TransmogOutfitTransactionTypeMeta then
  Enum.TransmogOutfitTransactionTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TransmogPendingType then
  Enum.TransmogPendingType = {
    Apply = 0,
    Revert = 1,
    ToggleOff = 3,
    ToggleOn = 2,
  }
end

if not Enum.TransmogPendingTypeMeta then
  Enum.TransmogPendingTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TransmogSearchType then
  Enum.TransmogSearchType = {
    BaseSets = 2,
    Items = 1,
    UsableSets = 3,
  }
end

if not Enum.TransmogSearchTypeMeta then
  Enum.TransmogSearchTypeMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.TransmogSituation then
  Enum.TransmogSituation = {
    AllEquipmentSets = 17,
    AllLocations = 2,
    AllMovement = 12,
    AllRacialForms = 19,
    AllSpecs = 0,
    EquipmentSets = 18,
    FormNative = 20,
    FormNonNative = 21,
    LocationArenas = 10,
    LocationBattlegrounds = 11,
    LocationCharacterSelect = 5,
    LocationDelves = 7,
    LocationDungeons = 8,
    LocationHouse = 4,
    LocationRaids = 9,
    LocationRested = 3,
    LocationWorld = 6,
    MovementFlyingMount = 16,
    MovementGroundMount = 15,
    MovementSwimming = 14,
    MovementUnmounted = 13,
    Spec = 1,
  }
end

if not Enum.TransmogSituationFlags then
  Enum.TransmogSituationFlags = {
    AllSituation = 4,
    DefaultsToOn = 8,
    DisabledSituation = 64,
    DynamicallyNamed = 16,
    IsPlayerFacing = 1,
    NoneSituation = 32,
    SpecUseTalentLoadout = 2,
  }
end

if not Enum.TransmogSituationFlagsMeta then
  Enum.TransmogSituationFlagsMeta = {
    MaxValue = 64,
    MinValue = 1,
    NumValues = 7,
  }
end

if not Enum.TransmogSituationGroupFlags then
  Enum.TransmogSituationGroupFlags = {
    DynamicallyCreatesGroups = 1,
  }
end

if not Enum.TransmogSituationGroupFlagsMeta then
  Enum.TransmogSituationGroupFlagsMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.TransmogSituationMeta then
  Enum.TransmogSituationMeta = {
    MaxValue = 21,
    MinValue = 0,
    NumValues = 22,
  }
end

if not Enum.TransmogSituationTrigger then
  Enum.TransmogSituationTrigger = {
    EquipmentSet = 6,
    EventOutfit = 8,
    Forms = 7,
    Location = 3,
    Manual = 1,
    Movement = 4,
    None = 0,
    Specialization = 5,
    TransmogUpdate = 2,
  }
end

if not Enum.TransmogSituationTriggerFlags then
  Enum.TransmogSituationTriggerFlags = {
    CanChangeLockedOutfit = 2,
    CanLockOutfit = 1,
    DisabledTrigger = 16,
    IsPlayerFacing = 4,
    SituationsAreExclusive = 8,
  }
end

if not Enum.TransmogSituationTriggerFlagsMeta then
  Enum.TransmogSituationTriggerFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.TransmogSituationTriggerMeta then
  Enum.TransmogSituationTriggerMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.TransmogSituationTriggerType then
  Enum.TransmogSituationTriggerType = {
    Automatic = 2,
    Manual = 1,
    None = 0,
    TransmogUpdate = 3,
  }
end

if not Enum.TransmogSituationTriggerTypeMeta then
  Enum.TransmogSituationTriggerTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.TransmogSlot then
  Enum.TransmogSlot = {
    Back = 2,
    Body = 4,
    Chest = 3,
    Feet = 10,
    Hand = 7,
    Head = 0,
    Legs = 9,
    Mainhand = 11,
    Offhand = 12,
    Shoulder = 1,
    Tabard = 5,
    Waist = 8,
    Wrist = 6,
  }
end

if not Enum.TransmogSlotMeta then
  Enum.TransmogSlotMeta = {
    MaxValue = 12,
    MinValue = 0,
    NumValues = 13,
  }
end

if not Enum.TransmogSource then
  Enum.TransmogSource = {
    Achievement = 7,
    CantCollect = 6,
    HiddenUntilCollected = 5,
    JournalEncounter = 1,
    None = 0,
    NotValidForTransmog = 9,
    Profession = 8,
    Quest = 2,
    TradingPost = 10,
    Vendor = 3,
    WorldDrop = 4,
  }
end

if not Enum.TransmogSourceMeta then
  Enum.TransmogSourceMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.TransmogType then
  Enum.TransmogType = {
    Appearance = 0,
    Illusion = 1,
  }
end

if not Enum.TransmogTypeMeta then
  Enum.TransmogTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TransmogUseErrorType then
  Enum.TransmogUseErrorType = {
    Ability = 3,
    Class = 7,
    Faction = 9,
    Holiday = 5,
    HotRecheckFailed = 6,
    ItemProficiency = 10,
    None = 0,
    PlayerCondition = 1,
    Race = 8,
    Reputation = 4,
    Skill = 2,
  }
end

if not Enum.TransmogUseErrorTypeMeta then
  Enum.TransmogUseErrorTypeMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.TtsBoolSetting then
  Enum.TtsBoolSetting = {
    AddCharacterNameToSpeech = 1,
    AlternateSystemVoice = 3,
    NarrateMyMessages = 4,
    PlayActivitySoundWhenNotFocused = 2,
    PlaySoundSeparatingChatLineBreaks = 0,
  }
end

if not Enum.TtsBoolSettingMeta then
  Enum.TtsBoolSettingMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.TtsVoiceTypeMeta then
  Enum.TtsVoiceTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TugOfWarMarkerArrowShownState then
  Enum.TugOfWarMarkerArrowShownState = {
    Always = 1,
    FlashOnMove = 2,
    Never = 0,
  }
end

if not Enum.TugOfWarMarkerArrowShownStateMeta then
  Enum.TugOfWarMarkerArrowShownStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.TugOfWarStyleValue then
  Enum.TugOfWarStyleValue = {
    ArchaeologyBrown = 1,
    DefaultYellow = 0,
  }
end

if not Enum.TugOfWarStyleValueMeta then
  Enum.TugOfWarStyleValueMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.TurnStrafeStyle then
  Enum.TurnStrafeStyle = {
    Custom = 2,
    Legacy = 1,
    Modern = 0,
  }
end

if not Enum.TurnStrafeStyleMeta then
  Enum.TurnStrafeStyleMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIActionType then
  Enum.UIActionType = {
    DefaultAction = 0,
    UpdateMapSystem = 1,
  }
end

if not Enum.UIActionTypeMeta then
  Enum.UIActionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UICovenantDisplayInfoFlags then
  Enum.UICovenantDisplayInfoFlags = {
    DisplayCovenantAsJourney = 1,
    UseJourneyRewardTrack = 2,
    UseJourneyUnlockToastText = 3,
  }
end

if not Enum.UICovenantDisplayInfoFlagsMeta then
  Enum.UICovenantDisplayInfoFlagsMeta = {
    MaxValue = 3,
    MinValue = 1,
    NumValues = 3,
  }
end

if not Enum.UICursorType then
  Enum.UICursorType = {
    ActionBar = 6,
    AmmoObsolete = 8,
    BattlePet = 16,
    ConduitCollectionItem = 19,
    Currency = 13,
    Default = 0,
    EquipmentSet = 12,
    Flyout = 14,
    GuildBank = 10,
    GuildBankMoney = 11,
    Item = 1,
    Macro = 7,
    Merchant = 5,
    Money = 2,
    Mount = 17,
    Outfit = 21,
    PerksProgramVendorItem = 20,
    Pet = 9,
    PetAction = 4,
    Spell = 3,
    Toy = 18,
    VoidItem = 15,
  }
end

if not Enum.UICursorTypeMeta then
  Enum.UICursorTypeMeta = {
    MaxValue = 21,
    MinValue = 0,
    NumValues = 22,
  }
end

if not Enum.UIFrameType then
  Enum.UIFrameType = {
    InterruptTutorial = 1,
    JailersTowerBuffs = 0,
  }
end

if not Enum.UIFrameTypeMeta then
  Enum.UIFrameTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIItemInteractionFlags then
  Enum.UIItemInteractionFlags = {
    AddCurrency = 16,
    ClickShowsFlyout = 8,
    ConfirmationHasDelay = 2,
    ConversionMode = 4,
    DisplayWithInset = 1,
    UsesCharges = 32,
  }
end

if not Enum.UIItemInteractionFlagsMeta then
  Enum.UIItemInteractionFlagsMeta = {
    MaxValue = 32,
    MinValue = 1,
    NumValues = 6,
  }
end

if not Enum.UIItemInteractionType then
  Enum.UIItemInteractionType = {
    CastSpell = 1,
    CleanseCorruption = 2,
    ItemConversion = 4,
    None = 0,
    RunecarverScrapping = 3,
  }
end

if not Enum.UIItemInteractionTypeMeta then
  Enum.UIItemInteractionTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.UIMapFlag then
  Enum.UIMapFlag = {
    AlwaysAllowTaxiPathing = 131072,
    AlwaysAllowUserWaypoints = 65536,
    DoNotShowOnNavbar = 524288,
    DoNotTranslateBranches = 512,
    FallbackToParentMap = 16,
    FlightMapAutoZoom = 16384,
    FlightMapShowZoomOut = 8192,
    ForceAllOverlayExplored = 4096,
    ForceAllowMapLinks = 262144,
    ForceOnNavbar = 32768,
    GarrisonMap = 8,
    HideArchaeologyDigs = 256,
    HideIcons = 1024,
    HideVignettes = 2048,
    IgnoreInTranslationsToParent = 2097152,
    IsCityMap = 1048576,
    NoHighlight = 1,
    NoHighlightTexture = 32,
    NoWorldPositions = 128,
    ShowOverlays = 2,
    ShowTaskObjectives = 64,
    ShowTaxiNodes = 4,
  }
end

if not Enum.UIMapFlagMeta then
  Enum.UIMapFlagMeta = {
    MaxValue = 2097152,
    MinValue = 1,
    NumValues = 22,
  }
end

if not Enum.UIMapGroupFlag then
  Enum.UIMapGroupFlag = {
    ShowIconsAcrossFloors = 1,
  }
end

if not Enum.UIMapGroupFlagMeta then
  Enum.UIMapGroupFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.UIMapSystem then
  Enum.UIMapSystem = {
    Adventure = 2,
    Minimap = 3,
    Taxi = 1,
    World = 0,
  }
end

if not Enum.UIMapSystemMeta then
  Enum.UIMapSystemMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.UIMapType then
  Enum.UIMapType = {
    Continent = 2,
    Cosmic = 0,
    Dungeon = 4,
    Micro = 5,
    Orphan = 6,
    World = 1,
    Zone = 3,
  }
end

if not Enum.UIMapTypeMeta then
  Enum.UIMapTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.UIModelSceneActorFlag then
  Enum.UIModelSceneActorFlag = {
    Deprecated1 = 1,
    UseCenterForOriginX = 2,
    UseCenterForOriginY = 4,
    UseCenterForOriginZ = 8,
  }
end

if not Enum.UIModelSceneActorFlagMeta then
  Enum.UIModelSceneActorFlagMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.UIModelSceneContext then
  Enum.UIModelSceneContext = {
    None = -1,
    PerksProgram = 0,
  }
end

if not Enum.UIModelSceneContextMeta then
  Enum.UIModelSceneContextMeta = {
    MaxValue = 0,
    MinValue = -1,
    NumValues = 2,
  }
end

if not Enum.UIModelSceneFlags then
  Enum.UIModelSceneFlags = {
    Autodress = 4,
    HideWeapon = 2,
    NoCameraSpin = 8,
    SheatheWeapon = 1,
  }
end

if not Enum.UIModelSceneFlagsMeta then
  Enum.UIModelSceneFlagsMeta = {
    MaxValue = 8,
    MinValue = 1,
    NumValues = 4,
  }
end

if not Enum.UISystemType then
  Enum.UISystemType = {
    InGameNavigation = 0,
  }
end

if not Enum.UISystemTypeMeta then
  Enum.UISystemTypeMeta = {
    MaxValue = 0,
    MinValue = 0,
    NumValues = 1,
  }
end

if not Enum.UITextureSliceMode then
  Enum.UITextureSliceMode = {
    Stretched = 0,
    Tiled = 1,
  }
end

if not Enum.UITextureSliceModeMeta then
  Enum.UITextureSliceModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetBlendModeType then
  Enum.UIWidgetBlendModeType = {
    Additive = 1,
    Opaque = 0,
  }
end

if not Enum.UIWidgetBlendModeTypeMeta then
  Enum.UIWidgetBlendModeTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetButtonEnabledState then
  Enum.UIWidgetButtonEnabledState = {
    Disabled = 0,
    Enabled = 1,
  }
end

if not Enum.UIWidgetButtonEnabledStateMeta then
  Enum.UIWidgetButtonEnabledStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetButtonIconType then
  Enum.UIWidgetButtonIconType = {
    Checkmark = 3,
    Exit = 0,
    RedX = 4,
    Speak = 1,
    Undo = 2,
  }
end

if not Enum.UIWidgetButtonIconTypeMeta then
  Enum.UIWidgetButtonIconTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.UIWidgetFlag then
  Enum.UIWidgetFlag = {
    UniversalWidget = 1,
  }
end

if not Enum.UIWidgetFlagMeta then
  Enum.UIWidgetFlagMeta = {
    MaxValue = 1,
    MinValue = 1,
    NumValues = 1,
  }
end

if not Enum.UIWidgetFontType then
  Enum.UIWidgetFontType = {
    Normal = 0,
    Outline = 2,
    Shadow = 1,
  }
end

if not Enum.UIWidgetFontTypeMeta then
  Enum.UIWidgetFontTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetHorizontalDirection then
  Enum.UIWidgetHorizontalDirection = {
    LeftToRight = 0,
    RightToLeft = 1,
  }
end

if not Enum.UIWidgetHorizontalDirectionMeta then
  Enum.UIWidgetHorizontalDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetLayoutDirection then
  Enum.UIWidgetLayoutDirection = {
    Default = 0,
    Horizontal = 2,
    HorizontalForceNewRow = 4,
    Overlap = 3,
    Vertical = 1,
  }
end

if not Enum.UIWidgetLayoutDirectionMeta then
  Enum.UIWidgetLayoutDirectionMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.UIWidgetModelSceneLayer then
  Enum.UIWidgetModelSceneLayer = {
    Back = 2,
    Front = 1,
    None = 0,
  }
end

if not Enum.UIWidgetModelSceneLayerMeta then
  Enum.UIWidgetModelSceneLayerMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetMotionType then
  Enum.UIWidgetMotionType = {
    Instant = 0,
    Smooth = 1,
  }
end

if not Enum.UIWidgetMotionTypeMeta then
  Enum.UIWidgetMotionTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetOverrideState then
  Enum.UIWidgetOverrideState = {
    Active = 1,
    Inactive = 0,
  }
end

if not Enum.UIWidgetOverrideStateMeta then
  Enum.UIWidgetOverrideStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UIWidgetRewardShownState then
  Enum.UIWidgetRewardShownState = {
    Hidden = 0,
    ShownEarned = 1,
    ShownUnearned = 2,
  }
end

if not Enum.UIWidgetRewardShownStateMeta then
  Enum.UIWidgetRewardShownStateMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetScale then
  Enum.UIWidgetScale = {
    Eighty = 2,
    Fifty = 5,
    Ninty = 1,
    OneHundred = 0,
    OneHundredEighty = 13,
    OneHundredFifty = 10,
    OneHundredForty = 9,
    OneHundredNinety = 14,
    OneHundredSeventy = 12,
    OneHundredSixty = 11,
    OneHundredTen = 6,
    OneHundredThirty = 8,
    OneHundredTwenty = 7,
    Seventy = 3,
    Sixty = 4,
    TwoHundred = 15,
  }
end

if not Enum.UIWidgetScaleMeta then
  Enum.UIWidgetScaleMeta = {
    MaxValue = 15,
    MinValue = 0,
    NumValues = 16,
  }
end

if not Enum.UIWidgetSetLayoutDirection then
  Enum.UIWidgetSetLayoutDirection = {
    Horizontal = 1,
    Overlap = 2,
    Vertical = 0,
  }
end

if not Enum.UIWidgetSetLayoutDirectionMeta then
  Enum.UIWidgetSetLayoutDirectionMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetSpellButtonCooldownType then
  Enum.UIWidgetSpellButtonCooldownType = {
    HideCooldown = 0,
    ShowCooldown = 1,
    ShowCooldownAndDisableOnCooldown = 2,
  }
end

if not Enum.UIWidgetSpellButtonCooldownTypeMeta then
  Enum.UIWidgetSpellButtonCooldownTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetTextFormatType then
  Enum.UIWidgetTextFormatType = {
    LeadingZeroesWithSixDigits = 3,
    None = 0,
    TimeOneLevel = 1,
    TimeTwoLevel = 2,
  }
end

if not Enum.UIWidgetTextFormatTypeMeta then
  Enum.UIWidgetTextFormatTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.UIWidgetTextSizeType then
  Enum.UIWidgetTextSizeType = {
    Huge27Pt = 3,
    Large20Pt = 8,
    Large24Pt = 2,
    Medium16Pt = 1,
    Medium18Pt = 7,
    Small10Pt = 5,
    Small11Pt = 6,
    Small12Pt = 0,
    Standard14Pt = 4,
  }
end

if not Enum.UIWidgetTextSizeTypeMeta then
  Enum.UIWidgetTextSizeTypeMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.UIWidgetTextureAndTextSizeType then
  Enum.UIWidgetTextureAndTextSizeType = {
    Huge = 3,
    Large = 2,
    Medium = 1,
    Medium2 = 5,
    Small = 0,
    Standard = 4,
  }
end

if not Enum.UIWidgetTextureAndTextSizeTypeMeta then
  Enum.UIWidgetTextureAndTextSizeTypeMeta = {
    MaxValue = 5,
    MinValue = 0,
    NumValues = 6,
  }
end

if not Enum.UIWidgetTooltipLocation then
  Enum.UIWidgetTooltipLocation = {
    Bottom = 8,
    BottomLeft = 1,
    BottomRight = 7,
    Default = 0,
    Left = 2,
    Right = 6,
    Top = 4,
    TopLeft = 3,
    TopRight = 5,
  }
end

if not Enum.UIWidgetTooltipLocationMeta then
  Enum.UIWidgetTooltipLocationMeta = {
    MaxValue = 8,
    MinValue = 0,
    NumValues = 9,
  }
end

if not Enum.UIWidgetUpdateAnimType then
  Enum.UIWidgetUpdateAnimType = {
    Flash = 1,
    FlashAndAnimateNumber = 2,
    None = 0,
  }
end

if not Enum.UIWidgetUpdateAnimTypeMeta then
  Enum.UIWidgetUpdateAnimTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UIWidgetVisualizationType then
  Enum.UIWidgetVisualizationType = {
    BulletTextList = 10,
    ButtonHeader = 30,
    CaptureBar = 1,
    CaptureZone = 17,
    DiscreteProgressSteps = 19,
    DoubleIconAndText = 5,
    DoubleStateIconRow = 14,
    DoubleStatusBar = 3,
    FillUpFrames = 24,
    HorizontalCurrencies = 9,
    IconAndText = 0,
    IconTextAndBackground = 4,
    IconTextAndCurrencies = 7,
    ItemDisplay = 27,
    MapPinAnimation = 26,
    PreyHuntProgress = 31,
    ScenarioHeaderCurrenciesAndBackground = 11,
    ScenarioHeaderDelves = 29,
    ScenarioHeaderTimer = 20,
    Spacer = 22,
    SpellDisplay = 13,
    StackedResourceTracker = 6,
    StatusBar = 2,
    TextColumnRow = 21,
    TextWithState = 8,
    TextWithSubtext = 25,
    TextureAndText = 12,
    TextureAndTextRow = 15,
    TextureWithAnimation = 18,
    TugOfWar = 28,
    UnitPowerBar = 23,
    ZoneControl = 16,
  }
end

if not Enum.UIWidgetVisualizationTypeMeta then
  Enum.UIWidgetVisualizationTypeMeta = {
    MaxValue = 31,
    MinValue = 0,
    NumValues = 32,
  }
end

if not Enum.UnitAuraSortDirection then
  Enum.UnitAuraSortDirection = {
    Normal = 0,
    Reverse = 1,
  }
end

if not Enum.UnitAuraSortDirectionMeta then
  Enum.UnitAuraSortDirectionMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UnitAuraSortRule then
  Enum.UnitAuraSortRule = {
    BigDefensive = 2,
    Default = 1,
    Expiration = 3,
    ExpirationOnly = 4,
    Name = 5,
    NameOnly = 6,
    Unsorted = 0,
  }
end

if not Enum.UnitAuraSortRuleMeta then
  Enum.UnitAuraSortRuleMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.UnitDamageAbsorbClampMode then
  Enum.UnitDamageAbsorbClampMode = {
    MaximumHealth = 2,
    MissingHealth = 0,
    MissingHealthWithoutIncomingHeals = 1,
  }
end

if not Enum.UnitDamageAbsorbClampModeMeta then
  Enum.UnitDamageAbsorbClampModeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.UnitHealAbsorbClampMode then
  Enum.UnitHealAbsorbClampMode = {
    CurrentHealth = 0,
    MaximumHealth = 1,
  }
end

if not Enum.UnitHealAbsorbClampModeMeta then
  Enum.UnitHealAbsorbClampModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UnitHealAbsorbMode then
  Enum.UnitHealAbsorbMode = {
    ReducedByIncomingHeals = 0,
    Total = 1,
  }
end

if not Enum.UnitHealAbsorbModeMeta then
  Enum.UnitHealAbsorbModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UnitIncomingHealClampMode then
  Enum.UnitIncomingHealClampMode = {
    MaximumHealth = 1,
    MissingHealth = 0,
  }
end

if not Enum.UnitIncomingHealClampModeMeta then
  Enum.UnitIncomingHealClampModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UnitMaximumHealthMode then
  Enum.UnitMaximumHealthMode = {
    Default = 0,
    WithAbsorbs = 1,
  }
end

if not Enum.UnitMaximumHealthModeMeta then
  Enum.UnitMaximumHealthModeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.UnitMirrorPetFlags then
  Enum.UnitMirrorPetFlags = {
    Dismissable = 2,
    ExtraPet = 16,
    RecentlyTamed = 4,
    Renameable = 1,
    Stampede = 8,
  }
end

if not Enum.UnitMirrorPetFlagsMeta then
  Enum.UnitMirrorPetFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.UnitSex then
  Enum.UnitSex = {
    Male = 0,
    Female = 1,
    None = 2,
    Both = 3,
    Neutral = 4,
  }
end

if not Enum.UnitSexMeta then
  Enum.UnitSexMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.ValidateNameResult then
  Enum.ValidateNameResult = {
    ConsecutiveSpaces = 13,
    DeclensionDoesntMatchBaseName = 16,
    Failure = 1,
    InvalidApostrophe = 9,
    InvalidCharacter = 5,
    InvalidSpace = 12,
    MixedLanguages = 6,
    MultipleApostrophes = 10,
    NoName = 2,
    Profane = 7,
    Reserved = 8,
    RussianConsecutiveSilentCharacters = 14,
    RussianSilentCharacterAtBeginningOrEnd = 15,
    SpacesDisallowed = 17,
    Success = 0,
    ThreeConsecutive = 11,
    TooLong = 4,
    TooShort = 3,
  }
end

if not Enum.ValidateNameResultMeta then
  Enum.ValidateNameResultMeta = {
    MaxValue = 17,
    MinValue = 0,
    NumValues = 18,
  }
end

if not Enum.VasPurchaseProgress then
  Enum.VasPurchaseProgress = {
    ApplyingLicense = 3,
    Complete = 7,
    Invalid = 0,
    PaymentPending = 2,
    PrePurchase = 1,
    ProcessingFactionChange = 6,
    Ready = 5,
    WaitingOnQueue = 4,
  }
end

if not Enum.VasPurchaseProgressMeta then
  Enum.VasPurchaseProgressMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.VasServiceType then
  Enum.VasServiceType = {
    AppearanceChange = 2,
    CharacterTransfer = 4,
    FactionChange = 1,
    FactionTransfer = 5,
    GuildFactionChange = 7,
    GuildFactionTransfer = 9,
    GuildNameChange = 6,
    GuildTransfer = 8,
    NameChange = 0,
    RaceChange = 3,
  }
end

if not Enum.VasServiceTypeMeta then
  Enum.VasServiceTypeMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.VasTransactionPurchaseResult then
  Enum.VasTransactionPurchaseResult = {
    BeginDbErrors = 20010,
    BeginProxyErrors = 17,
    DbAccountDoesNotOwnCharacter = 20017,
    DbAllianceNotEligible = 20027,
    DbAlreadyRenameFlagged = 20055,
    DbAuthenticatorInsufficient = 20073,
    DbBatchProcessingDisabled = 20028,
    DbBatchStateInvalid = 20067,
    DbBnetAccountNotProvided = 20077,
    DbBoostProcessingDisabled = 20095,
    DbBpayDeliveryPending = 20078,
    DbCagedPetInInventory = 20084,
    DbCannotMoveArenaCaptn = 20064,
    DbCannotMoveGuildmaster = 20012,
    DbCharLocked = 20026,
    DbCharNotLocked = 20025,
    DbCharacterDoesNotExist = 20019,
    DbCharacterWithoutGuild = 20070,
    DbCouldNotObtainCharLock = 20041,
    DbCustomizeAlreadyRequested = 20057,
    DbDeletedGuild = 20071,
    DbDuplicateCharacterName = 20015,
    DbDuplicateSupportRequest = 20063,
    DbEventNotActive = 20068,
    DbEventRealmClosed = 20038,
    DbFactionChangeTooSoon = 20061,
    DbGmSenorityInsufficient = 20072,
    DbGuildLevelInsufficient = 20074,
    DbGuildRankInsufficient = 20069,
    DbHasAuctions = 20033,
    DbHasBpayToken = 20079,
    DbHasCraftingOrders = 20092,
    DbHasHeirloomItem = 20080,
    DbHasMail = 20016,
    DbHasNewPlayerExperienceRestriction = 20091,
    DbHasUnclaimedCacheRewards = 20089,
    DbHouseOwnerRestriction = 20096,
    DbIneligibleMapID = 20076,
    DbIneligibleTargetRealm = 20022,
    DbInvalidName = 20094,
    DbInventoryCorrupt = 20040,
    DbLastCustomizeTooSoon = 20058,
    DbLastRenameTooRecent = 20052,
    DbLastRequestTooSoon = 20054,
    DbLastSaveTooDistant = 20083,
    DbLastSaveTooRecent = 20034,
    DbLockAlreadyRequested = 20044,
    DbLockNotAcknowledged = 20043,
    DbMaxCharactersOnServer = 20013,
    DbMoveInProgress = 20018,
    DbMultipleTransferCn = 20056,
    DbNameChangeSkipped = 20093,
    DbNameNotAvailable = 20051,
    DbNeedsEraChoice = 20090,
    DbNeedsLevelSquish = 20088,
    DbNewLeaderInvalid = 20086,
    DbNewNameTooLong = 20024,
    DbNoCopiesAvailable = 20020,
    DbNoCsiFound = 20065,
    DbNoMixedAlliance = 20014,
    DbNoneBatchQueueID = 20029,
    DbNoneLockID = 20042,
    DbNoneRealmForEvent = 20037,
    DbNoneSupportActionID = 20046,
    DbNoneSupportQueueID = 20045,
    DbNoneTemplateForEvent = 20039,
    DbNotMovebackable = 20048,
    DbNotRollbackable = 20047,
    DbOnBoostCooldown = 20085,
    DbPendingItemAudit = 20066,
    DbPvEPvPTransferNotAllowed = 20087,
    DbRaceClassComboIneligible = 20062,
    DbRealmNotEligible = 20011,
    DbRegionalDbNotFound = 20075,
    DbRenameAlreadyRequested = 20049,
    DbReservationDoesNotMatch = 20053,
    DbReservationExpired = 20050,
    DbResultAccountRestricted = 20082,
    DbResultSourceAccountBanned = 20081,
    DbTooMuchMoneyForLevel = 20032,
    DbTransferDateTooSoon = 20023,
    DbTransferNotCancellable = 20031,
    DbTransferNotFinalized = 20030,
    DbTransferNotLockable = 20035,
    DbTransferNotUnlockable = 20036,
    DbUnableToCustomize = 20060,
    DbUnderMinLevelReq = 20021,
    DbWaitingForPayment = 20059,
    DisallowedDestinationAccount = 9,
    DisallowedSourceAccount = 8,
    EndDbErrors = 20097,
    EndProxyErrors = 56,
    EndServerErrors = 16,
    InvalidBpayDeliverableID = 14,
    InvalidBpayProductID = 13,
    InvalidBpayProductType = 15,
    InvalidDestinationAccount = 6,
    InvalidDestinationRealm = 5,
    InvalidDistribution = 12,
    InvalidSourceAccount = 7,
    LowerBoxLevel = 11,
    None = 0,
    NoneCharacter = 2,
    NoneProduct = 3,
    OnlyOneVasAtATime = 4,
    ProxyAccountDataTooBig = 55,
    ProxyBadDbType = 35,
    ProxyBadObjType = 18,
    ProxyBadRequestContained = 33,
    ProxyBindFailed = 51,
    ProxyCannotDeleteArenaCaptain = 31,
    ProxyCannotDeleteGuildLeader = 30,
    ProxyCannotInsertNull = 45,
    ProxyCannotUndeleteTrialBoostCharacter = 46,
    ProxyCharacterHasWoWToken = 53,
    ProxyCharacterLevelTooHigh = 38,
    ProxyCharacterTransferred = 39,
    ProxyCharacterTransferredNoBoostInProgress = 40,
    ProxyDataInvalid = 22,
    ProxyDbError = 21,
    ProxyDeleteNotSupported = 28,
    ProxyExternalUpdateDetected = 37,
    ProxyGenericError = 44,
    ProxyInsertRequestContainedNoData = 49,
    ProxyInvalidKey = 50,
    ProxyIsReadonly = 34,
    ProxyLockedForToken = 42,
    ProxyLockedForTransfer = 26,
    ProxyLockedForUpgrade = 41,
    ProxyLockedForVas = 43,
    ProxyLocksFailed = 24,
    ProxyLostProxyConnection = 23,
    ProxyMissingResultsPtr = 25,
    ProxyMoneyLimitExceeded = 32,
    ProxyNoReturnValue = 27,
    ProxyObjDropped = 29,
    ProxyObjNotInDb = 19,
    ProxyOperationAlreadyInProgress = 36,
    ProxyPredicateFailed = 47,
    ProxyTooBusyBackOff = 54,
    ProxyTooManyRows = 52,
    ProxyUniqueKey = 20,
    ProxyWeeklyRewardNotFound = 48,
    UnknownError = 10,
    VasPurchaseDisabled = 1,
  }
end

if not Enum.VasTransactionPurchaseResultMeta then
  Enum.VasTransactionPurchaseResultMeta = {
    MaxValue = 20097,
    MinValue = 0,
    NumValues = 145,
  }
end

if not Enum.ViewArenaSizeMeta then
  Enum.ViewArenaSizeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ViewRaidSizeMeta then
  Enum.ViewRaidSizeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.VignetteObjectiveTypeMeta then
  Enum.VignetteObjectiveTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.VignetteType then
  Enum.VignetteType = {
    FyrakkFlight = 4,
    Normal = 0,
    PvPBounty = 1,
    Torghast = 2,
    Treasure = 3,
  }
end

if not Enum.VignetteTypeMeta then
  Enum.VignetteTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.Vocalerrorsounds then
  Enum.Vocalerrorsounds = {
    Abilitycooling = 50,
    Alreadyingroup = 20,
    Alreadyinguild = 21,
    Ammoonlyinbag = 28,
    Bagfull = 29,
    BoundNodrop = 4,
    Cantaffordbankslot = 22,
    CantattackNotarget = 38,
    CantattackNotstandingObsolete = 37,
    Cantattackrongdirection = 36,
    CantcastOutofrange = 46,
    Cantcreate = 18,
    Cantdrinkmore = 6,
    CanteatMoving = 24,
    Canteatmore = 7,
    Cantequip2HNoskill = 43,
    Cantequip2HSkill = 41,
    Cantequip2Hequipped = 42,
    Cantflyhere = 60,
    Cantinvite = 8,
    CantlearnLevel = 13,
    Cantloot = 17,
    CantlootDidntkill = 31,
    CantlootLocked = 33,
    CantlootNotstandingObsolete = 34,
    CantlootToofar = 35,
    CantlootWrongfacing = 32,
    Cantputbag = 26,
    Cantswap = 58,
    CanttaxiNomoney = 54,
    CanttradeSoulbound = 59,
    Cantuseitem = 51,
    Cantuselocked = 55,
    Cantusetoofar = 57,
    Chestinuse = 52,
    Declinegroup = 19,
    ExhaustedObsolete = 67,
    FoodcoolingObsolete = 53,
    Genericnotarget = 45,
    Guildpermissions = 62,
    Invaliditemtarget = 66,
    Invalidtarget = 11,
    Inventoryfull = 0,
    Inviteebusy = 9,
    Itemcooling = 5,
    Itemlocked = 61,
    Itemmaxcount = 30,
    Locked = 14,
    Mustequippitem = 49,
    Noenergy = 64,
    NoequipEver = 3,
    NoequipLevel = 2,
    Noequipslotavailable = 56,
    Noessence = 65,
    Nomana = 15,
    Norage = 63,
    Notabag = 25,
    Notenoughgold = 39,
    Notenoughmoney = 40,
    Notequippable = 44,
    Notwhiledead = 16,
    Outofammo = 1,
    Potioncooling = 47,
    Proficiencyneeded = 48,
    Spellcooling = 12,
    Targettoofar = 10,
    Toomanybankslots = 23,
    Wrongslot = 27,
  }
end

if not Enum.VocalerrorsoundsMeta then
  Enum.VocalerrorsoundsMeta = {
    MaxValue = 67,
    MinValue = 0,
    NumValues = 68,
  }
end

if not Enum.VoiceChannelErrorReason then
  Enum.VoiceChannelErrorReason = {
    IsBattleNetChannel = 1,
    Unknown = 0,
  }
end

if not Enum.VoiceChannelErrorReasonMeta then
  Enum.VoiceChannelErrorReasonMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.VoiceChatStatusCodeMeta then
  Enum.VoiceChatStatusCodeMeta = {
    MaxValue = 24,
    MinValue = 0,
    NumValues = 25,
  }
end

if not Enum.VoiceTtsStatusCode then
  Enum.VoiceTtsStatusCode = {
    DestinationQueueFull = 8,
    EngineAllocationFailed = 2,
    EnqueueNotNecessary = 9,
    InputTextEnqueued = 6,
    InternalError = 13,
    InvalidArgument = 12,
    InvalidEngineType = 1,
    ManagerNotFound = 11,
    MaxCharactersExceeded = 4,
    NotSupported = 3,
    SdkNotInitialized = 7,
    Success = 0,
    UtteranceBelowMinimumDuration = 5,
    UtteranceNotFound = 10,
  }
end

if not Enum.VoiceTtsStatusCodeMeta then
  Enum.VoiceTtsStatusCodeMeta = {
    MaxValue = 13,
    MinValue = 0,
    NumValues = 14,
  }
end

if not Enum.WarbandSceneFlags then
  Enum.WarbandSceneFlags = {
    AwardedAutomatically = 8,
    CannotBeSaved = 4,
    DoNotInclude = 1,
    HiddenUntilCollected = 2,
    IsDefault = 16,
  }
end

if not Enum.WarbandSceneFlagsMeta then
  Enum.WarbandSceneFlagsMeta = {
    MaxValue = 16,
    MinValue = 1,
    NumValues = 5,
  }
end

if not Enum.WeaponSlot then
  Enum.WeaponSlot = {
    MainHand = 0,
    OffHand = 1,
  }
end

if not Enum.WeaponSlotMeta then
  Enum.WeaponSlotMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WeeklyRewardChestActivityType then
  Enum.WeeklyRewardChestActivityType = {
    LFGDungeons = 1,
    Scenario = 0,
    World = 2,
  }
end

if not Enum.WeeklyRewardChestActivityTypeMeta then
  Enum.WeeklyRewardChestActivityTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WeeklyRewardChestClaimRewardResult then
  Enum.WeeklyRewardChestClaimRewardResult = {
    CountExceeded = 7,
    DbError = 5,
    InvalidSlot = 3,
    InvalidThreshold = 1,
    LockFailure = 6,
    PlayerNotFound = 2,
    Success = 0,
    TooManyItems = 4,
  }
end

if not Enum.WeeklyRewardChestClaimRewardResultMeta then
  Enum.WeeklyRewardChestClaimRewardResultMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.WeeklyRewardChestThresholdType then
  Enum.WeeklyRewardChestThresholdType = {
    Activities = 1,
    AlsoReceive = 4,
    Concession = 5,
    None = 0,
    Raid = 3,
    RankedPvP = 2,
    World = 6,
  }
end

if not Enum.WeeklyRewardChestThresholdTypeMeta then
  Enum.WeeklyRewardChestThresholdTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

if not Enum.WeeklyRewardProgressResult then
  Enum.WeeklyRewardProgressResult = {
    DbError = 3,
    NoPlayer = 4,
    NoSeason = 1,
    Success = 0,
    TimedOut = 2,
  }
end

if not Enum.WeeklyRewardProgressResultMeta then
  Enum.WeeklyRewardProgressResultMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

if not Enum.WidgetAnimationTypeMeta then
  Enum.WidgetAnimationTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetCurrencyClass then
  Enum.WidgetCurrencyClass = {
    Currency = 0,
    Item = 1,
  }
end

if not Enum.WidgetCurrencyClassMeta then
  Enum.WidgetCurrencyClassMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetEnabledState then
  Enum.WidgetEnabledState = {
    Artifact = 5,
    Black = 6,
    BrightBlue = 7,
    Disabled = 0,
    Green = 4,
    Red = 2,
    White = 3,
    Yellow = 1,
  }
end

if not Enum.WidgetEnabledStateMeta then
  Enum.WidgetEnabledStateMeta = {
    MaxValue = 7,
    MinValue = 0,
    NumValues = 8,
  }
end

if not Enum.WidgetGlowAnimType then
  Enum.WidgetGlowAnimType = {
    None = 0,
    Pulse = 1,
  }
end

if not Enum.WidgetGlowAnimTypeMeta then
  Enum.WidgetGlowAnimTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetIconSizeType then
  Enum.WidgetIconSizeType = {
    Large = 2,
    Medium = 1,
    Small = 0,
    Standard = 3,
  }
end

if not Enum.WidgetIconSizeTypeMeta then
  Enum.WidgetIconSizeTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.WidgetIconSourceType then
  Enum.WidgetIconSourceType = {
    Item = 1,
    Spell = 0,
  }
end

if not Enum.WidgetIconSourceTypeMeta then
  Enum.WidgetIconSourceTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetOpacityTypeMeta then
  Enum.WidgetOpacityTypeMeta = {
    MaxValue = 10,
    MinValue = 0,
    NumValues = 11,
  }
end

if not Enum.WidgetShowGlowStateMeta then
  Enum.WidgetShowGlowStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetShownState then
  Enum.WidgetShownState = {
    Hidden = 0,
    Shown = 1,
  }
end

if not Enum.WidgetShownStateMeta then
  Enum.WidgetShownStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.WidgetTextHorizontalAlignmentType then
  Enum.WidgetTextHorizontalAlignmentType = {
    Center = 1,
    Left = 0,
    Right = 2,
  }
end

if not Enum.WidgetTextHorizontalAlignmentTypeMeta then
  Enum.WidgetTextHorizontalAlignmentTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WidgetUnitPowerBarFlashMomentType then
  Enum.WidgetUnitPowerBarFlashMomentType = {
    FlashWhenMax = 0,
    FlashWhenMin = 1,
    NeverFlash = 2,
  }
end

if not Enum.WidgetUnitPowerBarFlashMomentTypeMeta then
  Enum.WidgetUnitPowerBarFlashMomentTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WoWEntitlementType then
  Enum.WoWEntitlementType = {
    Appearance = 4,
    AppearanceSet = 5,
    Battlepet = 2,
    GameTime = 6,
    Illusion = 8,
    Invalid = 9,
    Item = 0,
    Mount = 1,
    Title = 7,
    Toy = 3,
  }
end

if not Enum.WoWEntitlementTypeMeta then
  Enum.WoWEntitlementTypeMeta = {
    MaxValue = 9,
    MinValue = 0,
    NumValues = 10,
  }
end

if not Enum.WoWLabsAreaType then
  Enum.WoWLabsAreaType = {
    PlunderstormDropDense = 2,
    PlunderstormDropMedium = 1,
    PlunderstormDropSparse = 0,
  }
end

if not Enum.WoWLabsAreaTypeMeta then
  Enum.WoWLabsAreaTypeMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WorldCursorAnchorType then
  Enum.WorldCursorAnchorType = {
    Cursor = 2,
    Default = 1,
    Nameplate = 3,
    None = 0,
  }
end

if not Enum.WorldCursorAnchorTypeMeta then
  Enum.WorldCursorAnchorTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.WorldElapsedTimerTypesMeta then
  Enum.WorldElapsedTimerTypesMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.WorldQuestQuality then
  Enum.WorldQuestQuality = {
    Common = 0,
    Epic = 2,
    Rare = 1,
  }
end

if not Enum.WorldQuestQualityMeta then
  Enum.WorldQuestQualityMeta = {
    MaxValue = 2,
    MinValue = 0,
    NumValues = 3,
  }
end

if not Enum.ZoneControlActiveState then
  Enum.ZoneControlActiveState = {
    Active = 1,
    Inactive = 0,
  }
end

if not Enum.ZoneControlActiveStateMeta then
  Enum.ZoneControlActiveStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ZoneControlDangerFlashType then
  Enum.ZoneControlDangerFlashType = {
    ShowOnBadStates = 1,
    ShowOnBoth = 2,
    ShowOnGoodStates = 0,
    ShowOnNeither = 3,
  }
end

if not Enum.ZoneControlDangerFlashTypeMeta then
  Enum.ZoneControlDangerFlashTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ZoneControlFillTypeMeta then
  Enum.ZoneControlFillTypeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ZoneControlLeadingEdgeType then
  Enum.ZoneControlLeadingEdgeType = {
    NoLeadingEdge = 0,
    UseLeadingEdge = 1,
  }
end

if not Enum.ZoneControlLeadingEdgeTypeMeta then
  Enum.ZoneControlLeadingEdgeTypeMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.ZoneControlMode then
  Enum.ZoneControlMode = {
    BothStatesAreGood = 0,
    NeitherStateIsGood = 3,
    State1IsGood = 1,
    State2IsGood = 2,
  }
end

if not Enum.ZoneControlModeMeta then
  Enum.ZoneControlModeMeta = {
    MaxValue = 3,
    MinValue = 0,
    NumValues = 4,
  }
end

if not Enum.ZoneControlStateMeta then
  Enum.ZoneControlStateMeta = {
    MaxValue = 1,
    MinValue = 0,
    NumValues = 2,
  }
end

if not Enum.SecondsFormatterRoundingMeta then
  Enum.SecondsFormatterRoundingMeta = { MaxValue = 1, MinValue = 0, NumValues = 2 }
end

if not Enum.CooldownViewerSoundMeta then
  Enum.CooldownViewerSoundMeta = { MaxValue = 93, MinValue = 0, NumValues = 94 }
end

if not Enum.RecentAlliesFriendTagMeta then
  Enum.RecentAlliesFriendTagMeta = { MaxValue = 5, MinValue = 0, NumValues = 6 }
end

if not Enum.BattleNetFriendLevelMeta then
  Enum.BattleNetFriendLevelMeta = { MaxValue = 3, MinValue = 1, NumValues = 3 }
end

if not Enum.BattleNetFriendTagMeta then
  Enum.BattleNetFriendTagMeta = { MaxValue = 9, MinValue = 0, NumValues = 10 }
end

if not Enum.RaidDispelOverlayTypeMeta then
  Enum.RaidDispelOverlayTypeMeta = { MaxValue = 2, MinValue = 0, NumValues = 3 }
end

if not Enum.SocialSystemTypeMeta then
  Enum.SocialSystemTypeMeta = { MaxValue = 4, MinValue = 0, NumValues = 5 }
end

if not Enum.VisualAlertTypeMeta then
  Enum.VisualAlertTypeMeta = { MaxValue = 10, MinValue = 1, NumValues = 10 }
end

if not Enum.SocialUIPresenceTypeMeta then
  Enum.SocialUIPresenceTypeMeta = { MaxValue = 5, MinValue = 0, NumValues = 6 }
end

if not Enum.SocialUIBlockTypeMeta then
  Enum.SocialUIBlockTypeMeta = { MaxValue = 2, MinValue = 0, NumValues = 3 }
end

if not Enum.ScriptObjectAccessRestrictionMeta then
  Enum.ScriptObjectAccessRestrictionMeta = { MaxValue = 1, MinValue = 1, NumValues = 1 }
end

-- From Blizzard_APIDocumentationGenerated/PlayerHousingConstantsDocumentation.lua (12.1.0).
if not Enum.HousingBlueprintType then
  Enum.HousingBlueprintType = {
    Exterior = 4,
    House = 1,
    Interior = 3,
    None = 0,
    Room = 2,
  }
end

if not Enum.HousingBlueprintTypeMeta then
  Enum.HousingBlueprintTypeMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end

-- From Blizzard_APIDocumentationGenerated/HousingBlueprintConstantsDocumentation.lua (12.1.0).
if not Enum.HousingBlueprintContentType then
  Enum.HousingBlueprintContentType = {
    Decor = 3,
    Dye = 4,
    Fixture = 5,
    HouseType = 1,
    None = 0,
    Other = 6,
    Room = 2,
  }
end

if not Enum.HousingBlueprintContentTypeMeta then
  Enum.HousingBlueprintContentTypeMeta = {
    MaxValue = 6,
    MinValue = 0,
    NumValues = 7,
  }
end

-- From Blizzard_APIDocumentationGenerated/HousingBlueprintConstantsDocumentation.lua (12.1.0).
if not Enum.HousingBlueprintUnmetRequirementFlags then
  Enum.HousingBlueprintUnmetRequirementFlags = {
    HouseSizeLocked = 128,
    HouseTypeLocked = 64,
    InsufficientBudget = 1,
    MismatchedExteriorFaction = 32,
    MissingDecor = 8,
    MissingDye = 16,
    MissingFixture = 4,
    MissingRoom = 2,
  }
end

if not Enum.HousingBlueprintUnmetRequirementFlagsMeta then
  Enum.HousingBlueprintUnmetRequirementFlagsMeta = {
    MaxValue = 128,
    MinValue = 1,
    NumValues = 8,
  }
end

-- From Blizzard_APIDocumentationGenerated/AuraContainerSharedDocumentation.lua (12.1.0).
if not Enum.CustomAuraButtonDispelTypeTextureStyle then
  Enum.CustomAuraButtonDispelTypeTextureStyle = {
    Border = 0,
    BorderWithIcon = 1,
    CustomAsset = 4,
    Icon = 2,
    PreserveAsset = 3,
  }
end

if not Enum.CustomAuraButtonDispelTypeTextureStyleMeta then
  Enum.CustomAuraButtonDispelTypeTextureStyleMeta = {
    MaxValue = 4,
    MinValue = 0,
    NumValues = 5,
  }
end
