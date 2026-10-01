-- Catalog lifecycle retained from the temporary owner; data comes only from Rust.
local publish_items = ...

return function()
  local __wow_housing_all_category_id = Constants.HousingCatalogConsts.HOUSING_CATALOG_ALL_CATEGORY_ID
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
    searchItems = {},
    searchResults = {},
    callback = nil,
    inProgress = false,
    tagStatus = {},
  }

  local function refresh()
    -- Filters are retained parameters, not modeled matching predicates.
    state.searchItems = publish_items()
    state.searchResults = publish_items()
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
  function searcher:GetAllSearchItems() return state.searchItems end
  function searcher:GetCatalogSearchResults() return state.searchResults end
  function searcher:RunSearch()
    refresh()
  end

  refresh()
  return searcher
end
