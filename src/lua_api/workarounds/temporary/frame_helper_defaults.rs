//! Temporary frame helper methods installed around the CreateFrame boundary.
//!
//! These helpers model partial MapCanvas/data-provider behavior that is still
//! Lua-owned compatibility glue. Keep them out of the generic runtime surface
//! until the backed frame systems are implemented directly.

const FRAME_HELPER_DEFAULTS_LUA: &str = r#"
local function __wow_install_frame_helpers(frame)
  if frame == nil then
    return nil
  end

  if frame.AddDataProvider == nil then
    function frame:AddDataProvider(provider)
      local env = debug and debug.getfenv and debug.getfenv(self)
      local fields = type(env) == "table" and env[1] or nil
      if type(fields) ~= "table" then
        fields = {}
        if type(env) == "table" then
          env[1] = fields
        else
          return
        end
      end
      local providers = fields.dataProviders
      if type(providers) ~= "table" then
        providers = {}
        fields.dataProviders = providers
      end
      for i = 1, #providers do
        if providers[i] == provider then
          return
        end
      end
      providers[#providers + 1] = provider
      if type(provider) == "table" and type(provider.OnAdded) == "function" then
        pcall(provider.OnAdded, provider, self)
      end
      if type(provider) == "table" and provider.pin ~= nil then
        provider.pin.dataProvider = provider
      end
      if type(provider) == "table" and provider.pin == nil then
        provider.pin = { dataProvider = provider }
      end
    end
  end

  if frame.RemoveDataProvider == nil then
    function frame:RemoveDataProvider(provider)
      local env = debug and debug.getfenv and debug.getfenv(self)
      local providers = type(env) == "table" and env[1] and env[1].dataProviders or nil
      if type(providers) ~= "table" then
        return
      end
      for i = #providers, 1, -1 do
        if providers[i] == provider then
          table.remove(providers, i)
        end
      end
    end
  end

  if frame.IsInitialized == nil then
    function frame:IsInitialized()
      return type(self.layoutInfo) == "table" or type(self.systemInfo) == "table"
    end
  end

  -- A frame without edit-mode systemInfo counts as in its default position:
  -- the client has the method only on edit-mode systems, and its callers
  -- outside the mixin (AlertFrames.lua:416, EditModeUtil.lua:22) treat a
  -- missing method that way.
  if frame.IsInDefaultPosition == nil then
    function frame:IsInDefaultPosition()
      local info = self.systemInfo
      return type(info) ~= "table" or info.isInDefaultPosition == true
    end
  end

  return frame
end

if CreateFrame ~= nil and __wow_original_CreateFrame == nil then
  __wow_original_CreateFrame = CreateFrame

  function CreateFrame(...)
    local frameType = select(1, ...)
    local inherits = select(4, ...)
    if type(inherits) == "string" then
      if string.find(inherits, "MapCanvasFrameTemplate", 1, true) or
         string.find(inherits, "MapCanvasFrameScrollContainerTemplate", 1, true) then
        __wow_patch_map_canvas_scroll_container_methods()
      end
    end
    local created = __wow_install_frame_helpers(__wow_original_CreateFrame(...))
    if frameType == "GameTooltip" and created and created.SetFrameStrata ~= nil then
      created:SetFrameStrata("TOOLTIP")
    end
    local parent = select(3, ...)
    if type(parent) == "table" and type(inherits) == "string" then
      if string.find(inherits, "MapCanvasFrameScrollContainerTemplate", 1, true) then
        rawset(parent, "ScrollContainer", created)
      end
    end
    return created
  end
end

do
  local frameMeta = GetFrameMetatable and GetFrameMetatable()
  local frameIndex = frameMeta and frameMeta.__index
  if type(frameIndex) == "table" then
    if frameIndex.AddDataProvider == nil then
      -- Keep in sync with the AddDataProvider fallback in shared_bootstrap.lua:
      -- whichever installs first wins the == nil guard.
      function frameIndex:AddDataProvider(provider)
        local fields = debug.getfenv(self)
        local store = fields and fields[1]
        if type(store) ~= "table" then
          return
        end
        local providers = store.dataProviders
        if type(providers) ~= "table" then
          providers = {}
          store.dataProviders = providers
        end
        for i = 1, #providers do
          if providers[i] == provider then
            return
          end
        end
        providers[#providers + 1] = provider
        if type(provider) == "table" and type(provider.OnAdded) == "function" then
          pcall(provider.OnAdded, provider, self)
        end
        if type(provider) == "table" and provider.pin ~= nil then
          provider.pin.dataProvider = provider
        end
        if type(provider) == "table" and provider.pin == nil then
          provider.pin = { dataProvider = provider }
        end
      end
    end

    if frameIndex.RemoveDataProvider == nil then
      function frameIndex:RemoveDataProvider(provider)
        local fields = debug.getfenv(self)
        local providers = fields and fields[1] and fields[1].dataProviders
        if type(providers) ~= "table" then
          return
        end
        for i = #providers, 1, -1 do
          if providers[i] == provider then
            table.remove(providers, i)
          end
        end
      end
    end

    if frameIndex.IsInitialized == nil then
      function frameIndex:IsInitialized()
        return type(self.layoutInfo) == "table" or type(self.systemInfo) == "table"
      end
    end

    if frameIndex.IsInDefaultPosition == nil then
      function frameIndex:IsInDefaultPosition()
        local info = self.systemInfo
        return type(info) ~= "table" or info.isInDefaultPosition == true
      end
    end
  end
end

local function __wow_frame_fields(frame)
  local env = debug and debug.getfenv and debug.getfenv(frame)
  if type(env) ~= "table" then
    return nil
  end
  if type(env[1]) ~= "table" then
    env[1] = {}
  end
  return env[1]
end

local function __wow_remove_array_value(values, target)
  if type(values) ~= "table" then
    return
  end
  for index = #values, 1, -1 do
    if values[index] == target then
      table.remove(values, index)
      break
    end
  end
end

local function __wow_register_core_frame_methods()
  local mt = GetFrameMetatable and GetFrameMetatable()
  local methods = mt and mt.__index
  if type(methods) ~= "table" then
    return
  end

  if methods.IsInitialized == nil then
    function methods:IsInitialized()
      return type(self.layoutInfo) == "table" or type(self.systemInfo) == "table"
    end
  end

  if methods.IsInDefaultPosition == nil then
    function methods:IsInDefaultPosition()
      local systemInfo = self.systemInfo
      if type(systemInfo) == "table" and systemInfo.isInDefaultPosition ~= nil then
        return systemInfo.isInDefaultPosition == true
      end
      return type(systemInfo) ~= "table"
    end
  end

  if methods.AddDataProvider == nil then
    function methods:AddDataProvider(provider)
      local fields = __wow_frame_fields(self)
      if fields == nil or provider == nil then
        return
      end
      local providers = fields.dataProviders
      if type(providers) ~= "table" then
        providers = {}
        fields.dataProviders = providers
      end
      for _, existing in ipairs(providers) do
        if existing == provider then
          return
        end
      end
      table.insert(providers, provider)
      if type(provider) == "table" and provider.pin ~= nil then
        provider.pin.dataProvider = provider
      end
      if type(provider) == "table" and provider.pin == nil then
        provider.pin = { dataProvider = provider }
      end
    end
  end

  if methods.SetTitle == nil then
    function methods:SetTitle(title)
      self.title = title
      if self.TitleText and type(self.TitleText.SetText) == "function" then
        self.TitleText:SetText(title or "")
      elseif self.TitleContainer and self.TitleContainer.TitleText and type(self.TitleContainer.TitleText.SetText) == "function" then
        self.TitleContainer.TitleText:SetText(title or "")
      elseif self.Header and self.Header.Text and type(self.Header.Text.SetText) == "function" then
        self.Header.Text:SetText(title or "")
      end
    end
  end

  if methods.SetPortraitToAsset == nil then
    function methods:SetPortraitToAsset(texture)
      if self.GetPortrait and type(self.GetPortrait) == "function" then
        local portrait = self:GetPortrait()
        if portrait and type(portrait.SetTexture) == "function" then
          portrait:SetTexture(texture)
          return
        end
      end
      if self.PortraitContainer and self.PortraitContainer.portrait and type(self.PortraitContainer.portrait.SetTexture) == "function" then
        self.PortraitContainer.portrait:SetTexture(texture)
      end
    end
  end

  if methods.SetInterpolateScroll == nil then
    function methods:SetInterpolateScroll(enabled)
      self.interpolateScroll = enabled and true or false
    end
  end

  if methods.CanInterpolateScroll == nil then
    function methods:CanInterpolateScroll()
      return false
    end
  end

  if methods.Update == nil then
    function methods:Update()
      if type(self.updateCallback) == "function" then
        return self.updateCallback(self)
      end
    end
  end

  if methods.SetDirtyMethod == nil then
    function methods:SetDirtyMethod(method)
      self.dirtyCallback = function()
        method(self)
        self.dirty = nil
      end
    end
  end

  if methods.MarkDirty == nil then
    function methods:MarkDirty()
      if not self.dirty then
        if type(self.dirtyCallback) == "function" then
          RunNextFrame(self.dirtyCallback)
        end
      end
      self.dirty = true
    end
  end

  if methods.IsDirty == nil then
    function methods:IsDirty()
      return self.dirty
    end
  end

  function __wow_mark_nearest_layout_parent_dirty(frame)
    local parent = frame and frame.GetParent and frame:GetParent() or nil
    while parent do
      if __wow_mark_layout_frame_dirty(parent) then
        return
      end
      parent = parent.GetParent and parent:GetParent() or nil
    end
  end

  function __wow_mark_layout_frame_dirty(frame)
    if frame and frame.IsLayoutFrame and frame:IsLayoutFrame() then
      -- Blizzard's BaseLayoutMixin:MarkDirty installs its own OnUpdate; keep an
      -- already installed custom OnUpdate script in place.
      local currentOnUpdate = frame.GetScript and frame:GetScript("OnUpdate") or nil
      frame:MarkDirty()
      if currentOnUpdate and frame.GetScript and frame:GetScript("OnUpdate") ~= currentOnUpdate then
        frame:SetScript("OnUpdate", currentOnUpdate)
      end
      return true
    end
    return false
  end

  if methods.AddModule == nil then
    function methods:AddModule(module)
      local fields = __wow_frame_fields(self)
      if fields == nil or module == nil then
        return
      end
      local modules = fields.modules
      if type(modules) ~= "table" then
        modules = {}
        fields.modules = modules
      end
      for _, existing in ipairs(modules) do
        if existing == module then
          return
        end
      end
      table.insert(modules, module)
      if type(module.SetContainer) == "function" then
        module:SetContainer(self)
      end
    end
  end

  if methods.RemoveModule == nil then
    function methods:RemoveModule(module)
      local fields = __wow_frame_fields(self)
      local modules = fields and fields.modules
      if type(modules) ~= "table" then
        return
      end
      for i, existing in ipairs(modules) do
        if existing == module then
          table.remove(modules, i)
          break
        end
      end
    end
  end

  if methods.RemoveAllModules == nil then
    function methods:RemoveAllModules()
      local fields = __wow_frame_fields(self)
      if fields ~= nil then
        fields.modules = {}
      end
    end
  end

  if methods.HasModule == nil then
    function methods:HasModule(module)
      local fields = __wow_frame_fields(self)
      local modules = fields and fields.modules
      if type(modules) ~= "table" then
        return false
      end
      for _, existing in ipairs(modules) do
        if existing == module then
          return true
        end
      end
      return false
    end
  end

  if methods.RemoveDataProvider == nil then
    function methods:RemoveDataProvider(provider)
      local fields = __wow_frame_fields(self)
      local providers = fields and fields.dataProviders
      __wow_remove_array_value(providers, provider)
    end
  end
end

__wow_register_core_frame_methods()
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(FRAME_HELPER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn created_frames_get_data_provider_helpers() {
        let env = WowLuaEnv::new().expect("create Lua environment");
        let result: String = env
            .eval(
                r#"
                local frame = CreateFrame("Frame", "FrameHelperDefaultsTest", UIParent)
                local provider = { OnAdded = function(self, owner) self.owner = owner end }
                frame:AddDataProvider(provider)
                frame:AddDataProvider(provider)
                local fields = debug.getfenv(frame)
                local providers = fields and fields[1] and fields[1].dataProviders or {}
                if #providers ~= 1 then return "provider_count=" .. tostring(#providers) end
                if provider.owner ~= frame then return "provider_on_added_missing" end
                if not provider.pin or provider.pin.dataProvider ~= provider then return "pin_missing" end
                frame:RemoveDataProvider(provider)
                if #providers ~= 0 then return "provider_remove_failed" end
                if frame:IsInitialized() then return "fresh_frame_initialized" end
                frame.layoutInfo = {}
                if not frame:IsInitialized() then return "layout_frame_uninitialized" end
                -- no systemInfo: default position (AlertFrames.lua:416 semantics)
                if not frame:IsInDefaultPosition() then return "default_position_without_info" end
                frame.systemInfo = { isInDefaultPosition = true }
                if not frame:IsInDefaultPosition() then return "default_position_missing" end
                return "ok"
                "#,
            )
            .expect("frame helper probe should run");

        assert_eq!(result, "ok");
    }
}
