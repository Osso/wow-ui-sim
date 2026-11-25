//! Temporary C_LevelLink spell-lock state.
//!
//! Action locks are backed by simulator state. Spell locks still use a small
//! Lua-visible state table so tests and compatibility probes can seed locks
//! until spell-level progression is modeled.

const LEVEL_LINK_SPELL_LOCK_STATE_LUA: &str = r#"
C_LevelLink = C_LevelLink or __wow_namespace()

local state = rawget(C_LevelLink, "_state")
if state == nil or type(state) ~= "table" then
    state = {
        lockedSpells = {},
        lastSpellQuery = nil,
    }
    rawset(C_LevelLink, "_state", state)
elseif state.lockedSpells == nil then
    state.lockedSpells = {}
end

if rawget(C_LevelLink, "IsSpellLocked") == nil then
    function C_LevelLink.IsSpellLocked(spellID)
        local state = rawget(C_LevelLink, "_state")
        local normalized = tonumber(spellID)
        if normalized == nil then
            state.lastSpellQuery = nil
            return false
        end

        state.lastSpellQuery = normalized
        local entry = state.lockedSpells[normalized]
        if type(entry) == "table" then
            return entry.locked == true
        end
        return entry == true
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !crate::c_api::level_link::uses_rust_spell_locks() {
        lua.exec(LEVEL_LINK_SPELL_LOCK_STATE_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_level_link_spell_lock_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool, bool) = env
            .eval(
                r#"
                C_LevelLink._state.lockedSpells = {
                    [111] = true,
                    [222] = { locked = true },
                }

                return C_LevelLink.IsSpellLocked(111),
                       C_LevelLink.IsSpellLocked(222),
                       not C_LevelLink.IsSpellLocked(333)
                "#,
            )
            .expect("level link spell-lock defaults should be callable");

        assert_eq!(result, (true, true, true));
    }

    #[test]
    fn preserves_existing_level_link_spell_provider_and_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_LevelLink = C_LevelLink or __wow_namespace()
            C_LevelLink._state = {
                lockedSpells = { [5] = true },
                preserved = true,
            }

            function C_LevelLink.IsSpellLocked(_spellID)
                return "provider"
            end
            "#,
        )
        .expect("fixture should install existing level link provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (String, bool, bool) = env
            .eval(
                r#"
                return C_LevelLink.IsSpellLocked(5),
                       C_LevelLink._state.preserved == true,
                       C_LevelLink._state.lockedSpells[5] == true
                "#,
            )
            .expect("existing level link provider should remain callable");

        assert_eq!(result, ("provider".to_string(), true, true));
    }
}
