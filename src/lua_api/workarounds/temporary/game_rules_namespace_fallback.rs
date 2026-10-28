//! `C_GameRules` missing-member diagnostics and temporary cast-bar default.
//!
//! Game rule queries are Rust-backed by `SimState::game_rules`.
//! Unmodeled members remain nil.

const GAME_RULES_NAMESPACE_FALLBACK_LUA: &str = r#"
if type(C_GameRules) ~= "table" then
    C_GameRules = {}
end

GameRulesUtil = GameRulesUtil or {}
if rawget(GameRulesUtil, "ShouldShowPlayerCastBar") == nil then
    function GameRulesUtil.ShouldShowPlayerCastBar()
        return true
    end
end

local function GameRulesNamespaceFallback(t, key)
    if type(__wow_record_nil_symbol_access) == "function" then
        __wow_record_nil_symbol_access("C_GameRules", key, nil, nil)
    end
    return nil
end

local mt = getmetatable(C_GameRules)
if mt == nil then
    setmetatable(C_GameRules, { __index = GameRulesNamespaceFallback })
elseif mt.__index == nil then
    mt.__index = GameRulesNamespaceFallback
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(GAME_RULES_NAMESPACE_FALLBACK_LUA)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        lua.exec(
            r#"
            -- User-approved simulator default, not evidence of native behavior.
            -- Missing system: account-wide keybinding policy.
            function C_GameRules.IsAccountKeyBindingsEnabled()
                return false
            end
        "#,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn preserves_rust_backed_methods_and_leaves_unmodeled_members_nil() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("A_Admin.SetGameRule('Hardcore', true)")
            .expect("admin game rule should set");

        let result: String = env
            .eval(
                r#"
                if not C_GameRules.IsGameRuleActive("Hardcore") then
                    return "lost_rust_method"
                end
                if C_GameRules.NotAModeledRealmRule ~= nil then
                    return "fabricated_member"
                end
                if type(GameRulesUtil) ~= "table" then
                    return "missing_util"
                end
                if GameRulesUtil.ShouldShowPlayerCastBar() ~= true then
                    return "bad_cast_bar_rule"
                end
                return "ok"
                "#,
            )
            .expect("game rules fallback probe should run");

        assert_eq!(result, "ok");
    }
}
