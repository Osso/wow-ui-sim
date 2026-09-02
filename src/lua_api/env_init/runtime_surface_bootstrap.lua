local function __wow_noop()
end

-- WoW exposes debug.iscfunction (not a stock Lua 5.1 function): true for native
-- (C) functions, false for Lua closures. Back it with debug.getinfo's "what"
-- field (what == "C" for native functions).
if debug and rawget(debug, "iscfunction") == nil and type(debug.getinfo) == "function" then
  function debug.iscfunction(fn)
    if type(fn) ~= "function" then
      return false
    end
    local info = debug.getinfo(fn, "S")
    return info ~= nil and info.what == "C"
  end
end

-- DropCursorMoney(frame) drops the cursor's money stack onto the target frame
-- (a CoinPickupFrame) in real WoW. The simulator does not model cursor-money
-- drops, but Blizzard XML wires this as an OnLoad handler.
if DropCursorMoney == nil then
  function DropCursorMoney(_frame)
  end
end

-- GetInventorySlotInfo is registered from Rust
-- (src/lua_api/globals/inventory_slot.rs). Returns the canonical
-- (slotId, textureFileID, checkRelic) triple; case-insensitive on the
-- slot-name key.

-- C_PvP.GetZonePVPInfo is registered from Rust
-- (src/lua_api/globals/zone_text.rs) — reads SimState::world.pvp_type /
-- .is_sub_zone_pvp / .pvp_faction_name. Admin: A_Admin.SetZonePVP.

-- GetZoneText / GetSubZoneText / GetMinimapZoneText / GetRealZoneText are
-- registered from Rust (src/lua_api/globals/zone_text.rs), backed by
-- SimState::world. Tests drive the values via A_Admin.SetZone / SetSubZone
-- / SetInstanceInfo.
-- IsShiftKeyDown / IsControlKeyDown / IsAltKeyDown / IsMetaKeyDown /
-- IsModifierKeyDown are registered from Rust
-- (src/lua_api/globals/modifier_keys.rs), backed by SimState::modifier_keys.
-- Admin: A_Admin.SetShiftKeyDown(b) / SetControlKeyDown / SetAltKeyDown /
-- SetMetaKeyDown toggle individual keys.

-- GetGuildLogoInfo is registered from Rust (src/lua_api/globals/guild_logo.rs),
-- backed by SimState::world.guild_logo. Admin: A_Admin.SetGuildEmblem(filename,
-- bkgR, bkgG, bkgB, borderR, borderG, borderB, emblemR, emblemG, emblemB) —
-- all args optional, missing = 0 or "".

-- GetNetStats is registered from Rust (src/lua_api/globals/rilua_net_stats.rs)
-- and reads from SimState::net_stats so tests can inject values via
-- A_Admin.SetNetStats(bandwidthIn, bandwidthOut, latencyHome, latencyWorld).

-- StoreFrame_IsShown is registered from Rust (src/lua_api/globals/rilua_store_frame.rs)
-- and reads from SimState::store_frame_shown so tests can toggle it via
-- A_Admin.SetStoreFrameShown(true) to exercise MainMenuBarMicroButtons'
-- pushed-state rendering for the Store micro-button.

-- UnitIsPlayer / UnitIsHumanPlayer are registered from Rust
-- (src/lua_api/globals/unit_probes.rs).
-- It resolves tokens against SimState: "player"/"self" always true, "target"/
-- "focus" read the respective TargetInfo.is_player flag, "partyN" (N=1..4)
-- checks sim.party_members[N-1] is populated, everything else false.

local __wow_namespace_names = setmetatable({}, { __mode = "k" })
local __wow_namespace_mt = {
  __index = function(t, key)
    local removed = rawget(t, "__wow_ptr_removed_keys") or rawget(t, "__wow_removed_keys")
    if type(removed) == "table" and removed[key] then
      return nil
    end
    __wow_log_nil_symbol_access(__wow_namespace_names[t], key)
    local fn = function()
      return nil
    end
    rawset(t, key, fn)
    return fn
  end,
}

function __wow_log_nil_symbol_access(container, key)
  if type(__wow_record_nil_symbol_access) ~= "function" then
    return
  end

  local loggedNilSymbols = rawget(_G, "__wow_logged_nil_symbols")
  if type(loggedNilSymbols) ~= "table" then
    loggedNilSymbols = {}
    rawset(_G, "__wow_logged_nil_symbols", loggedNilSymbols)
  end
  local cacheKey = tostring(container) .. "\001" .. tostring(key)
  if loggedNilSymbols[cacheKey] then
    return
  end
  loggedNilSymbols[cacheKey] = true

  local source
  local line
  for level = 2, 8 do
    local info = debug.getinfo(level, "Sl")
    if info ~= nil and type(info.source) == "string" and info.source:sub(1, 1) == "@" then
      if info.source:find("runtime_surface_bootstrap.lua", 1, true) == nil then
        source = info.source
        line = info.currentline
        break
      end
    end
  end

  __wow_record_nil_symbol_access(container, key, source, line)
end

local function __wow_attach_namespace_name(namespace, name)
  if type(namespace) ~= "table" then
    return namespace
  end
  if name ~= nil then
    __wow_namespace_names[namespace] = name
  end
  local mt = getmetatable(namespace)
  if mt == nil then
    setmetatable(namespace, __wow_namespace_mt)
  elseif mt.__index == nil then
    setmetatable(namespace, __wow_namespace_mt)
  end
  return namespace
end

local function __wow_namespace(defaults)
  return __wow_attach_namespace_name(defaults or {})
end

local function __wow_merge_namespace(existing, defaults)
  local namespace = type(existing) == "table" and existing or {}
  for key, value in pairs(defaults or {}) do
    if rawget(namespace, key) == nil then
      rawset(namespace, key, value)
    end
  end
  local mt = getmetatable(namespace)
  if mt == nil or mt.__index == nil then
    setmetatable(namespace, getmetatable(__wow_namespace()))
  end
  return namespace
end

local __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY = 5
local __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE = 3
local __WOW_PURCHASE_ELIGIBILITY_OK = 0
local __WOW_VAS_SERVICE_NAME_CHANGE = 0
local __WOW_VAS_SERVICE_CHARACTER_TRANSFER = 4
local __WOW_VAS_SERVICE_GUILD_TRANSFER = 8

local function __wow_restore_secure_enum(enumName, values)
  local secureenv = rawget(_G, "__secureenv")
  if type(secureenv) ~= "table" then
    return
  end

  local enum = rawget(secureenv, "Enum")
  if type(enum) ~= "table" then
    enum = {}
    rawset(secureenv, "Enum", enum)
  end

  if rawget(enum, enumName) == nil then
    rawset(enum, enumName, values)
  end
end

