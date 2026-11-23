//! Temporary profession specialization defaults.
//!
//! Profession specialization data is not backed by the professions model yet.
//! Keep the small blacksmithing fixture here instead of in the central runtime
//! bootstrap so the compatibility boundary is explicit and removable.

const PROFESSION_SPECS_DEFAULTS_LUA: &str = r#"
if type(C_ProfSpecs) ~= "table" then
    C_ProfSpecs = {}
end

if rawget(C_ProfSpecs, "ShouldShowSpecTab") == nil then
    function C_ProfSpecs.ShouldShowSpecTab() return true end
end
if rawget(C_ProfSpecs, "GetDefaultSpecSkillLine") == nil then
    function C_ProfSpecs.GetDefaultSpecSkillLine() return 164 end
end
if rawget(C_ProfSpecs, "GetConfigIDForSkillLine") == nil then
    function C_ProfSpecs.GetConfigIDForSkillLine(skillLineID)
        if tonumber(skillLineID) == 164 then
            return 1
        end
        return nil
    end
end
if rawget(C_ProfSpecs, "GetSpecTabIDsForSkillLine") == nil then
    function C_ProfSpecs.GetSpecTabIDsForSkillLine(skillLineID)
        if tonumber(skillLineID) == 164 then
            return { 101 }
        end
        return {}
    end
end
if rawget(C_ProfSpecs, "GetTabInfo") == nil then
    function C_ProfSpecs.GetTabInfo(tabID)
        if tonumber(tabID) == 101 then
            return {
                tabID = 101,
                rootNodeID = 1001,
                name = "Armorsmithing",
                description = "",
                rootIconID = 0,
                highlights = {},
            }
        end
        return nil
    end
end
if rawget(C_ProfSpecs, "GetSpecTabInfo") == nil then
    function C_ProfSpecs.GetSpecTabInfo()
        return {
            enabled = false,
            errorReason = "",
        }
    end
end
if rawget(C_ProfSpecs, "GetCurrencyInfoForSkillLine") == nil then
    function C_ProfSpecs.GetCurrencyInfoForSkillLine(skillLineID)
        if tonumber(skillLineID) == 164 then
            return {
                numAvailable = 0,
                currencyName = "",
            }
        end
        return nil
    end
end
if rawget(C_ProfSpecs, "SkillLineHasSpecialization") == nil then
    function C_ProfSpecs.SkillLineHasSpecialization(skillLineID)
        return tonumber(skillLineID) == 164
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PROFESSION_SPECS_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_profession_spec_defaults_without_replacing_existing_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ProfSpecs.ShouldShowSpecTab = function() return "existing" end
            "#,
        )
        .expect("fixture should override an existing profession spec member");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("profession spec defaults should reapply");
        }

        let result: String = env
            .eval(
                r#"
                if C_ProfSpecs.ShouldShowSpecTab() ~= "existing" then return "show_spec" end
                if C_ProfSpecs.GetDefaultSpecSkillLine() ~= 164 then return "skill_line" end
                if C_ProfSpecs.GetConfigIDForSkillLine(164) ~= 1 then return "config" end
                local tabIDs = C_ProfSpecs.GetSpecTabIDsForSkillLine(164)
                if tabIDs[1] ~= 101 then return "tab_id" end
                local tabInfo = C_ProfSpecs.GetTabInfo(101)
                if tabInfo.rootNodeID ~= 1001 then return "root_node" end
                if C_ProfSpecs.GetSpecTabInfo().enabled ~= false then return "spec_tab" end
                if C_ProfSpecs.GetCurrencyInfoForSkillLine(164).numAvailable ~= 0 then return "currency" end
                if C_ProfSpecs.SkillLineHasSpecialization(164) ~= true then return "blacksmithing_specialization" end
                if C_ProfSpecs.SkillLineHasSpecialization(165) ~= false then return "unrelated_specialization" end
                return "ok"
                "#,
            )
            .expect("profession spec defaults should be callable");

        assert_eq!(result, "ok");
    }
}
