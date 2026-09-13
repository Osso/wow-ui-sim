local function __wow_noop()
end

local __wow_namespace_names = setmetatable({}, { __mode = "k" })
local __wow_namespace_mt = {
  __index = function(t, key)
    if key == nil then
      return nil
    end
    local value = function() end
    rawset(t, key, value)
    return value
  end,
}

local function __wow_namespace(defaults)
  defaults = defaults or {}
  return setmetatable(defaults, __wow_namespace_mt)
end

local function __wow_merge_namespace(existing, defaults)
  local namespace = existing
  if type(namespace) ~= "table" then
    namespace = __wow_namespace()
  else
    setmetatable(namespace, getmetatable(__wow_namespace()))
  end
  for key, value in pairs(defaults or {}) do
    if rawget(namespace, key) == nil then
      namespace[key] = value
    end
  end
  return namespace
end

local __wow_housing_entry_type = Enum.HousingCatalogEntryType and Enum.HousingCatalogEntryType.Decor or 0
local __wow_housing_all_category_id = Constants.HousingCatalogConsts.HOUSING_CATALOG_ALL_CATEGORY_ID

local __wow_housing_seeded_entries = {
  [1001] = {
    recordID = 1001,
    entryType = __wow_housing_entry_type,
    itemID = 1001,
    name = "Sunspire Chair",
    asset = nil,
    iconTexture = nil,
    iconAtlas = nil,
    uiModelSceneID = nil,
    categoryIDs = { __wow_housing_all_category_id, 101 },
    subcategoryIDs = { 1001 },
    dataTagsByID = {},
    size = Enum.HousingCatalogEntrySize and Enum.HousingCatalogEntrySize.Small or 2,
    placementCost = 1,
    totalNumStored = 1,
    remainingRedeemable = 0,
    totalNumPlaced = 1,
    destroyableInstanceCount = 1,
    isUniqueTrophy = false,
    isAllowedOutdoors = true,
    isAllowedIndoors = true,
    canCustomize = true,
    isPrefab = false,
    quality = 2,
    firstAcquisitionBonus = 0,
    sourceText = "",
  },
  [1002] = {
    recordID = 1002,
    entryType = __wow_housing_entry_type,
    itemID = 1002,
    name = "Azure Reading Lamp",
    asset = nil,
    iconTexture = nil,
    iconAtlas = nil,
    uiModelSceneID = nil,
    categoryIDs = { __wow_housing_all_category_id, 101 },
    subcategoryIDs = { 1002 },
    dataTagsByID = {},
    size = Enum.HousingCatalogEntrySize and Enum.HousingCatalogEntrySize.Small or 2,
    placementCost = 1,
    totalNumStored = 1,
    remainingRedeemable = 0,
    totalNumPlaced = 1,
    destroyableInstanceCount = 1,
    isUniqueTrophy = false,
    isAllowedOutdoors = true,
    isAllowedIndoors = true,
    canCustomize = true,
    isPrefab = false,
    quality = 2,
    firstAcquisitionBonus = 0,
    sourceText = "",
  },
}

local __wow_housing_seeded_variants = {
  [1001] = {
    [1] = { variantID = 1, productID = 91001, name = "Sunspire Chair", numStored = 1 },
    [2] = { variantID = 2, productID = 91002, name = "Azure Upholstery", numStored = 0 },
  },
  [1002] = {
    [1] = { variantID = 1, productID = 91003, name = "Azure Reading Lamp", numStored = 1 },
  },
}

local __wow_housing_seeded_featured_small_products = {
  {
    entryID = 1001,
    entryVariantID = { recordID = 1001, entryType = __wow_housing_entry_type, variantIdentifier = 1 },
    productID = 91001,
    price = 100,
    originalPrice = nil,
    canPreview = true,
  },
  {
    entryID = 1002,
    entryVariantID = { recordID = 1002, entryType = __wow_housing_entry_type, variantIdentifier = 1 },
    productID = 91003,
    price = 125,
    originalPrice = nil,
    canPreview = true,
  },
}

local __wow_housing_seeded_bundle_state = {
  [5001] = {
    productID = 5001,
    price = 500,
    originalPrice = nil,
    entryIDs = { 1001, 1002 },
    decorEntries = {
      { decorID = 1001, quantity = 1 },
      { decorID = 1002, quantity = 1 },
    },
    nonDecorProducts = {},
    canPreview = true,
    wasViewed = false,
  },
}

local __wow_housing_seeded_market_state = {
  [1001] = { price = 100, productID = 91001, bundleIDs = { 5001 }, isInCart = false, cartCount = 0, wasViewedInStore = false },
  [1002] = { price = 125, productID = 91003, bundleIDs = { 5001 }, isInCart = false, cartCount = 0, wasViewedInStore = false },
}

local CatalogShopConstants = rawget(_G, "CatalogShopConstants") or {
  ProductType = {
    Bundle = 1,
    Decor = 2,
  },
  CategoryLinks = {
    Featured = "featured",
    Housing = "housing",
  },
}
_G.CatalogShopConstants = CatalogShopConstants

