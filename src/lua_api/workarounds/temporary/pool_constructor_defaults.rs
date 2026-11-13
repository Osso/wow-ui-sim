//! Temporary pool constructor defaults for partial Blizzard addon loads.
//!
//! Blizzard SharedXML owns the real pool APIs. These defaults only cover
//! isolated addon/test paths where SharedXML pool helpers have not been loaded.

const POOL_CONSTRUCTOR_DEFAULTS_LUA: &str = r#"
local function __wow_pool_release_error(object, belongs)
  if belongs then
    return ("Attempted to release inactive object '%s'"):format(tostring(object))
  end
  return ("Attempted to release object '%s' that doesn't belong to this pool"):format(tostring(object))
end

local function __wow_pool_is_active_for_release(pool, object, canFailToFindObject)
  local active = pool.active[object] == true
  if active or canFailToFindObject then
    return active
  end
  error(__wow_pool_release_error(object, pool:DoesObjectBelongToPool(object)), 3)
end

if CreateFramePool == nil then
  function CreateFramePool(frameType, parent, template, resetter)
    local pool = {
      frameType = frameType or "Frame",
      parent = parent,
      template = template,
      resetter = resetter,
      inactive = {},
      active = {},
      known = {},
    }

    function pool:Acquire()
      local frame = table.remove(self.inactive)
      local isNew = false
      if frame == nil then
        frame = CreateFrame(self.frameType, nil, self.parent, self.template)
        isNew = true
        if self.resetter then
          self.resetter(self, frame, true, self.template)
        end
        self.known[frame] = true
      end
      self.active[frame] = true
      return frame, isNew
    end

    function pool:Release(frame, canFailToFindObject)
      if not __wow_pool_is_active_for_release(self, frame, canFailToFindObject) then
        return false
      end
      if self.resetter then
        self.resetter(self, frame, nil, self.template)
      elseif frame.Hide then
        frame:Hide()
      end
      self.active[frame] = nil
      table.insert(self.inactive, frame)
      return true
    end

    function pool:ReleaseAll()
      local frames = {}
      for frame in pairs(self.active) do
        table.insert(frames, frame)
      end
      for _, frame in ipairs(frames) do
        self:Release(frame)
      end
    end

    function pool:GetNumActive()
      local count = 0
      for _ in pairs(self.active) do
        count = count + 1
      end
      return count
    end

    function pool:IsActive(frame)
      return self.active[frame] == true
    end

    function pool:DoesObjectBelongToPool(frame)
      return self.known[frame] == true
    end

    function pool:EnumerateActive()
      local frames = {}
      for frame in pairs(self.active) do
        frames[#frames + 1] = frame
      end
      local index = 0
      return function()
        index = index + 1
        return frames[index]
      end
    end

    return pool
  end
end

if PartyMemberFramePool == nil and type(CreateFramePool) == "function" then
  PartyMemberFramePool = CreateFramePool("Frame", UIParent)
end
if PartyFrame ~= nil and PartyFrame.PartyMemberFramePool == nil then
  PartyFrame.PartyMemberFramePool = PartyMemberFramePool
end

local function __wow_make_region_pool(acquire_region)
  return function(parent, layer, subLevel, template, resetter)
    local pool = {
      parent = parent,
      layer = layer,
      subLevel = subLevel,
      template = template,
      resetter = resetter,
      inactive = {},
      active = {},
      known = {},
    }

    function pool:Acquire()
      local region = table.remove(self.inactive)
      local isNew = false
      if region == nil then
        region = acquire_region(self.parent, self.layer, self.subLevel, self.template)
        isNew = true
        if self.resetter then
          self.resetter(self, region, true, self.template)
        end
        self.known[region] = true
      end
      self.active[region] = true
      return region, isNew
    end

    function pool:Release(region, canFailToFindObject)
      if not __wow_pool_is_active_for_release(self, region, canFailToFindObject) then
        return false
      end
      if self.resetter then
        self.resetter(self, region, nil, self.template)
      end
      self.active[region] = nil
      table.insert(self.inactive, region)
      return true
    end

    function pool:GetNumActive()
      local count = 0
      for _ in pairs(self.active) do
        count = count + 1
      end
      return count
    end

    function pool:IsActive(region)
      return self.active[region] == true
    end

    function pool:DoesObjectBelongToPool(region)
      return self.known[region] == true
    end

    function pool:ReleaseAll()
      local regions = {}
      for region in pairs(self.active) do
        regions[#regions + 1] = region
      end
      for _, region in ipairs(regions) do
        self:Release(region)
      end
    end

    function pool:EnumerateActive()
      local regions = {}
      for region in pairs(self.active) do
        regions[#regions + 1] = region
      end
      local index = 0
      return function()
        index = index + 1
        return regions[index]
      end
    end

    return pool
  end
end

if CreateTexturePool == nil then
  CreateTexturePool = __wow_make_region_pool(function(parent, layer)
    return parent:CreateTexture(nil, layer or "ARTWORK")
  end)
end

if CreateFontStringPool == nil then
  CreateFontStringPool = __wow_make_region_pool(function(parent, layer)
    return parent:CreateFontString(nil, layer or "ARTWORK")
  end)
end

if CreateFramePoolCollection == nil then
  function CreateFramePoolCollection()
    local collection = { pools = {} }

    local function pool_key(frameType, parent, template, specialization)
      return table.concat({
        tostring(frameType or "Frame"),
        tostring(parent),
        tostring(template),
        tostring(specialization),
      }, "|")
    end

    local function find_pool_by_template(collection, template, specialization)
      for _, pool in pairs(collection.pools) do
        if pool.template == template and pool.specialization == specialization then
          return pool
        end
      end
      return nil
    end

    function collection:CreatePool(frameType, parent, template, resetter, _forbidden, specialization)
      local key = pool_key(frameType, parent, template, specialization)
      local pool = CreateFramePool(frameType, parent, template, resetter)
      pool.specialization = specialization
      self.pools[key] = pool
      return pool
    end

    function collection:GetOrCreatePool(frameType, parent, template, resetter, forbidden, specialization)
      local key = pool_key(frameType, parent, template, specialization)
      local pool = self.pools[key]
      if pool == nil then
        pool = self:CreatePool(frameType, parent, template, resetter, forbidden, specialization)
      end
      return pool
    end

    function collection:Acquire(template, specialization)
      local pool = find_pool_by_template(self, template, specialization)
      if pool == nil then
        return nil
      end
      return pool:Acquire()
    end

    function collection:GetNumActive()
      local total = 0
      for _, pool in pairs(self.pools) do
        total = total + (pool.GetNumActive and pool:GetNumActive() or 0)
      end
      return total
    end

    function collection:IsActive(object)
      for _, pool in pairs(self.pools) do
        if pool.IsActive and pool:IsActive(object) then
          return true
        end
      end
      return false
    end

    function collection:DoesObjectBelongToPool(object)
      for _, pool in pairs(self.pools) do
        if pool.DoesObjectBelongToPool and pool:DoesObjectBelongToPool(object) then
          return true
        end
      end
      return false
    end

    function collection:Release(object)
      for _, pool in pairs(self.pools) do
        if pool.Release and pool:Release(object, true) then
          return true
        end
      end
      error(__wow_pool_release_error(object, self:DoesObjectBelongToPool(object)), 2)
    end

    function collection:ReleaseAll()
      for _, pool in pairs(self.pools) do
        if pool.ReleaseAll then
          pool:ReleaseAll()
        end
      end
    end

    function collection:EnumerateActive()
      local objects = {}
      for _, pool in pairs(self.pools) do
        if pool.EnumerateActive then
          for object in pool:EnumerateActive() do
            objects[#objects + 1] = object
          end
        end
      end
      local index = 0
      return function()
        index = index + 1
        return objects[index]
      end
    end

    return collection
  end
end

if CreateFrameFactory == nil then
  function CreateFrameFactory()
    local factory = {
      templateInfoCache = CreateTemplateInfoCache and CreateTemplateInfoCache() or nil,
      poolCollection = CreateFramePoolCollection and CreateFramePoolCollection() or nil,
    }

    function factory:GetTemplateInfoCache()
      return self.templateInfoCache
    end

    function factory:Create(parent, frameTypeOrTemplate, resetFunc)
      local info = self.templateInfoCache and self.templateInfoCache:GetTemplateInfo(frameTypeOrTemplate) or nil
      local frameTemplate = nil
      local frameType = nil
      local specialization = nil

      if info then
        frameTemplate = frameTypeOrTemplate
        frameType = info.type
      else
        frameTemplate = ""
        frameType = type(frameTypeOrTemplate) == "string" and frameTypeOrTemplate or "Frame"
        specialization = frameType
      end

      if self.poolCollection and self.poolCollection.GetOrCreatePool then
        local pool = self.poolCollection:GetOrCreatePool(frameType, parent, frameTemplate, resetFunc, nil, specialization)
        local frame, isNew = pool:Acquire()
        return frame, isNew, info
      end

      local frame = CreateFrame(frameType, nil, parent, frameTemplate)
      if resetFunc then
        resetFunc(nil, frame, true, frameTemplate)
      end
      return frame, true, info
    end

    function factory:GetNumActive()
      if self.poolCollection and self.poolCollection.GetNumActive then
        return self.poolCollection:GetNumActive()
      end
      return 0
    end

    function factory:ReleaseAll()
      if self.poolCollection and self.poolCollection.ReleaseAll then
        self.poolCollection:ReleaseAll()
      end
    end

    function factory:Release(frame)
      if self.poolCollection and self.poolCollection.Release then
        return self.poolCollection:Release(frame)
      end
      return false
    end

    return factory
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(POOL_CONSTRUCTOR_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_pool_constructor_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(CreateFramePool) ~= "function" then return "frame_pool" end
                if type(CreateTexturePool) ~= "function" then return "texture_pool" end
                if type(CreateFontStringPool) ~= "function" then return "font_string_pool" end
                if type(CreateFramePoolCollection) ~= "function" then return "collection" end
                if type(CreateFrameFactory) ~= "function" then return "factory" end
                if type(PartyMemberFramePool) ~= "table" then return "party_pool" end
                if PartyMemberFramePool:GetNumActive() ~= 0 then return "party_pool_active" end
                if PartyFrame ~= nil and PartyFrame.PartyMemberFramePool ~= PartyMemberFramePool then return "party_frame_pool" end

                local pool = CreateFramePool("Frame", UIParent)
                local f1, isNew = pool:Acquire()
                if f1 == nil or not isNew or pool:GetNumActive() ~= 1 then return "acquire" end
                if not pool:IsActive(f1) or not pool:DoesObjectBelongToPool(f1) then return "active" end
                pool:Release(f1)
                local f2, isNewAgain = pool:Acquire()
                if f2 ~= f1 or isNewAgain then return "reuse" end

                local texturePool = CreateTexturePool(UIParent, "ARTWORK")
                local texture = texturePool:Acquire()
                if texture == nil or texturePool:GetNumActive() ~= 1 then return "texture" end

                local collection = CreateFramePoolCollection()
                local collectedPool = collection:GetOrCreatePool("Frame", UIParent, "")
                local collectedFrame = collectedPool:Acquire()
                if not collection:IsActive(collectedFrame) then return "collection_active" end
                collection:ReleaseAll()
                if collection:GetNumActive() ~= 0 then return "collection_release" end

                local factory = CreateFrameFactory()
                local factoryFrame = factory:Create(UIParent, "Frame")
                if factoryFrame == nil or factory:GetNumActive() ~= 1 then return "factory_create" end
                factory:ReleaseAll()
                if factory:GetNumActive() ~= 0 then return "factory_release" end
                return "ok"
                "#,
            )
            .expect("pool constructor probe should run");

        assert_eq!(result, "ok");
    }
}