__wow_restore_secure_enum("BattlepayCardType", {
  SmallCard = 0,
  MediumCard = 1,
  LargeHorizontalCard = 2,
  LargeVeritcalCard = 3,
  FullCardWithBuyButton = 4,
  MediumCardWithBuyButton = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
  LargeHorizontalCardWithBuyButton = 6,
  LargeVeritcalCardWithBuyButton = 7,
  FullCardWithNydusLinkButton = 8,
  LargeVeritcalPageableCardWithBuyButton = 9,
})
__wow_restore_secure_enum("BattlepayProductDecorator", {
  Boost = 0,
  Expansion = 1,
  WoWToken = 2,
  VasService = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
})
__wow_restore_secure_enum("PurchaseEligibility", {
  Ok = __WOW_PURCHASE_ELIGIBILITY_OK,
  PartiallyOwned = 1,
  Owned = 2,
  ExpansionTooLow = 3,
  ExpansionTooHigh = 4,
  MissingRequiredDeliverable = 5,
})
__wow_restore_secure_enum("VasServiceType", {
  NameChange = __WOW_VAS_SERVICE_NAME_CHANGE,
  FactionChange = 1,
  AppearanceChange = 2,
  RaceChange = 3,
  CharacterTransfer = __WOW_VAS_SERVICE_CHARACTER_TRANSFER,
  FactionTransfer = 5,
  GuildNameChange = 6,
  GuildFactionChange = 7,
  GuildTransfer = __WOW_VAS_SERVICE_GUILD_TRANSFER,
  GuildFactionTransfer = 9,
})

local function __wow_seed_namespace_names()
  for key, value in pairs(_G) do
    if type(key) == "string" and key:match("^C_[A-Za-z0-9_]+$") and type(value) == "table" then
      __wow_attach_namespace_name(value, key)
    end
  end
end

local function __wow_copy_table(source)
  local copy = {}
  for key, value in pairs(source or {}) do
    copy[key] = value
  end
  return copy
end

local function __wow_make_calendar_time(dayOffset, minuteOffset)
  local day = 14 + (tonumber(dayOffset) or 0)
  local totalMinutes = (12 * 60) + (tonumber(minuteOffset) or 0)
  local hour = math.floor(totalMinutes / 60)
  local minute = totalMinutes % 60
  while minute < 0 do
    minute = minute + 60
    hour = hour - 1
  end
  while minute >= 60 do
    minute = minute - 60
    hour = hour + 1
  end
  while hour < 0 do
    hour = hour + 24
    day = day - 1
  end
  while hour >= 24 do
    hour = hour - 24
    day = day + 1
  end
  return {
    year = 2026,
    month = 4,
    monthDay = day,
    weekday = 3,
    hour = hour,
    minute = minute,
  }
end

-- C_LFGList is state-backed via `src/lua_api/globals/lfg_list.rs`.
-- C_AddOnProfiler is state-backed via `src/c_api/c_addon_profiler.rs`.
-- C_Ping.GetDefaultPingOptions is a temporary workaround in
-- `src/lua_api/workarounds/temporary/ping_defaults.rs`.
-- C_ZoneAbility is state-backed via `src/lua_api/globals/missing_surface/zone_ability.rs`.

-- AccountStore / DamageMeter / CooldownViewer: Blizzard data-provider init
-- iterates the returned category / session / cooldown list with ipairs.
-- None of these subsystems are simulated; return empty tables.

-- Pet battles: lightly modeled, but fresh simulator state is not in a battle.
-- `GetNumPets` is compared numerically during PetBattleFrame OnLoad refresh,
-- so returning nil crashes `petIndex > GetNumPets(owner)`. Zero is the
-- accurate "no pets" answer.
-- C_PetBattles.GetNumPets / GetBattleState are registered from Rust
-- (src/lua_api/globals/pet_battles.rs), backed by SimState::pet_battles.
-- The earlier __wow_merge_namespace at the top of this file already
-- installed the C_PetBattles namespace with stub methods; our Rust
-- registration overrides the two that the PLAN called out.
local __wow_store_state = rawget(_G, "__wow_store_state")
if type(__wow_store_state) ~= "table" then
  local featuredGroupID = 501
  local featuredEntryID = 1003
  local featuredProductID = 2003
  local vasServiceType = __WOW_VAS_SERVICE_NAME_CHANGE
  local vasDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE
  local fullCardWithBuy = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY
  local purchasable = __WOW_PURCHASE_ELIGIBILITY_OK
  local regionUS = REGION_US or 1

  local featuredEntry = {
    entryID = featuredEntryID,
    productID = featuredProductID,
    sharedData = {
      name = "Apprentice Rider Bundle",
      description = "A seeded store product used for simulator storefront coverage.",
      tooltip = "A seeded store product used for simulator storefront coverage.",
      texture = "Interface\\Icons\\Ability_Mount_RidingHorse",
      productDecorator = vasDecorator,
      cardType = fullCardWithBuy,
      buyableHere = true,
      eligibility = purchasable,
      flags = 0,
      currentDollars = 10,
      currentCents = 0,
      normalDollars = 10,
      normalCents = 0,
      deliverables = {},
      cards = {},
      vasServiceType = vasServiceType,
      canChangeAccount = true,
      canChangeBNetAccount = true,
      boostType = nil,
      instructions = "",
    },
  }

  __wow_store_state = {
    available = true,
    duplicateKey = nil,
    disconnectOnLogout = false,
    failureCode = nil,
    failureReason = nil,
    confirmationProductID = featuredProductID,
    bnetGuid = 3001,
    gameAccounts = { "WoW2", "WoW3" },
    localAccounts = { WoW1 = 1001 },
    remoteAccounts = { WoW2 = 2002, WoW3 = 2003 },
    realms = {
      { virtualRealmAddress = 101, realmName = "Azeroth" },
      { virtualRealmAddress = 202, realmName = "Kalimdor" },
    },
    characters = {
      [101] = {
        { guid = 501001, name = "Simhero", realmName = "Azeroth", wowAccount = 1001, guildMaster = true },
        { guid = 501002, name = "Simshaman", realmName = "Azeroth", wowAccount = 1001, guildMaster = false },
      },
      [202] = {
        { guid = 602001, name = "KalimdorMage", realmName = "Kalimdor", wowAccount = 2002, guildMaster = false },
      },
    },
    productGroups = {
      { groupID = featuredGroupID, parentGroupID = 0 },
    },
    productGroupInfo = {
      [featuredGroupID] = {
        groupName = "Featured",
        texture = "Interface\\Icons\\INV_Misc_Coin_01",
        flags = 0,
        disabledTooltip = nil,
      },
    },
    productsByGroup = {
      [featuredGroupID] = { featuredEntryID },
    },
    entriesByID = {
      [featuredEntryID] = featuredEntry,
    },
    productsByID = {
      [featuredProductID] = featuredEntry,
    },
    currencyInfo = {
      sharedData = {
        regionID = regionUS,
        requireLicenseAccept = false,
        browseHasStar = false,
        hideBrowseNotice = false,
        hideConfirmationBrowseNotice = false,
        licenseAcceptText = "",
        formatShort = "%s",
        formatLong = "%s",
      },
    },
    completion = {
      productID = nil,
      guid = nil,
      realmName = nil,
    },
  }
  rawset(_G, "__wow_store_state", __wow_store_state)
end

local function __wow_store_realm_name(virtualRealmAddress)
  for _, realm in ipairs(__wow_store_state.realms) do
    if realm.virtualRealmAddress == virtualRealmAddress then
      return realm.realmName
    end
  end
  return nil
end

local function __wow_store_character_by_guid(guid)
  for _, realmCharacters in pairs(__wow_store_state.characters) do
    for _, character in ipairs(realmCharacters) do
      if character.guid == guid then
        return character
      end
    end
  end
  return nil
end

local function __wow_store_frame()
  local frame = rawget(_G, "StoreFrame")
  if frame ~= nil then
    return frame
  end

  local secureenv = rawget(_G, "__secureenv")
  if type(secureenv) == "table" then
    return rawget(secureenv, "StoreFrame")
  end
  return nil
end