local __wow_housing_seeded_product_infos = {
  [2003] = {
    catalogShopProductID = 2003,
    name = "Apprentice Rider Bundle",
    type = "Bundle",
    description = "A seeded store product used for simulator storefront coverage.",
    iconTexture = "Interface\\Icons\\Ability_Mount_RidingHorse",
    isFullyOwned = false,
    isPurchasePending = false,
    refundable = false,
    price = "10",
    originalPrice = "10",
    discountPercentage = 0,
    itemID = 0,
    mountID = 0,
    mountTypeName = "",
    speciesID = 0,
    transmogSetID = 0,
    itemModifiedAppearanceID = 0,
    subItems = {},
    subItemsLoaded = true,
    backgroundTexture = "shop-bg-map-blue",
    foregroundTexture = nil,
    smallCardBGTexture = nil,
    smallCardFGTexture = nil,
    wideCardBGTexture = nil,
    wideCardFGTexture = nil,
    previewIconTexture = nil,
    optionalWideCardBackgroundTexture = nil,
    isBundle = true,
    bundleChildrenSize = 2,
    licenseTermType = 0,
    licenseTermDuration = 0,
    virtualCurrencies = {},
    isHidden = false,
    hasPendingOrders = false,
    numBundleDetailCards = 2,
    isDynamicallyDiscounted = false,
    shouldShowOriginalPrice = false,
    wideCardBGOverrideProductURL = nil,
    previewBGOverrideProductURL = nil,
    previewSmallBGOverrideProductURL = nil,
    decorQuantity = nil,
    isVCProduct = false,
    containsHousingItem = true,
    creatureDisplayInfoIDs = {},
    spellVisualIDs = {},
    itemModifiedAppearanceIDs = {},
    mainHandItemModifiedAppearanceID = nil,
    offHandItemModifiedAppearanceID = nil,
    decorFileDataID = nil,
    houseTextureAtlas = nil,
    productType = 1,
    productIDList = { 20031, 20032 },
  },
  [20031] = {
    catalogShopProductID = 20031,
    name = "Apprentice Rider Saddle",
    type = "Decor",
    description = "Bundle child decor.",
    iconTexture = "Interface\\Icons\\INV_Misc_QuestionMark",
    isFullyOwned = false,
    isPurchasePending = false,
    refundable = false,
    price = "5",
    originalPrice = "5",
    discountPercentage = 0,
    itemID = 0,
    mountID = 0,
    mountTypeName = "",
    speciesID = 0,
    transmogSetID = 0,
    itemModifiedAppearanceID = 0,
    subItems = {},
    subItemsLoaded = true,
    backgroundTexture = "shop-bg-map-blue",
    foregroundTexture = nil,
    smallCardBGTexture = nil,
    smallCardFGTexture = nil,
    wideCardBGTexture = nil,
    wideCardFGTexture = nil,
    previewIconTexture = nil,
    optionalWideCardBackgroundTexture = nil,
    isBundle = false,
    bundleChildrenSize = 0,
    licenseTermType = 0,
    licenseTermDuration = 0,
    virtualCurrencies = {},
    isHidden = false,
    hasPendingOrders = false,
    numBundleDetailCards = 0,
    isDynamicallyDiscounted = false,
    shouldShowOriginalPrice = false,
    wideCardBGOverrideProductURL = nil,
    previewBGOverrideProductURL = nil,
    previewSmallBGOverrideProductURL = nil,
    decorQuantity = nil,
    isVCProduct = false,
    containsHousingItem = true,
    creatureDisplayInfoIDs = {},
    spellVisualIDs = {},
    itemModifiedAppearanceIDs = {},
    mainHandItemModifiedAppearanceID = nil,
    offHandItemModifiedAppearanceID = nil,
    decorFileDataID = 91001,
    houseTextureAtlas = nil,
    productType = 2,
  },
  [20032] = {
    catalogShopProductID = 20032,
    name = "Apprentice Rider Bridle",
    type = "Decor",
    description = "Bundle child decor.",
    iconTexture = "Interface\\Icons\\INV_Misc_QuestionMark",
    isFullyOwned = false,
    isPurchasePending = false,
    refundable = false,
    price = "5",
    originalPrice = "5",
    discountPercentage = 0,
    itemID = 0,
    mountID = 0,
    mountTypeName = "",
    speciesID = 0,
    transmogSetID = 0,
    itemModifiedAppearanceID = 0,
    subItems = {},
    subItemsLoaded = true,
    backgroundTexture = "shop-bg-map-blue",
    foregroundTexture = nil,
    smallCardBGTexture = nil,
    smallCardFGTexture = nil,
    wideCardBGTexture = nil,
    wideCardFGTexture = nil,
    previewIconTexture = nil,
    optionalWideCardBackgroundTexture = nil,
    isBundle = false,
    bundleChildrenSize = 0,
    licenseTermType = 0,
    licenseTermDuration = 0,
    virtualCurrencies = {},
    isHidden = false,
    hasPendingOrders = false,
    numBundleDetailCards = 0,
    isDynamicallyDiscounted = false,
    shouldShowOriginalPrice = false,
    wideCardBGOverrideProductURL = nil,
    previewBGOverrideProductURL = nil,
    previewSmallBGOverrideProductURL = nil,
    decorQuantity = nil,
    isVCProduct = false,
    containsHousingItem = true,
    creatureDisplayInfoIDs = {},
    spellVisualIDs = {},
    itemModifiedAppearanceIDs = {},
    mainHandItemModifiedAppearanceID = nil,
    offHandItemModifiedAppearanceID = nil,
    decorFileDataID = 91002,
    houseTextureAtlas = nil,
    productType = 2,
  },
}

local __wow_housing_preview_cart_state = {}

local __wow_housing_theme_set_names = {
  [1] = "Sunspire",
}

local function __wow_catalog_shop_emit_seeded_refresh(session_id)
  local frame = rawget(_G, "CatalogShopFrame")
  if type(frame) ~= "table" then
    return
  end

  frame.shoppingSessionUUIDStr = session_id

  local on_event = frame.OnEvent_CatalogShop
  if type(on_event) ~= "function" and type(frame.GetScript) == "function" then
    on_event = frame:GetScript("OnEvent")
  end

  if type(on_event) ~= "function" then
    return
  end

  on_event(frame, "CATALOG_SHOP_DATA_REFRESH", session_id)
  on_event(frame, "CATALOG_SHOP_FETCH_SUCCESS", session_id)

  local category_ids = C_CatalogShop and C_CatalogShop.GetAvailableCategoryIDs and C_CatalogShop.GetAvailableCategoryIDs() or nil
  local initial_category_id = type(category_ids) == "table" and category_ids[1] or nil
  if initial_category_id ~= nil and type(frame.OnCategorySelected) == "function" then
    frame:OnCategorySelected(initial_category_id)
  end
end

local __wow_housing_customize_mode_selected_decor = {
  decorGUID = "Decor-Selection-1001",
  decorID = 1001,
  name = "Sunspire Chair",
  isLocked = false,
  canBeCustomized = true,
  canBeRemoved = true,
  isAllowedOutdoors = true,
  isAllowedIndoors = true,
  isRefundable = false,
  dyeSlots = {},
  dataTagsByID = {},
  size = Enum.HousingCatalogEntrySize and Enum.HousingCatalogEntrySize.Small or 2,
}

local __wow_housing_decor_name_by_id = {
  [1001] = "Sunspire Chair",
  [1002] = "Azure Upholstery",
  [2001] = "Azure Reading Lamp",
  [91001] = "Sunspire Chair",
  [91002] = "Azure Upholstery",
  [91003] = "Azure Reading Lamp",
}

local __wow_housing_decor_icon_by_id = {
  [1001] = 0,
  [1002] = 0,
  [2001] = 0,
  [91001] = 0,
  [91002] = 0,
  [91003] = 0,
}

