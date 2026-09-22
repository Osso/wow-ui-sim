-- Simulator policy: select a contiguous interval window, round its last unit,
-- then normalize carries. Native defaults and boundary rules are unverified.
local function __wow_install_seconds_formatter_format(methods, render_duration_units, read_number, wrap_value)
  local type, error = type, error
  local floor, abs, min, max, huge = math.floor, math.abs, math.min, math.max, math.huge
  local durations = {[0] = 1, 60, 3600, 86400}

  local function enum_option(value, default, maximum, label)
    if value == nil then return default end
    if type(value) ~= "number" then error("SecondsFormatter " .. label .. " must be an enum", 3) end
    local integral = value == floor(value)
    local in_range = value >= 0 and value <= maximum
    if not integral or not in_range then error("SecondsFormatter invalid " .. label, 3) end
    return value
  end

  local function selected_interval(seconds, minimum, maximum)
    local interval = maximum
    while interval > minimum and seconds < durations[interval] do
      interval = interval - 1
    end
    return interval
  end

  local function last_interval(first, minimum, count)
    return max(minimum, first - min(count, 4) + 1)
  end

  local function round_seconds(seconds, last, fraction_digits, round_up)
    local quantum = durations[last]
    if fraction_digits == 3 and last == 0 then quantum = 0.001 end
    local scaled = seconds / quantum
    if scaled == huge then error("SecondsFormatter precision overflow", 3) end
    local rounded = (round_up and -floor(-scaled) or floor(scaled)) * quantum
    if rounded == huge then error("SecondsFormatter rounding overflow", 3) end
    return rounded
  end

  local function parts_for(seconds, first, last, fraction_digits)
    local parts = {}
    for interval = first, last, -1 do
      local value = seconds / durations[interval]
      if interval ~= last then value = floor(value) end
      seconds = max(0, seconds - value * durations[interval])
      local precision = interval == 0 and fraction_digits or 0
      if value > 0 then parts[#parts + 1] = {value, interval, precision} end
    end
    if #parts == 0 then parts[1] = {0, last, 0} end
    return parts
  end

  local function options_for(object, seconds, abbreviation)
    local minimum = methods.EvaluateMinInterval(object, seconds)
    local maximum, secret_maximum = read_number(methods.EvaluateMaxInterval(object, seconds))
    if minimum > maximum then error("SecondsFormatter minimum exceeds maximum", 3) end
    local width = enum_option(abbreviation, nil, 2, "abbreviation")
    if width == nil then width = enum_option(object.defaultAbbreviation, 0, 2, "abbreviation") end
    local rounding = enum_option(object.rounding, 1, 1, "rounding")
    local can_round_up = object.canRoundUpLastUnit
    if can_round_up == nil then can_round_up = true end
    if type(can_round_up) ~= "boolean" then error("SecondsFormatter round-up flag must be boolean", 3) end
    return minimum, maximum, methods.EvaluateDesiredUnitCount(object, seconds), width,
        rounding == 0 and can_round_up, secret_maximum
  end

  function methods:Format(input, abbreviation)
    -- Authenticate the receiver before dispatch, and never send decoded time to
    -- a configurable curve or addon conversion function.
    local approximate = methods.CanApproximate(self, input)
    local seconds, secret = read_number(input)
    local magnitude = abs(seconds)
    local prefix = seconds < 0 and "-" or ""
    if approximate then
      magnitude = methods.GetApproximationSeconds(self)
      prefix = "< "
    end
    local evaluation = secret and wrap_value(magnitude) or magnitude
    local minimum, maximum, count, width, round_up, secret_option = options_for(self, evaluation, abbreviation)
    secret = secret or secret_option
    local first = selected_interval(magnitude, minimum, maximum)
    local last = last_interval(first, minimum, count)
    local milliseconds = magnitude > 0 and magnitude < methods.GetMillisecondsThreshold(self)
    local precision = milliseconds and 3 or 0
    local rounded = round_seconds(magnitude, last, precision, round_up)
    first = selected_interval(rounded, minimum, maximum)
    last = last_interval(first, minimum, count)
    local parts = parts_for(rounded, first, last, precision)
    local text = prefix .. render_duration_units(parts, width, methods.GetStripIntervalWhitespace(self))
    return secret and wrap_value(text) or text
  end
  -- Inferred NumericFormatter interface required by unchanged AuraContainer consumers.
  methods.FormatNumber = methods.Format
end
