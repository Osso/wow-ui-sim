//! Temporary PvP talent defaults for `C_SpecializationInfo`.
//!
//! Core specialization data is modeled by `c_api::c_spec`; PvP talent rows,
//! selections, and lock state are not seeded yet. These inert defaults keep
//! Blizzard talent code loadable until real PvP talent state exists.

const PVP_TALENT_DEFAULTS_LUA: &str = r#"
C_SpecializationInfo = C_SpecializationInfo or __wow_namespace()

local function numberArgument(value)
    if type(value) == "number" then
        return value
    end

    return nil
end

if rawget(C_SpecializationInfo, "GetPvpTalentSlotUnlockLevel") == nil then
    function C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(slotIndex)
        slotIndex = numberArgument(slotIndex)
        if slotIndex == 1 then
            return 20
        elseif slotIndex == 2 then
            return 30
        elseif slotIndex == 3 then
            return 40
        end

        return 0
    end
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
        lua.exec(PVP_SLOT_DEFAULTS_LUA)?;
    }
    Ok(())
}

pub(crate) fn patch_loader(env: &crate::lua_api::LoaderEnv<'_>) {
    let _ = env.exec(PVP_TALENT_DEFAULTS_LUA);
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        let _ = env.exec(PVP_SLOT_DEFAULTS_LUA);
    }
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_pvp_talent_default_shapes() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, bool, i32, i32, i32, String, bool, bool, bool) = env
            .eval(
                r#"
                local slot = C_SpecializationInfo.GetPvpTalentSlotInfo(2)
                local talent = C_SpecializationInfo.GetPvpTalentInfo(123)
                local selected = C_SpecializationInfo.GetAllSelectedPvpTalentIDs()
                local setResult = C_SpecializationInfo.SetPvpTalentLocked(123, true)
                return slot.enabled,
                       slot.locked,
                       slot.selectedTalentID == nil,
                       slot.slotIndex,
                       #slot.availableTalentIDs,
                       C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(2),
                       talent.name,
                       talent.unlocked,
                       C_SpecializationInfo.IsPvpTalentLocked(123),
                       type(selected) == "table" and setResult == nil
                "#,
            )
            .expect("PvP talent defaults should be installed");

        assert_eq!(
            result,
            (
                true,
                false,
                true,
                2,
                0,
                30,
                "PvP Talent".to_string(),
                true,
                false,
                true,
            )
        );
    }

    #[test]
    fn rejects_invalid_pvp_talent_inputs() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, i32) = env
            .eval(
                r#"
                return C_SpecializationInfo.GetPvpTalentSlotInfo(4) == nil,
                       C_SpecializationInfo.GetPvpTalentInfo(0) == nil,
                       C_SpecializationInfo.GetPvpTalentSlotUnlockLevel(99)
                "#,
            )
            .expect("invalid PvP talent defaults should be inert");

        assert_eq!(result, (true, true, 0));
    }

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
