//! Temporary `C_Spell` static/default fallbacks.
//!
//! The Rust `C_Spell` surface owns state-backed spell metadata, links, costs,
//! cooldowns, charges, and flyout data. These defaults cover unmodeled
//! override, visibility, and Maw power-border state until those domains are
//! modeled.

const SPELL_STATIC_DEFAULTS_LUA: &str = r#"
C_Spell = C_Spell or __wow_namespace()

if rawget(C_Spell, "GetOverrideSpell") == nil then
    function C_Spell.GetOverrideSpell(spellID)
        return spellID
    end
end

if rawget(C_Spell, "GetVisibilityInfo") == nil then
    function C_Spell.GetVisibilityInfo(_spellID)
        return false, true, false
    end
end

if rawget(C_Spell, "GetMawPowerBorderAtlasBySpellID") == nil then
    function C_Spell.GetMawPowerBorderAtlasBySpellID(_spellID)
        return nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SPELL_STATIC_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_spell_static_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (override_id, maw_atlas_is_nil, charge_is_nil): (i64, bool, bool) = env
            .eval(
                r#"
                local charges = C_Spell.GetSpellCharges(116)
                return C_Spell.GetOverrideSpell(116),
                       C_Spell.GetMawPowerBorderAtlasBySpellID(116) == nil,
                       charges == nil
                "#,
            )
            .expect("spell static defaults should be callable");

        assert_eq!(override_id, 116);
        assert!(maw_atlas_is_nil);
        assert!(charge_is_nil);
    }

    #[test]
    fn preserves_existing_spell_static_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Spell = C_Spell or __wow_namespace()

            function C_Spell.GetOverrideSpell(_spellID)
                return 999
            end

            function C_Spell.GetMawPowerBorderAtlasBySpellID(_spellID)
                return "maw-border"
            end
            "#,
        )
        .expect("fixture should install existing spell static provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, String) = env
            .eval(
                r#"
                return C_Spell.GetOverrideSpell(116),
                       C_Spell.GetMawPowerBorderAtlasBySpellID(116)
                "#,
            )
            .expect("existing spell static provider should remain callable");

        assert_eq!(result, (999, "maw-border".to_string()));
    }
}
