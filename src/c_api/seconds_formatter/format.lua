-- PTR policy: select a contiguous interval window, round only its last unit,
-- then normalize carries. Native WoW defaults and boundary rules are unverified.
local function __wow_install_seconds_formatter_format(methods, render_duration_units)
  local durations = {[0] = 1, 60, 3600, 86400}

  local function enum_option(value, default, maximum, label)
    if value == nil then return default end
    if type(value) ~= "number" then error("SecondsFormatter " .. label .. " must be an enum", 3) end
    local integral = value == math.floor(value)
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
    return math.max(minimum, first - math.min(count, 4) + 1)
  end

  local function round_seconds(seconds, last, fraction_digits, round_up)
    local quantum = durations[last]
    if fraction_digits == 3 and last == 0 then quantum = 0.001 end
    local scaled = seconds / quantum
    if scaled == math.huge then error("SecondsFormatter precision overflow", 3) end
    local rounded = (round_up and math.ceil(scaled) or math.floor(scaled)) * quantum
    if rounded == math.huge then error("SecondsFormatter rounding overflow", 3) end
    return rounded
  end

  local function parts_for(seconds, first, last, fraction_digits)
    local parts = {}
    for interval = first, last, -1 do
      local value = seconds / durations[interval]
      if interval ~= last then value = math.floor(value) end
      seconds = math.max(0, seconds - value * durations[interval])
      local precision = interval == 0 and fraction_digits or 0
      if value > 0 then parts[#parts + 1] = {value, interval, precision} end
    end
    if #parts == 0 then parts[1] = {0, last, 0} end
    return parts
  end

  local function options_for(object, seconds, abbreviation)
    local minimum = object:EvaluateMinInterval(seconds)
    local maximum = object:EvaluateMaxInterval(seconds)
    if minimum > maximum then error("SecondsFormatter minimum exceeds maximum", 3) end
    local width = enum_option(abbreviation, nil, 2, "abbreviation")
    if width == nil then width = enum_option(object.defaultAbbreviation, 0, 2, "abbreviation") end
    local rounding = enum_option(object.rounding, 1, 1, "rounding")
    local can_round_up = object.canRoundUpLastUnit
    if can_round_up == nil then can_round_up = true end
    if type(can_round_up) ~= "boolean" then error("SecondsFormatter round-up flag must be boolean", 3) end
    return minimum, maximum, object:EvaluateDesiredUnitCount(seconds), width,
        rounding == 0 and can_round_up
  end

  function methods:Format(seconds, abbreviation)
    -- This both validates the receiver/seconds and consumes the stored threshold.
    local approximate = self:CanApproximate(seconds)
    local magnitude = math.abs(seconds)
    local prefix = seconds < 0 and "-" or ""
    if approximate then
      magnitude = self:GetApproximationSeconds()
      prefix = "< "
    end
    local minimum, maximum, count, width, round_up = options_for(self, magnitude, abbreviation)
    local first = selected_interval(magnitude, minimum, maximum)
    local last = last_interval(first, minimum, count)
    local milliseconds = magnitude > 0 and magnitude < self:GetMillisecondsThreshold()
    local precision = milliseconds and 3 or 0
    local rounded = round_seconds(magnitude, last, precision, round_up)
    first = selected_interval(rounded, minimum, maximum)
    last = last_interval(first, minimum, count)
    local parts = parts_for(rounded, first, last, precision)
    return prefix .. render_duration_units(parts, width)
  end
end
