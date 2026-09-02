//! Temporary Lua proxy-object factories for unmodeled C API userdata-like objects.
//!
//! `C_CurveUtil` and `C_FunctionContainers` should eventually be backed by real
//! simulator-side object types. Until then, keep the table-shaped Lua
//! compatibility objects in the workaround layer instead of central runtime
//! bootstrap.

const PROXY_OBJECT_FACTORIES_LUA: &str = r#"
C_CurveUtil = C_CurveUtil or __wow_namespace({
  CreateCurve = nil,
  CreateColorCurve = nil,
})

C_FunctionContainers = C_FunctionContainers or __wow_namespace({
  CreateCallback = nil,
})

C_StringUtil = C_StringUtil or __wow_namespace({
  CreateSecondsFormatter = nil,
})

ProxyUtil = ProxyUtil or {}
ProxyConvertableMixin = ProxyConvertableMixin or {}
ProxyUtil.CreateProxy = ProxyUtil.CreateProxy or function(value) return value end
ProxyUtil.CreateProxyMixin = ProxyUtil.CreateProxyMixin or function() return {} end
ProxyUtil.SetPrivateReference = ProxyUtil.SetPrivateReference or __wow_noop
ProxyUtil.ReleasePrivateReference = ProxyUtil.ReleasePrivateReference or __wow_noop

if type(ProxyConvertableMixin.Init) ~= "function" then
  function ProxyConvertableMixin:Init(proxy, proxies, permitOverwrite)
    self.proxy = proxy or self
    if proxies and type(proxies.AddProxy) == "function" then
      proxies:AddProxy(self, permitOverwrite)
    end
    self.__proxy_tags = self.__proxy_tags or {}
    return self.__proxy_tags
  end
end

if type(ProxyConvertableMixin.ToProxy) ~= "function" then
  function ProxyConvertableMixin:ToProxy()
    return self.proxy or self
  end
end

if type(ProxyUtil.CreateProxyDirectory) ~= "function"
  or type(ProxyUtil.CreateProxyDirectory().AddProxy) ~= "function"
then
  function ProxyUtil.CreateProxyDirectory()
    local proxies = {
      __private_by_public = setmetatable({}, { __mode = "k" }),
      __public_by_private = setmetatable({}, { __mode = "k" }),
    }

    function proxies:AddProxy(object, _permitOverwrite)
      local public = object and type(object.ToProxy) == "function" and object:ToProxy() or object
      if public ~= nil then
        self.__private_by_public[public] = object
        self.__public_by_private[object] = public
      end
    end

    function proxies:RemoveProxy(public)
      local private = self.__private_by_public[public]
      self.__private_by_public[public] = nil
      if private ~= nil then
        self.__public_by_private[private] = nil
      end
    end

    function proxies:ToPrivate(public)
      return self.__private_by_public[public] or public
    end

    function proxies:ToPublic(private)
      return self.__public_by_private[private] or private
    end

    return proxies
  end
end

local __wow_proxy_object_id = 1

local function __wow_next_proxy_label(prefix)
  local label = prefix .. ":" .. tostring(__wow_proxy_object_id)
  __wow_proxy_object_id = __wow_proxy_object_id + 1
  return label
end

local function __wow_make_proxy_object(prefix, methods, initial_state)
  local object = initial_state or {}
  local label = __wow_next_proxy_label(prefix)
  return setmetatable(object, {
    __index = function(t, key)
      local value = rawget(t, key)
      if value ~= nil then
        return value
      end
      return methods[key]
    end,
    __newindex = function(t, key, value)
      if methods[key] ~= nil then
        error("read-only key: " .. tostring(key), 2)
      end
      rawset(t, key, value)
    end,
    __tostring = function()
      return label
    end,
  })
end

local function __wow_clone_proxy_points(points)
  local copy = {}
  for index = 1, #(points or {}) do
    local point = points[index]
    copy[index] = {
      x = point.x,
      y = point.y,
    }
  end
  return copy
end

local function __wow_copy_proxy_table(source)
  local copy = {}
  if type(source) ~= "table" then
    return copy
  end
  for key, value in pairs(source) do
    copy[key] = value
  end
  return copy
end

