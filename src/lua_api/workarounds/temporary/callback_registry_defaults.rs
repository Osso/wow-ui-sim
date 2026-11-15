//! Temporary callback registry defaults for isolated Blizzard addon loads.
//!
//! Real callback registry behavior is owned by Blizzard SharedXML. These
//! fallbacks keep tests and partial addon loads functional without leaving the
//! compatibility surface hidden in the central runtime bootstrap.

const CALLBACK_REGISTRY_DEFAULTS_LUA: &str = r#"
if type(CallbackRegistryMixin) ~= "table" then
  CallbackRegistryMixin = {}
end

if type(CallbackRegistryMixin.OnLoad) ~= "function" then
  function CallbackRegistryMixin:OnLoad()
    self.__callbacks = self.__callbacks or {}
    self.Event = self.Event or {}
  end
end

if type(CallbackRegistryMixin.SetUndefinedEventsAllowed) ~= "function" then
  function CallbackRegistryMixin:SetUndefinedEventsAllowed(allowed)
    self.__allowUndefinedEvents = not not allowed
  end
end

if type(CallbackRegistryMixin.GenerateCallbackEvents) ~= "function" then
  function CallbackRegistryMixin:GenerateCallbackEvents(events)
    self:OnLoad()
    if type(events) ~= "table" then
      return
    end
    for _, eventName in ipairs(events) do
      self.Event[eventName] = eventName
    end
  end
end

if type(CallbackRegistryMixin.RegisterCallback) ~= "function" then
  function CallbackRegistryMixin:RegisterCallback(eventName, callback, owner)
    self:OnLoad()
    if type(callback) ~= "function" then
      return nil
    end
    local callbacks = self.__callbacks[eventName]
    if callbacks == nil then
      callbacks = {}
      self.__callbacks[eventName] = callbacks
    end
    local handle = { callback = callback, owner = owner }
    callbacks[#callbacks + 1] = handle
    return handle
  end
end

if type(CallbackRegistryMixin.UnregisterCallback) ~= "function" then
  function CallbackRegistryMixin:UnregisterCallback(eventName, ownerOrHandle)
    local callbacks = self.__callbacks and self.__callbacks[eventName]
    if callbacks == nil then
      return
    end
    for index = #callbacks, 1, -1 do
      local entry = callbacks[index]
      if entry == ownerOrHandle or entry.owner == ownerOrHandle then
        table.remove(callbacks, index)
      end
    end
  end
end

if type(CallbackRegistryMixin.TriggerEvent) ~= "function" then
  function CallbackRegistryMixin:TriggerEvent(eventName, ...)
    local callbacks = self.__callbacks and self.__callbacks[eventName]
    if callbacks == nil then
      return
    end
    for _, entry in ipairs(callbacks) do
      if entry.owner ~= nil then
        entry.callback(entry.owner, ...)
      else
        entry.callback(...)
      end
    end
  end
end

if type(EventRegistry) ~= "table" then
  EventRegistry = CreateFromMixins(CallbackRegistryMixin)
  EventRegistry:OnLoad()
end

if type(CVarCallbackRegistry) ~= "table" then
  CVarCallbackRegistry = CreateFromMixins(CallbackRegistryMixin)
  CVarCallbackRegistry:OnLoad()
end

if type(CVarCallbackRegistry.SetCVarCachable) ~= "function" then
  function CVarCallbackRegistry:SetCVarCachable(name)
    self.__cvars = self.__cvars or {}
    self.__cvars[name] = true
  end
end

function CVarCallbackRegistry:GetCVarValueBool(name)
  return GetCVarBool(name) == true
end

if type(CallbackRegistrantMixin) ~= "table" then
  CallbackRegistrantMixin = {}
end

if type(CallbackRegistrantMixin.AddEventMethodInternal) ~= "function" then
  function CallbackRegistrantMixin:AddEventMethodInternal(handlersTable, callbackRegistry, event, handlerMethod)
    local info = self:CreateEventRegistrationInfo(callbackRegistry, event, handlerMethod)
    table.insert(handlersTable, info)
    return info
  end
end

if type(CallbackRegistrantMixin.GetDynamicCallbackRegistrantHandlers) ~= "function" then
  function CallbackRegistrantMixin:GetDynamicCallbackRegistrantHandlers()
    self.callbackRegistrantHandlers = self.callbackRegistrantHandlers or {}
    return self.callbackRegistrantHandlers
  end
end

if type(CallbackRegistrantMixin.GetStaticCallbackRegistrantHandlers) ~= "function" then
  function CallbackRegistrantMixin:GetStaticCallbackRegistrantHandlers()
    self.staticCallbackRegistrantHandlers = self.staticCallbackRegistrantHandlers or {}
    return self.staticCallbackRegistrantHandlers
  end
end

if type(CallbackRegistrantMixin.CreateEventRegistrationInfo) ~= "function" then
  function CallbackRegistrantMixin:CreateEventRegistrationInfo(callbackRegistry, event, handlerMethod)
    return {
      callbackRegistry = callbackRegistry,
      event = event,
      handlerMethod = handlerMethod,
      registered = false,
    }
  end
end

if type(CallbackRegistrantMixin.RegisterFromRegistrationInfo) ~= "function" then
  function CallbackRegistrantMixin:RegisterFromRegistrationInfo(info)
    if info.registered then
      return
    end
    if type(info.callbackRegistry) ~= "table" or type(info.callbackRegistry.RegisterCallback) ~= "function" then
      return
    end
    info.callbackRegistry:RegisterCallback(info.event, info.handlerMethod, self)
    info.registered = true
  end
end

if type(CallbackRegistrantMixin.UnregisterFromRegistrationInfo) ~= "function" then
  function CallbackRegistrantMixin:UnregisterFromRegistrationInfo(info)
    if not info.registered then
      return
    end
    if type(info.callbackRegistry) == "table" and type(info.callbackRegistry.UnregisterCallback) == "function" then
      info.callbackRegistry:UnregisterCallback(info.event, self)
    end
    info.registered = false
  end
end

if type(CallbackRegistrantMixin.UnregisterAllInternal) ~= "function" then
  function CallbackRegistrantMixin:UnregisterAllInternal(handlersTable)
    for _, info in ipairs(handlersTable) do
      self:UnregisterFromRegistrationInfo(info)
    end
  end
end

if type(CallbackRegistrantMixin.AddStaticEventMethod) ~= "function" then
  function CallbackRegistrantMixin:AddStaticEventMethod(callbackRegistry, event, handlerMethod)
    local info = self:AddEventMethodInternal(self:GetStaticCallbackRegistrantHandlers(), callbackRegistry, event, handlerMethod)
    self:RegisterFromRegistrationInfo(info)
    return info
  end
end

if type(CallbackRegistrantMixin.AddDynamicEventMethod) ~= "function" then
  function CallbackRegistrantMixin:AddDynamicEventMethod(callbackRegistry, event, handlerMethod)
    local info = self:AddEventMethodInternal(self:GetDynamicCallbackRegistrantHandlers(), callbackRegistry, event, handlerMethod)
    if type(self.IsShown) == "function" and self:IsShown() then
      self:RegisterFromRegistrationInfo(info)
    end
    return info
  end
end

if type(CallbackRegistrantMixin.RemoveStaticEventMethod) ~= "function" then
  function CallbackRegistrantMixin:RemoveStaticEventMethod(callbackRegistry, event, _handlerMethod)
    local handlers = self:GetStaticCallbackRegistrantHandlers()
    for index, info in ipairs(handlers) do
      if info.callbackRegistry == callbackRegistry and info.event == event then
        self:UnregisterFromRegistrationInfo(info)
        table.remove(handlers, index)
        break
      end
    end
  end
end

if type(CallbackRegistrantMixin.UnregisterAllEventMethods) ~= "function" then
  function CallbackRegistrantMixin:UnregisterAllEventMethods()
    self:UnregisterAllInternal(self:GetDynamicCallbackRegistrantHandlers())
    self:UnregisterAllInternal(self:GetStaticCallbackRegistrantHandlers())
  end
end

if type(CallbackRegistrantMixin.OnShow) ~= "function" then
  function CallbackRegistrantMixin:OnShow()
    for _, info in ipairs(self:GetDynamicCallbackRegistrantHandlers()) do
      self:RegisterFromRegistrationInfo(info)
    end
  end
end

if type(CallbackRegistrantMixin.OnHide) ~= "function" then
  function CallbackRegistrantMixin:OnHide()
    self:UnregisterAllInternal(self:GetDynamicCallbackRegistrantHandlers())
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CALLBACK_REGISTRY_DEFAULTS_LUA)?;
    Ok(())
}

