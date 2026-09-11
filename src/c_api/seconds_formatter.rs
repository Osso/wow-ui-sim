//! Modeled numeric configuration for simulator-owned SecondsFormatter proxies.
//!
//! This source shares the temporary factory's Lua chunk so the installer stays
//! local rather than exposing a new global or C_StringUtil helper API.

#[cfg(feature = "retail-12-1-5")]
mod render;

pub(crate) const FORMAT_LUA: &str = include_str!("seconds_formatter/format.lua");

pub(crate) fn renderer(state: &mut rilua::vm::state::LuaState) -> rilua::Val {
    #[cfg(feature = "retail-12-1-5")]
    {
        render::callback(state)
    }
    #[cfg(not(feature = "retail-12-1-5"))]
    {
        let _ = state;
        rilua::Val::Nil
    }
}

pub(crate) const CONFIGURATION_LUA: &str = r#"
local function __wow_install_seconds_formatter_configuration(methods, render_duration_units)
  local configurations = setmetatable({}, { __mode = "k" })

  local function configuration(object)
    local values = configurations[object]
    if values == nil then
      error("SecondsFormatter configuration requires a formatter receiver", 3)
    end
    return values
  end

  local function require_number(value)
    local is_number = type(value) == "number"
    local is_nan = value ~= value
    local is_infinite = value == math.huge or value == -math.huge
    local is_finite_number = is_number and not is_nan and not is_infinite
    if not is_finite_number then
      error("SecondsFormatter configuration requires a finite number", 3)
    end
    return value
  end

  local function set_number(object, key, value)
    configuration(object)[key] = require_number(value)
  end

  local function require_interval(value)
    require_number(value)
    local is_integer = value == math.floor(value)
    local in_range = value >= 0 and value <= 3
    if not is_integer or not in_range then
      error("SecondsFormatter interval must be an integer from 0 to 3", 3)
    end
    return value
  end

  local function require_evaluation(object, seconds)
    local values = configuration(object)
    require_number(seconds)
    return values
  end

  function methods:SetApproximationSeconds(seconds)
    set_number(self, "approximationSeconds", seconds)
  end

  function methods:GetApproximationSeconds()
    return configuration(self).approximationSeconds
  end

  function methods:SetMillisecondsThreshold(threshold)
    set_number(self, "millisecondsThreshold", threshold)
  end

  function methods:GetMillisecondsThreshold()
    return configuration(self).millisecondsThreshold
  end

  function methods:CanApproximate(seconds)
    local values = require_evaluation(self, seconds)
    return seconds > 0 and seconds < values.approximationSeconds
  end

  function methods:EvaluateMinInterval(seconds)
    require_evaluation(self, seconds)
    return require_interval(self.minInterval)
  end

  function methods:EvaluateMaxInterval(seconds)
    require_evaluation(self, seconds)
    local curve = self.maxIntervalCurve
    if curve ~= nil then
      return require_interval(curve:Evaluate(seconds))
    end
    return require_interval(self.maxInterval)
  end

  function methods:EvaluateDesiredUnitCount(seconds)
    require_evaluation(self, seconds)
    local count = require_number(self.desiredUnitCount)
    local is_integer = count == math.floor(count)
    if not is_integer or count < 1 then
      error("SecondsFormatter desired unit count must be a positive integer", 3)
    end
    return count
  end

  if render_duration_units ~= nil then
    __wow_install_seconds_formatter_format(methods, render_duration_units)
  end

  return function(object)
    configurations[object] = { approximationSeconds = 0, millisecondsThreshold = 0 }
    object.minInterval = 0 -- Seconds
    object.maxInterval = 3 -- Days
    object.desiredUnitCount = 1
    return object
  end
end
"#;
