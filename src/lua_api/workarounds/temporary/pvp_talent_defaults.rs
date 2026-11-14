//! Temporary PvP talent defaults for `C_SpecializationInfo`.
//!
//! Core specialization data and Retail lock state are modeled by `c_api::c_spec`;
//! PvP talent rows and selections are not seeded yet. These defaults keep
//! Blizzard talent code loadable while those data sources remain unmodeled.

const PVP_TALENT_DEFAULTS_LUA: &str = r#"
C_SpecializationInfo = C_SpecializationInfo or __wow_namespace()

local function numberArgument(value)
    if type(value) == "number" then
        return value
    end

    return nil
end

if rawget(C_SpecializationInfo, "GetPvpTalentInfo") == nil then
    function C_SpecializationInfo.GetPvpTalentInfo(talentID)
        talentID = numberArgument(talentID)
        if talentID == nil or talentID <= 0 then
            return nil
        end

        return {
            talentID = talentID,
            name = "PvP Talent",
            icon = "Interface\\Icons\\Spell_Holy_PowerWordShield",
            unlocked = true,
            dependenciesUnmet = false,
            dependenciesUnmetReason = nil,
        }
    end
end

if rawget(C_SpecializationInfo, "GetPvpTalentUnlockLevel") == nil then
    function C_SpecializationInfo.GetPvpTalentUnlockLevel()
        return 20
    end
end

if rawget(C_SpecializationInfo, "GetInspectSelectedPvpTalent") == nil then
    function C_SpecializationInfo.GetInspectSelectedPvpTalent()
        return nil
    end
end

if rawget(C_SpecializationInfo, "GetAllSelectedPvpTalentIDs") == nil then
    function C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
        return {}
    end
end

if rawget(C_SpecializationInfo, "IsPvpTalentLocked") == nil then
    function C_SpecializationInfo.IsPvpTalentLocked()
        return false
    end
end

if rawget(C_SpecializationInfo, "SetPvpTalentLocked") == nil then
    function C_SpecializationInfo.SetPvpTalentLocked()
    end
end
"#;

const PVP_TALENT_SLOT_UNLOCK_LEVEL_DEFAULTS_LUA: &str = r#"
if rawget(C_SpecializationInfo, "GetPvpTalentSlotUnlockLevel") == nil then
    function C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(slotIndex)
        if type(slotIndex) ~= "number" then
            return 0
        elseif slotIndex == 1 then
            return 20
        elseif slotIndex == 2 then
            return 30
        elseif slotIndex == 3 then
            return 40
        end

        return 0
    end
end
"#;

const PVP_SLOT_DEFAULTS_LUA: &str = r#"
local function numberArgument(value)
    return type(value) == "number" and value or nil
end
local function isValidPvpTalentSlot(slotIndex)
    return slotIndex == 1 or slotIndex == 2 or slotIndex == 3
end
if rawget(C_SpecializationInfo, "GetPvpTalentSlotInfo") == nil then
    function C_SpecializationInfo.GetPvpTalentSlotInfo(slotIndex)
        slotIndex = numberArgument(slotIndex)
        if slotIndex == nil or not isValidPvpTalentSlot(slotIndex) then
            return nil
        end

            return {
                enabled = true,
                locked = false,
                slotIndex = slotIndex,
                availableTalentIDs = {},
            }
    end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PVP_TALENT_DEFAULTS_LUA)?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        lua.exec(PVP_TALENT_SLOT_UNLOCK_LEVEL_DEFAULTS_LUA)?;
        lua.exec(PVP_SLOT_DEFAULTS_LUA)?;
    }
    Ok(())
}

pub(crate) fn patch_loader(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = env.exec(PVP_TALENT_DEFAULTS_LUA);
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        let _ = env.exec(PVP_TALENT_SLOT_UNLOCK_LEVEL_DEFAULTS_LUA);
        let _ = env.exec(PVP_SLOT_DEFAULTS_LUA);
    }
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn preserves_existing_specialization_provider_methods() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_SpecializationInfo.GetPvpTalentInfo(talentID)
                return { talentID = talentID, name = "Existing" }
            end
            "#,
        )
        .expect("fixture should install provider method");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (String, String) = env
            .eval(
                r#"
                local talent = C_SpecializationInfo.GetPvpTalentInfo(55)
                return talent.name, type(C_SpecializationInfo.GetPvpTalentSlotInfo)
                "#,
            )
            .expect("existing PvP talent provider should be preserved");

        assert_eq!(result, ("Existing".to_string(), "function".to_string()));
    }
}

/// Preserve the pre-R73 unmodelled Retail slot result, not a native default.
/// Retire when native no-host slot behavior is established.
#[cfg(feature = "client-retail")]
pub(crate) fn pvp_slot_default(state: &mut rilua::vm::state::LuaState, index: usize) -> rilua::Val {
    use crate::lua_api::methods::{create_table, table_set};
    use rilua::Val;
    if !(1..=3).contains(&index) {
        return Val::Nil;
    }
    let result = create_table(state);
    table_set(state, result, "enabled", Val::Bool(true));
    table_set(state, result, "locked", Val::Bool(false));
    table_set(state, result, "slotIndex", Val::Num(index as f64));
    let available = create_table(state);
    table_set(state, result, "availableTalentIDs", available);
    result
}