local __wow_housing_decor_info_by_guid = {
  ["Decor-Selection-1001"] = {
    decorGUID = "Decor-Selection-1001",
    decorID = 1001,
    name = "Sunspire Chair",
    isLocked = false,
    canBeCustomized = true,
    canBeRemoved = true,
    isAllowedOutdoors = true,
    isAllowedIndoors = true,
    isRefundable = false,
    dyeSlots = {},
    dataTagsByID = {},
    size = Enum.HousingCatalogEntrySize and Enum.HousingCatalogEntrySize.Small or 2,
  },
  ["Decor-Selection-2001"] = {
    decorGUID = "Decor-Selection-2001",
    decorID = 2001,
    name = "Azure Reading Lamp",
    isLocked = false,
    canBeCustomized = true,
    canBeRemoved = true,
    isAllowedOutdoors = true,
    isAllowedIndoors = true,
    isRefundable = false,
    dyeSlots = {},
    dataTagsByID = {},
    size = Enum.HousingCatalogEntrySize and Enum.HousingCatalogEntrySize.Small or 2,
  },
}

local __wow_housing_decor_state = {
  preview = false,
  gridVisible = false,
  selectedDecorGUID = "Decor-Selection-2001",
  hoveredDecorGUID = nil,
  placedDecor = {
    __wow_housing_decor_info_by_guid["Decor-Selection-1001"],
    __wow_housing_decor_info_by_guid["Decor-Selection-2001"],
  },
}

local __wow_housing_neighborhood_state = {
  houseInfo = {
    plotID = 27,
    houseName = "Sunspire Retreat",
    ownerName = "Simhero",
    plotCost = 0,
    neighborhoodName = "Dawnmeadow",
    moveOutTime = nil,
    plotReserved = false,
    neighborhoodGUID = "Neighborhood-Dawnmeadow",
    houseGUID = "House-Sunspire-Retreat",
  },
  neighborhoodInfo = {
    neighborhoodType = "Public",
    neighborhoodOwnerType = Enum.NeighborhoodOwnerType and Enum.NeighborhoodOwnerType.None or 0,
    neighborhoodName = "Dawnmeadow",
    neighborhoodGUID = "Neighborhood-Dawnmeadow",
    ownerGUID = "Player-Simhero",
    suggestionReason = nil,
    ownerName = "Simhero",
    locationName = "Dawnmeadow",
  },
}

local __wow_housing_exterior_state = {
  decorHidden = false,
  selectedExteriorType = 1,
  selectedExteriorTypeName = "Sunspire Cottage",
  selectedSize = Enum.HousingFixtureSize and Enum.HousingFixtureSize.Medium or 3,
  selectedFixturePoint = {
    fixtureID = 1,
    pointID = 1,
    name = "Front Door",
  },
  baseFixtureInfo = {
    selectedStyleFixtureID = 101,
    selectedVariantFixtureID = 201,
    styleOptions = {
      { fixtureID = 101, name = "Sunspire Base", colorID = 1 },
      { fixtureID = 102, name = "Moonspire Base", colorID = 2 },
    },
    currentStyleVariantOptions = {
      { fixtureID = 201, name = "Sunspire Base", colorID = 1 },
      { fixtureID = 202, name = "Moonspire Base", colorID = 2 },
    },
  },
  roofFixtureInfo = {
    selectedStyleFixtureID = 301,
    selectedVariantFixtureID = 401,
    styleOptions = {
      { fixtureID = 301, name = "Sunspire Roof", colorID = 1 },
      { fixtureID = 302, name = "Moonspire Roof", colorID = 2 },
    },
    currentStyleVariantOptions = {
      { fixtureID = 401, name = "Sunspire Roof", colorID = 1 },
      { fixtureID = 402, name = "Moonspire Roof", colorID = 2 },
    },
  },
}

local function __wow_housing_clone_table(source)
  local copy = {}
  for key, value in pairs(source or {}) do
    copy[key] = value
  end
  return copy
end

local function __wow_housing_copy_core_fixture_info(info)
  if not info then
    return nil
  end
  local copy = __wow_housing_clone_table(info)
  copy.styleOptions = __wow_housing_clone_table(info.styleOptions)
  copy.currentStyleVariantOptions = __wow_housing_clone_table(info.currentStyleVariantOptions)
  return copy
end

local function __wow_housing_variant_id(entry_id, variant_id)
  return {
    recordID = entry_id,
    entryType = __wow_housing_entry_type,
    variantIdentifier = variant_id,
  }
end

local function __wow_housing_list_contains(list, needle)
  for _, value in ipairs(list or {}) do
    if value == needle then
      return true
    end
  end
  return false
end

local function __wow_housing_copy_variant_info(entry_id, variant_id)
  local variant_info = __wow_housing_seeded_variants[entry_id] and __wow_housing_seeded_variants[entry_id][variant_id]
  if not variant_info then
    return nil
  end
  local info = __wow_housing_clone_table(variant_info)
  info.entryVariantID = __wow_housing_variant_id(entry_id, variant_id)
  info.variantID = variant_id
  return info
end

local function __wow_housing_copy_entry_info(entry_id)
  local entry = __wow_housing_seeded_entries[entry_id]
  if not entry then
    return nil
  end
  local info = __wow_housing_clone_table(entry)
  info.entryID = __wow_housing_variant_id(entry_id, 0)
  return info
end

local function __wow_housing_copy_featured_small_products()
  local infos = {}
  for index, item in ipairs(__wow_housing_seeded_featured_small_products) do
    local copy = __wow_housing_clone_table(item)
    copy.entryVariantID = __wow_housing_variant_id(item.entryID, 1)
    copy.entryID = item.entryID
    infos[index] = copy
  end
  return infos
end

local function __wow_housing_copy_bundle_info(bundle_product_id)
  local bundle = __wow_housing_seeded_bundle_state[bundle_product_id]
  if not bundle then
    return nil
  end
  local info = __wow_housing_clone_table(bundle)
  info.entryIDs = __wow_housing_clone_table(bundle.entryIDs)
  info.decorEntries = {}
  for index, decor_entry in ipairs(bundle.decorEntries) do
    info.decorEntries[index] = __wow_housing_clone_table(decor_entry)
  end
  info.nonDecorProducts = __wow_housing_clone_table(bundle.nonDecorProducts)
  return info
end

local function __wow_housing_copy_market_info(decor_id)
  local market = __wow_housing_seeded_market_state[decor_id]
  if not market then
    return nil
  end
  local info = __wow_housing_clone_table(market)
  info.bundleIDs = __wow_housing_clone_table(market.bundleIDs)
  info.isInCart = market.cartCount > 0
  info.cartCount = market.cartCount
  info.wasViewedInStore = market.wasViewedInStore
  return info
end

