//! Temporary formatting and utility globals for partial Blizzard loads.
//!
//! These helpers mirror small Blizzard utility surfaces that are expected by
//! startup Lua, but they are still shallow compatibility defaults rather than a
//! modeled formatting subsystem.

const FORMATTING_UTILITY_DEFAULTS_LUA: &str = r##"
if GetText == nil then
  function GetText(token)
    if type(token) ~= "string" then
      return token
    end
    local value = rawget(_G, token)
    return value ~= nil and value or token
  end
end

BACK = BACK or "Back"
NEXT = NEXT or "Next"
PREVIEW = PREVIEW or "Preview"
CUSTOMIZE = CUSTOMIZE or "Customize"
FINISH = FINISH or "Finish"
EVERY_X_PERCENT = EVERY_X_PERCENT or "%d%%"
TRANSMOGRIFY_TOOLTIP_APPEARANCE_KNOWN = TRANSMOGRIFY_TOOLTIP_APPEARANCE_KNOWN or "Known"
ERR_QUEST_SESSION_RESULT_RESYNC = ERR_QUEST_SESSION_RESULT_RESYNC or "Resync"
CLASS_SORT_ORDER = CLASS_SORT_ORDER or { "WARRIOR", "PALADIN", "HUNTER", "ROGUE", "PRIEST", "DEATHKNIGHT", "SHAMAN", "MAGE", "WARLOCK", "MONK", "DRUID", "DEMONHUNTER", "EVOKER" }
MAX_CLASSES = MAX_CLASSES or #CLASS_SORT_ORDER

if BreakUpLargeNumbers == nil then
  function BreakUpLargeNumbers(value)
    return tostring(value)
  end
end

if CalculateStringEditDistance == nil then
  function CalculateStringEditDistance(firstString, secondString)
    if type(firstString) ~= "string" or type(secondString) ~= "string" then
      return 0
    end
    local firstLen = #firstString
    local secondLen = #secondString
    if firstLen == 0 then return secondLen end
    if secondLen == 0 then return firstLen end

    local previousRow = {}
    for column = 0, secondLen do
      previousRow[column] = column
    end

    local currentRow = {}
    for row = 1, firstLen do
      currentRow[0] = row
      local firstChar = firstString:byte(row)
      for column = 1, secondLen do
        local substitutionCost = (firstChar == secondString:byte(column)) and 0 or 1
        local deletion = previousRow[column] + 1
        local insertion = currentRow[column - 1] + 1
        local substitution = previousRow[column - 1] + substitutionCost
        currentRow[column] = math.min(deletion, insertion, substitution)
      end
      for column = 0, secondLen do
        previousRow[column] = currentRow[column]
      end
    end

    return previousRow[secondLen]
  end
end

if strsplittable == nil then
  function strsplittable(delimiter, input, limit)
    return { strsplit(delimiter, input, limit) }
  end
end

if MergeTable == nil then
  function MergeTable(dest, src)
    if type(dest) ~= "table" or type(src) ~= "table" then
      return dest
    end
    for key, value in pairs(src) do
      dest[key] = value
    end
    return dest
  end
end

if tFilter == nil then
  function tFilter(t, predicate)
    if type(t) ~= "table" or type(predicate) ~= "function" then
      return t
    end
    local out = 1
    local len = #t
    for i = 1, len do
      local value = t[i]
      if value ~= nil and predicate(value, i, t) then
        if out ~= i then
          t[out] = value
        end
        out = out + 1
      end
    end
    for i = out, len do
      t[i] = nil
    end
    return t
  end
end

if mapvalues == nil then
  function mapvalues(fn, ...)
    local count = select("#", ...)
    if count == 0 then
      return
    end

    local values = {}
    for index = 1, count do
      values[index] = fn(select(index, ...))
    end

    return unpack(values, 1, count)
  end
end

do
  local stringMeta = getmetatable("")
  local function splitStringMethod(self, delimiterOrInput, limit)
    if type(self) == "string" and type(delimiterOrInput) == "string" then
      local inputIsEmpty = #delimiterOrInput == 0
      local inputIsLonger = #self <= 4 and #delimiterOrInput > #self
      local receiverLooksLikeDelimiter = #self <= 4 and self:match("^%W+$") ~= nil
      if inputIsEmpty or inputIsLonger or receiverLooksLikeDelimiter then
        return strsplit(self, delimiterOrInput, limit)
      end
    end
    return strsplit(delimiterOrInput, self, limit)
  end
  if type(stringMeta) == "table" then
    local stringIndex = stringMeta.__index
    if type(stringIndex) == "table" then
      function stringIndex:split(delimiter, limit)
        return splitStringMethod(self, delimiter, limit)
      end
    end

    function stringMeta:split(delimiter, limit)
      return splitStringMethod(self, delimiter, limit)
    end
  end
