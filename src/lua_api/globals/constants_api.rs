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

    // Create the auto-vivifying Constants table via Lua code.
    // Any access to an undefined subtable (e.g., Constants.Foo) returns an
    // empty auto-vivifying table, and accessing a leaf on that returns nil
    // instead of erroring.
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

        -- LFG_ROLEConstants (from LFGConstantsDocumentation)
        Constants.LFG_ROLEConstants.LFG_ROLE_NO_ROLE = -1
        Constants.LFG_ROLEConstants.LFG_ROLE_ANY = 3  -- Enum.LFGRoleMeta.NumValues

        -- LFGConstsExposed
        Constants.LFGConstsExposed.GROUP_FINDER_MAX_ACTIVITY_CAPACITY = 16

        -- MoneyFormattingConstants
        Constants.MoneyFormattingConstants.GOLD_REWARD_THRESHOLD_TO_HIDE_COPPER = 10

        -- TimerunningConsts (referenced by GameRulesUtil)
        Constants.TimerunningConsts.TIMERUNNING_SEASON_NONE = 0
        Constants.TimerunningConsts.TIMERUNNING_SEASON_PANDARIA = 1

        -- AccountStoreConsts (referenced by GameRulesUtil)
        Constants.AccountStoreConsts.PlunderstormStoreFrontID = 0
        Constants.AccountStoreConsts.WowhackStoreFrontID = 0

        -- InventoryConstants (referenced by Blizzard_FrameXMLBase/Constants.lua)
        Constants.InventoryConstants.NumBagSlots = 5
        Constants.InventoryConstants.NumReagentBagSlots = 1
    "#,
    )
    .exec()?;

    Ok(())
}
