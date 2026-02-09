//! WoW Constants table containing game constant namespaces.
//!
//! This module registers the global `Constants` table. Unlike `Enum`, which
//! contains enumerations, `Constants` contains named constant values grouped
//! into namespaces (e.g., `Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE`).
//!
//! The table uses auto-vivifying metatables so that accessing undefined
//! subtables returns an empty table instead of nil, preventing crashes
//! when Blizzard code references constants we haven't stubbed yet.

use mlua::{Lua, Result};

/// Register the Constants table with auto-vivifying metatables.
pub fn register_constants_api(lua: &Lua) -> Result<()> {
    register_constants_table(lua)?;
    register_color_globals(lua)?;
    register_raid_class_colors(lua)?;
    Ok(())
}

/// Create the auto-vivifying Constants table and populate constant namespaces.
fn register_constants_table(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        local function make_autovivify()
            local mt = {
                __index = function(t, k)
                    local sub = setmetatable({}, getmetatable(t))
                    rawset(t, k, sub)
                    return sub
                end
            }
            return setmetatable({}, mt)
        end

        Constants = make_autovivify()

        Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE = -1
        Constants.LFG_ROLEConstants.LFG_ROLE_ANY = 3
        Constants.LFGConstsExposed.GROUP_FINDER_MAX_ACTIVITY_CAPACITY = 16
        Constants.MoneyFormattingConstants.GOLD_REWARD_THRESHOLD_TO_HIDE_COPPER = 10
        Constants.TimerunningConsts.TIMERUNNING_SEASON_NONE = 0
        Constants.TimerunningConsts.TIMERUNNING_SEASON_PANDARIA = 1
        Constants.AccountStoreConsts.PlunderstormStoreFrontID = 0
        Constants.AccountStoreConsts.WowhackStoreFrontID = 0
        Constants.InventoryConstants.NumBagSlots = 5
        Constants.InventoryConstants.NumReagentBagSlots = 1
        Constants.EditModeConsts.EditModeDefaultGridSpacing = 100
        Constants.EditModeConsts.EditModeMinGridSpacing = 20
        Constants.EditModeConsts.EditModeMaxGridSpacing = 300
        Constants.EditModeConsts.EditModeMaxLayoutsPerType = 5
        Constants.MajorFactionsConsts.PLUNDERSTORM_MAJOR_FACTION_ID = 0

        Constants.ChatFrameConstants.MaxChatWindows = 10
        Constants.ChatFrameConstants.MaxChatChannels = 20
        Constants.ChatFrameConstants.MaxCharacterNameBytes = 100
    "#,
    )
    .exec()
}

/// Register global color objects (faction colors, font colors, PvP colors).
fn register_color_globals(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        local function makeColor(r, g, b, a)
            a = a or 1.0
            local c = { r = r, g = g, b = b, a = a }
            function c:GetRGB() return self.r, self.g, self.b end
            function c:GetRGBA() return self.r, self.g, self.b, self.a end
            function c:GetRGBAsBytes()
                return math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255)
            end
            function c:GenerateHexColor()
                return string.format("%02x%02x%02x",
                    math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
            end
            function c:GenerateHexColorMarkup() return "|cff" .. self:GenerateHexColor() end
            function c:WrapTextInColorCode(text) return "|cff" .. self:GenerateHexColor() .. text .. "|r" end
            return c
        end

        PLAYER_FACTION_COLOR_HORDE = makeColor(0.90196, 0.05098, 0.07059)
        PLAYER_FACTION_COLOR_ALLIANCE = makeColor(0.29412, 0.33333, 0.91373)
        NORMAL_FONT_COLOR = makeColor(1.0, 0.82, 0.0)
        HIGHLIGHT_FONT_COLOR = makeColor(1.0, 1.0, 1.0)
        RED_FONT_COLOR = makeColor(1.0, 0.1, 0.1)
        GREEN_FONT_COLOR = makeColor(0.1, 1.0, 0.1)
        GRAY_FONT_COLOR = makeColor(0.5, 0.5, 0.5)
        YELLOW_FONT_COLOR = makeColor(1.0, 1.0, 0.0)
        LIGHTYELLOW_FONT_COLOR = makeColor(1.0, 1.0, 0.6)
        ORANGE_FONT_COLOR = makeColor(1.0, 0.5, 0.25)
        WHITE_FONT_COLOR = makeColor(1.0, 1.0, 1.0)
        DISABLED_FONT_COLOR = makeColor(0.5, 0.5, 0.5)
        DIM_RED_FONT_COLOR = makeColor(0.8, 0.1, 0.1)
        PVP_SCOREBOARD_HORDE_CELL_COLOR = makeColor(1.0, 0.18, 0.18)
        PVP_SCOREBOARD_ALLIANCE_CELL_COLOR = makeColor(0.36, 0.45, 1.0)
        FACTION_RED_COLOR = makeColor(0.8, 0.13, 0.13)
        FACTION_ORANGE_COLOR = makeColor(0.93, 0.53, 0.13)
        FACTION_YELLOW_COLOR = makeColor(0.8, 0.73, 0.13)
        FACTION_GREEN_COLOR = makeColor(0.13, 0.8, 0.13)
        OBJECTIVE_TRACKER_BLOCK_HEADER_COLOR = makeColor(1.0, 0.82, 0.0)
        PANEL_BACKGROUND_COLOR = makeColor(0.15, 0.15, 0.15)
        LIGHTBLUE_FONT_COLOR = makeColor(0.51176, 0.77255, 1.0)
    "#,
    )
    .exec()
}

/// Register `RAID_CLASS_COLORS` - maps class file names to color objects.
fn register_raid_class_colors(lua: &Lua) -> Result<()> {
    lua.load(
        r#"
        local function makeClassColor(r, g, b)
            local c = { r = r, g = g, b = b }
            function c:GetRGB() return self.r, self.g, self.b end
            function c:GetRGBA() return self.r, self.g, self.b, 1.0 end
            function c:GenerateHexColor()
                return string.format("%02x%02x%02x",
                    math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
            end
            function c:WrapTextInColorCode(text) return "|cff" .. self:GenerateHexColor() .. text .. "|r" end
            return c
        end

        RAID_CLASS_COLORS = {
            WARRIOR     = makeClassColor(0.78, 0.61, 0.43),
            PALADIN     = makeClassColor(0.96, 0.55, 0.73),
            HUNTER      = makeClassColor(0.67, 0.83, 0.45),
            ROGUE       = makeClassColor(1.00, 0.96, 0.41),
            PRIEST      = makeClassColor(1.00, 1.00, 1.00),
            DEATHKNIGHT = makeClassColor(0.77, 0.12, 0.23),
            SHAMAN      = makeClassColor(0.00, 0.44, 0.87),
            MAGE        = makeClassColor(0.25, 0.78, 0.92),
            WARLOCK     = makeClassColor(0.53, 0.53, 0.93),
            MONK        = makeClassColor(0.00, 1.00, 0.60),
            DRUID       = makeClassColor(1.00, 0.49, 0.04),
            DEMONHUNTER = makeClassColor(0.64, 0.19, 0.79),
            EVOKER      = makeClassColor(0.20, 0.58, 0.50),
        }
    "#,
    )
    .exec()
}
