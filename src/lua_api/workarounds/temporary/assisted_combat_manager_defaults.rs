//! Temporary `AssistedCombatManager` table defaults.
//!
//! Assisted combat rotation state is not modeled yet. These defaults keep
//! Blizzard action-bar and spellbook code loading while preserving the
//! table-shaped manager surface behind an explicit workaround boundary.

const ASSISTED_COMBAT_MANAGER_DEFAULTS_LUA: &str = r#"
AssistedCombatManager = type(AssistedCombatManager) == "table" and AssistedCombatManager or {}

if rawget(AssistedCombatManager, "GetActionSpellID") == nil then
  function AssistedCombatManager:GetActionSpellID()
    return 0
  end
end

if rawget(AssistedCombatManager, "ShouldHighlightSpellbookSpell") == nil then
  function AssistedCombatManager:ShouldHighlightSpellbookSpell(_spellID)
    return false
  end
end

"#;

const ASSISTED_COMBAT_MANAGER_OWNER_FALLBACK_LUA: &str = r#"
if rawget(AssistedCombatManager, "HasActionSpell") == nil then
  function AssistedCombatManager:HasActionSpell()
    return false
  end
end

if rawget(AssistedCombatManager, "GetActionSpellDescription") == nil then
  function AssistedCombatManager:GetActionSpellDescription()
    return ""
  end
end

if rawget(AssistedCombatManager, "AddSpellTooltipLine") == nil then
  function AssistedCombatManager:AddSpellTooltipLine(_tooltip, _spellID, _overriddenSpellID)
  end
end

if rawget(AssistedCombatManager, "SetCanHighlightSpellbookSpells") == nil then
  function AssistedCombatManager:SetCanHighlightSpellbookSpells(_enabled)
  end
end
"#;

fn profile_has_original_assisted_combat_owner() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    )
}

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(ASSISTED_COMBAT_MANAGER_DEFAULTS_LUA)?;
    if !profile_has_original_assisted_combat_owner() {
        lua.exec(ASSISTED_COMBAT_MANAGER_OWNER_FALLBACK_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_assisted_combat_manager_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("assisted combat defaults should apply");
        }

        let expected_owner_fallback = !super::profile_has_original_assisted_combat_owner();
        let has_action_spell_fallback: bool = env
            .eval("return type(AssistedCombatManager.HasActionSpell) == 'function'")
            .expect("action-spell fallback availability probe should run");
        let has_description_fallback: bool = env
            .eval("return type(AssistedCombatManager.GetActionSpellDescription) == 'function'")
            .expect("description fallback availability probe should run");
        let has_tooltip_fallback: bool = env
            .eval("return type(AssistedCombatManager.AddSpellTooltipLine) == 'function'")
            .expect("tooltip fallback availability probe should run");
        let has_highlight_fallback: bool = env
            .eval("return type(AssistedCombatManager.SetCanHighlightSpellbookSpells) == 'function'")
            .expect("highlight fallback availability probe should run");
        assert_eq!(has_action_spell_fallback, expected_owner_fallback);
        assert_eq!(has_description_fallback, expected_owner_fallback);
        assert_eq!(has_tooltip_fallback, expected_owner_fallback);
        assert_eq!(has_highlight_fallback, expected_owner_fallback);

        let result: String = env
            .eval(
                r#"
                if type(AssistedCombatManager) ~= "table" then return "missing_manager" end
                if type(AssistedCombatManager.HasActionSpell) == "function"
                    and AssistedCombatManager:HasActionSpell() ~= false
                then return "has_action_spell" end
                if AssistedCombatManager:GetActionSpellID() ~= 0 then return "spell_id" end
                if type(AssistedCombatManager.GetActionSpellDescription) == "function"
                    and AssistedCombatManager:GetActionSpellDescription() ~= ""
                then return "description" end
                if AssistedCombatManager:ShouldHighlightSpellbookSpell(116) ~= false then return "highlight" end
                if type(AssistedCombatManager.SetCanHighlightSpellbookSpells) == "function" then
                    AssistedCombatManager:SetCanHighlightSpellbookSpells(true)
                end
                if type(AssistedCombatManager.AddSpellTooltipLine) == "function" then
                    AssistedCombatManager:AddSpellTooltipLine({}, 116, nil)
                end
                return "ok"
                "#,
            )
            .expect("assisted combat defaults should be callable");

        assert_eq!(result, "ok");
    }
}
