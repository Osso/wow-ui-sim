//! Temporary Dispatcher surface fallback.
//!
//! This is a simulator-owned replacement for the Blizzard_Dispatcher addon when
//! startup paths need Dispatcher semantics before, or after, the Blizzard addon
//! has loaded. Keep it explicit in the workaround layer until the Dispatcher
//! compatibility surface is modeled as a first-class Rust/Lua subsystem.

const DISPATCHER_SURFACE_LUA: &str = r#"local function __wow_dispatcher_invoke_callback(callbackData, ...)
  local callback = callbackData and callbackData.Callback
  if type(callback) == "function" then
    return callback(...)
  end
  if type(callback) ~= "table" then
    return nil
  end

  local method = callback[callbackData.EventFunctionOrScript]
  if type(method) == "function" then
    return method(callback, ...)
  end
  return nil
end

local function __wow_dispatcher_find_id(callbackTable, ownerOrID)
  if type(callbackTable) ~= "table" or ownerOrID == nil then
    return nil
  end
  if type(ownerOrID) == "number" then
    return ownerOrID
  end

  for id, callbackData in pairs(callbackTable) do
    if type(callbackData) == "table" and callbackData.Callback == ownerOrID then
      return id
    end
  end
  return nil
end

local function __wow_dispatcher_collect_ids(callbackTable)
  local ids = {}
  if type(callbackTable) ~= "table" then
    return ids
  end
  for id in pairs(callbackTable) do
    table.insert(ids, id)
  end
  return ids
end