local function __wow_housing_copy_product_info(product_id)
  local product = __wow_housing_seeded_product_infos[product_id]
  if not product then
    return nil
  end
  local copy = __wow_housing_clone_table(product)
  copy.subItems = __wow_housing_clone_table(product.subItems)
  copy.virtualCurrencies = __wow_housing_clone_table(product.virtualCurrencies)
  copy.creatureDisplayInfoIDs = __wow_housing_clone_table(product.creatureDisplayInfoIDs)
  copy.spellVisualIDs = __wow_housing_clone_table(product.spellVisualIDs)
  copy.itemModifiedAppearanceIDs = __wow_housing_clone_table(product.itemModifiedAppearanceIDs)
  copy.productIDList = __wow_housing_clone_table(product.productIDList)
  return copy
end

local function __wow_housing_copy_product_display_info(product_id)
  local product = __wow_housing_seeded_product_infos[product_id]
  if not product then
    return nil
  end
  return {
    defaultPreviewModelSceneID = 0,
    defaultCardModelSceneID = 0,
    defaultWideCardModelSceneID = 0,
    itemID = product.itemID or 0,
    overridePreviewModelSceneID = nil,
    overrideCardModelSceneID = nil,
    overrideWideCardModelSceneID = nil,
    creatureDisplayInfoIDs = __wow_housing_clone_table(product.creatureDisplayInfoIDs),
    spellVisualIDs = __wow_housing_clone_table(product.spellVisualIDs),
    mainHandItemModifiedAppearanceID = product.mainHandItemModifiedAppearanceID,
    offHandItemModifiedAppearanceID = product.offHandItemModifiedAppearanceID,
    itemModifiedAppearanceIDs = __wow_housing_clone_table(product.itemModifiedAppearanceIDs),
    iconFileDataID = nil,
    iconTextureKit = nil,
    productType = product.productType,
    itemDescription = product.description,
    hasUnknownLicense = false,
    productPMTURL = nil,
    additionalProductPMTURLs = {},
    otherProductImageAtlasName = nil,
    otherProductGameTitleBaseTag = nil,
    otherProductGameType = nil,
    customLoopingSoundStart = nil,
    customLoopingSoundMiddle = nil,
    customLoopingSoundEnd = nil,
    specialActorID_1 = nil,
    specialActorID_2 = nil,
    specialActorID_3 = nil,
    specialActorID_4 = nil,
    specialActorID_5 = nil,
    gameFlavorID = nil,
    decorFileDataID = product.decorFileDataID,
    quantity = product.decorQuantity,
    houseTextureAtlas = product.houseTextureAtlas,
  }
end

