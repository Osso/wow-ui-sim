//! Temporary EditMode cache/default layout API surface.
//!
//! `C_EditMode` cache parsing and default account-layout state are currently
//! Lua-owned compatibility state used by SavedVariables loading and Blizzard
//! EditMode startup. Keep that ownership explicit instead of leaving it in the
//! generic runtime bootstrap.

const EDIT_MODE_CACHE_DEFAULTS_LUA: &str = r##"
if C_EditMode == nil then
  C_EditMode = __wow_namespace()
end
if rawget(C_EditMode, "GetAccountSettings") == nil then
  local function __wow_copy_edit_mode_value(value)
    if type(value) ~= "table" then
      return value
    end

    local copy = {}
    for key, child in pairs(value) do
      copy[__wow_copy_edit_mode_value(key)] = __wow_copy_edit_mode_value(child)
    end
    return copy
  end

  local function __wow_default_edit_mode_account_setting(setting)
    if setting == Enum.EditModeAccountSetting.ShowGrid then
      return 0
    elseif setting == Enum.EditModeAccountSetting.GridSpacing then
      return Constants.EditModeConsts.EditModeDefaultGridSpacing or 100
    elseif setting == Enum.EditModeAccountSetting.SettingsExpanded then
      return 0
    elseif setting == Enum.EditModeAccountSetting.EnableAdvancedOptions then
      return 0
    end
    return 1
  end

  local __wow_edit_mode_layout_state = {
    layouts = {},
    activeLayout = 1,
  }
  local __wow_edit_mode_account_setting_state = nil

  local function __wow_build_default_edit_mode_account_settings()
    local settings = {}
    for _, setting in pairs(Enum.EditModeAccountSetting or {}) do
      if type(setting) == "number" then
        table.insert(settings, {
          setting = setting,
          value = __wow_default_edit_mode_account_setting(setting),
        })
      end
    end
    table.sort(settings, function(a, b) return a.setting < b.setting end)
    return settings
  end

  local function __wow_merge_edit_mode_account_settings(accountSettings)
    local merged = __wow_build_default_edit_mode_account_settings()
    local bySetting = {}
    for _, settingInfo in ipairs(merged) do
      bySetting[settingInfo.setting] = settingInfo
    end

    for _, settingInfo in ipairs(accountSettings or {}) do
      local existing = bySetting[settingInfo.setting]
      if existing then
        existing.value = settingInfo.value
      else
        table.insert(merged, {
          setting = settingInfo.setting,
          value = settingInfo.value,
        })
      end
    end

    table.sort(merged, function(a, b) return a.setting < b.setting end)
    return merged
  end

  local __wow_edit_mode_frame_points = {
    [0] = "TOPLEFT",
    [1] = "TOP",
    [2] = "TOPRIGHT",
    [3] = "LEFT",
    [4] = "CENTER",
    [5] = "RIGHT",
    [6] = "BOTTOMLEFT",
    [7] = "BOTTOM",
    [8] = "BOTTOMRIGHT",
  }

  local function __wow_edit_mode_tokens(text)
    local tokens = {}
    if type(text) ~= "string" then
      return tokens
    end
    text = string.gsub(text, "%z", "")
    for token in string.gmatch(text, "%S+") do
      table.insert(tokens, token)
    end
    return tokens
  end

  local function __wow_edit_mode_read(text, cursor)
    local first, last = string.find(text, "%S+", cursor)
    if first then
      return string.sub(text, first, last), last + 1
    end
    return nil, #text + 1
  end

  local function __wow_edit_mode_read_number(text, cursor)
    local token
    token, cursor = __wow_edit_mode_read(text, cursor)
    return tonumber(token), cursor
  end

  local function __wow_edit_mode_read_count(text, cursor)
    local count
    count, cursor = __wow_edit_mode_read_number(text, cursor)
    if count and count >= 0 and count % 1 == 0 then
      return count, cursor
    end
  end

  local function __wow_edit_mode_decode_settings(encoded)
    local settings = {}
    if type(encoded) ~= "string" then
      return settings
    end
    local lastSetting = nil
    local lastInfo = nil
    local placeValue = 1
    for i = 1, string.len(encoded), 2 do
      local settingByte = string.byte(encoded, i)
      local valueByte = string.byte(encoded, i + 1)
      if settingByte and valueByte then
        local setting = settingByte - 35
        local valueChunk = valueByte - 35
        if setting == lastSetting and lastInfo then
          placeValue = placeValue * 90
          lastInfo.value = lastInfo.value + (valueChunk * placeValue)
        else
          lastInfo = {
            setting = setting,
            value = valueChunk,
          }
          table.insert(settings, lastInfo)
          lastSetting = setting
          placeValue = 1
        end
      end
    end
    return settings
  end

  local function __wow_edit_mode_parse_system(text, cursor)
    local system, systemIndex, isInDefaultPosition, point, relativePoint
    local relativeTo, offsetX, offsetY, secondaryAnchor, settingsText
    system, cursor = __wow_edit_mode_read_number(text, cursor)
    systemIndex, cursor = __wow_edit_mode_read_number(text, cursor)
    isInDefaultPosition, cursor = __wow_edit_mode_read_number(text, cursor)
    point, cursor = __wow_edit_mode_read_number(text, cursor)
    relativePoint, cursor = __wow_edit_mode_read_number(text, cursor)
    relativeTo, cursor = __wow_edit_mode_read(text, cursor)
    offsetX, cursor = __wow_edit_mode_read_number(text, cursor)
    offsetY, cursor = __wow_edit_mode_read_number(text, cursor)
    secondaryAnchor, cursor = __wow_edit_mode_read(text, cursor)
    settingsText, cursor = __wow_edit_mode_read(text, cursor)
    if not (system and systemIndex and isInDefaultPosition and point and relativePoint
        and relativeTo and not tonumber(relativeTo) and offsetX and offsetY
        and secondaryAnchor and settingsText) then
      return nil, cursor
    end
    if systemIndex >= 0 then
      systemIndex = systemIndex + 1
    end

    local statusTrackingSystem = Enum and Enum.EditModeSystem
      and Enum.EditModeSystem.StatusTrackingBar
    local hidden = system == statusTrackingSystem and isInDefaultPosition == 0

    return {
      system = system,
      systemIndex = systemIndex,
      hidden = hidden,
      isInDefaultPosition = isInDefaultPosition ~= 0,
      anchorInfo = {
        point = __wow_edit_mode_frame_points[point] or "CENTER",
        relativeTo = relativeTo,
        relativePoint = __wow_edit_mode_frame_points[relativePoint] or "CENTER",
        offsetX = offsetX,
        offsetY = offsetY,
      },
      settings = __wow_edit_mode_decode_settings(settingsText),
    }, cursor
  end

  local function __wow_edit_mode_read_layout_name(text, cursor, byteCount)
    local afterName = cursor + byteCount + 1
    if byteCount > 0 and afterName <= #text and string.find(text, "^%s", cursor)
        and string.find(text, "^%s", afterName) then
      return string.sub(text, cursor + 1, afterName - 1), afterName
    end
  end

  local function __wow_edit_mode_read_layouts(text, cursor, layoutCount, hasMetadata)
    local layouts = {}
    for _ = 1, layoutCount do
      local layoutStart = cursor
      local prefix, afterPrefix = __wow_edit_mode_read(text, cursor)
      if not prefix then
        return layouts, layoutStart
      end
      local prefixNumber = tonumber(prefix)
      if not prefixNumber or prefixNumber < 0 or prefixNumber % 1 ~= 0 then
        return layouts, layoutStart
      end
      local layoutName, afterName, layoutIndex
      if hasMetadata then
        -- Forever stores a numeric field between the byte-counted name and system count.
        layoutName, afterName = __wow_edit_mode_read_layout_name(text, afterPrefix, prefixNumber)
        if not layoutName then return layouts, layoutStart end
        local metadata
        metadata, afterName = __wow_edit_mode_read_count(text, afterName)
        if not metadata then return layouts, layoutStart end
      else
        -- Older inputs also use this field as an explicit index with a single-token name.
        layoutIndex = prefixNumber
        layoutName, afterName = __wow_edit_mode_read(text, afterPrefix)
      end
      local systemCount
      systemCount, cursor = __wow_edit_mode_read_count(text, afterName)
      if not (layoutName and systemCount) then return layouts, layoutStart end
      local systems = {}
      for systemIndex = 1, systemCount do
        local system
        system, cursor = __wow_edit_mode_parse_system(text, cursor)
        if not system then return layouts, layoutStart end
        -- Blizzard looks up singleton systems such as ChatFrame with no index.
        if hasMetadata and system.systemIndex == -1 then
          system.systemIndex = nil
        end
        systems[systemIndex] = system
      end
      table.insert(layouts, {
        layoutIndex = layoutIndex,
        layoutName = layoutName,
        layoutType = Enum.EditModeLayoutType.Account,
        systems = systems,
      })
    end

    return layouts, cursor
  end

  local function __wow_edit_mode_parse_account_cache(text)
    if type(text) ~= "string" then return {}, {}, false end
    text = string.gsub(text, "%z", "")
    local layoutCount, cursor = __wow_edit_mode_read_count(text, 1)
    if not layoutCount then return {}, {}, false end
    local accountSettingCount
    accountSettingCount, cursor = __wow_edit_mode_read_count(text, cursor)
    if not accountSettingCount then return {}, {}, layoutCount > 0 end

    local accountSettings = {}
    for setting = 0, accountSettingCount - 1 do
      local value
      value, cursor = __wow_edit_mode_read_number(text, cursor)
      if not value then return {}, accountSettings, layoutCount > 0 end
      table.insert(accountSettings, { setting = setting, value = value })
    end

    local layouts, layoutsEnd = __wow_edit_mode_read_layouts(text, cursor, layoutCount, true)
    if __wow_edit_mode_read(text, layoutsEnd) then
      local legacy, legacyEnd = __wow_edit_mode_read_layouts(text, cursor, layoutCount, false)
      -- A truncated record can leave both formats with the same number of
      -- layouts. Prefer the one that consumed more of the complete records.
      if legacyEnd >= layoutsEnd then
        layouts = legacy
      end
    end
    return layouts, accountSettings, #layouts < layoutCount
  end

  local function __wow_edit_mode_active_layout_from_character_cache(text, activeSpecIndex)
    local tokens = __wow_edit_mode_tokens(text)
    local active = tonumber(tokens[activeSpecIndex or 1])
    if active and active > 0 then
      return active
    end
    for _, token in ipairs(tokens) do
      active = tonumber(token)
      if active and active > 0 then
        return active
      end
    end
    return nil
  end

  local function __wow_edit_mode_active_layout_from_override(layouts, preferredLayout)
    if type(preferredLayout) ~= "string" or preferredLayout == "" then
      return nil
    end

    local presetCount = Enum.EditModePresetLayoutsMeta.NumValues
    local preferredIndex = tonumber(preferredLayout)
    if preferredIndex and preferredIndex > 0 then
      -- Preserve explicit custom ordinal/ID selection; character-cache indices
      -- already address the combined preset-plus-custom list and never enter here.
      if layouts[preferredIndex] then
        return presetCount + preferredIndex
      end
      for index, layout in ipairs(layouts) do
        if layout.layoutIndex == preferredIndex then
          return presetCount + index
        end
      end
      return preferredIndex
    end

    for index, layout in ipairs(layouts or {}) do
      if layout.layoutName == preferredLayout then
        return presetCount + index
      end
    end

    local loweredPreferredLayout = string.lower(preferredLayout)
    for index, layout in ipairs(layouts or {}) do
      local layoutName = type(layout.layoutName) == "string" and layout.layoutName or ""
      if string.lower(layoutName) == loweredPreferredLayout then
        return presetCount + index
      end
    end

    return nil
  end

  function C_EditMode.GetAccountSettings()
    if __wow_edit_mode_account_setting_state == nil then
      __wow_edit_mode_account_setting_state = __wow_build_default_edit_mode_account_settings()
    end
    return __wow_copy_edit_mode_value(__wow_edit_mode_account_setting_state)
  end

  function C_EditMode.GetLayouts()
    return __wow_copy_edit_mode_value(__wow_edit_mode_layout_state)
  end

  function C_EditMode.SaveLayouts(saveInfo)
    if type(saveInfo) ~= "table" then
      return
    end

    __wow_edit_mode_layout_state = {
      layouts = __wow_copy_edit_mode_value(saveInfo.layouts or {}),
      activeLayout = saveInfo.activeLayout or __wow_edit_mode_layout_state.activeLayout or 1,
    }
  end

  function C_EditMode.SetActiveLayout(layoutIndex)
    if type(layoutIndex) == "number" then
      __wow_edit_mode_layout_state.activeLayout = layoutIndex
    end
  end

  function C_EditMode.SetAccountSetting(setting, value)
    if __wow_edit_mode_account_setting_state == nil then
      __wow_edit_mode_account_setting_state = __wow_build_default_edit_mode_account_settings()
    end
    for _, settingInfo in ipairs(__wow_edit_mode_account_setting_state) do
      if settingInfo.setting == setting then
        settingInfo.value = value
        return
      end
    end
    table.insert(__wow_edit_mode_account_setting_state, { setting = setting, value = value })
    table.sort(__wow_edit_mode_account_setting_state, function(a, b) return a.setting < b.setting end)
  end

  function C_EditMode.__LoadCache(accountCache, characterCache, activeSpecIndex, preferredLayout)
    local layouts, accountSettings, missingLayouts = __wow_edit_mode_parse_account_cache(accountCache)
    local activeLayout = __wow_edit_mode_active_layout_from_character_cache(characterCache, activeSpecIndex)
    activeLayout = __wow_edit_mode_active_layout_from_override(layouts, preferredLayout) or activeLayout
    __wow_edit_mode_layout_state = {
      layouts = layouts,
      activeLayout = activeLayout or __wow_edit_mode_layout_state.activeLayout or 1,
    }
    if #accountSettings > 0 then
      __wow_edit_mode_account_setting_state = __wow_merge_edit_mode_account_settings(accountSettings)
    end
    return missingLayouts
  end
end
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(EDIT_MODE_CACHE_DEFAULTS_LUA)?;
    Ok(())
}