C_StoreSecure = __wow_merge_namespace(rawget(_G, "C_StoreSecure"), {})
C_StoreSecure.IsAvailable = function()
  return __wow_store_state.available == true
end
C_StoreSecure.HasPurchaseList = function()
  return true
end
C_StoreSecure.HasProductList = function()
  return true
end
C_StoreSecure.HasDistributionList = function()
  return true
end
C_StoreSecure.HasPurchaseInProgress = function()
  return false
end
C_StoreSecure.GetCurrencyID = function()
  return 1
end
C_StoreSecure.GetCurrencyInfo = function()
  return __wow_store_state.currencyInfo
end
C_StoreSecure.GetPurchaseList = function()
  if StoreFrame and type(StoreFrame.IsShown) == "function" and StoreFrame:IsShown() then
    FireEvent("STORE_PURCHASE_LIST_UPDATED")
  end
  return true
end
C_StoreSecure.GetProductList = function()
  local storeFrame = __wow_store_frame()
  local storeShown = storeFrame and type(storeFrame.IsShown) == "function" and storeFrame:IsShown()
  if storeShown then
    FireEvent("STORE_PRODUCTS_UPDATED")
    FireEvent("PRODUCT_DISTRIBUTIONS_UPDATED")
    if type(StoreFrame_OnEvent) == "function" then
      StoreFrame_OnEvent(storeFrame, "STORE_PRODUCTS_UPDATED")
    elseif type(StoreFrame_UpdateSelectedCategory) == "function" then
      StoreFrame_UpdateSelectedCategory()
      if type(StoreFrame_SetCategory) == "function" then
        StoreFrame_SetCategory(true)
      end
    end
  end
  return true
end
C_StoreSecure.GetProductGroups = function()
  return __wow_store_state.productGroups
end
C_StoreSecure.GetProductGroupInfo = function(groupID)
  return __wow_store_state.productGroupInfo[groupID]
end
C_StoreSecure.GetProducts = function(groupID)
  return __wow_store_state.productsByGroup[groupID] or {}
end
C_StoreSecure.GetEntryInfo = function(entryID)
  return __wow_store_state.entriesByID[entryID]
end
C_StoreSecure.GetProductInfo = function(productID)
  return __wow_store_state.productsByID[productID]
end
C_StoreSecure.GetWoWAccountGUIDFromName = function(accountName, isLocalAccount)
  if isLocalAccount then
    return __wow_store_state.localAccounts[accountName]
  end
  return __wow_store_state.remoteAccounts[accountName]
end
C_StoreSecure.ValidateBnetTransfer = function(_email)
  FireEvent("VAS_TRANSFER_VALIDATION_UPDATE", false)
end
C_StoreSecure.GetBnetTransferInfo = function()
  return __wow_store_state.bnetGuid, __wow_store_state.gameAccounts
end
C_StoreSecure.GetRealmList = function()
  return __wow_store_state.realms
end
C_StoreSecure.GetVASRealmList = function()
  return __wow_store_state.realms
end
C_StoreSecure.GetCharactersForRealm = function(virtualRealmAddress, guildOnly)
  local allCharacters = __wow_store_state.characters[virtualRealmAddress] or {}
  if not guildOnly then
    return allCharacters
  end

  local guildCharacters = {}
  for _, character in ipairs(allCharacters) do
    if character.guildMaster then
      table.insert(guildCharacters, character)
    end
  end
  return guildCharacters
end
C_StoreSecure.GetCharacterInfoByGUID = function(guid)
  return __wow_store_character_by_guid(guid)
end
C_StoreSecure.GetEligibleRacesForVASService = function(_guid, _serviceType)
  return {
    { raceID = 1, raceName = "Human", isAlliedRace = false },
    { raceID = 29, raceName = "Void Elf", isAlliedRace = true },
  }
end
C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID = function(guid)
  if guid == 501001 then
    return {
      guildName = "Simulator Guild",
      guildMasterName = "Simleader",
    }
  end
  return nil
end
C_StoreSecure.GetVasServiceType = function(productID)
  local product = C_StoreSecure.GetProductInfo(productID)
  return product and product.sharedData and product.sharedData.vasServiceType or nil
end
C_StoreSecure.IsRegionLocked = function()
  return false
end
C_StoreSecure.GetLastProductListResponseError = function()
  return 0
end
C_StoreSecure.GetVASErrors = function()
  return {}
end
C_StoreSecure.RequestRealmGuildMasterInfo = function(virtualRealmAddress)
  FireEvent("STORE_GUILD_MASTER_INFO_RECEIVED", virtualRealmAddress)
end
C_StoreSecure.RequestCharacterGuildFollowInfo = function(guid, _virtualRealmAddress)
  FireEvent("STORE_GUILD_FOLLOW_INFO_RECEIVED", guid, { transferredRealm = "Kalimdor" })
end
C_StoreSecure.OpenNydusLink = function(entryID)
  local entry = C_StoreSecure.GetEntryInfo(entryID)
  if entry then
    __wow_store_state.confirmationProductID = entry.productID
  end
end
C_StoreSecure.GetConfirmationInfo = function()
  return __wow_store_state.confirmationProductID, "Blizzard Balance", nil, nil, 10, 0
end
C_StoreSecure.GetUnrevokedBoostInfo = function()
  return "Level 70 Character Boost", "Simhero", "Azeroth"
end
C_StoreSecure.PurchaseVASProduct = function(productID, guid, _newName, _guildName, _guildMasterGuid, destinationRealmAddress)
  local realmName = __wow_store_realm_name(destinationRealmAddress)
  local duplicateKey = string.format("%s:%s:%s", tostring(productID), tostring(guid), tostring(realmName))
  if __wow_store_state.duplicateKey == duplicateKey then
    __wow_store_state.failureCode = Enum.StoreError and Enum.StoreError.Other or 1
    __wow_store_state.failureReason = "DuplicateVASPurchase"
    return false
  end

  __wow_store_state.duplicateKey = duplicateKey
  __wow_store_state.completion.productID = productID
  __wow_store_state.completion.guid = guid
  __wow_store_state.completion.realmName = realmName
  return true
end
C_StoreSecure.GetVASCompletionInfo = function()
  return __wow_store_state.completion.productID, __wow_store_state.completion.guid, __wow_store_state.completion.realmName, __wow_store_state.disconnectOnLogout == true
end
C_StoreSecure.GetFailureInfo = function()
  return __wow_store_state.failureCode, __wow_store_state.failureReason
end
C_StoreSecure.AckFailure = function()
  __wow_store_state.failureCode = nil
  __wow_store_state.failureReason = nil
end
C_StoreSecure.ClearPreGeneratedExternalTransactionID = function()
  __wow_store_state.duplicateKey = nil
end
C_StoreSecure.SetDisconnectOnLogout = function(shouldDisconnect)
  __wow_store_state.disconnectOnLogout = shouldDisconnect == true
end
C_StoreSecure.SetVASProductReady = function(isReady)
  if isReady then
    FireEvent("STORE_VAS_PURCHASE_COMPLETE")
  end
end
C_StoreSecure.RequestAllDynamicPriceInfo = __wow_noop
C_StoreSecure.HasDynamicPriceData = function()
  return true
end
C_StoreSecure.IsDynamicBundle = function()
  return false
end