end

if tAppendAll == nil then
  function tAppendAll(tbl, addedArray)
    if type(tbl) ~= "table" or type(addedArray) ~= "table" then
      return tbl
    end

    for _, value in ipairs(addedArray) do
      table.insert(tbl, value)
    end

    return tbl
  end
end

if GetScreenDPIScale == nil then
  function GetScreenDPIScale()
    return 1
  end
end

if difftime == nil then
  if os ~= nil and os.difftime ~= nil then
    difftime = os.difftime
  else
    function difftime(time1, time0)
      return (time1 or 0) - (time0 or 0)
    end
  end
end

if FindInTableIf == nil then
  function FindInTableIf(tbl, predicate)
    if type(tbl) ~= "table" or type(predicate) ~= "function" then
      return nil
    end
    for index, value in ipairs(tbl) do
      if predicate(value, index) then
        return index, value
      end
    end
    return nil
  end
end

local function __wow_install_string_color_helper(name, color)
  local method = "SetColor" .. name
  if string[method] == nil then
    string[method] = function(self)
      return color:WrapTextInColorCode(self)
    end
  end
end

__wow_install_string_color_helper("Orange", CreateColor(1.00, 0.50, 0.25))
__wow_install_string_color_helper("Yellow", CreateColor(1.00, 0.82, 0.00))
__wow_install_string_color_helper("AddonBlue", CreateColor(0.11, 0.57, 0.76))

if string.K_ReplaceVars == nil then
  string.K_ReplaceVars = function(self, vars)
    local text = tostring(self or "")
    if type(vars) ~= "table" then
      return text
    end
    return (text:gsub("({([^}]+)})", function(whole, key)
      local replacement = vars[key]
      if replacement == nil then
        return whole
      end
      return tostring(replacement)
    end))
  end
end

if string.K_AddDefaultValueText == nil then
  string.K_AddDefaultValueText = function(self)
    return tostring(self or "")
  end
end