local function __wow_housing_catalog_search_results(searcher_state)
  local results = {}
  local search_text = (searcher_state.searchText or ""):lower()
  local filtered_category = searcher_state.filteredCategoryID
  local filtered_subcategory = searcher_state.filteredSubcategoryID
  local base_variant_only = searcher_state.baseVariantOnly

  for entry_id, entry in pairs(__wow_housing_seeded_entries) do
    local entry_matches_category = (not filtered_category) or filtered_category == __wow_housing_all_category_id or __wow_housing_list_contains(entry.categoryIDs, filtered_category)
    local entry_matches_subcategory = (not filtered_subcategory) or __wow_housing_list_contains(entry.subcategoryIDs, filtered_subcategory)
    local entry_matches_search = search_text == "" or entry.name:lower():find(search_text, 1, true) ~= nil
    if entry_matches_category and entry_matches_subcategory and entry_matches_search then
      local variants = __wow_housing_seeded_variants[entry_id] or {}
      for variant_id, variant in pairs(variants) do
        if (not base_variant_only) or variant_id == 1 then
          local variant_id_table = __wow_housing_variant_id(entry_id, variant_id)
          results[#results + 1] = variant_id_table
        end
      end
    end
  end

  table.sort(results, function(lhs, rhs)
    if lhs.recordID ~= rhs.recordID then
      return lhs.recordID < rhs.recordID
    end
    return (lhs.variantIdentifier or 0) < (rhs.variantIdentifier or 0)
  end)
  return results
end

local function __wow_housing_make_catalog_searcher()
  local state = {
    searchText = nil,
    filteredCategoryID = __wow_housing_all_category_id,
    filteredSubcategoryID = nil,
    sortType = Enum.HousingCatalogSortType and Enum.HousingCatalogSortType.Alphabetical or 0,
    customizableOnly = false,
    allowedIndoors = true,
    allowedOutdoors = true,
    collected = true,
    uncollected = true,
    firstAcquisitionBonusOnly = false,
    storedOnly = false,
    baseVariantOnly = false,
    editorModeContext = nil,
    searchResults = {},
    callback = nil,
    inProgress = false,
    tagStatus = {},
  }

  local function refresh()
    state.searchResults = __wow_housing_catalog_search_results(state)
    state.inProgress = false
    if state.callback then
      state.callback()
    end
  end

  local searcher = {}
  function searcher:SetResultsUpdatedCallback(callback)
    state.callback = callback
  end
  function searcher:SetAutoUpdateOnParamChanges(_enabled) end
  function searcher:SetStoredOnly(enabled) state.storedOnly = not not enabled end
  function searcher:IsStoredOnlyActive() return state.storedOnly end
  function searcher:SetBaseVariantOnly(enabled) state.baseVariantOnly = not not enabled end
  function searcher:IsBaseVariantOnlyActive() return state.baseVariantOnly end
  function searcher:SetEditorModeContext(mode) state.editorModeContext = mode end
  function searcher:GetEditorModeContext() return state.editorModeContext end
  function searcher:SetAllowedIndoors(enabled) state.allowedIndoors = not not enabled end
  function searcher:IsAllowedIndoorsActive() return state.allowedIndoors end
  function searcher:SetAllowedOutdoors(enabled) state.allowedOutdoors = not not enabled end
  function searcher:IsAllowedOutdoorsActive() return state.allowedOutdoors end
  function searcher:SetCollected(enabled) state.collected = not not enabled end
  function searcher:IsCollectedActive() return state.collected end
  function searcher:SetUncollected(enabled) state.uncollected = not not enabled end
  function searcher:IsUncollectedActive() return state.uncollected end
  function searcher:SetCustomizableOnly(enabled) state.customizableOnly = not not enabled end
  function searcher:IsCustomizableOnlyActive() return state.customizableOnly end
  function searcher:SetFirstAcquisitionBonusOnly(enabled) state.firstAcquisitionBonusOnly = not not enabled end
  function searcher:IsFirstAcquisitionBonusOnlyActive() return state.firstAcquisitionBonusOnly end
  function searcher:SetSortType(sortType) state.sortType = sortType end
  function searcher:GetSortType() return state.sortType end
  function searcher:SetFilteredCategoryID(categoryID) state.filteredCategoryID = categoryID or __wow_housing_all_category_id end
  function searcher:GetFilteredCategoryID() return state.filteredCategoryID end
  function searcher:SetFilteredSubcategoryID(subcategoryID) state.filteredSubcategoryID = subcategoryID end
  function searcher:GetFilteredSubcategoryID() return state.filteredSubcategoryID end
  function searcher:SetSearchText(searchText) state.searchText = searchText end
  function searcher:GetSearchText() return state.searchText end
  function searcher:SetFilterTagStatus(groupID, tagID, enabled)
    state.tagStatus[groupID] = state.tagStatus[groupID] or {}
    state.tagStatus[groupID][tagID] = not not enabled
  end
  function searcher:GetFilterTagStatus(groupID, tagID)
    return state.tagStatus[groupID] and state.tagStatus[groupID][tagID] or false
  end
  function searcher:SetAllInFilterTagGroup(groupID, enabled)
    state.tagStatus[groupID] = state.tagStatus[groupID] or {}
    state.tagStatus[groupID].__all = not not enabled
  end
  function searcher:IsSearchInProgress() return state.inProgress end
  function searcher:GetSearchCount() return #state.searchResults end
  function searcher:GetNumSearchItems() return #state.searchResults end
  function searcher:GetAllSearchItems() return state.searchResults end
  function searcher:GetCatalogSearchResults() return state.searchResults end
  function searcher:RunSearch()
    refresh()
  end

  refresh()
  return searcher
end

C_CatalogShop = __wow_merge_namespace(C_CatalogShop, {
  IsShop2Enabled = function() return false end,
  HasNewProducts = function() return false end,
  GetAvailableCategoryIDs = function() return { __wow_housing_all_category_id, 101 } end,
  GetProductIDsForCategory = function(categoryID)
    if categoryID == __wow_housing_all_category_id then
      return { 2003, 20031, 20032 }
    elseif categoryID == 101 then
      return { 2003 }
    elseif categoryID == 102 then
      return { 20031, 20032 }
    end
    return {}
  end,
  GetProductIDsForCategorySection = function(categoryID, sectionID)
    if sectionID ~= 1 then
      return {}
    end
    return C_CatalogShop.GetProductIDsForCategory(categoryID)
  end,
  GetCategoryInfo = function(categoryID)
    if categoryID == __wow_housing_all_category_id then
      return { ID = categoryID, displayName = "All", iconTexture = "Interface\\Icons\\INV_Misc_QuestionMark", linkTag = "all", isDisabled = false, showPersistentRefundButton = false }
    elseif categoryID == 101 then
      return { ID = categoryID, displayName = "Featured", iconTexture = "Interface\\Icons\\INV_Misc_QuestionMark", linkTag = "featured", isDisabled = false, showPersistentRefundButton = false }
    elseif categoryID == 102 then
      return { ID = categoryID, displayName = "Decor", iconTexture = "Interface\\Icons\\INV_Misc_QuestionMark", linkTag = "housing", isDisabled = false, showPersistentRefundButton = false }
    end
    return nil
  end,
  GetProductSortOrder = function(_categoryID, _sectionID, productID)
    local product = __wow_housing_seeded_product_infos[productID]
    if product then
      return product.sortOrder or productID
    end
    return productID
  end,
  GetSectionIDsForCategory = function(categoryID)
    if categoryID == __wow_housing_all_category_id then
      return { 1 }
    end
    return { 1 }
  end,
  GetCategorySectionInfo = function(categoryID, sectionID)
    return {
      ID = sectionID,
      displayName = "Featured",
      parentCatalogShopCategoryInfoID = categoryID,
      cardType = nil,
      scrollGridSize = 1,
      shouldShowRecommendationOptOutDisclaimer = false,
    }
  end,
  GetFailureInfo = function() return nil, nil end,
  RefreshVirtualCurrencyBalance = __wow_noop,
  GetVirtualCurrencyBalance = function() return 0 end,
  OpenCatalogShopInteractionFromShop = function()
    local session_id = "seeded-catalog-shop-session"
    __wow_catalog_shop_emit_seeded_refresh(session_id)
    return session_id
  end,
  OpenCatalogShopInteractionFromHouse = function()
    local session_id = "seeded-catalog-house-session"
    __wow_catalog_shop_emit_seeded_refresh(session_id)
    return session_id
  end,
  CloseCatalogShopInteraction = __wow_noop,
  GetFirstCategoryByProductID = function(productID)
    if productID == 2003 or productID == 20031 or productID == 20032 then
      return C_CatalogShop.GetCategoryInfo(101)
    end
    return nil
  end,
  ShouldShowHousingWarning = function() return false end,
  GetProductInfo = function(productID) return __wow_housing_copy_product_info(productID) end,
  GetCatalogShopProductDisplayInfo = function(productID) return __wow_housing_copy_product_display_info(productID) end,
  GetRefundableDecors = function()
    return {}, 0
  end,
  GetProductIDsForBundle = function(productID)
    local product = __wow_housing_seeded_product_infos[productID]
    if not product or not product.isBundle then
      return {}
    end
    local children = {}
    for index, child_id in ipairs(product.productIDList or {}) do
      children[index] = { childProductID = child_id, displayOrder = index, quantityInBundle = 1 }
    end
    return children
  end,
  GetSpellVisualInfoForMount = function() return nil end,
  PurchaseProduct = __wow_noop,
  ConfirmHousingPurchase = __wow_noop,
  ProductDisplayedTelemetry = __wow_noop,
  OnLegalDisclaimerClicked = __wow_noop,
  FindBestCurrencyProductForNeededAmount = function() return nil end,
  IsProductIncludedInAnyBundle = function(productID)
    return productID == 20031 or productID == 20032
  end,
  GetProductAvailabilityTimeRemainingSecs = function() return 1 end,
  OnLegalPersonalizedOptOutClicked = __wow_noop,
  ProductSelectedTelemetry = __wow_noop,
})

-- Housing service flag stays in Rust, but the seeded housing/catalog UI
-- namespaces need enough state for the full Blizzard UI to load.
-- C_Housing.IsHousingServiceEnabled is registered from Rust
-- (src/lua_api/globals/housing.rs), backed by SimState::housing_service_enabled.
-- Admin: A_Admin.SetHousingServiceEnabled(b?).
-- Merge stub-namespace fallback so other unimplemented C_Housing members
-- resolve to the no-op metamethod.
C_Housing = __wow_merge_namespace(C_Housing, {})
C_HousingBasicMode = __wow_merge_namespace(C_HousingBasicMode, {
  IsPlacingNewDecor = function() return false end,
  IsDecorSelected = function() return C_HousingDecor.IsDecorSelected() end,
  GetSelectedDecorInfo = function() return C_HousingDecor.GetSelectedDecorInfo() end,
  IsHouseExteriorSelected = function() return false end,
  CommitDecorMovement = __wow_noop,
  CommitHouseExteriorPosition = __wow_noop,
  CancelActiveEditing = __wow_noop,
  FinishPlacingNewDecor = __wow_noop,
  StartPlacingNewDecor = __wow_noop,
  StartPlacingPreviewDecor = __wow_noop,
  IsGridSnapEnabled = function() return false end,
  SetGridSnapEnabled = __wow_noop,
  IsGridVisible = function() return false end,
  SetGridVisible = __wow_noop,
})
C_HousingExpertMode = __wow_merge_namespace(C_HousingExpertMode, {
  IsDecorSelected = function() return C_HousingDecor.IsDecorSelected() end,
  IsHouseExteriorSelected = function() return false end,
  GetSelectedDecorInfo = function() return C_HousingDecor.GetSelectedDecorInfo() end,
  CancelActiveEditing = __wow_noop,
  GetPrecisionSubmode = function() return 0 end,
  SetPrecisionSubmode = __wow_noop,
  GetPrecisionSubmodeRestriction = function() return 0 end,
  ResetPrecisionChanges = __wow_noop,
})
C_HousingLayout = __wow_merge_namespace(C_HousingLayout, {
  GetSpentPlacementBudget = function() return 0 end,
  GetRoomPlacementBudget = function() return 10 end,
  HasRoomPlacementBudget = function() return false end,
  HasAnySelections = function() return false end,
  GetSelectedFloorplan = function() return nil end,
  SelectFloorplan = __wow_noop,
  DeselectFloorplan = __wow_noop,
  GetViewedFloor = function() return 0 end,
  SetViewedFloor = __wow_noop,
  GetNumFloors = function() return 1 end,
  AnyRoomsOnFloor = function() return false end,
  IsDraggingRoom = function() return false end,
  StartDrag = __wow_noop,
  StopDrag = __wow_noop,
  StopDraggingRoom = __wow_noop,
  CancelActiveLayoutEditing = __wow_noop,
  IsBaseRoom = function() return false end,
  GetSelectedDoor = function() return nil end,
  HasValidConnection = function() return false end,
  RemoveRoom = __wow_noop,
  RotateRoom = __wow_noop,
})
C_HousingDecor = __wow_merge_namespace(C_HousingDecor, {
  CancelActiveEditing = __wow_noop,
  CommitDecorMovement = __wow_noop,
  EnterPreviewState = function() __wow_housing_decor_state.preview = true end,
  ExitPreviewState = function() __wow_housing_decor_state.preview = false end,
  GetAllPlacedDecor = function()
    local placed = {}
    for _, info in ipairs(__wow_housing_decor_state.placedDecor) do
      placed[#placed + 1] = __wow_housing_clone_table(info)
    end
    return placed
  end,
  GetDecorHyperlink = function(decorID)
    local name = __wow_housing_decor_name_by_id[decorID]
    if not name then
      return nil
    end
    return string.format("|cff66bbff|Hhousingdecor:%d|h[%s]|h|r", decorID, name)
  end,
  GetDecorIcon = function(decorID)
    return __wow_housing_decor_icon_by_id[decorID] or 0
  end,
  GetDecorInstanceInfoForGUID = function(decorGUID)
    return __wow_housing_decor_info_by_guid[decorGUID] and __wow_housing_clone_table(__wow_housing_decor_info_by_guid[decorGUID]) or nil
  end,
  GetDecorName = function(decorID)
    return __wow_housing_decor_name_by_id[decorID] or ""
  end,
  GetHoveredDecorInfo = function()
    local decorGUID = __wow_housing_decor_state.hoveredDecorGUID
    return decorGUID and __wow_housing_decor_info_by_guid[decorGUID] and __wow_housing_clone_table(__wow_housing_decor_info_by_guid[decorGUID]) or nil
  end,
  GetHoveredDecorDebugInfo = function()
    return C_HousingDecor.GetHoveredDecorInfo()
  end,
  GetMaxPlacementBudget = function() return 100 end,
  GetNumDecorPlaced = function() return #__wow_housing_decor_state.placedDecor end,
  GetNumPreviewDecor = function() return __wow_housing_decor_state.preview and 1 or 0 end,
  GetPreviewDyesOnSelectedDecor = function() return {} end,
  GetRecentlyUsedDyes = function() return {} end,
  GetSelectedDecorInfo = function()
    local decorGUID = __wow_housing_decor_state.selectedDecorGUID
    return decorGUID and __wow_housing_decor_info_by_guid[decorGUID] and __wow_housing_clone_table(__wow_housing_decor_info_by_guid[decorGUID]) or nil
  end,
  GetSpentPlacementBudget = function() return 20 end,
  HasMaxPlacementBudget = function() return false end,
  IsDecorSelected = function() return __wow_housing_decor_state.selectedDecorGUID ~= nil end,
  IsGridVisible = function() return __wow_housing_decor_state.gridVisible end,
  IsHouseExteriorHovered = function() return false end,
  IsHoveringDecor = function() return __wow_housing_decor_state.hoveredDecorGUID ~= nil end,
  IsModeDisabledForPreviewState = function() return false end,
  IsPreviewState = function() return __wow_housing_decor_state.preview end,
  RemovePlacedDecorEntry = __wow_noop,
  RemoveSelectedDecor = __wow_noop,
  SetGridVisible = function(visible) __wow_housing_decor_state.gridVisible = not not visible end,
  SetPlacedDecorEntryHovered = function(decorGUID, hovered)
    __wow_housing_decor_state.hoveredDecorGUID = hovered and decorGUID or nil
  end,
  SetPlacedDecorEntrySelected = function(decorGUID, selected)
    __wow_housing_decor_state.selectedDecorGUID = selected and decorGUID or nil
  end,
})
C_HousingCustomizeMode = __wow_merge_namespace(C_HousingCustomizeMode, {
  CancelActiveEditing = __wow_noop,
  GetHoveredRoomComponentInfo = function() return nil end,
  GetRecentlyUsedThemeSets = function() return { 1 } end,
  GetRecentlyUsedWallpapers = function() return { 1 } end,
  GetSelectedDecorInfo = function()
    return __wow_housing_clone_table(__wow_housing_customize_mode_selected_decor)
  end,
  GetSelectedRoomComponentInfo = function() return nil end,
  GetThemeSetInfo = function(themeSetID)
    return __wow_housing_theme_set_names[themeSetID]
  end,
  GetWallpapersForRoomComponentType = function(_type)
    return { { roomComponentTextureRecID = 1, name = "Sunspire Plaster" } }
  end,
  IsDecorSelected = function() return true end,
  IsHoveringRoomComponent = function() return false end,
  IsRoomComponentSelected = function() return false end,
  ApplyThemeToSelectedRoomComponent = __wow_noop,
  ApplyWallpaperToSelectedRoomComponent = __wow_noop,
  ApplyWallpaperToAllWalls = __wow_noop,
  ApplyThemeToRoom = __wow_noop,
  ClearTargetRoomComponent = __wow_noop,
  CommitDyesForSelectedDecor = function() return true end,
  GetNumDyesToSpendOnSelectedDecor = function() return 0 end,
  GetNumDyesToRemoveOnSelectedDecor = function() return 0 end,
  GetPreviewDyesOnSelectedDecor = function() return {} end,
  GetRecentlyUsedDyes = function() return {} end,
  RoomComponentSupportsVariant = function() return false end,
  SetGridSnapEnabled = __wow_noop,
  SetGridVisible = __wow_noop,
  SetRoomComponentCeilingType = __wow_noop,
  SetRoomComponentDoorType = __wow_noop,
  SetFreePlaceEnabled = __wow_noop,
})
C_HousingNeighborhood = __wow_merge_namespace(C_HousingNeighborhood, {
  CanReturnAfterVisitingHouse = function() return false end,
  CancelInviteToNeighborhood = __wow_noop,
  DemoteToResident = __wow_noop,
  GetCornerstoneHouseInfo = function()
    return __wow_housing_neighborhood_state.houseInfo and __wow_housing_clone_table(__wow_housing_neighborhood_state.houseInfo) or nil
  end,
  GetCornerstoneNeighborhoodInfo = function()
    return __wow_housing_neighborhood_state.neighborhoodInfo and __wow_housing_clone_table(__wow_housing_neighborhood_state.neighborhoodInfo) or nil
  end,
  GetCornerstonePurchaseMode = function() return 0 end,
  GetCurrentNeighborhoodTextureSuffix = function() return "dawnmeadow" end,
  GetDiscountedMovePrice = function() return 0 end,
  GetMoveCooldownTime = function() return 0 end,
  GetNeighborhoodMapData = function() return {} end,
  GetNeighborhoodName = function() return "Dawnmeadow" end,
  GetNeighborhoodPlotName = function(plotID) return string.format("Plot %s", tostring(plotID)) end,
  GetPreviousHouseIdentifier = function() return "Sunspire Retreat" end,
  HasPermissionToPurchase = function() return false end,
  InvitePlayerToNeighborhood = __wow_noop,
  IsNeighborhoodManager = function() return false end,
  IsNeighborhoodOwner = function() return true end,
  IsPlayerInOtherPlayersPlot = function() return false end,
  IsPlotAvailableForPurchase = function() return false end,
  IsPlotOwnedByPlayer = function() return true end,
  OnBulletinBoardClosed = __wow_noop,
  OnCornerstoneClosed = function()
    __wow_housing_neighborhood_state.houseInfo = nil
    __wow_housing_neighborhood_state.neighborhoodInfo = nil
  end,
  PromoteToManager = __wow_noop,
  RequestNeighborhoodInfo = __wow_noop,
  RequestNeighborhoodRoster = __wow_noop,
  RequestPendingNeighborhoodInvites = __wow_noop,
  SetDesiredNeighborhoodType = __wow_noop,
  TransferNeighborhoodOwnership = __wow_noop,
  TryEvictPlayer = __wow_noop,
  TryMoveHouse = __wow_noop,
  TryPurchasePlot = __wow_noop,
})
C_HouseExterior = __wow_merge_namespace(C_HouseExterior, {
  CancelActiveExteriorEditing = __wow_noop,
  GetCoreFixtureOptionsInfo = function(coreFixtureType)
    if coreFixtureType == Enum.HousingFixtureType.Base then
      return __wow_housing_copy_core_fixture_info(__wow_housing_exterior_state.baseFixtureInfo)
    elseif coreFixtureType == Enum.HousingFixtureType.Roof then
      return __wow_housing_copy_core_fixture_info(__wow_housing_exterior_state.roofFixtureInfo)
    end
    return nil
  end,
  GetCurrentHouseExteriorSize = function() return __wow_housing_exterior_state.selectedSize end,
  GetCurrentHouseExteriorType = function() return __wow_housing_exterior_state.selectedExteriorType, __wow_housing_exterior_state.selectedExteriorTypeName end,
  GetHouseExteriorSizeOptions = function()
    return {
      selectedSize = __wow_housing_exterior_state.selectedSize,
      options = {
        { size = Enum.HousingFixtureSize and Enum.HousingFixtureSize.Medium or 3, name = "Medium" },
        { size = Enum.HousingFixtureSize and Enum.HousingFixtureSize.Large or 4, name = "Large" },
      },
    }
  end,
  GetHouseExteriorTypeOptions = function()
    return {
      selectedExteriorType = __wow_housing_exterior_state.selectedExteriorType,
      options = {
        { houseExteriorTypeID = 1, name = "Sunspire Cottage" },
        { houseExteriorTypeID = 2, name = "Sunspire Manor" },
      },
    }
  end,
  GetSelectedFixturePointInfo = function()
    return __wow_housing_exterior_state.selectedFixturePoint and __wow_housing_clone_table(__wow_housing_exterior_state.selectedFixturePoint) or nil
  end,
  GetHoveredFixtureDebugInfo = function() return nil end,
  HasHoveredFixture = function() return false end,
  HasSelectedFixturePoint = function() return true end,
  IsAnyDecorAttachedToCoreFixture = function(coreFixtureType)
    return coreFixtureType == Enum.HousingFixtureType.Base
  end,
  IsAnyDecorAttachedToDoor = function() return true end,
  IsAnyDecorAttachedToHouseExterior = function() return true end,
  IsAnyDecorAttachedToSelectedFixturePoint = function() return true end,
  IsExteriorDecorHidden = function() return __wow_housing_exterior_state.decorHidden end,
  RemoveFixtureFromSelectedPoint = __wow_noop,
  SelectCoreFixtureOption = __wow_noop,
  SelectFixtureOption = __wow_noop,
  SetExteriorDecorHidden = function(decorHidden)
    __wow_housing_exterior_state.decorHidden = not not decorHidden
  end,
  SetHouseExteriorSize = __wow_noop,
  SetHouseExteriorType = __wow_noop,
})
C_HousingCatalog = __wow_merge_namespace(C_HousingCatalog, {
  CreateCatalogSearcher = function()
    return __wow_housing_make_catalog_searcher()
  end,
  DeletePreviewCartDecor = __wow_noop,
  DestroyEntry = __wow_noop,
  GetAllFilterTagGroups = function() return {} end,
  GetAllVariantInfosForEntry = function(entryID)
    local entry_id = type(entryID) == "table" and entryID.recordID or entryID
    local variants = __wow_housing_seeded_variants[entry_id]
    if not variants then
      return {}
    end
    local results = {}
    for variantID, _variant in pairs(variants) do
      results[#results + 1] = __wow_housing_copy_variant_info(entry_id, variantID)
    end
    table.sort(results, function(lhs, rhs)
      return (lhs.variantID or 0) < (rhs.variantID or 0)
    end)
    return results
  end,
  GetBundleInfo = function(bundleCatalogShopProductID)
    return __wow_housing_copy_bundle_info(bundleCatalogShopProductID)
  end,
  GetCartSizeLimit = function() return 20 end,
  GetCatalogCategoryInfo = function(categoryID)
    if categoryID == __wow_housing_all_category_id then
      return { ID = categoryID, orderIndex = 0, name = "All", icon = nil, subcategoryIDs = { 1001, 1002 }, anyStoredEntries = true }
    elseif categoryID == 101 then
      return { ID = categoryID, orderIndex = 1, name = "Featured", icon = nil, subcategoryIDs = { 1001 }, anyStoredEntries = true }
    elseif categoryID == 102 then
      return { ID = categoryID, orderIndex = 2, name = "Decor", icon = nil, subcategoryIDs = { 1001, 1002 }, anyStoredEntries = true }
    end
    return nil
  end,
  GetCatalogEntryInfo = function(entryVariantID)
    local entry_id = type(entryVariantID) == "table" and entryVariantID.recordID or entryVariantID
    return __wow_housing_copy_entry_info(entry_id)
  end,
  GetCatalogEntryInfoByItem = function(itemInfo)
    local item_id = type(itemInfo) == "table" and (itemInfo.itemID or itemInfo.id) or itemInfo
    return __wow_housing_copy_entry_info(item_id)
  end,
  GetCatalogEntryInfoByRecordID = function(entryType, recordID)
    if entryType ~= __wow_housing_entry_type then
      return nil
    end
    return __wow_housing_copy_entry_info(recordID)
  end,
  GetCatalogEntryRefundTimeStampByRecordID = function() return nil end,
  GetCatalogEntryVariantInfo = function(entryID, variantID)
    local entry_id = type(entryID) == "table" and entryID.recordID or entryID
    local variant_id = type(entryID) == "table" and entryID.variantIdentifier or variantID or 1
    return __wow_housing_copy_variant_info(entry_id, variant_id)
  end,
  GetCatalogSubcategoryInfo = function(subcategoryID)
    if subcategoryID == 1001 then
      return { ID = 1001, orderIndex = 1, parentCategoryID = 102, name = "Seating", icon = nil, anyStoredEntries = true }
    elseif subcategoryID == 1002 then
      return { ID = 1002, orderIndex = 2, parentCategoryID = 102, name = "Lighting", icon = nil, anyStoredEntries = true }
    end
    return nil
  end,
  GetDecorMaxOwnedCount = function() return 99 end,
  GetDecorTotalOwnedCount = function() return 2, 0 end,
  GetDestroyableInstanceCount = function(entryVariantID)
    local entry_id = type(entryVariantID) == "table" and entryVariantID.recordID or entryVariantID
    local variant_id = type(entryVariantID) == "table" and entryVariantID.variantIdentifier or 1
    local variant = __wow_housing_seeded_variants[entry_id] and __wow_housing_seeded_variants[entry_id][variant_id]
    return variant and variant.numStored or 0
  end,
  GetFeaturedBundles = function()
    local featured = { __wow_housing_copy_bundle_info(5001) }
    return featured
  end,
  GetFeaturedSmallProducts = function()
    return __wow_housing_copy_featured_small_products()
  end,
  GetMarketInfoForDecor = function(decorID)
    return __wow_housing_copy_market_info(decorID)
  end,
  HasFeaturedEntries = function() return true end,
  HousingMarketActionAddToCart = function(productID)
    local market = __wow_housing_seeded_market_state[productID]
    if not market then
      return false
    end
    market.cartCount = market.cartCount + 1
    return true
  end,
  HousingMarketActionClearCart = function()
    for _decorID, market in pairs(__wow_housing_seeded_market_state) do
      market.cartCount = 0
    end
  end,
  HousingMarketActionRemoveFromCart = function(productID)
    local market = __wow_housing_seeded_market_state[productID]
    if not market then
      return false
    end
    market.cartCount = 0
    return true
  end,
  HousingMarketActionViewBundle = function(bundleProductID)
    local bundle = __wow_housing_seeded_bundle_state[bundleProductID]
    if not bundle then
      return false
    end
    bundle.wasViewed = true
    return true
  end,
  HousingMarketActionViewInStore = function(productID)
    local market = __wow_housing_seeded_market_state[productID]
    if not market then
      return false
    end
    market.wasViewedInStore = true
    return true
  end,
  IsPreviewCartItemShown = function(decorGUID)
    return __wow_housing_preview_cart_state[decorGUID] or false
  end,
  PromotePreviewDecor = function(decorID, previewDecorGUID)
    __wow_housing_preview_cart_state[previewDecorGUID] = true
    return true
  end,
  RequestHousingMarketInfoRefresh = __wow_noop,
  RequestHousingMarketRefundInfo = __wow_noop,
  SearchCatalogCategories = function(_searchParams)
    return { __wow_housing_all_category_id, 101, 102 }
  end,
  SearchCatalogSubcategories = function(_searchParams)
    return { 1001, 1002 }
  end,
  SetPreviewCartItemShown = function(decorGUID, shown)
    __wow_housing_preview_cart_state[decorGUID] = not not shown
  end,
  IsProductIncludedInAnyBundle = function(productID)
    return productID == 20031 or productID == 20032
  end,
  GetProductAvailabilityTimeRemainingSecs = function() return 1 end,
})
