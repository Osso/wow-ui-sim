//! Temporary EnvironmentCleanup runtime-surface restore.
//!
//! This workaround restores shared bootstrap globals after cleanup. The
//! cleanup-removed secure namespaces remain absent from the public environment.

use crate::lua_api::SimState;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::LuaApiMut;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn restore_post_cleanup_globals(
    lua: &mut rilua::Lua,
    _state: Rc<RefCell<SimState>>,
) -> crate::Result<()> {
    crate::lua_api::env_init::init_shared_bootstrap(lua)?;
    crate::lua_api::env_init::init_runtime_surface_bootstrap(lua)?;
    crate::lua_api::env_init::init_enum_globals(lua)?;
    crate::lua_api::globals::strings::restore_missing_ui_strings(lua)?;
    super::debug_environment_defaults::apply_bootstrap(lua)?;
    crate::lua_api::globals::real::secure_helpers::register(lua)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    crate::lua_api::globals::real::input_interface::register_all(lua)?;
    super::gamepad_cursor_control_defaults::apply_bootstrap(lua)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    {
        crate::c_api::secure_transfer::register_secure_environment(lua.state_mut())?;
        crate::lua_api::workarounds::refresh_missing_globals_for_secure_addon(lua.state_mut())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    const INITIAL_FORMATTER_SETUP: &str = r#"
            assert(type(C_StringUtil.CreateSecondsFormatter) == 'function',
                'formatter must exist before restoration')
            secondsRestore = {
                formatter = C_StringUtil.CreateSecondsFormatter(),
            }
            "#;

    const INITIAL_SECURE_FORMATTER_SETUP: &str = r#"
                local public_factory = C_StringUtil.CreateSecondsFormatter
                local secure_factory = __secureenv.C_StringUtil.CreateSecondsFormatter
                assert(type(secure_factory) == 'function',
                    'secure formatter factory must exist before restoration')
                assert(not rawequal(public_factory, secure_factory),
                    'native initial publication constructs separate factory functions')
                secondsRestore.secure_formatter = secure_factory()
                "#;

    const ORDINARY_FORMATTER_READY_PROBE: &str = r#"
                return type(C_StringUtil) == 'table'
                    and type(C_StringUtil.CreateSecondsFormatter) == 'function'
                "#;

    const ORDINARY_FORMATTED_PROBE: &str = r#"
                return secondsRestore.formatter:Format(12),
                    C_StringUtil.CreateSecondsFormatter():Format(12)
                "#;

    const SECURE_FORMATTER_READY_PROBE: &str = r#"
                    return type(__secureenv.C_StringUtil) == 'table'
                        and type(__secureenv.C_StringUtil.CreateSecondsFormatter) == 'function'
                    "#;

    const SECURE_FORMATTED_PROBE: &str = r#"
                    return secondsRestore.secure_formatter:Format(12),
                        __secureenv.C_StringUtil.CreateSecondsFormatter():Format(12)
                    "#;

    #[test]
    fn post_cleanup_restore_preserves_seconds_formatter_namespace_and_factory() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let expected = if cfg!(feature = "native-duration-formatting") {
            "12 seconds"
        } else {
            "12"
        };
        let retail_secure_publication = crate::client_profile::is_active_retail_12_1_5();
        install_initial_seconds_formatters(&env, retail_secure_publication);
        assert_initial_seconds_formatters(&env, expected, retail_secure_publication);
        env.restore_post_cleanup_globals();
        assert_restored_ordinary_seconds_formatters(&env, expected);
        if retail_secure_publication {
            assert_restored_secure_seconds_formatters(&env, expected);
        }
    }

    fn install_initial_seconds_formatters(env: &WowLuaEnv, retail_secure_publication: bool) {
        env.exec(INITIAL_FORMATTER_SETUP)
            .expect("ordinary formatter exists before restoration");
        if retail_secure_publication {
            env.exec(INITIAL_SECURE_FORMATTER_SETUP)
                .expect("Retail publishes a distinct secure formatter factory");
        }
    }

    fn assert_initial_seconds_formatters(
        env: &WowLuaEnv,
        expected: &str,
        retail_secure_publication: bool,
    ) {
        let initial: String = env
            .eval("return secondsRestore.formatter:Format(12)")
            .expect("initial formatter handles the numeric fixture");
        assert_eq!(initial, expected);
        if retail_secure_publication {
            let secure_initial: String = env
                .eval("return secondsRestore.secure_formatter:Format(12)")
                .expect("initial secure formatter handles the numeric fixture");
            assert_eq!(secure_initial, expected);
        }
    }

    fn assert_restored_ordinary_seconds_formatters(env: &WowLuaEnv, expected: &str) {
        let ordinary_ready: bool = env
            .eval(ORDINARY_FORMATTER_READY_PROBE)
            .expect("ordinary formatter publication should remain callable");
        assert!(ordinary_ready);
        let ordinary_formatted: (String, String) = env
            .eval(ORDINARY_FORMATTED_PROBE)
            .expect("existing and newly created ordinary formatters still work");
        assert_eq!(ordinary_formatted, (expected.into(), expected.into()));
    }

    fn assert_restored_secure_seconds_formatters(env: &WowLuaEnv, expected: &str) {
        let secure_ready: bool = env
            .eval(SECURE_FORMATTER_READY_PROBE)
            .expect("secure formatter publication should remain callable");
        assert!(secure_ready);
        let secure_formatted: (String, String) = env
            .eval(SECURE_FORMATTED_PROBE)
            .expect("existing and newly created secure formatters still work");
        assert_eq!(secure_formatted, (expected.into(), expected.into()));
    }

    #[test]
    fn post_cleanup_restore_reinstalls_gamepad_cursor_defaults() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        env.exec(
            r#"
            CanAutoSetGamePadCursorControl = nil
            SetGamePadCursorControl = nil
            "#,
        )
        .expect("gamepad cursor-control defaults should clear");

        env.restore_post_cleanup_globals();

        let restored: (String, String, bool, bool) = env
            .eval(
                r#"
                return type(CanAutoSetGamePadCursorControl),
                    type(SetGamePadCursorControl),
                    CanAutoSetGamePadCursorControl(true) == false,
                    pcall(SetGamePadCursorControl, true)
                "#,
            )
            .expect("restored gamepad cursor-control defaults should run");
        assert_eq!(restored, ("function".into(), "function".into(), true, true));
    }

    #[test]
    fn post_cleanup_restore_does_not_report_its_own_store_namespace_lookup() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        env.exec("C_StoreSecure = nil")
            .expect("store namespace should clear");
        env.state().borrow_mut().nil_symbol_accesses.clear();

        env.restore_post_cleanup_globals();

        let store_accesses = env
            .state()
            .borrow()
            .nil_symbol_accesses
            .iter()
            .filter(|access| access.key == "C_StoreSecure")
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            store_accesses.is_empty(),
            "runtime bootstrap must not attribute its own namespace restoration as a client lookup: {store_accesses:?}"
        );
    }

    #[test]
    fn post_cleanup_restore_preserves_existing_deprecation_constants() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let expected_store_buy: String = env
            .eval("return BLIZZARD_STORE_BUY")
            .expect("bootstrap store string should be readable");
        env.exec(
            r#"
            LE_AUTOCOMPLETE_PRIORITY_OTHER = Enum.AutoCompletePriority.Other
            LE_AUTOCOMPLETE_PRIORITY_INTERACTED = Enum.AutoCompletePriority.Interacted
            LE_AUTOCOMPLETE_PRIORITY_IN_GROUP = Enum.AutoCompletePriority.InGroup
            LE_AUTOCOMPLETE_PRIORITY_GUILD = Enum.AutoCompletePriority.Guild
            LE_AUTOCOMPLETE_PRIORITY_FRIEND = Enum.AutoCompletePriority.Friend
            LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER = Enum.AutoCompletePriority.AccountCharacter
            LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER_SAME_REALM = Enum.AutoCompletePriority.AccountCharacterSameRealm

            COMBATLOG_OBJECT_RAIDTARGET1 = Enum.CombatLogObjectTarget.Raidtarget1
            COMBATLOG_OBJECT_RAIDTARGET2 = Enum.CombatLogObjectTarget.Raidtarget2
            COMBATLOG_OBJECT_RAIDTARGET3 = Enum.CombatLogObjectTarget.Raidtarget3
            COMBATLOG_OBJECT_RAIDTARGET4 = Enum.CombatLogObjectTarget.Raidtarget4
            COMBATLOG_OBJECT_RAIDTARGET5 = Enum.CombatLogObjectTarget.Raidtarget5
            COMBATLOG_OBJECT_RAIDTARGET6 = Enum.CombatLogObjectTarget.Raidtarget6
            COMBATLOG_OBJECT_RAIDTARGET7 = Enum.CombatLogObjectTarget.Raidtarget7
            COMBATLOG_OBJECT_RAIDTARGET8 = Enum.CombatLogObjectTarget.Raidtarget8

            BLIZZARD_STORE_BUY = nil
            "#,
        )
        .expect("Blizzard deprecation constants should assign");

        env.restore_post_cleanup_globals();

        let restored_store_buy: String = env
            .eval("return BLIZZARD_STORE_BUY")
            .expect("missing store string should be restored");
        assert_eq!(restored_store_buy, expected_store_buy);

        let autocomplete: (i32, i32, i32, i32, i32, i32, i32) = env
            .eval(
                r#"
                return LE_AUTOCOMPLETE_PRIORITY_OTHER,
                    LE_AUTOCOMPLETE_PRIORITY_INTERACTED,
                    LE_AUTOCOMPLETE_PRIORITY_IN_GROUP,
                    LE_AUTOCOMPLETE_PRIORITY_GUILD,
                    LE_AUTOCOMPLETE_PRIORITY_FRIEND,
                    LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER,
                    LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER_SAME_REALM
                "#,
            )
            .expect("autocomplete constants should remain readable");
        assert_eq!(autocomplete, (0, 1, 2, 3, 4, 5, 6));

        let raid_targets: (i32, i32, i32, i32, i32, i32, i32, i32) = env
            .eval(
                r#"
                return COMBATLOG_OBJECT_RAIDTARGET1,
                    COMBATLOG_OBJECT_RAIDTARGET2,
                    COMBATLOG_OBJECT_RAIDTARGET3,
                    COMBATLOG_OBJECT_RAIDTARGET4,
                    COMBATLOG_OBJECT_RAIDTARGET5,
                    COMBATLOG_OBJECT_RAIDTARGET6,
                    COMBATLOG_OBJECT_RAIDTARGET7,
                    COMBATLOG_OBJECT_RAIDTARGET8
                "#,
            )
            .expect("raid-target constants should remain readable");
        assert_eq!(raid_targets, (1, 2, 4, 8, 16, 32, 64, 128));
    }
}