local __wow_store_secure_state = {
  available = true,
  has_purchase_list = true,
  has_product_list = true,
  has_distribution_list = true,
  region_locked = false,
  last_product_list_response_error = 0,
  vas_errors = {},
  failure_code = nil,
  failure_reason = nil,
  confirmation_product_id = nil,
  confirmation_wallet_name = "Blizzard Balance",
  confirmation_current_dollars = 10,
  confirmation_current_cents = 0,
  completion_product_id = nil,
  completion_guid = nil,
  completion_realm_name = nil,
  completion_should_handle = false,
  disconnect_on_logout = false,
  purchase_in_progress = false,
  pre_generated_external_transaction_id = false,
  bnet_transfer_guid = 3001,
  bnet_transfer_game_accounts = { "WoW2", "WoW3" },
  bnet_transfer_validated = false,
}

local __wow_store_realms = {
  { realmName = "Azeroth", virtualRealmAddress = 101 },
  { realmName = "Kalimdor", virtualRealmAddress = 102 },
}

local __wow_store_characters = {
  [101] = {
    {
      guid = 501001,
      name = "Simhero",
      realmName = "Azeroth",
      currentServer = 101,
      classFileName = "WARRIOR",
      className = "Warrior",
      level = 70,
      raceName = "Human",
      faction = 0,
      wowAccount = 1001,
      createScreenIconAtlas = "",
    },
    {
      guid = 501002,
      name = "Simalt",
      realmName = "Azeroth",
      currentServer = 101,
      classFileName = "MAGE",
      className = "Mage",
      level = 70,
      raceName = "Void Elf",
      faction = 1,
      wowAccount = 1002,
      createScreenIconAtlas = "",
    },
  },
  [102] = {
    {
      guid = 502001,
      name = "KalimdorHero",
      realmName = "Kalimdor",
      currentServer = 102,
      classFileName = "PRIEST",
      className = "Priest",
      level = 70,
      raceName = "Night Elf",
      faction = 1,
      wowAccount = 2001,
      createScreenIconAtlas = "",
    },
  },
}

local __wow_store_guild_master_info = {
  [501001] = {
    guildName = "Simulator Guild",
    guildMasterName = "Simleader",
    guildMasterGuid = 501001,
  },
}

local __wow_store_product_groups = {
  {
    groupID = 22,
    parentGroupID = nil,
    groupName = "Services",
    texture = "Interface\\Icons\\INV_Misc_QuestionMark",
    flags = 0,
    disabledTooltip = nil,
  },
}

local __wow_store_products = {
  [2003] = {
    productID = 2003,
    sharedData = {
      name = "Apprentice Rider Bundle",
      description = "Simulator store product.",
      tooltip = "",
      texture = "Interface\\Icons\\INV_Misc_Note_02",
      productDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
      vasServiceType = __WOW_VAS_SERVICE_NAME_CHANGE,
      cardType = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
      flags = 0,
      eligibility = __WOW_PURCHASE_ELIGIBILITY_OK,
      buyableHere = true,
      currentDollars = 10,
      currentCents = 0,
      normalDollars = 10,
      normalCents = 0,
      instructions = "",
      canChangeAccount = true,
      canChangeBNetAccount = true,
      canChangeRealm = true,
      deliverables = {},
      cards = {},
    },
  },
  [189] = {
    productID = 189,
    sharedData = {
      name = "Character Transfer",
      description = "Simulator character transfer.",
      tooltip = "",
      texture = "Interface\\Icons\\INV_Misc_Note_02",
      productDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
      vasServiceType = __WOW_VAS_SERVICE_CHARACTER_TRANSFER,
      cardType = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
      flags = 0,
      eligibility = __WOW_PURCHASE_ELIGIBILITY_OK,
      buyableHere = true,
      currentDollars = 25,
      currentCents = 0,
      normalDollars = 25,
      normalCents = 0,
      instructions = "",
      canChangeAccount = true,
      canChangeBNetAccount = true,
      canChangeRealm = true,
      deliverables = {},
      cards = {},
    },
  },
  [239] = {
    productID = 239,
    sharedData = {
      name = "Character Transfer Bundle",
      description = "Simulator transfer bundle.",
      tooltip = "",
      texture = "Interface\\Icons\\INV_Misc_Note_02",
      productDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
      vasServiceType = __WOW_VAS_SERVICE_CHARACTER_TRANSFER,
      cardType = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
      flags = 0,
      eligibility = __WOW_PURCHASE_ELIGIBILITY_OK,
      buyableHere = true,
      currentDollars = 25,
      currentCents = 0,
      normalDollars = 25,
      normalCents = 0,
      instructions = "",
      canChangeAccount = true,
      canChangeBNetAccount = true,
      canChangeRealm = true,
      deliverables = {},
      cards = {},
    },
  },
  [476] = {
    productID = 476,
    sharedData = {
      name = "Guild Transfer",
      description = "Simulator guild transfer.",
      tooltip = "",
      texture = "Interface\\Icons\\INV_Misc_Note_02",
      productDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
      vasServiceType = __WOW_VAS_SERVICE_GUILD_TRANSFER,
      cardType = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
      flags = 0,
      eligibility = __WOW_PURCHASE_ELIGIBILITY_OK,
      buyableHere = true,
      currentDollars = 35,
      currentCents = 0,
      normalDollars = 35,
      normalCents = 0,
      instructions = "",
      canChangeAccount = true,
      canChangeBNetAccount = true,
      canChangeRealm = true,
      deliverables = {},
      cards = {},
    },
  },
  [477] = {
    productID = 477,
    sharedData = {
      name = "Guild Transfer Bundle",
      description = "Simulator guild transfer bundle.",
      tooltip = "",
      texture = "Interface\\Icons\\INV_Misc_Note_02",
      productDecorator = __WOW_BATTLEPAY_DECORATOR_VAS_SERVICE,
      vasServiceType = __WOW_VAS_SERVICE_GUILD_TRANSFER,
      cardType = __WOW_BATTLEPAY_CARD_MEDIUM_WITH_BUY,
      flags = 0,
      eligibility = __WOW_PURCHASE_ELIGIBILITY_OK,
      buyableHere = true,
      currentDollars = 35,
      currentCents = 0,
      normalDollars = 35,
      normalCents = 0,
      instructions = "",
      canChangeAccount = true,
      canChangeBNetAccount = true,
      canChangeRealm = true,
      deliverables = {},
      cards = {},
    },
  },
}

local function __wow_store_realm_name(address)
  if address == 101 then
    return "Azeroth"
  elseif address == 102 or address == 202 then
    return "Kalimdor"
  end
  return tostring(address or "")
end

local function __wow_store_find_character(guid)
  for _, realmCharacters in pairs(__wow_store_characters) do
    for _, character in ipairs(realmCharacters) do
      if character.guid == guid then
        return character
      end
    end
  end
  return nil
end

local function __wow_store_product(productID)
  return __wow_store_products[tonumber(productID) or -1]
end

C_StoreSecure = C_StoreSecure or __wow_namespace()
if rawget(C_StoreSecure, "_state") == nil then
  C_StoreSecure._state = __wow_store_secure_state
end
if rawget(C_StoreSecure, "IsAvailable") == nil then
  function C_StoreSecure.IsAvailable() return __wow_store_secure_state.available end
end
if rawget(C_StoreSecure, "HasPurchaseList") == nil then
  function C_StoreSecure.HasPurchaseList() return __wow_store_secure_state.has_purchase_list end
end
if rawget(C_StoreSecure, "HasProductList") == nil then
  function C_StoreSecure.HasProductList() return __wow_store_secure_state.has_product_list end