local function __wow_ensure_dispatcher_surface()
  local existing = rawget(_G, "Dispatcher")
  if type(existing) == "table" and rawget(existing, "__wow_sim_dispatcher") == true then
    return
  end

  DISPATCHER_VERSION = 2.0

  local dispatcher = {
    __wow_sim_dispatcher = true,
    EventFrame = nil,
    NextEventID = 1,
    NextFunctionID = 1,
    NextScriptID = 1,
    Events = {},
    Functions = {
      Global = {},
      Owners = {},
    },
    Scripts = {},
  }

  function dispatcher:_CreateCallbackData(eventFunctionOrScript, callback, oneTime)
    return {
      EventFunctionOrScript = eventFunctionOrScript,
      Callback = callback,
      OneTime = oneTime == true,
    }
  end

  function dispatcher:Initialize()
    if type(self.EventFrame) == "table" then
      return
    end

    self.EventFrame = CreateFrame("Frame", "DispatcherFrame")
    self.EventFrame:SetScript("OnEvent", function(_, event, ...)
      self:OnEvent(event, ...)
    end)
  end

  function dispatcher:RegisterEvent(event, callback, oneTime)
    self:Initialize()

    if type(event) ~= "string" then
      return nil
    end
    if type(callback) == "table" then
      self:UnregisterEvent(event, callback)
    end

    local callbacks = self.Events[event]
    if type(callbacks) ~= "table" then
      callbacks = {}
      self.Events[event] = callbacks
      if event == "OnUpdate" then
        self.EventFrame:SetScript("OnUpdate", function(_, elapsed)
          self:OnEvent("OnUpdate", elapsed)
        end)
      else
        self.EventFrame:RegisterEvent(event)
      end
    end

    local id = self.NextEventID
    self.NextEventID = id + 1
    callbacks[id] = self:_CreateCallbackData(event, callback, oneTime)
    return id
  end

  function dispatcher:UnregisterEvent(event, ownerOrID)
    local callbacks = self.Events[event]
    if type(callbacks) ~= "table" then
      return
    end

    local id = __wow_dispatcher_find_id(callbacks, ownerOrID)
    if id ~= nil then
      callbacks[id] = nil
    end

    if next(callbacks) ~= nil then
      return
    end

    self.Events[event] = nil
    if type(self.EventFrame) ~= "table" then
      return
    end
    if event == "OnUpdate" then
      self.EventFrame:SetScript("OnUpdate", nil)
    else
      self.EventFrame:UnregisterEvent(event)
    end
  end

  function dispatcher:UnregisterAllEvents(owner)
    for event, callbacks in pairs(self.Events) do
      if __wow_dispatcher_find_id(callbacks, owner) ~= nil then
        self:UnregisterEvent(event, owner)
      end
    end
  end

  function dispatcher:OnEvent(event, ...)
    local callbacks = self.Events[event]
    if type(callbacks) ~= "table" then
      return
    end

    local idsToRemove = {}
    for _, id in ipairs(__wow_dispatcher_collect_ids(callbacks)) do
      local callbackData = callbacks[id]
      if type(callbackData) == "table" then
        __wow_dispatcher_invoke_callback(callbackData, ...)
        if callbackData.OneTime then
          table.insert(idsToRemove, id)
        end
      end
    end

    for _, id in ipairs(idsToRemove) do
      self:UnregisterEvent(event, id)
    end
  end

  function dispatcher:_GetFunctionBucket(functionOwner, functionName)
    if type(functionOwner) == "table" then
      local owned = self.Functions.Owners[functionOwner]
      return type(owned) == "table" and owned[functionName] or nil
    end
    return self.Functions.Global[functionName]
  end

  function dispatcher:_SetFunctionTarget(functionOwner, functionName, func)
    if type(functionOwner) == "table" then
      functionOwner[functionName] = func
    else
      _G[functionName] = func
    end
  end

  function dispatcher:RegisterFunction(functionOwner, functionName, callback, oneTime)
    if type(functionOwner) ~= "table" then
      functionOwner, functionName, callback, oneTime = nil, functionOwner, functionName, callback
    end

    if type(functionName) ~= "string" then
      return nil
    end

    local original = type(functionOwner) == "table" and functionOwner[functionName] or _G[functionName]
    if type(original) ~= "function" then
      return nil
    end

    local bucket = self:_GetFunctionBucket(functionOwner, functionName)
    if type(bucket) ~= "table" then
      bucket = {
        callbacks = {},
        original = original,
      }

      if type(functionOwner) == "table" then
        local owned = self.Functions.Owners[functionOwner]
        if type(owned) ~= "table" then
          owned = {}
          self.Functions.Owners[functionOwner] = owned
        end
        owned[functionName] = bucket
      else
        self.Functions.Global[functionName] = bucket
      end

      local dispatcher_ref = self
      local wrapper = function(...)
        bucket.original(...)
        dispatcher_ref:OnSecureFunc(functionOwner, functionName, ...)
      end
      bucket.wrapper = wrapper
      self:_SetFunctionTarget(functionOwner, functionName, wrapper)
    end

    local id = self.NextFunctionID
    self.NextFunctionID = id + 1
    bucket.callbacks[id] = self:_CreateCallbackData(functionName, callback, oneTime)
    return id
  end

  function dispatcher:UnregisterFunction(functionOwner, functionName, ownerOrID)
    if type(functionOwner) ~= "table" then
      functionOwner, functionName, ownerOrID = nil, functionOwner, functionName
    end

    local bucket = self:_GetFunctionBucket(functionOwner, functionName)
    if type(bucket) ~= "table" then
      return
    end

    local id = __wow_dispatcher_find_id(bucket.callbacks, ownerOrID)
    if id ~= nil then
      bucket.callbacks[id] = nil
    end

    if next(bucket.callbacks) ~= nil then
      return
    end

    self:_SetFunctionTarget(functionOwner, functionName, bucket.original)
    if type(functionOwner) == "table" then
      local owned = self.Functions.Owners[functionOwner]
      if type(owned) == "table" then
        owned[functionName] = nil
        if next(owned) == nil then
          self.Functions.Owners[functionOwner] = nil
        end
      end
    else
      self.Functions.Global[functionName] = nil
    end
  end

  function dispatcher:UnregisterAllFunctions(owner)
    for functionName, bucket in pairs(self.Functions.Global) do
      if __wow_dispatcher_find_id(bucket.callbacks, owner) ~= nil then
        self:UnregisterFunction(functionName, owner)
      end
    end

    for functionOwner, owned in pairs(self.Functions.Owners) do
      for functionName, bucket in pairs(owned) do
        if __wow_dispatcher_find_id(bucket.callbacks, owner) ~= nil then
          self:UnregisterFunction(functionOwner, functionName, owner)
        end
      end
    end
  end

  function dispatcher:OnSecureFunc(functionOwner, functionName, ...)
    local bucket = self:_GetFunctionBucket(functionOwner, functionName)
    if type(bucket) ~= "table" then
      return
    end

    local idsToRemove = {}
    for _, id in ipairs(__wow_dispatcher_collect_ids(bucket.callbacks)) do
      local callbackData = bucket.callbacks[id]
      if type(callbackData) == "table" then
        __wow_dispatcher_invoke_callback(callbackData, ...)
        if callbackData.OneTime then
          table.insert(idsToRemove, id)
        end
      end
    end

    for _, id in ipairs(idsToRemove) do
      self:UnregisterFunction(functionOwner, functionName, id)
    end
  end

  function dispatcher:RegisterScript(frame, script, callback, oneTime)
    if type(frame) ~= "table" or type(script) ~= "string" or not frame:HasScript(script) then
      return nil
    end

    local frameScripts = self.Scripts[frame]
    if type(frameScripts) ~= "table" then
      frameScripts = {}
      self.Scripts[frame] = frameScripts
    end

    local callbacks = frameScripts[script]
    if type(callbacks) ~= "table" then
      callbacks = {}
      frameScripts[script] = callbacks
      frame:HookScript(script, function(...)
        self:OnScript(frame, script, ...)
      end)
    end

    local id = self.NextScriptID
    self.NextScriptID = id + 1
    callbacks[id] = self:_CreateCallbackData(script, callback, oneTime)
    return id
  end

  function dispatcher:UnregisterScript(frame, script, ownerOrID)
    local frameScripts = self.Scripts[frame]
    local callbacks = type(frameScripts) == "table" and frameScripts[script] or nil
    if type(callbacks) ~= "table" then
      return
    end

    local id = __wow_dispatcher_find_id(callbacks, ownerOrID)
    if id ~= nil then
      callbacks[id] = nil
    end
  end

  function dispatcher:UnregisterAllScripts(owner)
    for frame, frameScripts in pairs(self.Scripts) do
      for script, callbacks in pairs(frameScripts) do
        if __wow_dispatcher_find_id(callbacks, owner) ~= nil then
          self:UnregisterScript(frame, script, owner)
        end
      end
    end
  end

  function dispatcher:OnScript(frame, script, ...)
    local frameScripts = self.Scripts[frame]
    local callbacks = type(frameScripts) == "table" and frameScripts[script] or nil
    if type(callbacks) ~= "table" then
      return
    end

    local idsToRemove = {}
    for _, id in ipairs(__wow_dispatcher_collect_ids(callbacks)) do
      local callbackData = callbacks[id]
      if type(callbackData) == "table" then
        __wow_dispatcher_invoke_callback(callbackData, ...)
        if callbackData.OneTime then
          table.insert(idsToRemove, id)
        end
      end
    end

    for _, id in ipairs(idsToRemove) do
      self:UnregisterScript(frame, script, id)
    end
  end

  function dispatcher:UnregisterAll(owner)
    self:UnregisterAllEvents(owner)
    self:UnregisterAllFunctions(owner)
    self:UnregisterAllScripts(owner)
  end

  Dispatcher = dispatcher
  dispatcher:Initialize()
