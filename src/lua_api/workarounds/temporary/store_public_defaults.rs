//! Temporary `C_StorePublic` availability defaults.
//!
//! Visibility stays here; the verified Retail/PTR `IsEnabled` query lives in `c_api`.
//! Supplied group eligibility is implemented separately under `c_api`.

const STORE_PUBLIC_DEFAULTS_LUA: &str = r#"
C_StorePublic = C_StorePublic or __wow_namespace()
if type(rawget(C_StorePublic, "_state")) ~= "table" then
    rawset(C_StorePublic, "_state", {
        shown = false,
        contextKey = nil,
    })
end

if rawget(C_StorePublic, "IsEnabled") == nil then
    function C_StorePublic.IsEnabled()
        return true
    end
end

if rawget(C_StorePublic, "EventStoreUISetShown") == nil then
    function C_StorePublic.EventStoreUISetShown(shown, contextKey)
        local state = rawget(C_StorePublic, "_state")
        state.shown = shown and true or false
        state.contextKey = contextKey
    end
end
"#;

const STORE_FRAME_ESCAPE_DEFAULTS_LUA: &str = r#"
if type(StoreFrame_EscapePressed) == "function" and not rawget(_G, "__wow_store_frame_escape_wrapped") then
    local originalEscapePressed = StoreFrame_EscapePressed
    StoreFrame_EscapePressed = function(...)
        if StoreFrame == nil then
            return false
        end
        return originalEscapePressed(...)
    end
    rawset(_G, "__wow_store_frame_escape_wrapped", true)
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(STORE_PUBLIC_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &crate::lua_api::WowLuaEnv) {
    let _ = env.exec(STORE_FRAME_ESCAPE_DEFAULTS_LUA);
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_store_public_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, String) = env
            .eval(
                r#"
                C_StorePublic.EventStoreUISetShown(true, "services")
                return rawget(C_StorePublic, "_state").shown,
                       rawget(C_StorePublic, "_state").contextKey
                "#,
            )
            .expect("C_StorePublic visibility state should run");

        assert_eq!(result, (true, "services".to_string()));
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
            env.exec("assert(rawget(C_StorePublic, 'DoesGroupHavePurchaseableProducts') == nil)")
                .expect("no declaration-backed Wrath query");
            return;
        }
        // Preserve the old group-501 assertion with an explicit observation,
        // not an invented reduction from its seeded secure-product membership.
        env.state()
            .borrow_mut()
            .store_public_eligibility
            .insert(501, true);
        let eligibility: bool = env
            .eval("return C_StorePublic.DoesGroupHavePurchaseableProducts(501)")
            .expect("supplied group observation should be queryable");
        assert!(eligibility);
    }
}
