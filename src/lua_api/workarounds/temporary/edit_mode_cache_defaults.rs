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

  local function __wow_edit_mode_read(tokens, cursor)
    return tokens[cursor], cursor + 1
  end

  local function __wow_edit_mode_read_number(tokens, cursor, fallback)
    local token
    token, cursor = __wow_edit_mode_read(tokens, cursor)
    return tonumber(token) or fallback or 0, cursor
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

  local function __wow_edit_mode_parse_system(tokens, cursor)
    local system, systemIndex, isInDefaultPosition, point, relativePoint
    local relativeTo, offsetX, offsetY, settingsText
    system, cursor = __wow_edit_mode_read_number(tokens, cursor)
    systemIndex, cursor = __wow_edit_mode_read_number(tokens, cursor, -1)
    if systemIndex >= 0 then
      systemIndex = systemIndex + 1
    end
    isInDefaultPosition, cursor = __wow_edit_mode_read_number(tokens, cursor)
    point, cursor = __wow_edit_mode_read_number(tokens, cursor)
    relativePoint, cursor = __wow_edit_mode_read_number(tokens, cursor)
    relativeTo, cursor = __wow_edit_mode_read(tokens, cursor)
    offsetX, cursor = __wow_edit_mode_read_number(tokens, cursor)
    offsetY, cursor = __wow_edit_mode_read_number(tokens, cursor)
    _, cursor = __wow_edit_mode_read(tokens, cursor)
    settingsText, cursor = __wow_edit_mode_read(tokens, cursor)

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
        relativeTo = relativeTo or "UIParent",
        relativePoint = __wow_edit_mode_frame_points[relativePoint] or "CENTER",
        offsetX = offsetX,
        offsetY = offsetY,
      },
      settings = __wow_edit_mode_decode_settings(settingsText),
    }, cursor
  end

  local function __wow_edit_mode_parse_account_cache(text)
    local tokens = __wow_edit_mode_tokens(text)
    local cursor = 1
    local layoutCount, accountSettingCount
    layoutCount, cursor = __wow_edit_mode_read_number(tokens, cursor)
    accountSettingCount, cursor = __wow_edit_mode_read_number(tokens, cursor)

    local accountSettings = {}
    for setting = 0, accountSettingCount - 1 do
      local value
      value, cursor = __wow_edit_mode_read_number(tokens, cursor)
      table.insert(accountSettings, { setting = setting, value = value })
    end

    local layouts = {}
    for _ = 1, layoutCount do
      -- A 12.1 client writes only the count and the account settings; do not
      -- fabricate nameless layouts from an exhausted token stream.
      if cursor > #tokens then
        break
      end
      local layoutIndex, layoutName, systemCount
      layoutIndex, cursor = __wow_edit_mode_read_number(tokens, cursor)
      layoutName, cursor = __wow_edit_mode_read(tokens, cursor)
      systemCount, cursor = __wow_edit_mode_read_number(tokens, cursor)
      local systems = {}
      for systemIndex = 1, systemCount do
        systems[systemIndex], cursor = __wow_edit_mode_parse_system(tokens, cursor)
      end
      table.insert(layouts, {
        layoutIndex = layoutIndex,
        layoutName = layoutName or "",
        layoutType = Enum.EditModeLayoutType.Account,
        systems = systems,
      })
    end

    return layouts, accountSettings
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

    local preferredIndex = tonumber(preferredLayout)
    if preferredIndex and preferredIndex > 0 then
      return preferredIndex
    end

    for index, layout in ipairs(layouts or {}) do
      if layout.layoutName == preferredLayout then
        return index
      end
    end

    local loweredPreferredLayout = string.lower(preferredLayout)
    for index, layout in ipairs(layouts or {}) do
      local layoutName = type(layout.layoutName) == "string" and layout.layoutName or ""
      if string.lower(layoutName) == loweredPreferredLayout then
        return index
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
    local layouts, accountSettings = __wow_edit_mode_parse_account_cache(accountCache)
    local activeLayout = __wow_edit_mode_active_layout_from_character_cache(characterCache, activeSpecIndex)
    activeLayout = __wow_edit_mode_active_layout_from_override(layouts, preferredLayout) or activeLayout
    __wow_edit_mode_layout_state = {
      layouts = layouts,
      activeLayout = activeLayout or __wow_edit_mode_layout_state.activeLayout or 1,
    }
    if #accountSettings > 0 then
      __wow_edit_mode_account_setting_state = __wow_merge_edit_mode_account_settings(accountSettings)
    end
  end
end
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(EDIT_MODE_CACHE_DEFAULTS_LUA)?;
    Ok(())
}