end
if rawget(C_StoreSecure, "HasDistributionList") == nil then
  function C_StoreSecure.HasDistributionList() return __wow_store_secure_state.has_distribution_list end
end
function C_StoreSecure.HasPurchaseInProgress()
  return __wow_store_secure_state.purchase_in_progress
end
if rawget(C_StoreSecure, "IsRegionLocked") == nil then
  function C_StoreSecure.IsRegionLocked() return __wow_store_secure_state.region_locked end
end
if rawget(C_StoreSecure, "GetLastProductListResponseError") == nil then
  function C_StoreSecure.GetLastProductListResponseError() return __wow_store_secure_state.last_product_list_response_error end
end
if rawget(C_StoreSecure, "GetVASErrors") == nil then
  function C_StoreSecure.GetVASErrors() return __wow_store_secure_state.vas_errors end
end
if rawget(C_StoreSecure, "GetCurrencyInfo") == nil then
  function C_StoreSecure.GetCurrencyInfo()
    return {
      sharedData = {
        regionID = 1,
        formatShort = "%s",
        formatLong = "%s",
        licenseAcceptText = "",
        requireLicenseAccept = false,
        browseHasStar = false,
        hideBrowseNotice = false,
        hideConfirmationBrowseNotice = false,
      },
    }
  end
end
if rawget(C_StoreSecure, "GetProductGroups") == nil then
  function C_StoreSecure.GetProductGroups() return __wow_store_product_groups end
end
if rawget(C_StoreSecure, "GetProductGroupInfo") == nil then
  function C_StoreSecure.GetProductGroupInfo(groupID)
    for _, group in ipairs(__wow_store_product_groups) do
      if group.groupID == groupID then
        return group
      end
    end
    return nil
  end
end
if rawget(C_StoreSecure, "GetProducts") == nil then
  function C_StoreSecure.GetProducts(groupID)
    if groupID == 22 then
      return { 2003, 189, 239, 476, 477 }
    end
    return {}
  end
end
if rawget(C_StoreSecure, "GetEntryInfo") == nil then
  function C_StoreSecure.GetEntryInfo(entryID) return __wow_store_product(entryID) end
end
if rawget(C_StoreSecure, "GetProductInfo") == nil then
  function C_StoreSecure.GetProductInfo(productID) return __wow_store_product(productID) end
end
if rawget(C_StoreSecure, "IsDynamicBundle") == nil then
  function C_StoreSecure.IsDynamicBundle(_productID) return false end
end
if rawget(C_StoreSecure, "HasDynamicPriceData") == nil then
  function C_StoreSecure.HasDynamicPriceData(_productID) return true end
end
if rawget(C_StoreSecure, "RequestAllDynamicPriceInfo") == nil then
  function C_StoreSecure.RequestAllDynamicPriceInfo() return nil end
end
if rawget(C_StoreSecure, "GetProductList") == nil then
  function C_StoreSecure.GetProductList()
    __wow_store_secure_state.has_product_list = true
    FireEvent("STORE_PRODUCTS_UPDATED")
    local storeFrame = __wow_store_frame()
    if storeFrame and type(StoreFrame_OnEvent) == "function" then
      StoreFrame_OnEvent(storeFrame, "STORE_PRODUCTS_UPDATED")
    end
    return nil
  end
end
if rawget(C_StoreSecure, "GetPurchaseList") == nil then
  function C_StoreSecure.GetPurchaseList()
    __wow_store_secure_state.has_purchase_list = true
    FireEvent("STORE_PURCHASE_LIST_UPDATED")
    return nil
  end
end
if rawget(C_StoreSecure, "GetDistributionList") == nil then
  function C_StoreSecure.GetDistributionList()
    __wow_store_secure_state.has_distribution_list = true
    return {}
  end
end
function C_StoreSecure.GetFailureInfo()
  return __wow_store_secure_state.failure_code, __wow_store_secure_state.failure_reason
end
function C_StoreSecure.AckFailure()
  __wow_store_secure_state.failure_code = nil
  __wow_store_secure_state.failure_reason = nil
end
function C_StoreSecure.ClearPreGeneratedExternalTransactionID()
  __wow_store_secure_state.pre_generated_external_transaction_id = false
end
function C_StoreSecure.OpenNydusLink(productID)
  local normalized = tonumber(productID) or 0
  if normalized == 1003 then
    normalized = 2003
  end
  local product = __wow_store_product(normalized)
  if product then
    __wow_store_secure_state.confirmation_product_id = normalized
    __wow_store_secure_state.confirmation_wallet_name = "Blizzard Balance"
    __wow_store_secure_state.confirmation_current_dollars = product.sharedData.currentDollars
    __wow_store_secure_state.confirmation_current_cents = product.sharedData.currentCents
  end
end
function C_StoreSecure.GetConfirmationInfo()
  return __wow_store_secure_state.confirmation_product_id, __wow_store_secure_state.confirmation_wallet_name, nil, nil, __wow_store_secure_state.confirmation_current_dollars, __wow_store_secure_state.confirmation_current_cents
end
if rawget(C_StoreSecure, "GetUnrevokedBoostInfo") == nil then
  function C_StoreSecure.GetUnrevokedBoostInfo()
    return "Level 70 Character Boost", "Simhero", "Azeroth"
  end
end
function C_StoreSecure.GetVASCompletionInfo()
  return __wow_store_secure_state.completion_product_id, __wow_store_secure_state.completion_guid, __wow_store_secure_state.completion_realm_name, __wow_store_secure_state.completion_should_handle
end
function C_StoreSecure.SetDisconnectOnLogout(disconnectOnLogout)
  __wow_store_secure_state.disconnect_on_logout = disconnectOnLogout and true or false
  if __wow_store_secure_state.completion_product_id then
    __wow_store_secure_state.completion_should_handle = __wow_store_secure_state.disconnect_on_logout
  end
end
function C_StoreSecure.SetVASProductReady(ready)
  if ready and __wow_store_secure_state.completion_product_id then
    __wow_store_secure_state.purchase_in_progress = false
    FireEvent("STORE_VAS_PURCHASE_COMPLETE")
  end
end
function C_StoreSecure.PurchaseVASProduct(productID, guid, _name, _oldGuildName, _newGuildMasterGuid, realmValue, _wowAccountGuid, _bnetAccountGuid, _transferFactionChangeBundle, _isGuildFollow)
  if __wow_store_secure_state.completion_product_id and __wow_store_secure_state.pre_generated_external_transaction_id then
    __wow_store_secure_state.failure_code = Enum.StoreError.Other
    __wow_store_secure_state.failure_reason = "DuplicateVASPurchase"
    return false
  end

  local product = __wow_store_product(productID)
  if not product then
    __wow_store_secure_state.failure_code = Enum.StoreError.Other
    __wow_store_secure_state.failure_reason = "UnknownVASProduct"
    return false
  end

  __wow_store_secure_state.confirmation_product_id = productID
  __wow_store_secure_state.confirmation_wallet_name = "Blizzard Balance"
  __wow_store_secure_state.confirmation_current_dollars = product.sharedData.currentDollars
  __wow_store_secure_state.confirmation_current_cents = product.sharedData.currentCents
  __wow_store_secure_state.completion_product_id = productID
  __wow_store_secure_state.completion_guid = guid
  __wow_store_secure_state.completion_realm_name = __wow_store_realm_name(realmValue)
  __wow_store_secure_state.completion_should_handle = __wow_store_secure_state.disconnect_on_logout
  __wow_store_secure_state.purchase_in_progress = true
  __wow_store_secure_state.pre_generated_external_transaction_id = true
  __wow_store_secure_state.failure_code = nil
  __wow_store_secure_state.failure_reason = nil
  return true
