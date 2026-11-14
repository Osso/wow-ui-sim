//! Temporary UI color defaults.
//!
//! These values keep Blizzard startup code and addon probes working until the
//! simulator has a real color registry surface.

use rilua::LuaApiMut;

const COLOR_DEFAULTS_LUA: &str = r#"
local function __wow_make_color(r, g, b, a)
  local color = {
    r = r or 1,
    g = g or 1,
    b = b or 1,
    a = a or 1,
  }

  function color:GetRGB()
    return self.r, self.g, self.b
  end

  function color:GetRGBA()
    return self.r, self.g, self.b, self.a
  end

  local function channel_byte(value)
    return math.floor((value or 0) * 255 + 0.5)
  end

  function color:GetRGBAsBytes()
    return channel_byte(self.r), channel_byte(self.g), channel_byte(self.b)
  end

  function color:GetRGBAAsBytes()
    return channel_byte(self.r), channel_byte(self.g), channel_byte(self.b), channel_byte(self.a or 1)
  end

  function color:GenerateHexColor()
    return string.format("FF%02X%02X%02X", math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
  end

  function color:GenerateHexColorNoAlpha()
    return string.format("%02X%02X%02X", self:GetRGBAsBytes())
  end

  function color:GenerateHexColorMarkup()
    return "|c" .. self:GenerateHexColor()
  end

  function color:WrapTextInColorCode(text)
    return self:GenerateHexColorMarkup() .. tostring(text or "") .. "|r"
  end

  return color
end

if CreateColor == nil then
  function CreateColor(r, g, b, a)
    return __wow_make_color(r, g, b, a)
  end
end

local function __wow_color_merge_namespace(existing, defaults)
  local namespace = type(existing) == "table" and existing or {}
  for key, value in pairs(defaults or {}) do
    if rawget(namespace, key) == nil then
      rawset(namespace, key, value)
    end
  end
  return namespace
end

-- Simulator fallback payload; native flags apply to a separate native handler.
C_UIColor = __wow_color_merge_namespace(C_UIColor, {
  GetColors = function()
    return {
      { baseTag = "HIGHLIGHT_FONT_COLOR", color = { r = 1, g = 1, b = 1, a = 1 } },
      { baseTag = "PLAYER_FACTION_COLOR_HORDE", color = { r = 1, g = 0.1, b = 0.1, a = 1 } },
      { baseTag = "PLAYER_FACTION_COLOR_ALLIANCE", color = { r = 0.2, g = 0.4, b = 1, a = 1 } },
      { baseTag = "NORMAL_FONT_COLOR", color = { r = 1, g = 0.82, b = 0, a = 1 } },
      -- Blizzard_Professions panels look up the tradeskill experience bar
      -- fill color by baseTag in the C_UIColor.GetColors() return value.
      { baseTag = "TRADESKILL_EXPERIENCE_COLOR", color = { r = 0.25, g = 0.25, b = 0.75, a = 1 } },
    }
  end,
})

QuestDifficultyColors = QuestDifficultyColors or {}
QuestDifficultyColors.trivial = QuestDifficultyColors.trivial or { r = 0.50, g = 0.50, b = 0.50 }
QuestDifficultyColors.standard = QuestDifficultyColors.standard or { r = 0.25, g = 0.75, b = 0.25 }
QuestDifficultyColors.difficult = QuestDifficultyColors.difficult or { r = 1.00, g = 1.00, b = 0.00 }
QuestDifficultyColors.verydifficult = QuestDifficultyColors.verydifficult or { r = 1.00, g = 0.50, b = 0.25 }
QuestDifficultyColors.impossible = QuestDifficultyColors.impossible or { r = 1.00, g = 0.10, b = 0.10 }

QuestDifficultyHighlightColors = QuestDifficultyHighlightColors or {}
QuestDifficultyHighlightColors.trivial = QuestDifficultyHighlightColors.trivial or { r = 0.70, g = 0.70, b = 0.70 }
QuestDifficultyHighlightColors.standard = QuestDifficultyHighlightColors.standard or { r = 0.50, g = 1.00, b = 0.50 }
QuestDifficultyHighlightColors.difficult = QuestDifficultyHighlightColors.difficult or { r = 1.00, g = 1.00, b = 0.50 }
QuestDifficultyHighlightColors.verydifficult = QuestDifficultyHighlightColors.verydifficult or { r = 1.00, g = 0.75, b = 0.50 }
QuestDifficultyHighlightColors.impossible = QuestDifficultyHighlightColors.impossible or { r = 1.00, g = 0.40, b = 0.40 }

local function __wow_clamp_normalized(value)
  local number = tonumber(value) or 0
  return math.min(math.max(number, 0), 1)
end

local function __wow_normalize_hue(value)
  local hue = tonumber(value) or 0
  return (hue % 1 + 1) % 1
end

C_ColorUtil = __wow_color_merge_namespace(C_ColorUtil, {
  ConvertHSLToHSV = function(h, s, l)
    local hue = __wow_normalize_hue(h)
    local saturation = __wow_clamp_normalized(s)
    local lightness = __wow_clamp_normalized(l)
    local value = lightness + saturation * math.min(lightness, 1 - lightness)
    local valueSaturation = value == 0 and 0 or 2 * (1 - lightness / value)
    return hue, valueSaturation, value
  end,
  ConvertHSVToHSL = function(h, s, v)
    local hue = __wow_normalize_hue(h)
    local saturation = __wow_clamp_normalized(s)
    local value = __wow_clamp_normalized(v)
    local lightness = value * (1 - saturation / 2)
    local lightnessSaturation = (lightness == 0 or lightness == 1)
      and 0
      or (value - lightness) / math.min(lightness, 1 - lightness)
    return hue, lightnessSaturation, lightness
  end,
  ConvertHSVToRGB = function(h, s, v)
    local hue = __wow_normalize_hue(h)
    local saturation = __wow_clamp_normalized(s)
    local value = __wow_clamp_normalized(v)
    if saturation == 0 then
      return value, value, value
    end

    local sector = math.floor(hue * 6)
    local fraction = hue * 6 - sector
    local p = value * (1 - saturation)
    local q = value * (1 - saturation * fraction)
    local t = value * (1 - saturation * (1 - fraction))
    if sector % 6 == 0 then return value, t, p end
    if sector % 6 == 1 then return q, value, p end
    if sector % 6 == 2 then return p, value, t end
    if sector % 6 == 3 then return p, q, value end
    if sector % 6 == 4 then return t, p, value end
    return value, p, q
  end,
  ConvertRGBToHSV = function(r, g, b)
    local red = __wow_clamp_normalized(r)
    local green = __wow_clamp_normalized(g)
    local blue = __wow_clamp_normalized(b)
    local maximum = math.max(red, green, blue)
    local minimum = math.min(red, green, blue)
    local delta = maximum - minimum
    if delta == 0 then
      return -1, 0, maximum
    end

    local hue
    if maximum == red then
      hue = ((green - blue) / delta) % 6
    elseif maximum == green then
      hue = (blue - red) / delta + 2
    else
      hue = (red - green) / delta + 4
    end
    return (hue / 6) % 1, delta / maximum, maximum
  end,
  GenerateTextColorCode = function(color)
    local r = math.floor((color.r or 1) * 255)
    local g = math.floor((color.g or 1) * 255)
    local b = math.floor((color.b or 1) * 255)
    return string.format("ff%02x%02x%02x", r, g, b)
  end,
  WrapTextInColor = function(text, color)
    return "|c" .. C_ColorUtil.GenerateTextColorCode(color) .. tostring(text or "") .. "|r"
  end,
  WrapTextInColorCode = function(text, colorCode)
    local code = tostring(colorCode or "ffffffff"):gsub("^|c", "")
    return "|c" .. code .. tostring(text or "") .. "|r"
  end,
})

C_ColorOverrides = __wow_color_merge_namespace(C_ColorOverrides, {
  -- ColorOverridesDocumentation: ColorOverride -> nullable ColorOverrideInfo.
  -- Missing system: color override registry. Empty-state simulator policy;
  -- retire this query when c_api owns that registry and its supplied records.
  GetColorOverrideInfo = function(_overrideType)
    return nil
  end,
  GetColorForQuality = function(_quality)
    return CreateColor(1, 1, 1, 1)
  end,
})

C_PvP = __wow_color_merge_namespace(C_PvP, {
  IsInBrawl = function()
    return false
  end,
  IsSoloShuffle = function()
    return false
  end,
  GetArenaCrowdControlInfo = function(_unit)
    return nil, 0, 0
  end,
})
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(COLOR_DEFAULTS_LUA)?;
    register_secure_ui_color_fallback(lua)?;
    Ok(())
}

