//! Modeled numeric configuration for simulator-owned SecondsFormatter proxies.
//!
//! This source shares the temporary factory's Lua chunk so the installer stays
//! local rather than exposing a new global or C_StringUtil helper API.

pub(crate) const CONFIGURATION_LUA: &str = r#"
local function __wow_install_seconds_formatter_configuration(methods)
  local configurations = setmetatable({}, { __mode = "k" })

  local function configuration(object)
    local values = configurations[object]
    if values == nil then
      error("SecondsFormatter configuration requires a formatter receiver", 3)
    end
    return values
  end

  local function set_number(object, key, value)
    if type(value) ~= "number" or value ~= value
      or value == math.huge or value == -math.huge then
      error("SecondsFormatter configuration requires a finite number", 3)
    end
    configuration(object)[key] = value
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

  return function(object)
    configurations[object] = { approximationSeconds = 0, millisecondsThreshold = 0 }
    return object
  end
end
"#;