end
if rawget(C_StoreSecure, "PurchaseProduct") == nil then
  function C_StoreSecure.PurchaseProduct(productID)
    return C_StoreSecure.PurchaseVASProduct(productID, 0, nil, nil, nil, 101, nil, nil, false, false)
  end
end
if rawget(C_StoreSecure, "PurchaseProductConfirm") == nil then
  function C_StoreSecure.PurchaseProductConfirm(confirm, _dollars, _cents)
    if confirm and __wow_store_secure_state.completion_product_id then
      __wow_store_secure_state.purchase_in_progress = false
      FireEvent("STORE_VAS_PURCHASE_COMPLETE")
    end
    return true
  end
end
if rawget(C_StoreSecure, "ValidateBnetTransfer") == nil then
  function C_StoreSecure.ValidateBnetTransfer(_email)
    __wow_store_secure_state.bnet_transfer_validated = true
    FireEvent("VAS_TRANSFER_VALIDATION_UPDATE", false)
  end
end
if rawget(C_StoreSecure, "GetBnetTransferInfo") == nil then
  function C_StoreSecure.GetBnetTransferInfo()
    return __wow_store_secure_state.bnet_transfer_guid, __wow_store_secure_state.bnet_transfer_game_accounts
  end
end
if rawget(C_StoreSecure, "GetWoWAccountGUIDFromName") == nil then
  function C_StoreSecure.GetWoWAccountGUIDFromName(name, isLocal)
    if isLocal and name == "WoW1" then
      return 1001
    elseif not isLocal and name == "WoW2" then
      return 2002
    end
    return nil
  end
end
if rawget(C_StoreSecure, "GetRealmList") == nil then
  function C_StoreSecure.GetRealmList() return __wow_store_realms end
end
if rawget(C_StoreSecure, "GetVASRealmList") == nil then
  function C_StoreSecure.GetVASRealmList() return __wow_store_realms end
end
if rawget(C_StoreSecure, "GetCharactersForRealm") == nil then
  function C_StoreSecure.GetCharactersForRealm(realmAddress, guildOnly)
    local realmCharacters = __wow_store_characters[tonumber(realmAddress) or -1] or {}
    local filtered = {}
    for _, character in ipairs(realmCharacters) do
      if not guildOnly or character.guid == 501001 then
        table.insert(filtered, character)
      end
    end
    return filtered
  end
end
if rawget(C_StoreSecure, "GetCharacterInfoByGUID") == nil then
  function C_StoreSecure.GetCharacterInfoByGUID(guid)
    return __wow_store_find_character(tonumber(guid) or -1)
  end
end
if rawget(C_StoreSecure, "GetEligibleRacesForVASService") == nil then
  function C_StoreSecure.GetEligibleRacesForVASService(_characterGuid, vasServiceType)
    if vasServiceType == __WOW_VAS_SERVICE_NAME_CHANGE then
      return {
        { raceName = "Human", isAlliedRace = false, isHeritageArmorUnlocked = true },
        { raceName = "Void Elf", isAlliedRace = true, isHeritageArmorUnlocked = true },
      }
    end
    return {}
  end
end
if rawget(C_StoreSecure, "GetVASGuildMasterInfoForCharacterByGUID") == nil then
  function C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(guid)
    return __wow_store_guild_master_info[tonumber(guid) or -1]
  end
end
if rawget(C_StoreSecure, "GetVasServiceType") == nil then
  function C_StoreSecure.GetVasServiceType(productID)
    local normalized = tonumber(productID) or -1
    if normalized == 2003 then
      return __WOW_VAS_SERVICE_NAME_CHANGE
    elseif normalized == 189 or normalized == 239 then
      return __WOW_VAS_SERVICE_CHARACTER_TRANSFER
    elseif normalized == 476 or normalized == 477 then
      return __WOW_VAS_SERVICE_GUILD_TRANSFER
    end
    return nil
  end
end
if rawget(C_StoreSecure, "RequestRealmGuildMasterInfo") == nil then
  function C_StoreSecure.RequestRealmGuildMasterInfo(realmAddress)
    FireEvent("STORE_GUILD_MASTER_INFO_RECEIVED", realmAddress)
  end
end
if rawget(C_StoreSecure, "RequestCharacterGuildFollowInfo") == nil then
  function C_StoreSecure.RequestCharacterGuildFollowInfo(guid, realmAddress)
    FireEvent("STORE_GUILD_FOLLOW_INFO_RECEIVED", guid, { transferredRealm = __wow_store_realm_name(realmAddress) })
  end
end
if rawget(C_StoreSecure, "AckFailure") == nil then
  function C_StoreSecure.AckFailure()
    __wow_store_secure_state.failure_code = nil
    __wow_store_secure_state.failure_reason = nil
  end
end
if rawget(C_StoreSecure, "ClearPreGeneratedExternalTransactionID") == nil then
  function C_StoreSecure.ClearPreGeneratedExternalTransactionID()
    __wow_store_secure_state.pre_generated_external_transaction_id = false
  end
end

-- GetAvailableLocaleInfo is registered from Rust
-- (src/lua_api/globals/real/locale_info.rs). Returns the 12-locale retail list
-- as { localeId, localeName, englishName, displayName } entries.
-- GuildControlSetRank / GuildControlGetRankName / GuildControlGetNumRanks /
-- GuildControlGetRankFlags are registered from Rust
-- (src/lua_api/globals/guild_control.rs), backed by SimState::world.guild_ranks.
-- Admin: A_Admin.SetGuildRanks({ {name="Leader", flags={true,...}}, ... }).
local function __wow_find_first_scroll_frame_child(parent)
  if parent == nil or type(parent.GetChildren) ~= "function" then
    return nil
  end

  local count = parent:GetNumChildren()
  for index = 1, count do
    local child = select(index, parent:GetChildren())
    if type(child) == "table" then
      local isScrollFrame =
        (type(child.IsObjectType) == "function" and child:IsObjectType("ScrollFrame")) or
        (type(child.GetObjectType) == "function" and child:GetObjectType() == "ScrollFrame")
      if isScrollFrame then
        return child
      end
    end
  end

  return nil
end

local function __wow_ensure_map_canvas_scroll_container(frame)
  if type(frame) ~= "table" then
    return nil
  end

  local existing = rawget(frame, "ScrollContainer")
  if existing ~= nil then
    return existing
  end

  local scroll = __wow_find_first_scroll_frame_child(frame)
  if scroll ~= nil then
    rawset(frame, "ScrollContainer", scroll)
  end
  return scroll
end