local function __wow_curve_methods(prefix)
  local methods = {}

  function methods:AddPoint(x, y)
    self.points[#self.points + 1] = { x = x or 0, y = y or 0 }
  end

  function methods:ClearPoints()
    self.points = {}
  end

  function methods:SetType(curveType)
    self.curveType = curveType or 0
  end

  function methods:GetPointCount()
    return #self.points
  end

  function methods:Evaluate(x)
    local points = self.points
    if #points == 0 then
      return 0
    end
    if #points == 1 then
      return points[1].y
    end

    local target = x or 0
    for index = 1, #points - 1 do
      local left = points[index]
      local right = points[index + 1]
      if target <= right.x then
        local dx = right.x - left.x
        if dx == 0 then
          return right.y
        end
        local fraction = (target - left.x) / dx
        return left.y + (right.y - left.y) * fraction
      end
    end

    return points[#points].y
  end

  function methods:Copy()
    return __wow_make_proxy_object(prefix, methods, {
      points = __wow_clone_proxy_points(self.points),
      curveType = self.curveType,
    })
  end

  return methods
end

if rawget(C_CurveUtil, "CreateCurve") == nil then
  local curveMethods = __wow_curve_methods("LuaCurveObject")
  function C_CurveUtil.CreateCurve()
    return __wow_make_proxy_object("LuaCurveObject", curveMethods, {
      points = {},
      curveType = 0,
    })
  end
end

if rawget(C_CurveUtil, "CreateColorCurve") == nil then
  local colorCurveMethods = __wow_curve_methods("LuaColorCurveObject")
  function C_CurveUtil.CreateColorCurve()
    return __wow_make_proxy_object("LuaColorCurveObject", colorCurveMethods, {
      points = {},
      curveType = 0,
    })
  end
end

if rawget(C_StringUtil, "CreateSecondsFormatter") == nil then
  -- Surface taken from Blizzard_APIDocumentationGenerated/SecondsFormatterAPIDocumentation.lua.
  -- Blizzard_AuraContainer/Blizzard_AuraContainerShared.lua calls SetRounding at
  -- line 94, at file scope; a missing method there costs the whole file. Setters
  -- store, getters read back, predicates read the same field; only Format
  -- produces text.
  local secondsFormatterMethods = {}

  local properties = {
    "ApproximationSeconds", "CanApproximate", "CanRoundUpIntervals",
    "CanRoundUpLastUnit", "ConvertToLower", "DefaultAbbreviation",
    "DesiredUnitCount", "DesiredUnitCountCurve", "MaxInterval",
    "MaxIntervalCurve", "MillisecondsThreshold", "MinInterval",
    "MinIntervalCurve", "Rounding", "StripIntervalWhitespace",
  }

  for _, name in ipairs(properties) do
    local field = name:sub(1, 1):lower() .. name:sub(2)
    secondsFormatterMethods["Set" .. name] = function(self, value)
      self[field] = value
    end
    secondsFormatterMethods["Get" .. name] = function(self)
      return self[field]
    end
  end

  -- Documented as predicates rather than getters.
  for _, name in ipairs({ "CanApproximate", "CanRoundUpIntervals", "CanRoundUpLastUnit" }) do
    local field = name:sub(1, 1):lower() .. name:sub(2)
    secondsFormatterMethods[name] = function(self)
      return self[field] == true
    end
  end

  -- Documented as evaluating the configured interval/count for a given seconds
  -- value; without a modeled curve the configured value is the answer.
  secondsFormatterMethods.EvaluateMinInterval = function(self)
    return self.minInterval
  end
  secondsFormatterMethods.EvaluateMaxInterval = function(self)
    return self.maxInterval
  end
  secondsFormatterMethods.EvaluateDesiredUnitCount = function(self)
    return self.desiredUnitCount
  end

  function secondsFormatterMethods:Reset()
    for _, name in ipairs(properties) do
      self[name:sub(1, 1):lower() .. name:sub(2)] = nil
    end
  end

  function secondsFormatterMethods:Format(seconds)
    return tostring(seconds or 0)
  end

  function secondsFormatterMethods:FormatZero()
    return self:Format(0)
  end

  function C_StringUtil.CreateSecondsFormatter()
    return __wow_make_proxy_object("SecondsFormatter", secondsFormatterMethods, {})
  end
end

if rawget(C_FunctionContainers, "CreateCallback") == nil then
  -- Userdata-backed C objects, mirroring retail (verified via
  -- docs/addons/TimerCallbackProbe + Interface/AddOns/FcTest): type()=="userdata",
  -- getmetatable()==false, read-only methods, per-instance field storage, and
  -- proxy/handle equality (a fired ticker's argument == the handle but is a
  -- distinct table key, sharing the handle's fields). Modeled on wowless's
  -- luaobjects: one shared metatable per object kind (built via newproxy(true)),
  -- instances via newproxy(prototype), and a weak map from each userdata to its
  -- backing state table. __eq compares backing identity, so a proxy and its
  -- source compare equal while remaining distinct as raw table keys.
  local backing = setmetatable({}, { __mode = "k" })

  local function buildPrototype(methods)
    local proto = newproxy(true)
    local mt = getmetatable(proto)
    mt.__index = function(u, key)
      local method = methods[key]
      if method ~= nil then
        return method
      end
      local state = backing[u]
      return state and state[key]
    end
    mt.__newindex = function(u, key, value)
      if methods[key] ~= nil then
        error("attempt to assign read-only key '" .. tostring(key) .. "'", 2)
      end
      local state = backing[u]
      if state then
        state[key] = value
      end
    end
    mt.__eq = function(a, b)
      return backing[a] == backing[b]
    end
    mt.__metatable = false -- set last; getmetatable() returns false afterwards
    return proto
  end

  -- A fresh object: distinct userdata with its own backing state table.
  local function newObject(proto, initialState)
    local u = newproxy(proto)
    backing[u] = initialState or {}
    return u
  end

  -- A proxy of an object: distinct userdata sharing the SAME backing, so it
  -- compares == to the source and shares fields, but is a distinct table key.
  local function newProxyOf(u)
    local state = backing[u]
    if not state then
      return u
    end
    local p = newproxy(u) -- shares u's metatable
    backing[p] = state
    return p
  end

  -- Retail rejects C functions as callbacks. debug.getinfo reports what=="C"
  -- for native functions and "Lua"/"main" for Lua closures (works for closures
  -- with upvalues, unlike string.dump).
  local function isLuaFunction(fn)
    if type(fn) ~= "function" then
      return false
    end
    local info = debug.getinfo(fn, "S")
    return info ~= nil and info.what ~= "C"
  end

  local methods = {}

  -- Cancelling a container cancels every timer it backs. One container can back
  -- multiple timers (the same callback object fed into multiple C_Timer.New*
  -- calls), so the bound timer handles are tracked in a list.
  function methods:Cancel()
    self._cancelled = true
    local handles = self._timerHandles
    if handles then
      for index = 1, #handles do
        local handle = handles[index]
        if type(handle) == "table" and type(handle.Cancel) == "function" then
          handle:Cancel()
        end
      end
    end
  end

  function methods:IsCancelled()
    return self._cancelled == true
  end

  -- Invoke calls the wrapped function and returns nothing (retail/wowless).
  function methods:Invoke(...)
    if self._cancelled then
      return
    end
    local callback = self._callback
    if type(callback) == "function" then
      callback(...)
    end
  end

  local containerProto = buildPrototype(methods)

  function C_FunctionContainers.CreateCallback(fn)
    if not isLuaFunction(fn) then
      error("Usage: C_FunctionContainers.CreateCallback(callback)", 2)
    end
    return newObject(containerProto, { _callback = fn, _cancelled = false })
  end

  -- Real-WoW C_Timer contract (verified on retail 12.0.7 via
  -- docs/addons/TimerCallbackProbe): a ticker IS a FunctionContainer.
  -- C_Timer.After/NewTimer/NewTicker accept either a plain function or a
  -- FunctionContainer as the callback, and NewTimer/NewTicker return the
  -- callback container itself. Feeding a returned ticker back into another
  -- C_Timer.New* call therefore reuses the same callback object, while each
  -- registration keeps its own independent iteration count.
  --
  -- The Rust C_Timer engine schedules and fires plain functions, so this layer
  -- only adds the container contract on top: coerce the callback to a
  -- container, register its function, and hand back the container.
  if C_Timer and rawget(C_Timer, "__wow_container_wrapped") == nil then
    local CreateCallback = C_FunctionContainers.CreateCallback
    local rawAfter = C_Timer.After
    local rawNewTimer = C_Timer.NewTimer
    local rawNewTicker = C_Timer.NewTicker

    local function asContainer(callback, fnName)
      local kind = type(callback)
      if kind == "function" then
        return CreateCallback(callback)
      end
      if kind == "userdata" and backing[callback] then
        return callback
      end
      error("bad argument #2 to '" .. fnName .. "' (function or callback expected)", 3)
    end

    local function bindHandle(container, handle)
      local handles = container._timerHandles
      if not handles then
        handles = {}
        container._timerHandles = handles
      end
      handles[#handles + 1] = handle
      -- The container has no id of its own (retail tickers are opaque), but the
      -- Rust engine and timer-queue test helpers locate a pending timer by id.
      -- Surface the underlying timer's __id; for a reused container backing
      -- multiple timers, the most recent registration wins.
      if type(handle) == "table" then
        container.__id = handle.__id
      end
    end

    -- The engine fires this closure each tick; it invokes the wrapped function
    -- with a proxy of the container (retail passes a proxy of the ticker that
    -- compares == to the handle), and stops if the container was cancelled. The
    -- proxy is built once per registration to avoid per-tick allocation.
    local function makeInvoker(container)
      local fn = container._callback
      local proxy = newProxyOf(container)
      return function()
        if container._cancelled then
          return
        end
        return fn(proxy)
      end
    end

    function C_Timer.After(seconds, callback)
      local container = asContainer(callback, "After")
      return rawAfter(seconds, makeInvoker(container))
    end

    function C_Timer.NewTimer(seconds, callback)
      local container = asContainer(callback, "NewTimer")
      bindHandle(container, rawNewTimer(seconds, makeInvoker(container)))
      return container
    end

    function C_Timer.NewTicker(seconds, callback, iterations)
      local container = asContainer(callback, "NewTicker")
      bindHandle(container, rawNewTicker(seconds, makeInvoker(container), iterations))
      return container
    end

    C_Timer.__wow_container_wrapped = true
  end
end

if CreateAbbreviateConfig == nil then
  local abbreviateMethods = {}

  function abbreviateMethods:GetAbbreviateNumberData()
    return self._abbreviateNumberData
  end

  function abbreviateMethods:SetAbbreviateNumberData(data)
    self._abbreviateNumberData = data
  end

  function CreateAbbreviateConfig(initial)
    local state = __wow_copy_proxy_table(initial)
    state._abbreviateNumberData = state._abbreviateNumberData
    return __wow_make_proxy_object("AbbreviateConfig", abbreviateMethods, state)
  end
end

if CreateUnitHealPredictionCalculator == nil then
  local healPredictionMethods = {}

  local function healPredictionDefaultValues()
    return {
      health = 0,
      healthMax = 0,
      totalDamageAbsorbs = 0,
      totalHealAbsorbs = 0,
      totalIncomingHeals = 0,
      totalIncomingHealsFromHealer = 0,
    }
  end

  local function healPredictionCopyValues(values)
    values = values or {}
    return {
      health = values.health or 0,
      healthMax = values.healthMax or 0,
      totalDamageAbsorbs = values.totalDamageAbsorbs or 0,
      totalHealAbsorbs = values.totalHealAbsorbs or 0,
      totalIncomingHeals = values.totalIncomingHeals or 0,
      totalIncomingHealsFromHealer = values.totalIncomingHealsFromHealer or 0,
    }
  end

  function healPredictionMethods:Reset()
    self._damageAbsorbClampMode = 0
    self._healAbsorbClampMode = 0
    self._healAbsorbMode = 0
    self._incomingHealClampMode = 0
    self._incomingHealOverflowPercent = 1
    self._maximumHealthMode = 0
    self._predictedValues = healPredictionDefaultValues()
    self._hasSecretValues = false
  end

  function healPredictionMethods:GetIncomingHeals()
    local values = self._predictedValues or healPredictionDefaultValues()
    local total = values.totalIncomingHeals or 0
    local healer = values.totalIncomingHealsFromHealer or 0
    return total, healer, total - healer, false
  end

  function healPredictionMethods:GetDamageAbsorbs()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.totalDamageAbsorbs or 0, false
  end

  function healPredictionMethods:GetHealAbsorbs()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.totalHealAbsorbs or 0, false
  end

  function healPredictionMethods:GetDamageAbsorbClampMode()
    return self._damageAbsorbClampMode or 0
  end

  function healPredictionMethods:GetHealAbsorbClampMode()
    return self._healAbsorbClampMode or 0
  end

  function healPredictionMethods:GetHealAbsorbMode()
    return self._healAbsorbMode or 0
  end

  function healPredictionMethods:GetIncomingHealClampMode()
    return self._incomingHealClampMode or 0
  end

  function healPredictionMethods:GetIncomingHealOverflowPercent()
    return self._incomingHealOverflowPercent or 1
  end

  function healPredictionMethods:GetCurrentHealth()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.health or 0
  end

  function healPredictionMethods:GetMaximumHealth()
    local values = self._predictedValues or healPredictionDefaultValues()
    return values.healthMax or 0
  end

  function healPredictionMethods:GetMaximumHealthMode()
    return self._maximumHealthMode or 0
  end

  function healPredictionMethods:GetPredictedValues()
    return healPredictionCopyValues(self._predictedValues)
  end

  function healPredictionMethods:HasSecretValues()
    return self._hasSecretValues == true
  end

  function healPredictionMethods:ResetPredictedValues()
    self._predictedValues = healPredictionDefaultValues()
  end

  function healPredictionMethods:SetDamageAbsorbClampMode(mode)
    self._damageAbsorbClampMode = mode or 0
  end

  function healPredictionMethods:SetHealAbsorbClampMode(mode)
    self._healAbsorbClampMode = mode or 0
  end

  function healPredictionMethods:SetHealAbsorbMode(mode)
    self._healAbsorbMode = mode or 0
  end

  function healPredictionMethods:SetIncomingHealClampMode(mode)
    self._incomingHealClampMode = mode or 0
  end

  function healPredictionMethods:SetIncomingHealOverflowPercent(percent)
    self._incomingHealOverflowPercent = percent or 1
  end

  function healPredictionMethods:SetMaximumHealthMode(mode)
    self._maximumHealthMode = mode or 0
  end

  function healPredictionMethods:SetPredictedValues(values)
    self._predictedValues = healPredictionCopyValues(values)
  end

  function healPredictionMethods:SetToDefaults()
    self:Reset()
  end

  function CreateUnitHealPredictionCalculator()
    local calculator = __wow_make_proxy_object("UnitHealPredictionCalculator", healPredictionMethods, {})
    calculator:Reset()
    return calculator
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PROXY_OBJECT_FACTORIES_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_proxy_factories() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_CurveUtil.CreateCurve) ~= "function" then return "curve" end
                if type(C_CurveUtil.CreateColorCurve) ~= "function" then return "color_curve" end
                if type(C_FunctionContainers.CreateCallback) ~= "function" then return "callback" end
                if type(C_StringUtil.CreateSecondsFormatter) ~= "function" then return "seconds_formatter_factory" end
                if type(ProxyUtil.CreateProxy) ~= "function" then return "proxy" end
                if type(ProxyUtil.CreateProxyMixin) ~= "function" then return "proxy_mixin" end
                if type(ProxyUtil.CreateProxyDirectory) ~= "function" then return "proxy_directory" end
                if type(ProxyUtil.CreateProxyDirectory().AddProxy) ~= "function" then return "proxy_directory_add" end
                if type(ProxyConvertableMixin) ~= "table" then return "convertable_mixin" end
                if type(ProxyConvertableMixin.Init) ~= "function" then return "convertable_init" end
                if type(ProxyConvertableMixin.ToProxy) ~= "function" then return "convertable_to_proxy" end
                if type(CreateAbbreviateConfig) ~= "function" then return "abbreviate" end
                if type(CreateUnitHealPredictionCalculator) ~= "function" then return "heal_prediction" end
                local curve = C_CurveUtil.CreateCurve()
                curve:AddPoint(0, 10)
                curve:AddPoint(10, 20)
                if curve:Evaluate(5) ~= 15 then return "evaluate" end
                local formatter = C_StringUtil.CreateSecondsFormatter()
                if type(formatter) ~= "table" then return "seconds_formatter_type" end
                formatter:SetDefaultAbbreviation(Enum.SecondsFormatterAbbreviation.OneLetter)
                formatter:SetMinInterval(Enum.SecondsFormatterInterval.Seconds)
                formatter:SetMaxIntervalCurve(curve)
                formatter:SetDesiredUnitCount(1)
                if formatter.defaultAbbreviation ~= Enum.SecondsFormatterAbbreviation.OneLetter then return "seconds_formatter_abbrev" end
                local invoked = nil
                local callback = C_FunctionContainers.CreateCallback(function(value) invoked = value end)
                if type(callback) ~= "userdata" then return "callback_type" end
                callback:Invoke(41)
                if invoked ~= 41 then return "invoke" end
                local value = { name = "proxy-value" }
                if ProxyUtil.CreateProxy(value) ~= value then return "proxy_identity" end
                local directory = ProxyUtil.CreateProxyDirectory()
                if directory:ToPrivate(value) ~= value then return "to_private" end
                if directory:ToPublic(value) ~= value then return "to_public" end
                local private = {}
                local public = {}
                private.ToProxy = ProxyConvertableMixin.ToProxy
                ProxyConvertableMixin.Init(private, public, directory)
                if directory:ToPrivate(public) ~= private then return "registered_private" end
                if directory:ToPublic(private) ~= public then return "registered_public" end
                local config = CreateAbbreviateConfig({})
                config:SetAbbreviateNumberData({ value = 1 })
                if config:GetAbbreviateNumberData().value ~= 1 then return "config" end
                local prediction = CreateUnitHealPredictionCalculator()
                prediction:SetDamageAbsorbClampMode(2)
                if prediction:GetDamageAbsorbClampMode() ~= 2 then return "prediction" end
                return "ok"
                "#,
            )
            .expect("proxy factory probe should run");

        assert_eq!(result, "ok");
    }
}
