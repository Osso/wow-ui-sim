local new_configuration, write_configuration, read_number, wrap_value, render_duration_units, render_numeric_text = ...
local type, error, setmetatable, getmetatable, newproxy = type, error, setmetatable, getmetatable, newproxy
local floor, huge, tostring = math.floor, math.huge, tostring
local configurations = setmetatable({}, { __mode = "k" })
local methods = {}

local function configuration(object)
  local values = configurations[object]
  if values == nil then error("SecondsFormatter configuration requires a formatter receiver", 3) end
  return values
end

local function require_number(value)
  if type(value) ~= "number" or value ~= value or value == huge or value == -huge then
    error("SecondsFormatter configuration requires a finite number", 3)
  end
  return value
end

local function set_value(object, key, value)
  write_configuration(configuration(object), key, value)
end

local function set_number(object, key, value)
  configuration(object)
  set_value(object, key, require_number(value))
end

local function require_interval(value)
  require_number(value)
  if value ~= floor(value) or value < 0 or value > 3 then
    error("SecondsFormatter interval must be an integer from 0 to 3", 3)
  end
  return value
end

local function require_evaluation(object, seconds)
  local values = configuration(object)
  local number = read_number(seconds)
  return values, number
end

local function set_boolean(object, key, value)
  configuration(object)
  if type(value) ~= "boolean" then error("SecondsFormatter " .. key .. " requires a boolean", 3) end
  set_value(object, key, value)
end

-- A static value and its curve are alternatives: setting one clears the other,
-- and the static getter reports nil while a curve is configured.
local function set_static(object, key, value)
  set_value(object, key, value)
  set_value(object, key .. "Curve", nil)
end

local function static_value(object, key)
  local values = configuration(object)
  if values[key .. "Curve"] ~= nil then return nil end
  return values[key]
end

function methods:SetDefaultAbbreviation(value) set_value(self, "defaultAbbreviation", value) end
function methods:GetDefaultAbbreviation() return configuration(self).defaultAbbreviation end
function methods:SetRounding(value) set_value(self, "rounding", value) end
function methods:SetCanRoundUpLastUnit(value) set_value(self, "canRoundUpLastUnit", value) end
function methods:CanRoundUpLastUnit()
  -- Format treats an unset flag as enabled.
  return configuration(self).canRoundUpLastUnit ~= false
end
function methods:SetCanRoundUpIntervals(value) set_boolean(self, "canRoundUpIntervals", value) end
function methods:CanRoundUpIntervals() return configuration(self).canRoundUpIntervals end
function methods:SetConvertToLower(value) set_boolean(self, "convertToLower", value) end
function methods:GetConvertToLower() return configuration(self).convertToLower end
function methods:SetMinInterval(value) set_static(self, "minInterval", value) end
function methods:GetMinInterval() return static_value(self, "minInterval") end
function methods:SetMinIntervalCurve(value) set_value(self, "minIntervalCurve", value) end
function methods:GetMinIntervalCurve() return configuration(self).minIntervalCurve end
function methods:SetMaxInterval(value) set_static(self, "maxInterval", value) end
function methods:GetMaxInterval() return static_value(self, "maxInterval") end
function methods:SetMaxIntervalCurve(value) set_value(self, "maxIntervalCurve", value) end
function methods:GetMaxIntervalCurve() return configuration(self).maxIntervalCurve end
function methods:SetDesiredUnitCount(value) set_static(self, "desiredUnitCount", value) end
function methods:GetDesiredUnitCount() return static_value(self, "desiredUnitCount") end
function methods:SetDesiredUnitCountCurve(value) set_value(self, "desiredUnitCountCurve", value) end
function methods:GetDesiredUnitCountCurve() return configuration(self).desiredUnitCountCurve end
function methods:Reset()
  configuration(self)
  configurations[self] = new_configuration()
end

if Enum.SecondsFormatterIntervalWhitespace ~= nil then
  function methods:SetStripIntervalWhitespace(mode)
    configuration(self)
    require_number(mode)
    if mode ~= 0 and mode ~= 1 and mode ~= 2 then
      error("SecondsFormatter whitespace mode must be an integer from 0 to 2", 2)
    end
    set_value(self, "stripIntervalWhitespace", mode)
  end
  function methods:GetStripIntervalWhitespace() return configuration(self).stripIntervalWhitespace end
  function methods:GetRounding() return configuration(self).rounding end
end

function methods:SetApproximationSeconds(value) set_number(self, "approximationSeconds", value) end
function methods:GetApproximationSeconds() return configuration(self).approximationSeconds end
function methods:SetMillisecondsThreshold(value) set_number(self, "millisecondsThreshold", value) end
function methods:GetMillisecondsThreshold() return configuration(self).millisecondsThreshold end

function methods:CanApproximate(seconds)
  local values, number = require_evaluation(self, seconds)
  return number > 0 and number < values.approximationSeconds
end

local function require_unit_count(count)
  require_number(count)
  if count ~= floor(count) or count < 1 then
    error("SecondsFormatter desired unit count must be a positive integer", 3)
  end
  return count
end

-- Evaluates the configured curve for `key`, or validates its static value.
local function evaluate_setting(object, seconds, key, validate)
  local values = require_evaluation(object, seconds)
  local curve = values[key .. "Curve"]
  if curve ~= nil then
    -- Keep authenticated wrappers intact at the configurable callback boundary.
    local value, secret = read_number(curve:Evaluate(seconds))
    value = validate(value)
    return secret and wrap_value(value) or value
  end
  return validate(values[key])
end

function methods:EvaluateMinInterval(seconds)
  return evaluate_setting(self, seconds, "minInterval", require_interval)
end

function methods:EvaluateMaxInterval(seconds)
  return evaluate_setting(self, seconds, "maxInterval", require_interval)
end

function methods:EvaluateDesiredUnitCount(seconds)
  return evaluate_setting(self, seconds, "desiredUnitCount", require_unit_count)
end

if render_duration_units ~= nil then
  __wow_install_seconds_formatter_format(methods, render_duration_units, read_number, wrap_value)
else
  -- Unchanged, explicitly unsupported formatting on profiles without the
  -- native capability. This is not a failed-render fallback for PTR/Forever.
  function methods:Format(seconds)
    configuration(self)
    return tostring(seconds or 0)
  end
  -- Common numeric contract retains this profile's existing primitive-number
  -- output without decoding secret time for an arbitrary tostring metamethod.
  function methods:FormatNumber(input)
    configuration(self)
    return render_numeric_text(input)
  end
end

function methods:FormatZero(abbreviation) return methods.Format(self, 0, abbreviation) end

local prototype = newproxy(true)
local metatable = getmetatable(prototype)
metatable.__index = function(object, key)
  local values = configuration(object)
  local method = methods[key]
  if method ~= nil then return method end
  return values[key]
end
metatable.__newindex = function() error("SecondsFormatter fields are read-only", 2) end
metatable.__metatable = false

function C_StringUtil.CreateSecondsFormatter()
  local object = newproxy(prototype)
  configurations[object] = new_configuration()
  return object
end

-- Host consumers retain these closures, not the configuration store or public
-- property lookup. Identity validation receives only a receiver, never timing.
return function(object) return configurations[object] ~= nil end, methods.FormatNumber