end

__wow_ensure_dispatcher_surface()
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(DISPATCHER_SURFACE_LUA)?;
    Ok(())
}

/// Re-install the simulator Dispatcher surface after Blizzard_Dispatcher
/// loads. Wired through `apply_blizzard_post_load_patches`; the Lua
/// `hooksecurefunc(C_AddOns, "LoadAddOn", ...)` route no longer works because
/// the shared bootstrap hooksecurefunc deliberately refuses that target.
pub(crate) fn patch_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) -> crate::Result<()> {
    env.exec(DISPATCHER_SURFACE_LUA)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_dispatcher_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(Dispatcher) ~= "table" then return "dispatcher" end
                if Dispatcher.__wow_sim_dispatcher ~= true then return "marker" end
                if type(Dispatcher.RegisterEvent) ~= "function" then return "register_event" end
                if type(Dispatcher.UnregisterAll) ~= "function" then return "unregister_all" end
                if DISPATCHER_VERSION ~= 2.0 then return "version" end
                return "ok"
                "#,
            )
            .expect("Dispatcher surface probe should run");

        assert_eq!(result, "ok");
    }

    modern_engine_namespace_fixture! {
    #[test]
    fn reapplies_after_blizzard_dispatcher_load() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Dispatcher = { __wow_sim_dispatcher = false }
            C_AddOns.LoadAddOn("Blizzard_Dispatcher")
            "#,
        )
        .expect("dispatcher reload fixture should run");

        let restored: bool = env
            .eval("return type(Dispatcher) == 'table' and Dispatcher.__wow_sim_dispatcher == true")
            .expect("Dispatcher restore probe should run");

        assert!(restored);
    }
    }
}
