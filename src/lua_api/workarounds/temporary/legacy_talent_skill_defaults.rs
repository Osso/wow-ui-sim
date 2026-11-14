//! Temporary legacy talent, PvP talent, arena, and skill-window defaults.
//!
//! The simulator does not yet model legacy talent grids, retail PvP talent
//! selections, arena rosters, or the removed legacy skill window.

use crate::lua_api::methods::table_get;
use crate::lua_bridge::stack_val;
use rilua::{LuaResult, Val, vm::state::LuaState};

pub(crate) fn get_legacy_talent_info(state: &mut LuaState) -> LuaResult<u32> {
    let query = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(query, Val::Table(_)) {
        return Err(rilua::runtime_error("GetTalentInfo requires a query table"));
    }
    validate_talent_index(state, query, "groupIndex")?;
    let has_tier = validate_talent_index(state, query, "tier")?;
    let has_column = validate_talent_index(state, query, "column")?;
    validate_talent_index(state, query, "specializationIndex")?;
    validate_talent_index(state, query, "talentIndex")?;
    for field in ["isInspect", "isPet"] {
        let value = table_get(state, query, field);
        rilua::table_security::unwrap_secret(state, value)?;
    }
    let target = table_get(state, query, "target");
    let target = rilua::table_security::unwrap_secret(state, target)?;
    if !matches!(target, Val::Nil | Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error("query.target requires a unit token"));
    }
    if !has_tier {
        return Err(rilua::runtime_error("query.tier must be specified."));
    }
    if !has_column {
        return Err(rilua::runtime_error("query.column must be specified."));
    }
    // The Retail legacy tier/column grid is not modeled. Forever uses c_api::forever_talents.
    state.push(Val::Nil);
    Ok(1)
}

/// Validate a provided index and report whether the field was supplied.
fn validate_talent_index(state: &mut LuaState, query: Val, field: &str) -> LuaResult<bool> {
    let value = table_get(state, query, field);
    let value = rilua::table_security::unwrap_secret(state, value)?;
    match value {
        Val::Nil => Ok(false),
        Val::Num(number) if number.is_finite() && (0.0..=f64::from(u32::MAX)).contains(&number) => {
            Ok(true)
        }
        _ => Err(rilua::runtime_error(format!(
            "query.{field} requires an index"
        ))),
    }
}

const LEGACY_TALENT_ROW_DEFAULTS_LUA: &str = r#"
if rawget(_G, "GetNumTalentTabs") == nil then
    function GetNumTalentTabs()
        return 0
    end
end

if rawget(_G, "GetTalentInfo") == nil then
    function GetTalentInfo(_tabIndex, _talentIndex)
        return nil
    end
end

"#;

const LEGACY_TALENT_SKILL_DEFAULTS_LUA: &str = r#"
if rawget(_G, "GetTalentInfoBySpecialization") == nil then
    function GetTalentInfoBySpecialization(_specIndex, _tier, _column)
        return nil
    end
end

if rawget(_G, "GetArenaOpponentSpec") == nil then
    function GetArenaOpponentSpec(_opponentIndex)
        return 0
    end
end

"#;

const LEGACY_PVP_TALENT_SLOT_INFO_DEFAULT_LUA: &str = r#"
local function numberArgument(value)
    if type(value) == "number" then
        return value
    end

    return nil
end

local function isValidPvpTalentSlot(slotIndex)
    return slotIndex == 1 or slotIndex == 2 or slotIndex == 3
end

if rawget(_G, "GetPvpTalentSlotInfo") == nil then
    function GetPvpTalentSlotInfo(slotIndex)
        slotIndex = numberArgument(slotIndex)
        if slotIndex == nil or not isValidPvpTalentSlot(slotIndex) then
            return nil
        end

        return {
            enabled = true,
            locked = false,
            selectedTalentID = 0,
            slotIndex = slotIndex,
        }
    end
end
"#;

#[cfg(not(any(
    feature = "client-wrath",
    feature = "client-era",
    feature = "client-anniversary"
)))]
const LEGACY_SKILL_DEFAULTS_LUA: &str = r#"
if rawget(_G, "GetNumSkillLines") == nil then
    function GetNumSkillLines()
        return 0
    end
end

if rawget(_G, "GetSkillLineInfo") == nil then
    function GetSkillLineInfo(_index)
        return nil
    end
end

if rawget(_G, "GetSelectedSkill") == nil then
    function GetSelectedSkill()
        return 0
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        lua.exec(LEGACY_TALENT_ROW_DEFAULTS_LUA)?;
    }
    lua.exec(LEGACY_TALENT_SKILL_DEFAULTS_LUA)?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        // Retail registers no legacy global alias for this slot query.
        lua.exec(LEGACY_PVP_TALENT_SLOT_INFO_DEFAULT_LUA)?;
    }
    #[cfg(not(any(
        feature = "client-wrath",
        feature = "client-era",
        feature = "client-anniversary"
    )))]
    lua.exec(LEGACY_SKILL_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    retail_removed_legacy_globals_fixture! {
    #[cfg(feature = "client-ptr")]
    #[test]
    fn installs_legacy_talent_skill_default_shapes() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, bool, bool, i32, i32, i32, bool, i32) = env
            .eval(
                r#"
                local talent = GetTalentInfo(1, 1)
                local specTalent = GetTalentInfoBySpecialization(1, 1, 1)
                local slot = GetPvpTalentSlotInfo(2)
                local missingSlot = GetPvpTalentSlotInfo(4)
                local skill = GetSkillLineInfo(1)
                return GetNumTalentTabs(),
                       talent == nil,
                       specTalent == nil,
                       slot.slotIndex,
                       slot.selectedTalentID,
                       GetArenaOpponentSpec(1),
                       missingSlot == nil and skill == nil,
                       GetSelectedSkill()
                "#,
            )
            .expect("legacy talent/skill defaults should be callable");

        assert_eq!(result, (0, true, true, 2, 0, 0, true, 0));
    }
    }

    #[test]
    fn preserves_existing_legacy_talent_skill_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function GetArenaOpponentSpec(_opponentIndex)
                return 70
            end
            "#,
        )
        .expect("fixture should install existing provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let spec_id: i32 = env
            .eval("return GetArenaOpponentSpec(1)")
            .expect("existing provider should remain callable");

        assert_eq!(spec_id, 70);
    }
}
