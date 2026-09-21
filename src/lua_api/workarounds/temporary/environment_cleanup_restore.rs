//! Temporary EnvironmentCleanup runtime-surface restore.
//!
//! The simulator loads Blizzard_EnvironmentCleanup in the same run as later UI
//! addons, so globals that the cleanup file nils still need to be restored for
//! the rest of startup. Keep that repair out of the generic globals surface.

use crate::lua_api::SimState;
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
    crate::c_api::register_utility_bootstrap_tables(lua.state_mut())?;
    super::debug_environment_defaults::apply_bootstrap(lua)?;
    super::gamepad_cursor_control_defaults::apply_bootstrap(lua)?;
    super::ui_parent_panel_toggles::apply_bootstrap(lua)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn post_cleanup_restore_preserves_seconds_formatter_namespace_and_factory() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let expected = if cfg!(feature = "client-ptr") {
            "12 seconds"
        } else {
            "12"
        };
        env.exec(
            r#"
            assert(type(C_StringUtil.CreateSecondsFormatter) == 'function',
                'formatter must exist before restoration')
            secondsRestore = {
                namespace = C_StringUtil,
                factory = C_StringUtil.CreateSecondsFormatter,
                formatter = C_StringUtil.CreateSecondsFormatter(),
            }
            assert(__secureenv.C_StringUtil == secondsRestore.namespace)
            assert(__secureenv.C_StringUtil.CreateSecondsFormatter == secondsRestore.factory)
            "#,
        )
        .expect("initial public and secure environments share the formatter factory");
        let initial: String = env
            .eval("return secondsRestore.formatter:Format(12)")
            .expect("initial formatter handles the numeric fixture");
        assert_eq!(initial, expected);

        env.restore_post_cleanup_globals();

        env.exec(
            r#"
            assert(C_StringUtil == secondsRestore.namespace,
                'restoration must preserve the namespace')
            assert(C_StringUtil.CreateSecondsFormatter == secondsRestore.factory,
                'restoration must preserve the existing factory')
            assert(__secureenv.C_StringUtil == C_StringUtil,
                'public and secure namespaces must remain consistent')
            assert(__secureenv.C_StringUtil.CreateSecondsFormatter == secondsRestore.factory)
            "#,
        )
        .expect("restoration preserves public and secure formatter publication");
        let formatted: (String, String, String) = env
            .eval(
                r#"
                return secondsRestore.formatter:Format(12),
                    C_StringUtil.CreateSecondsFormatter():Format(12),
                    __secureenv.C_StringUtil.CreateSecondsFormatter():Format(12)
                "#,
            )
            .expect("existing and newly constructed formatters still format numeric input");
        assert_eq!(
            formatted,
            (expected.into(), expected.into(), expected.into())
        );
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