/// Addon-load variant: goes through `LoaderEnv::with_state` so a runtime
/// `LoadAddOn` initiated from inside Lua (RefCell already mutably borrowed by
/// the outer entry point) doesn't panic on a second `borrow_mut`.
pub(crate) fn patch_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) -> crate::Result<()> {
    env.exec(CALLBACK_REGISTRY_DEFAULTS_LUA)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_callback_registry_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(CallbackRegistryMixin.RegisterCallback) ~= "function" then return "register" end
                if type(CVarCallbackRegistry.SetCVarCachable) ~= "function" then return "cvar" end
                if type(CallbackRegistrantMixin.AddStaticEventMethod) ~= "function" then return "registrant" end

                local owner = { count = 0 }
                local registry = CreateFromMixins(CallbackRegistryMixin)
                registry:OnLoad()
                local handle = registry:RegisterCallback("Example.Event", function(self, value)
                  self.count = self.count + value
                end, owner)
                registry:TriggerEvent("Example.Event", 2)
                if owner.count ~= 2 then return "trigger" end
                registry:UnregisterCallback("Example.Event", handle)
                registry:TriggerEvent("Example.Event", 2)
                if owner.count ~= 2 then return "unregister" end

                local registrant = CreateFromMixins(CallbackRegistrantMixin)
                registrant:AddStaticEventMethod(registry, "Example.Event", function(self, value)
                  self.count = (self.count or 0) + value
                end)
                registry:TriggerEvent("Example.Event", 3)
                if registrant.count ~= 3 then return "static" end

                CVarCallbackRegistry:SetCVarCachable("testCVar")
                if CVarCallbackRegistry.__cvars.testCVar ~= true then return "cvar_state" end
                return "ok"
                "#,
            )
            .expect("callback registry probe should run");

        assert_eq!(result, "ok");
    }
}
