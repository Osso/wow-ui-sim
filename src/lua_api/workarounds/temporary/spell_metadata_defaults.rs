//! Temporary `C_Spell` metadata/count defaults.
//!
//! Local spell names, icons, targeting, and cooldowns are backed elsewhere.
//! Retail 12.1.5 VERIFIED: IsSpellPassive and IsPriorityAura are false for identifier zero or a missing record.
//! Their positive-record flags, ranged-auto shared ID, and press-hold status remain UNKNOWN.
//! Cast counts before 12.0.0 and display counts before 12.0.5 retain numeric zero.

const SPELL_METADATA_DEFAULTS_LUA: &str = r#"
C_Spell = C_Spell or __wow_namespace()

local function installSpellDefault(name, fn)
    if rawget(C_Spell, name) == nil then
        C_Spell[name] = fn
    end
end

local function returnFalse(_spellID)
    return false
end

installSpellDefault("IsSpellPassive", returnFalse)
installSpellDefault("IsRangedAutoAttackSpell", returnFalse)
installSpellDefault("IsPressHoldReleaseSpell", returnFalse)

installSpellDefault("IsPriorityAura", returnFalse)
"#;

#[cfg(not(feature = "retail-12-0-0"))]
const SPELL_CAST_COUNT_DEFAULT_LUA: &str = r#"
local function installSpellDefault(name, fn)
    if rawget(C_Spell, name) == nil then
        C_Spell[name] = fn
    end
end

installSpellDefault("GetSpellCastCount", function(_spellID)
    return 0
end)
"#;

#[cfg(not(feature = "retail-12-0-5"))]
const SPELL_DISPLAY_COUNT_DEFAULT_LUA: &str = r#"
local function installSpellDefault(name, fn)
    if rawget(C_Spell, name) == nil then
        C_Spell[name] = fn
    end
end

installSpellDefault("GetSpellDisplayCount", function(_spellID, _maxDisplayCount)
    return 0
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SPELL_METADATA_DEFAULTS_LUA)?;
    apply_count_defaults(lua)?;
    Ok(())
}

fn apply_count_defaults(_lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "retail-12-0-0"))]
    _lua.exec(SPELL_CAST_COUNT_DEFAULT_LUA)?;
    #[cfg(not(feature = "retail-12-0-5"))]
    _lua.exec(SPELL_DISPLAY_COUNT_DEFAULT_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(feature = "client-retail")]
    #[test]
    fn installs_spell_metadata_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, bool, bool, bool, bool) = env
            .eval(
                r#"
                return type(C_Spell.IsSpellPassive) == "function",
                    type(C_Spell.IsRangedAutoAttackSpell) == "function",
                    type(C_Spell.IsPressHoldReleaseSpell) == "function",
                    type(C_Spell.IsPriorityAura) == "function",
                    type(C_Spell.GetSpellCastCount) == "function",
                    type(C_Spell.GetSpellDisplayCount) == "function"
                "#,
            )
            .expect("spell metadata functions should be callable");

        assert_eq!(result, (true, true, true, true, true, true));
    }

    #[test]
    fn preserves_existing_spell_metadata_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Spell = C_Spell or __wow_namespace()

            function C_Spell.IsSpellPassive(_spellID)
                return true
            end
            function C_Spell.GetSpellCastCount(_spellID)
                return 7
            end
            function C_Spell.IsPriorityAura(_spellID)
                return true
            end
            "#,
        )
        .expect("fixture should install existing C_Spell providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (bool, i32, bool) = env
            .eval(
                r#"
                return C_Spell.IsSpellPassive(116),
                    C_Spell.GetSpellCastCount(116),
                    C_Spell.IsPriorityAura(116)
                "#,
            )
            .expect("existing spell metadata providers should remain callable");

        assert_eq!(result, (true, 7, true));
    }
}
