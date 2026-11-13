//! Temporary `C_StorePublic` availability defaults.
//!
//! Visibility stays here; the verified Retail/PTR/Forever `IsEnabled` query lives in `c_api`.
//! Supplied group eligibility is implemented separately under `c_api`.

const STORE_PUBLIC_DEFAULTS_LUA: &str = r#"
C_StorePublic = C_StorePublic or __wow_namespace()
if type(rawget(C_StorePublic, "_state")) ~= "table" then
    rawset(C_StorePublic, "_state", {
        shown = false,
        contextKey = nil,
    })
end

if rawget(C_StorePublic, "EventStoreUISetShown") == nil then
    function C_StorePublic.EventStoreUISetShown(shown, contextKey)
        local state = rawget(C_StorePublic, "_state")
        state.shown = shown and true or false
        state.contextKey = contextKey
    end
end
"#;

// Guessed fallback retained only on Mists, Era, Anniversary and Wrath.
// Those profiles have no R38/R75/R87 evidence and remain out of scope.
const UNEVIDENCED_STORE_AVAILABILITY_FALLBACK_LUA: &str = r#"
if rawget(C_StorePublic, "IsEnabled") == nil then
    function C_StorePublic.IsEnabled()
        return true
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
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Mists
            | crate::client_profile::ClientProfile::Era
            | crate::client_profile::ClientProfile::Anniversary
            | crate::client_profile::ClientProfile::Wrath
    ) {
        lua.exec(UNEVIDENCED_STORE_AVAILABILITY_FALLBACK_LUA)?;
    }
    Ok(())
}

pub(crate) fn patch(env: &crate::lua_api::WowLuaEnv) {
    let _ = env.exec(STORE_FRAME_ESCAPE_DEFAULTS_LUA);
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(any(
        feature = "client-ptr",
        feature = "client-retail",
        feature = "client-wowforever"
    ))]
    #[test]
    fn evidenced_profiles_do_not_guess_missing_store_availability() {
        let env = WowLuaEnv::new().unwrap();
        env.exec("C_StorePublic.IsEnabled = nil").unwrap();
        super::apply_bootstrap(&mut env.lua.borrow_mut()).unwrap();
        env.exec("assert(rawget(C_StorePublic, 'IsEnabled') == nil)")
            .unwrap();
    }

    #[cfg(feature = "client-wowforever")]
    #[test]
    fn forever_store_enabled_reads_server_mirror_in_game_and_glue() {
        for screen in [
            crate::screen::ScreenKind::Game,
            crate::screen::ScreenKind::CharacterSelect,
        ] {
            let env = WowLuaEnv::new().unwrap();
            env.set_screen_mode(screen);
            assert!(
                !env.eval::<bool>("return C_StorePublic.IsEnabled()")
                    .unwrap()
            );
            env.set_server_mirrored_cvar("bpayStoreEnable", "1")
                .unwrap();
            assert!(
                env.eval::<bool>("return C_StorePublic.IsEnabled()")
                    .unwrap()
            );
            env.set_server_mirrored_cvar("bpayStoreEnable", "0")
                .unwrap();
            assert!(
                !env.eval::<bool>("return C_StorePublic.IsEnabled()")
                    .unwrap()
            );
        }
    }

    #[test]
    fn installs_store_public_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) {
            env.set_server_mirrored_cvar("bpayStoreEnable", "0")
                .expect("store default input should be accepted");
        }

        let result: (bool, bool, String) = env
            .eval(
                r#"
                C_StorePublic.EventStoreUISetShown(true, "services")
                return C_StorePublic.IsEnabled(),
                       rawget(C_StorePublic, "_state").shown,
                       rawget(C_StorePublic, "_state").contextKey
                "#,
            )
            .expect("C_StorePublic visibility state should run");

        let expected_enabled = !matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
                | crate::client_profile::ClientProfile::WowForever
        );
        assert_eq!(result, (expected_enabled, true, "services".to_string()));
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