if GetMoneyString == nil then
  local function __wow_separate_thousands(n)
    local digits = tostring(n)
    if #digits <= 3 then
      return digits
    end
    local out = digits:sub(-3)
    local i = #digits - 3
    while i > 0 do
      local chunk_start = math.max(1, i - 2)
      out = digits:sub(chunk_start, i) .. "," .. out
      i = chunk_start - 1
    end
    return out
  end

  function GetMoneyString(money, separateThousands)
    money = math.floor(tonumber(money) or 0)
    if money < 0 then money = 0 end
    local gold = math.floor(money / 10000)
    local silver = math.floor((money - gold * 10000) / 100)
    local copper = money % 100
    local gold_text = separateThousands and __wow_separate_thousands(gold) or tostring(gold)
    local parts = {}
    if gold > 0 then parts[#parts + 1] = gold_text .. "g" end
    if silver > 0 then parts[#parts + 1] = silver .. "s" end
    if copper > 0 or #parts == 0 then parts[#parts + 1] = copper .. "c" end
    return table.concat(parts, " ")
  end
end

if GetColorForCurrencyReward == nil then
  function GetColorForCurrencyReward(_currencyID, _rewardQuantity, defaultColor)
    if defaultColor ~= nil then
      return defaultColor
    end
    if HIGHLIGHT_FONT_COLOR ~= nil then
      return HIGHLIGHT_FONT_COLOR
    end
    return CreateColor(1, 1, 1, 1)
  end
end

local __wow_console_font_height = 14

if ConsoleGetColorFromType == nil then
  function ConsoleGetColorFromType(_colorType)
    return CreateColor(1, 1, 1)
  end
end

if ConsoleGetFontHeight == nil then
  function ConsoleGetFontHeight()
    return __wow_console_font_height
  end
end

if ConsoleSetFontHeight == nil then
  function ConsoleSetFontHeight(fontHeightInPixels)
    __wow_console_font_height = tonumber(fontHeightInPixels) or __wow_console_font_height
  end
end

if AbbreviateLargeNumbers == nil then
  function AbbreviateLargeNumbers(value)
    return tostring(math.floor(tonumber(value) or 0))
  end
end

if AbbreviateNumbers == nil then
  function AbbreviateNumbers(value)
    return tostring(value or 0)
  end
end
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(FORMATTING_UTILITY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_formatting_utility_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            EVERY_X_PERCENT = nil
            TRANSMOGRIFY_TOOLTIP_APPEARANCE_KNOWN = nil
            ERR_QUEST_SESSION_RESULT_RESYNC = nil
            CLASS_SORT_ORDER = nil
            MAX_CLASSES = nil
            "#,
        )
        .expect("fixture should clear moved constants");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("formatting defaults should reapply");
        }

        let result: String = env
            .eval(
                r#"
                TOKEN_TEXT = "Resolved"
                if GetText("TOKEN_TEXT") ~= "Resolved" then return "gettext" end
                if GetText("MISSING_TOKEN") ~= "MISSING_TOKEN" then return "gettext_missing" end
                if GetText(42) ~= 42 then return "gettext_passthrough" end
                if BACK ~= "Back" or NEXT ~= "Next" or PREVIEW ~= "Preview" then return "text_defaults" end
                if EVERY_X_PERCENT ~= "%d%%" then return "every_percent" end
                if TRANSMOGRIFY_TOOLTIP_APPEARANCE_KNOWN ~= "Known" then return "transmog_known" end
                if type(ERR_QUEST_SESSION_RESULT_RESYNC) ~= "string" or ERR_QUEST_SESSION_RESULT_RESYNC == "" then return "quest_resync" end
                if CLASS_SORT_ORDER[1] ~= "WARRIOR" or CLASS_SORT_ORDER[13] ~= "EVOKER" then return "class_sort_order" end
                if MAX_CLASSES ~= 13 then return "max_classes" end
                if BreakUpLargeNumbers(12345) ~= "12345" then return "breakup" end
                if CalculateStringEditDistance("kitten", "sitting") ~= 3 then return "edit_distance" end
                local split = strsplittable(",", "a,b")
                if split[1] ~= "a" or split[2] ~= "b" then return "strsplittable" end
                local merged = MergeTable({ a = 1 }, { b = 2 })
                if merged.a ~= 1 or merged.b ~= 2 then return "merge_table" end
                local filtered = tFilter({ 1, 2, 3, 4 }, function(value) return value % 2 == 0 end)
                if #filtered ~= 2 or filtered[1] ~= 2 or filtered[2] ~= 4 then return "tfilter" end
                local firstMapped, secondMapped = mapvalues(function(value) return value * 2 end, 2, 3)
                if firstMapped ~= 4 or secondMapped ~= 6 then return "mapvalues" end
                local first, second = ("a,b,c"):split(",")
                if first ~= "a" or second ~= "b" then return "split_method" end
                local appended = tAppendAll({ "a" }, { "b", "c" })
                if #appended ~= 3 or appended[2] ~= "b" or appended[3] ~= "c" then return "append" end
                if GetScreenDPIScale() ~= 1 then return "dpi" end
                if difftime(10, 3) ~= 7 then return "difftime" end
                local index, value = FindInTableIf({ "a", "b", "c" }, function(v) return v == "b" end)
                if index ~= 2 or value ~= "b" then return "find" end
                if ("label"):SetColorOrange():sub(1, 2) ~= "|c" then return "orange" end
                if ("Hello {name}"):K_ReplaceVars({ name = "Sim" }) ~= "Hello Sim" then return "replace" end
                if ("Plain"):K_AddDefaultValueText() ~= "Plain" then return "default_text" end
                if GetMoneyString(1234567, true) ~= "123g 45s 67c" then return "money" end
                if type(GetColorForCurrencyReward(1, 1).GetRGB) ~= "function" then return "currency_color" end
                ConsoleSetFontHeight(18)
                if ConsoleGetFontHeight() ~= 18 then return "console_font" end
                if type(ConsoleGetColorFromType(0).GetRGB) ~= "function" then return "console_color" end
                if AbbreviateLargeNumbers(123.9) ~= "123" then return "abbrev" end
                if AbbreviateNumbers(456.7) ~= "456.7" then return "abbrev_numbers" end
                return "ok"
                "#,
            )
            .expect("formatting utility defaults probe should run");

        assert_eq!(result, "ok");
    }
}
