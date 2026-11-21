//! Temporary `C_Spell` target-spell defaults for profiles without the model.
//!
//! Retail and Forever provide these queries from `c_api::target_spell`.

const SPELL_TARGET_DEFAULTS_LUA: &str = r#"
C_Spell = C_Spell or __wow_namespace()

local function installSpellTargetDefault(name)
    if rawget(C_Spell, name) == nil then
        C_Spell[name] = function()
            return false
        end
    end
end

installSpellTargetDefault("TargetSpellIsEnchanting")
installSpellTargetDefault("TargetSpellJumpsUpgradeTrack")
installSpellTargetDefault("TargetSpellReplacesBonusTree")
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SPELL_TARGET_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_false_spell_target_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, bool) = env
            .eval(
                r#"
                return C_Spell.TargetSpellIsEnchanting(),
                    C_Spell.TargetSpellJumpsUpgradeTrack(),
                    C_Spell.TargetSpellReplacesBonusTree()
                "#,
            )
            .expect("spell target defaults should be callable");

        assert_eq!(result, (false, false, false));
    }

    #[test]
    fn preserves_existing_spell_target_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Spell.TargetSpellIsEnchanting()
                return true
            end
            "#,
        )
        .expect("fixture should install existing spell target provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let enchanting: bool = env
            .eval("return C_Spell.TargetSpellIsEnchanting()")
            .expect("existing spell target provider should remain callable");

        assert!(enchanting);
    }
}