local function __wow_patch_map_canvas_scroll_container_methods()
  if rawget(_G, "__wow_map_canvas_scroll_container_patched") then
    return
  end
  if type(MapCanvasMixin) ~= "table" then
    return
  end

  if type(MapCanvasMixin.SetMapID) == "function" then
    local originalSetMapID = MapCanvasMixin.SetMapID
    MapCanvasMixin.SetMapID = function(self, ...)
      if __wow_ensure_map_canvas_scroll_container(self) == nil then
        local mapID = ...
        self.mapID = mapID
        if C_Map and type(C_Map.GetMapArtID) == "function" then
          self.mapArtID = C_Map.GetMapArtID(mapID)
        end
        return
      end
      return originalSetMapID(self, ...)
    end
  end

  if type(MapCanvasMixin.GetCanvas) == "function" then
    MapCanvasMixin.GetCanvas = function(self, ...)
      local scroll = __wow_ensure_map_canvas_scroll_container(self)
      return scroll and scroll.Child or nil
    end
  end

  if type(MapCanvasMixin.GetCanvasContainer) == "function" then
    MapCanvasMixin.GetCanvasContainer = function(self, ...)
      return __wow_ensure_map_canvas_scroll_container(self)
    end
  end

  if type(MapCanvasMixin.OnFrameSizeChanged) == "function" then
    local originalOnFrameSizeChanged = MapCanvasMixin.OnFrameSizeChanged
    MapCanvasMixin.OnFrameSizeChanged = function(self, ...)
      if __wow_ensure_map_canvas_scroll_container(self) == nil then
        return
      end
      return originalOnFrameSizeChanged(self, ...)
    end
  end

  __wow_map_canvas_scroll_container_patched = true
end

if rawget(_G, "PVEFrame_ToggleFrame") == nil then
  function PVEFrame_ToggleFrame(...)
    local loadAddOn = C_AddOns and C_AddOns.LoadAddOn
    if type(loadAddOn) == "function" then
      pcall(loadAddOn, "Blizzard_GroupFinder")
    end

    local loaded = rawget(_G, "PVEFrame_ToggleFrame")
    if type(loaded) == "function" and loaded ~= PVEFrame_ToggleFrame then
      return loaded(...)
    end
  end
end

__wow_patch_map_canvas_scroll_container_methods()

local function __wow_patch_fog_of_war_pin_methods()
  if rawget(_G, "__wow_fog_of_war_pin_methods_patched") then
    return
  end
  if type(FogOfWarPinMixin) ~= "table" then
    return
  end

  if type(FogOfWarPinMixin.OnMapChanged) == "function" then
    FogOfWarPinMixin.OnMapChanged = function(self)
      local mapID = nil
      if type(self.GetMap) == "function" then
        local map = self:GetMap()
        if map ~= nil and type(map.GetMapID) == "function" then
          mapID = map:GetMapID()
        end
      end

      if (mapID == nil or mapID == 0) and C_Map ~= nil and type(C_Map.GetCurrentMapID) == "function" then
        mapID = C_Map.GetCurrentMapID()
      end

      if type(self.SetUiMapID) == "function" then
        self:SetUiMapID(mapID)
      end

      if type(self.TryFindingBestFogOfWarID) == "function" then
        self:TryFindingBestFogOfWarID(true)
      elseif (mapID == nil or mapID == 0) and type(self.Hide) == "function" then
        self:Hide()
      end
    end
  end

  rawset(_G, "__wow_fog_of_war_pin_methods_patched", true)
end

__wow_patch_fog_of_war_pin_methods()

local function __wow_patch_character_select_nav_bar()
  if rawget(_G, "__wow_character_select_nav_bar_patched") then
    return
  end
  if type(CharacterSelectNavBarMixin) ~= "table" then
    return
  end

  if type(CharacterSelectNavBarMixin.SetRealmsButtonEnabled) == "function" then
    local original_set_realms_button_enabled = CharacterSelectNavBarMixin.SetRealmsButtonEnabled
    CharacterSelectNavBarMixin.SetRealmsButtonEnabled = function(self, enabled)
      if type(self) ~= "table" or self.RealmsButton == nil then
        return
      end
      return original_set_realms_button_enabled(self, enabled)
    end
  end

  rawset(_G, "__wow_character_select_nav_bar_patched", true)
end

__wow_patch_character_select_nav_bar()

local function __wow_patch_uiparent_onupdate_worklists()
  if type(FCF_OnUpdate) == "function" and rawget(_G, "__wow_fcf_onupdate_wrapper") ~= FCF_OnUpdate then
    local original_fcf_onupdate = FCF_OnUpdate
    local wrapper = function(elapsed)
      if type(CHAT_FRAMES) == "table" and next(CHAT_FRAMES) == nil then
        return
      end
      return original_fcf_onupdate(elapsed)
    end
    FCF_OnUpdate = wrapper
    rawset(_G, "__wow_fcf_onupdate_wrapper", wrapper)
  end

  if type(ButtonPulse_OnUpdate) == "function"
    and rawget(_G, "__wow_button_pulse_onupdate_wrapper") ~= ButtonPulse_OnUpdate then
    local original_button_pulse_onupdate = ButtonPulse_OnUpdate
    local wrapper = function(elapsed)
      if type(PULSEBUTTONS) == "table" and next(PULSEBUTTONS) == nil then
        return
      end
      return original_button_pulse_onupdate(elapsed)
    end
    ButtonPulse_OnUpdate = wrapper
    rawset(_G, "__wow_button_pulse_onupdate_wrapper", wrapper)
  end

  if type(AnimatedShine_OnUpdate) == "function"
    and rawget(_G, "__wow_animated_shine_onupdate_wrapper") ~= AnimatedShine_OnUpdate then
    local original_animated_shine_onupdate = AnimatedShine_OnUpdate
    local wrapper = function(elapsed)
      if type(SHINES_TO_ANIMATE) == "table" and next(SHINES_TO_ANIMATE) == nil then
        return
      end
      return original_animated_shine_onupdate(elapsed)
    end
    AnimatedShine_OnUpdate = wrapper
    rawset(_G, "__wow_animated_shine_onupdate_wrapper", wrapper)
  end

  if type(UIParent) == "table"
    and type(UIParent.GetScript) == "function"
    and type(UIParent.SetScript) == "function" then
    local wrapper = rawget(_G, "__wow_ui_parent_onupdate_worklist_wrapper")
    if UIParent:GetScript("OnUpdate") ~= wrapper then
      wrapper = function(self, elapsed)
        if type(CHAT_FRAMES) ~= "table" or next(CHAT_FRAMES) ~= nil then
          FCF_OnUpdate(elapsed)
        end
        if type(PULSEBUTTONS) ~= "table" or next(PULSEBUTTONS) ~= nil then
          ButtonPulse_OnUpdate(elapsed)
        end
        if type(SHINES_TO_ANIMATE) ~= "table" or next(SHINES_TO_ANIMATE) ~= nil then
          AnimatedShine_OnUpdate(elapsed)
        end
        if type(HelpOpenWebTicketButton_OnUpdate) == "function" then
          HelpOpenWebTicketButton_OnUpdate(HelpOpenWebTicketButton, elapsed)
        end
      end
      UIParent:SetScript("OnUpdate", wrapper)
      rawset(_G, "__wow_ui_parent_onupdate_worklist_wrapper", wrapper)
    end
  end
end

__wow_patch_uiparent_onupdate_worklists()

-- hooksecurefunc(C_AddOns, "LoadAddOn", ...) is deliberately refused by the
-- shared bootstrap, so these re-apply hooks run from Rust via
-- apply_blizzard_post_load_patches (patch_runtime_surface_for_addon_load).
-- Exposed as globals so post-load patch snippets (and the CreateFrame wrapper
-- in frame_helper_defaults.rs) can reach them from other chunks.
rawset(_G, "__wow_patch_character_select_nav_bar", __wow_patch_character_select_nav_bar)
rawset(_G, "__wow_patch_uiparent_onupdate_worklists", __wow_patch_uiparent_onupdate_worklists)
rawset(_G, "__wow_patch_map_canvas_scroll_container_methods", __wow_patch_map_canvas_scroll_container_methods)
rawset(_G, "__wow_patch_fog_of_war_pin_methods", __wow_patch_fog_of_war_pin_methods)

