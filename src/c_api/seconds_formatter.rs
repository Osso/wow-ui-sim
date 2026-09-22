//! Modeled numeric configuration for simulator-owned SecondsFormatter proxies.
//!
//! This source shares the temporary factory's Lua chunk so the installer stays
//! local rather than exposing a new global or C_StringUtil helper API.

#[cfg(feature = "retail-12-1-5")]
mod render;

/// The pinned current Retail, PTR, and Forever API declares this numeric mode.
/// Historical profiles retain their existing formatter surface.
pub(crate) fn register_enums(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<()> {
    if !cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        return Ok(());
    }
    use crate::lua_api::methods::{create_table, table_set_static};
    use rilua::Val;

    let enums = super::helpers::ensure_namespace(state, "Enum")?;
    let values = create_table(state);
    for (name, value) in [
        ("Preserve", 0.0),
        ("Strip", 1.0),
        ("StripIgnoreLocale", 2.0),
    ] {
        table_set_static(state, values, name, Val::Num(value));
    }
    table_set_static(
        state,
        Val::Table(enums),
        "SecondsFormatterIntervalWhitespace",
        values,
    );
    let metadata = create_table(state);
    for (name, value) in [("MinValue", 0.0), ("MaxValue", 2.0), ("NumValues", 3.0)] {
        table_set_static(state, metadata, name, Val::Num(value));
    }
    let enum_meta = super::helpers::ensure_namespace(state, "EnumMeta")?;
    table_set_static(
        state,
        Val::Table(enum_meta),
        "SecondsFormatterIntervalWhitespace",
        metadata,
    );
    Ok(())
}

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

  if Enum.SecondsFormatterIntervalWhitespace ~= nil then
    function methods:SetStripIntervalWhitespace(mode)
      require_number(mode)
      if mode ~= 0 and mode ~= 1 and mode ~= 2 then
        error("SecondsFormatter whitespace mode must be an integer from 0 to 2", 2)
      end
      configuration(self).stripIntervalWhitespace = mode
    end

    function methods:GetStripIntervalWhitespace()
      return configuration(self).stripIntervalWhitespace
    end
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
    configurations[object] = {
      approximationSeconds = 0, millisecondsThreshold = 0, stripIntervalWhitespace = 0,
    }
    object.minInterval = 0 -- Seconds
    object.maxInterval = 3 -- Days
    object.desiredUnitCount = 1
    return object
  end
end
"#;
