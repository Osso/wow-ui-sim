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

function methods:SetDefaultAbbreviation(value) set_value(self, "defaultAbbreviation", value) end
function methods:SetRounding(value) set_value(self, "rounding", value) end
function methods:SetCanRoundUpLastUnit(value) set_value(self, "canRoundUpLastUnit", value) end
function methods:SetMinInterval(value) set_value(self, "minInterval", value) end
function methods:SetMaxInterval(value)
  set_value(self, "maxInterval", value)
  set_value(self, "maxIntervalCurve", nil)
end
function methods:SetMaxIntervalCurve(value) set_value(self, "maxIntervalCurve", value) end
function methods:SetDesiredUnitCount(value) set_value(self, "desiredUnitCount", value) end

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

function methods:EvaluateMinInterval(seconds)
  local values = require_evaluation(self, seconds)
  return require_interval(values.minInterval)
end

function methods:EvaluateMaxInterval(seconds)
  local values = require_evaluation(self, seconds)
  local curve = values.maxIntervalCurve
  if curve ~= nil then
    -- Keep authenticated wrappers intact at the configurable callback boundary.
    local maximum, secret = read_number(curve:Evaluate(seconds))
    local interval = require_interval(maximum)
    return secret and wrap_value(interval) or interval
  end
  return require_interval(values.maxInterval)
end

function methods:EvaluateDesiredUnitCount(seconds)
  local values = require_evaluation(self, seconds)
  local count = require_number(values.desiredUnitCount)
  if count ~= floor(count) or count < 1 then
    error("SecondsFormatter desired unit count must be a positive integer", 3)
  end
  return count
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