AUTOCOMPLETE_LIST = AUTOCOMPLETE_LIST or {}
AUTOCOMPLETE_LIST.ADDFRIEND = AUTOCOMPLETE_LIST.ADDFRIEND or {}
if type(setprinthandler) ~= "function" then
  function setprinthandler() end
end

if rawget(_G, "ToggleCollectionsJournal") == nil then
  function ToggleCollectionsJournal(tabIndex)
    if DISALLOW_FRAME_TOGGLING then
      return
    end
    if not CollectionsJournal and type(CollectionsJournal_LoadUI) == "function" then
      CollectionsJournal_LoadUI()
    end
    if CollectionsJournal and type(SetCollectionsJournalShown) == "function" then
      local tabMatches = not tabIndex or tabIndex == PanelTemplates_GetSelectedTab(CollectionsJournal)
      local isShown = CollectionsJournal:IsShown() and tabMatches
      SetCollectionsJournalShown(not isShown, tabIndex)
    elseif CollectionsJournal then
      if CollectionsJournal:IsShown() then
        CollectionsJournal:Hide()
      else
        CollectionsJournal:Show()
      end
    end
  end
end

if rawget(_G, "ToggleEncounterJournal") == nil then
  function ToggleEncounterJournal()
    if DISALLOW_FRAME_TOGGLING then
      return
    end
    if not EncounterJournal and type(EncounterJournal_LoadUI) == "function" then
      EncounterJournal_LoadUI()
    end
    if not EncounterJournal and type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
      C_AddOns.LoadAddOn("Blizzard_EncounterJournal")
    end
    if EncounterJournal then
      if EncounterJournal:IsShown() then
        if type(HideUIPanel) == "function" then
          HideUIPanel(EncounterJournal)
        else
          EncounterJournal:Hide()
        end
      else
        if type(ShowUIPanel) == "function" then
          ShowUIPanel(EncounterJournal)
        else
          EncounterJournal:Show()
        end
      end
    end
  end
end

if CreateTemplateInfoCache == nil then
  function CreateTemplateInfoCache()
    local cache = {
      templateInfos = {},
      infoAddedCallback = nil,
    }

    function cache:Init()
    end

    function cache:SetInfoAddedCallback(callback)
      self.infoAddedCallback = callback
    end

    function cache:FlushTemplateInfos()
      self.templateInfos = {}
    end

    function cache:GetTemplateInfo(frameTemplate)
      local info = self.templateInfos[frameTemplate]
      if info == nil and C_XMLUtil and C_XMLUtil.GetTemplateInfo then
        info = C_XMLUtil.GetTemplateInfo(frameTemplate)
        self.templateInfos[frameTemplate] = info
      end
      if info ~= nil and self.infoAddedCallback then
        self.infoAddedCallback(info)
      end
      return info
    end

    function cache:GetTemplateInfos()
      return self.templateInfos
    end

    cache:Init()
    return cache
  end
end

local __global_mt = getmetatable(_G) or {}
local __prev_index = __global_mt.__index
local __prev_newindex = __global_mt.__newindex
local function __wow_is_color_constant_key(key)
  if type(key) ~= "string" then
    return false
  end
  if key:match("_COLOR$") then
    return true
  end
  if not key:match("_COLOR_[A-Z0-9_]+$") then
    return false
  end
  return not key:match("_COLOR_CODE")
     and not key:match("_COLOR_TABLE")
     and not key:match("_COLOR_ATLASES")
end
local function __wow_preserve_nil_global(key)
  if type(key) ~= "string" then
    return false
  end
  return key:match("^SLASH_[A-Z0-9_]+%d+$") ~= nil
      or key:match("^EMOTE%d+_CMD%d+$") ~= nil
      or key:match("^EMOTE%d+_TOKEN$") ~= nil
end
local function __wow_prepare_global_assignment(key, value)
  local prepare = rawget(_G, "__wow_prepare_temporary_global_assignment")
  if type(prepare) == "function" then
    return prepare(key, value)
  end
  return value
end
__global_mt.__index = function(t, key)
  local value = nil
  if __prev_index ~= nil then
    if type(__prev_index) == "function" then
      value = __prev_index(t, key)
    else
      value = __prev_index[key]
    end
  end
  if value ~= nil then
    return value
  end
  if __wow_preserve_nil_global(key) then
    return nil
  end

  if key == "HIGHLIGHT_FONT_COLOR" then
    value = CreateColor(1, 1, 1, 1)
  elseif __wow_is_color_constant_key(key) then
    value = CreateColor(1, 1, 1, 1)
  elseif key == "PLAYER_FACTION_COLOR_HORDE" then
    value = CreateColor(1, 0.1, 0.1, 1)
  elseif key == "PLAYER_FACTION_COLOR_ALLIANCE" then
    value = CreateColor(0.2, 0.4, 1, 1)
  elseif type(key) == "string" and key:match("^C_[A-Za-z0-9_]+$") then
    __wow_log_nil_symbol_access("_G", key)
    value = __wow_attach_namespace_name(__wow_namespace(), key)
  elseif type(key) == "string" and key:match("^ERR_") then
    value = key
  end

  if value ~= nil then
    rawset(t, key, value)
    return value
  end
  if debug.isglobalindex() then
    __wow_log_nil_symbol_access("_G", key)
  end
  return nil
end
local __wow_record_public_global_publication = rawget(_G, "__wow_record_public_global_publication")
rawset(_G, "__wow_record_public_global_publication", nil)

local function __wow_record_public_global_assignment(t, key, value)
  if t ~= _G or type(key) ~= "string" or value == nil or key:sub(1, 2) == "C_" then
    return
  end
  if type(__wow_record_public_global_publication) == "function" then
    __wow_record_public_global_publication(key)
  end
end

__global_mt.__newindex = function(t, key, value)
  value = __wow_prepare_global_assignment(key, value)
  local taint = debug and debug.getstacktaint and debug.getstacktaint()
  if __prev_newindex ~= nil then
    if type(__prev_newindex) == "function" then
      __prev_newindex(t, key, value)
    else
      __prev_newindex[key] = value
      if taint and type(__sim_mark_slot_taint) == "function" then
        __sim_mark_slot_taint(__prev_newindex, key, taint)
      end
    end
  else
    rawset(t, key, value)
    if taint and type(__sim_mark_slot_taint) == "function" then
      __sim_mark_slot_taint(t, key, taint)
    end
  end
  __wow_record_public_global_assignment(t, key, value)
end
setmetatable(_G, __global_mt)
__wow_seed_namespace_names()

-- GetBuildOption(name) is documented in BuildDocumentation.lua and answered
-- from the client's build configuration. The only option Blizzard Lua reads
-- ("RestrictedAuraAPI", TargetFrameAuraShared.lua:32) selects the forbidden
-- aura-button templates, which the simulator does not model, so every option
-- reads as off. Without the function the unit-frame aura containers' OnLoad
-- died on a nil call.
if rawget(_G, "GetBuildOption") == nil then
  function GetBuildOption(name)
    return false
  end
end

-- table.create(narr, nrec) is the client's preallocating constructor (seven
-- Blizzard call sites, e.g. Blizzard_AuraContainerGroups.lua:27). Lua 5.1 has
-- no preallocation; a plain table is equivalent.
if table.create == nil then
  function table.create(_narr, _nrec)
    return {}
  end
end