fn register_secure_ui_color_fallback(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }

    // UNKNOWN (R88k.7): preserve the existing simulator copy; the Lua placeholder is not the native flags-3 handler.
    let state = lua.state_mut();
    let ordinary = crate::c_api::ensure_namespace(state, "C_UIColor")?;
    let secure = crate::lua_bridge::namespace_peer(state, ordinary)?
        .ok_or_else(|| rilua::runtime_error("C_UIColor secure namespace peer missing"))?;
    let get_colors =
        crate::lua_api::methods::table_get_static(state, rilua::Val::Table(ordinary), "GetColors");
    let secure_get_colors =
        crate::lua_api::methods::table_get_static(state, rilua::Val::Table(secure), "GetColors");
    if matches!(get_colors, rilua::Val::Function(_)) && matches!(secure_get_colors, rilua::Val::Nil)
    {
        crate::lua_api::methods::table_set_static(
            state,
            rilua::Val::Table(secure),
            "GetColors",
            get_colors,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_color_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_UIColor.GetColors) ~= "function" then return "ui_colors" end
                local foundTradeskillColor = false
                for _, entry in ipairs(C_UIColor.GetColors()) do
                  if entry.baseTag == "TRADESKILL_EXPERIENCE_COLOR" then
                    foundTradeskillColor = entry.color.b == 0.75
                  end
                end
                if not foundTradeskillColor then return "tradeskill" end
                if C_ColorUtil.GenerateTextColorCode({ r = 1, g = 0.5, b = 0 }) ~= "ffff7f00" then return "text_code" end
                if C_ColorUtil.WrapTextInColorCode("Ready", "ff112233") ~= "|cff112233Ready|r" then return "wrap" end
                local color = CreateColor(0.25, 0.5, 0.75, 0.8)
                local r, g, b, a = color:GetRGBA()
                if r ~= 0.25 or g ~= 0.5 or b ~= 0.75 or a ~= 0.8 then return "rgba" end
                local rb, gb, bb, ab = color:GetRGBAAsBytes()
                if rb ~= 64 or gb ~= 128 or bb ~= 191 or ab ~= 204 then return "bytes" end
                if color:GenerateHexColor() ~= "FF3F7FBF" then return "hex" end
                if color:GenerateHexColorNoAlpha() ~= "4080BF" then return "hex_no_alpha" end
                if color:WrapTextInColorCode("Ready") ~= "|cFF3F7FBFReady|r" then return "color_wrap" end
                if QuestDifficultyColors.impossible.g ~= 0.10 then return "quest" end
                if QuestDifficultyHighlightColors.standard.g ~= 1.00 then return "highlight" end
                return "ok"
                "#,
            )
            .expect("color defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[cfg(feature = "retail-12-0-0")]
    #[test]
    fn patch_12_0_0_color_util_conversions_and_wrapping() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local function assert_close(actual, expected, label)
                  if math.abs(actual - expected) > 0.000001 then return label end
                end

                local h, s, v = C_ColorUtil.ConvertRGBToHSV(1, 0, 0)
                if assert_close(h, 0, "rgb_h") or assert_close(s, 1, "rgb_s") or assert_close(v, 1, "rgb_v") then
                  return assert_close(h, 0, "rgb_h") or assert_close(s, 1, "rgb_s") or assert_close(v, 1, "rgb_v")
                end
                h, s, v = C_ColorUtil.ConvertRGBToHSV(0.25, 0.5, 0.75)
                if assert_close(h, 0.5833333333333334, "fractional_h")
                    or assert_close(s, 0.6666666666666666, "fractional_s")
                    or assert_close(v, 0.75, "fractional_v") then
                  return assert_close(h, 0.5833333333333334, "fractional_h")
                    or assert_close(s, 0.6666666666666666, "fractional_s")
                    or assert_close(v, 0.75, "fractional_v")
                end
                h, s, v = C_ColorUtil.ConvertRGBToHSV(0, 0, 0)
                if h ~= -1 or s ~= 0 or v ~= 0 then return "black" end
                h, s, v = C_ColorUtil.ConvertRGBToHSV(1, 1, 1)
                if h ~= -1 or s ~= 0 or v ~= 1 then return "white" end

                local r, g, b = C_ColorUtil.ConvertHSVToRGB(0, 1, 1)
                if r ~= 1 or g ~= 0 or b ~= 0 then return "hsv_red" end
                r, g, b = C_ColorUtil.ConvertHSVToRGB(1 / 3, 1, 1)
                if assert_close(r, 0, "hsv_green_r") or assert_close(g, 1, "hsv_green_g") or assert_close(b, 0, "hsv_green_b") then
                  return "hsv_green"
                end
                r, g, b = C_ColorUtil.ConvertHSVToRGB(1, 0, 0.5)
                if r ~= 0.5 or g ~= 0.5 or b ~= 0.5 then return "hsv_hue_boundary" end

                h, s, v = C_ColorUtil.ConvertHSVToHSL(0, 1, 1)
                if h ~= 0 or s ~= 1 or v ~= 0.5 then return "hsv_hsl_red" end
                h, s, v = C_ColorUtil.ConvertHSVToHSL(0, 0, 0.5)
                if h ~= 0 or s ~= 0 or v ~= 0.5 then return "hsv_hsl_gray" end
                h, s, v = C_ColorUtil.ConvertHSLToHSV(0, 1, 0.5)
                if h ~= 0 or s ~= 1 or v ~= 1 then return "hsl_hsv_red" end
                h, s, v = C_ColorUtil.ConvertHSLToHSV(0, 0, 0.5)
                if h ~= 0 or s ~= 0 or v ~= 0.5 then return "hsl_hsv_gray" end

                h, s, v = C_ColorUtil.ConvertRGBToHSV(nil, nil, nil)
                if h ~= -1 or s ~= 0 or v ~= 0 then return "nil_rgb" end
                r, g, b = C_ColorUtil.ConvertHSVToRGB(nil, nil, nil)
                if r ~= 0 or g ~= 0 or b ~= 0 then return "nil_hsv" end
                h, s, v = C_ColorUtil.ConvertHSVToHSL(nil, nil, nil)
                if h ~= 0 or s ~= 0 or v ~= 0 then return "nil_hsl" end
                h, s, v = C_ColorUtil.ConvertHSLToHSV(nil, nil, nil)
                if h ~= 0 or s ~= 0 or v ~= 0 then return "nil_hsv_from_hsl" end

                if C_ColorUtil.WrapTextInColor("Ready", { r = 1, g = 0.5, b = 0 }) ~= "|cffff7f00Ready|r" then
                  return "wrap_text"
                end
                if C_ColorUtil.WrapTextInColor("", { r = 1, g = 0.5, b = 0 }) ~= "|cffff7f00|r" then
                  return "wrap_empty"
                end
                return "ok"
                "#,
            )
            .expect("ColorUtil probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_color_override_and_pvp_defaults_without_replacing_existing_methods() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local color = C_ColorOverrides.GetColorForQuality(1)
                if color.r ~= 1 or color.g ~= 1 or color.b ~= 1 or color.a ~= 1 then return "color" end
                if C_PvP.IsInBrawl() ~= false then return "brawl" end
                if C_PvP.IsSoloShuffle() ~= false then return "shuffle" end
                local spellID, startTime, duration = C_PvP.GetArenaCrowdControlInfo("player")
                if spellID ~= nil or startTime ~= 0 or duration ~= 0 then return "cc" end
                C_PvP.SetLocklistMap(566)
                if C_PvP.GetLocklistMap(1) ~= 566 then return "state-backed" end
                return "ok"
                "#,
            )
            .expect("color/PvP defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_color_override_and_pvp_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ColorOverrides.GetColorForQuality = function()
                return { r = 0.1, g = 0.2, b = 0.3, a = 0.4 }
            end
            C_PvP.IsInBrawl = function()
                return true
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                local color = C_ColorOverrides.GetColorForQuality(1)
                if color.r ~= 0.1 or color.g ~= 0.2 or color.b ~= 0.3 or color.a ~= 0.4 then return "color" end
                if C_PvP.IsInBrawl() ~= true then return "brawl" end
                return "ok"
                "#,
            )
            .expect("existing color/PvP functions should remain callable");

        assert_eq!(result, "ok");
    }
}
