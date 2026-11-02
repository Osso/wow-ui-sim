use super::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use rilua::{Lua, LuaApiMut};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

struct Fixture {
    root: TempDir,
    env: WowLuaEnv,
    manager: Option<SavedVariablesManager>,
}

impl Fixture {
    fn new() -> Self {
        let root = tempdir().unwrap();
        let env = WowLuaEnv::new().unwrap();
        let mut manager = SavedVariablesManager::with_storage_dir(root.path().join("saved"));
        manager.set_character("Hero", "Realm");
        register(&env, &mut manager).unwrap();
        env.exec("AccountDB = { value = 1 }; CharacterDB = { value = 2 }")
            .unwrap();
        Self {
            root,
            env,
            manager: Some(manager),
        }
    }

    fn account_path(&self) -> PathBuf {
        self.root.path().join("saved/Probe.lua")
    }

    fn character_path(&self) -> PathBuf {
        self.root.path().join("saved/Realm/Hero/Probe.lua")
    }

    fn finish(&mut self) -> crate::Result<()> {
        self.env.finalize_session(&mut self.manager)
    }

    fn save(&self) {
        self.manager
            .as_ref()
            .unwrap()
            .save_all(self.env.rilua_mut().state_mut())
            .unwrap();
    }

    fn read_back(&self, assertions: &str) {
        let mut lua = Lua::new().unwrap();
        lua.exec(&fs::read_to_string(self.account_path()).unwrap())
            .unwrap();
        lua.exec(&fs::read_to_string(self.character_path()).unwrap())
            .unwrap();
        lua.exec(assertions).unwrap();
    }
}

fn register(env: &WowLuaEnv, manager: &mut SavedVariablesManager) -> crate::Result<()> {
    manager.init_for_addon(
        env.rilua_mut().state_mut(),
        "Probe",
        &["AccountDB".into()],
        &["CharacterDB".into()],
    )
}

fn write(path: &Path, content: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn finalization_saves_both_scopes_once() {
    let mut fixture = Fixture::new();
    // Historical diagnostics do not become a newly failed shutdown callback.
    fixture
        .env
        .state()
        .borrow_mut()
        .lua_errors
        .push("old error".into());
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
    let account = fs::read(fixture.account_path()).unwrap();
    let character = fs::read(fixture.character_path()).unwrap();
    fixture
        .env
        .exec("AccountDB.value = 31; CharacterDB.value = 32")
        .unwrap();
    fixture.finish().unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), account);
    assert_eq!(fs::read(fixture.character_path()).unwrap(), character);
}

#[test]
fn serialization_failure_can_be_retried_after_the_value_is_repaired() {
    let mut fixture = Fixture::new();
    fixture.save();
    let account = fs::read(fixture.account_path()).unwrap();
    fixture.env.exec("AccountDB = function() end").unwrap();
    let error = fixture.finish().unwrap_err().to_string();
    assert!(error.contains("function"), "{error}");
    assert!(fixture.env.is_ui_environment_usable());
    assert_eq!(fs::read(fixture.account_path()).unwrap(), account);
    fixture.env.exec("AccountDB = { value = 99 }").unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 99 and CharacterDB.value == 2)");
}

#[test]
fn output_write_failure_can_be_retried_after_the_path_is_repaired() {
    let mut fixture = Fixture::new();
    fs::write(
        fixture.root.path().join("saved"),
        b"blocks output directory",
    )
    .unwrap();
    let error = fixture.finish().unwrap_err().to_string();
    assert!(!error.is_empty());
    assert!(fixture.env.is_ui_environment_usable());
    fs::remove_file(fixture.root.path().join("saved")).unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
}

#[test]
fn failed_reload_save_consumes_request_and_retries_after_output_repair() {
    let mut fixture = Fixture::new();
    fixture.env.state().borrow_mut().in_world = true;
    fixture
        .env
        .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
    fs::write(
        fixture.root.path().join("saved"),
        b"blocks output directory",
    )
    .unwrap();
    fixture.env.exec("AccountDB.value = 7; ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).is_err());
    assert!(fixture.env.is_ui_environment_usable());
    assert!(fixture.env.state().borrow().in_world);
    assert!(!fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture.env.exec("assert(AccountDB.value == 7)").unwrap();
    fs::remove_file(fixture.root.path().join("saved")).unwrap();
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 2)");
}

#[cfg(feature = "client-wrath")]
#[test]
fn failed_finish_restores_wrath_world_and_protected_action_state_for_recovery() {
    let mut fixture = Fixture::new();
    {
        let mut state = fixture.env.state().borrow_mut();
        state.in_world = true;
        state.player.in_combat = true;
        state
            .world
            .mirror_timers
            .push(crate::lua_api::state::MirrorTimer {
                name: "BREATH".into(),
                max_value: 60.0,
                ..Default::default()
            });
    }
    fixture.env.exec("AccountDB = function() end").unwrap();
    assert!(fixture.finish().is_err());
    assert!(
        fixture
            .env
            .eval::<bool>("return InCombatLockdown()")
            .unwrap()
    );
    {
        let state = fixture.env.state().borrow();
        assert!(state.in_world && state.player.in_combat);
        assert!(!state.reload_allows_protected_actions);
        assert_eq!(state.world.mirror_timers.len(), 1);
        assert_eq!(state.world.mirror_timers[0].name, "BREATH");
    }
    fixture.env.exec("AccountDB = { value = 7 }").unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 2)");
}

#[cfg(feature = "client-wrath")]
#[test]
fn first_wrath_recovery_without_prior_layout_does_not_save_failed_geometry() {
    let mut fixture = Fixture::new();
    fixture.save();
    let layout = fixture
        .root
        .path()
        .join("saved/Realm/Hero/layout-local.txt");
    assert!(!layout.exists());
    fixture
        .env
        .mark_addon_initialization_failed("Probe", "probe startup failed");
    fixture.env.exec("AccountDB.value = 99").unwrap();
    fixture
        .env
        .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    assert!(fixture.env.is_ui_environment_usable());
    fixture.env.exec("assert(AccountDB.value == 1)").unwrap();
    assert!(
        !layout.exists(),
        "recovery must not manufacture a snapshot from failed state"
    );
}

#[cfg(feature = "client-wrath")]
#[test]
fn wrath_recovery_restores_prior_layout_and_preserves_its_bytes() {
    let mut fixture = Fixture::new();
    fixture.save();
    let layout = fixture
        .root
        .path()
        .join("saved/Realm/Hero/layout-local.txt");
    let saved = b"Version: 1\nFrame: RecoveryPlaced\nFrameLevel: 7\nAnchor: TOPLEFT\nX: 17\nY: -23\nW: 111\nH: 55\n";
    write(&layout, saved);
    fixture.env.exec("local f = CreateFrame('Frame', 'RecoveryPlaced', UIParent); f:SetMovable(true); f:SetResizable(true); f:SetUserPlaced(true); f:SetSize(999, 888); f:SetPoint('CENTER', UIParent, 'CENTER')").unwrap();
    fixture
        .env
        .mark_addon_initialization_failed("Probe", "probe startup failed");
    fixture.env.set_reload_loader(|env, manager| {
        register(env, manager.as_mut().unwrap())?;
        env.exec("local f = CreateFrame('Frame', 'RecoveryPlaced', UIParent); f:SetMovable(true); f:SetResizable(true); f:SetSize(9, 8); f:SetPoint('CENTER', UIParent, 'CENTER')")
    });
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture.env.exec("local p, r, rp, x, y = RecoveryPlaced:GetPoint(); assert(p == 'TOPLEFT' and r == UIParent and rp == p and x == 17 and y == -23); assert(RecoveryPlaced:GetWidth() == 111 and RecoveryPlaced:GetHeight() == 55); assert(RecoveryPlaced:IsUserPlaced())").unwrap();
    assert_eq!(fs::read(&layout).unwrap(), saved);
}

#[cfg(feature = "client-wrath")]
#[test]
fn wrath_recovery_preserves_errors_for_existing_unreadable_or_invalid_layout() {
    for directory in [false, true] {
        let mut fixture = Fixture::new();
        fixture.save();
        let layout = fixture
            .root
            .path()
            .join("saved/Realm/Hero/layout-local.txt");
        if directory {
            fs::create_dir(&layout).unwrap();
        } else {
            write(&layout, &[0xff]);
        }
        fixture
            .env
            .mark_addon_initialization_failed("Probe", "probe startup failed");
        fixture
            .env
            .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
        fixture.env.exec("ReloadUI()").unwrap();
        let error = fixture
            .env
            .service_reload(&mut fixture.manager)
            .unwrap_err()
            .to_string();
        assert!(error.contains("read placed layout"), "{error}");
        assert!(!fixture.env.is_ui_environment_usable());
        assert!(layout.exists());
    }
}

#[cfg(feature = "client-wrath")]
#[test]
fn normal_wrath_reload_requires_its_freshly_saved_layout() {
    let mut fixture = Fixture::new();
    let layout = fixture
        .root
        .path()
        .join("saved/Realm/Hero/layout-local.txt");
    fixture.env.set_reload_loader(move |env, manager| {
        register(env, manager.as_mut().unwrap())?;
        assert!(
            layout.is_file(),
            "normal reload must write layout before reconstruction"
        );
        fs::remove_file(&layout).unwrap();
        Ok(())
    });
    fixture.env.exec("ReloadUI()").unwrap();
    let error = fixture
        .env
        .service_reload(&mut fixture.manager)
        .unwrap_err()
        .to_string();
    assert!(error.contains("read placed layout"), "{error}");
    assert!(!fixture.env.is_ui_environment_usable());
}

#[test]
fn failed_input_is_preserved_and_repaired_source_can_reload_or_finish() {
    for reload in [false, true] {
        let root = tempdir().unwrap();
        let source = root.path().join("saved/Probe.lua");
        let original = b"AccountDB = { invalid";
        write(&source, original);
        let mut env = WowLuaEnv::new().unwrap();
        env.set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
        let mut manager = SavedVariablesManager::with_storage_dir(root.path().join("saved"));
        assert!(register(&env, &mut manager).is_err());
        env.exec("AccountDB = { value = 'must not save' }; InputSentinel = true")
            .unwrap();
        let mut manager = Some(manager);
        let error = if reload {
            env.exec("ReloadUI()").unwrap();
            env.service_reload(&mut manager).unwrap_err().to_string()
        } else {
            env.finalize_session(&mut manager).unwrap_err().to_string()
        };
        assert!(error.contains("Probe account SavedVariables"), "{error}");
        assert_eq!(fs::read(&source).unwrap(), original);
        assert!(env.eval::<bool>("return InputSentinel").unwrap());
        assert!(env.is_ui_environment_usable());
        assert!(!env.service_reload(&mut manager).unwrap());
        write(&source, b"AccountDB = { value = 'repaired input' }\n");
        if reload {
            env.exec("ReloadUI()").unwrap();
            assert!(env.service_reload(&mut manager).unwrap());
        }
        env.finalize_session(&mut manager).unwrap();
        let mut read_back = Lua::new().unwrap();
        read_back
            .exec(&fs::read_to_string(&source).unwrap())
            .unwrap();
        read_back
            .exec("assert(AccountDB.value == 'repaired input')")
            .unwrap();
    }
}

#[test]
fn failed_reload_cannot_save_partial_values_and_can_reconstruct_again() {
    let mut fixture = Fixture::new();
    fixture.env.set_reload_loader(|env, manager| {
        register(env, manager.as_mut().unwrap())?;
        env.exec("AccountDB.value = 99")?;
        Err(crate::Error::Other("probe initialization failed".into()))
    });
    fixture.env.exec("AccountDB.value = 7; ReloadUI()").unwrap();
    let error = fixture
        .env
        .service_reload(&mut fixture.manager)
        .unwrap_err()
        .to_string();
    assert!(error.contains("probe initialization failed"));
    assert!(!fixture.env.is_ui_environment_usable());
    assert!(!fixture.env.service_reload(&mut fixture.manager).unwrap());
    assert_eq!(
        fixture.env.eval::<f64>("return AccountDB.value").unwrap(),
        99.0
    );
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 2)");
    assert!(fixture.finish().unwrap_err().to_string().contains(&error));
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 2)");
    fixture
        .env
        .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    assert!(fixture.env.is_ui_environment_usable());
    fixture
        .env
        .exec("assert(AccountDB.value == 7); CharacterDB.value = 8")
        .unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 8)");
}

#[test]
fn supplied_identity_class_and_latest_values_survive_reload_then_finish() {
    let mut fixture = Fixture::new();
    {
        let mut state = fixture.env.state().borrow_mut();
        state.player.name = "Hero".into();
        state.player.realm_name = Some("Realm".into());
        state.player.class_index = 8;
    }
    fixture.env.set_reload_loader(|env, manager| {
        register(env, manager.as_mut().unwrap())?;
        env.exec(
            "assert(UnitName('player') == 'Hero' and GetRealmName() == 'Realm'); \
             local _, class, index = UnitClass('player'); assert(class == 'MAGE' and index == 8)",
        )
    });
    fixture
        .env
        .exec("AccountDB.value = 7; CharacterDB.value = 8; ReloadUI()")
        .unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture
        .env
        .exec("assert(AccountDB.value == 7 and CharacterDB.value == 8); CharacterDB.value = 9")
        .unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 7 and CharacterDB.value == 9)");
}

#[test]
fn failed_addon_can_recover_without_saving_its_uninitialized_values() {
    let mut fixture = Fixture::new();
    fixture.save();
    fixture
        .env
        .mark_addon_initialization_failed("Probe", "probe startup failed");
    fixture
        .env
        .state()
        .borrow_mut()
        .lua_errors
        .push("historical probe diagnostic".into());
    fixture
        .env
        .state()
        .borrow_mut()
        .runtime_addon_diagnostics
        .warnings
        .push("historical runtime file warning".into());
    fixture.env.exec("AccountDB.value = 99").unwrap();
    assert!(
        fixture
            .finish()
            .unwrap_err()
            .to_string()
            .contains("probe startup failed")
    );
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
    fixture.env.set_reload_loader(|env, manager| {
        register(env, manager.as_mut().unwrap())?;
        env.mark_addon_initialized("Probe");
        Ok(())
    });
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture.env.exec("assert(AccountDB.value == 1)").unwrap();
    assert!(
        fixture
            .env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .any(|error| error == "historical probe diagnostic")
    );
    assert_eq!(
        fixture.env.drain_runtime_addon_diagnostics().warnings,
        vec!["historical runtime file warning"]
    );
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
}

#[test]
fn disabled_failed_owner_does_not_overwrite_its_data_or_block_healthy_output() {
    let mut fixture = Fixture::new();
    fixture.save();
    fixture.env.register_addon(crate::lua_api::AddonInfo {
        folder_name: "Probe".into(),
        enabled: false,
        ..Default::default()
    });
    fixture
        .env
        .mark_addon_initialization_failed("Probe", "probe startup failed");
    fixture
        .manager
        .as_mut()
        .unwrap()
        .init_for_addon(
            fixture.env.rilua_mut().state_mut(),
            "Healthy",
            &["HealthyDB".into()],
            &[],
        )
        .unwrap();
    fixture
        .env
        .exec("AccountDB.value = 99; HealthyDB = { value = 77 }")
        .unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
    let mut read_back = Lua::new().unwrap();
    read_back
        .exec(&fs::read_to_string(fixture.root.path().join("saved/Healthy.lua")).unwrap())
        .unwrap();
    read_back.exec("assert(HealthyDB.value == 77)").unwrap();
}

#[test]
fn unregistered_failure_and_disabled_persistence_do_not_block_generic_completion() {
    for persistence in [false, true] {
        let mut fixture = Fixture::new();
        fixture
            .env
            .mark_addon_initialization_failed("Unregistered", "diagnostic only");
        if !persistence {
            fixture.manager = None;
        }
        fixture.finish().unwrap();
        assert_eq!(fixture.account_path().exists(), persistence);
    }
}

#[cfg(any(feature = "client-wrath", feature = "client-wowforever"))]
#[test]
fn shutdown_callback_writes_complete_before_both_scopes_are_saved() {
    let mut fixture = Fixture::new();
    fixture.env.state().borrow_mut().in_world = true;
    fixture.env.exec(r#"
        ShutdownEvents = {}
        local observer = CreateFrame('Frame')
        for _, event in ipairs({'PLAYER_REGEN_ENABLED', 'PLAYER_LEAVING_WORLD', 'PLAYER_LOGOUT', 'ADDONS_UNLOADING'}) do
            observer:RegisterEvent(event)
        end
        observer:SetScript('OnEvent', function(_, event, ...)
            ShutdownEvents[#ShutdownEvents + 1] = event
            if event == 'ADDONS_UNLOADING' then
                assert(select('#', ...) == 1 and (...) == true)
                AccountDB.unloaded = true
            else
                assert(select('#', ...) == 0)
            end
            if event == 'PLAYER_LOGOUT' then
                AccountDB.value = 41
                CharacterDB.value = 42
            end
            AccountDB.events = table.concat(ShutdownEvents, ',')
        end)
    "#).unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 41 and CharacterDB.value == 42)");
    if cfg!(feature = "client-wowforever") {
        fixture.read_back("assert(AccountDB.events == 'PLAYER_LEAVING_WORLD,PLAYER_LOGOUT,ADDONS_UNLOADING' and AccountDB.unloaded == true)");
    } else {
        fixture.read_back("assert(AccountDB.events == 'PLAYER_REGEN_ENABLED,PLAYER_LEAVING_WORLD,PLAYER_LOGOUT' and AccountDB.unloaded == nil)");
    }
}

#[cfg(any(feature = "client-wrath", feature = "client-wowforever"))]
#[test]
fn callback_failure_preserves_data_and_retry_can_succeed_after_handler_repair() {
    let mut fixture = Fixture::new();
    fixture.save();
    let account = fs::read(fixture.account_path()).unwrap();
    let character = fs::read(fixture.character_path()).unwrap();
    fixture
        .env
        .exec(
            r#"
        seterrorhandler(function() end)
        local bad, good = CreateFrame('Frame'), CreateFrame('Frame')
        ShutdownBadFrame = bad
        bad:RegisterEvent('PLAYER_LOGOUT')
        good:RegisterEvent('PLAYER_LOGOUT')
        bad:SetScript('OnEvent', function() error('shutdown probe failed') end)
        good:SetScript('OnEvent', function() AccountDB.value = 91; CharacterDB.value = 92 end)
    "#,
        )
        .unwrap();
    let error = fixture.finish().unwrap_err().to_string();
    assert!(error.contains("shutdown probe failed"), "{error}");
    fixture
        .env
        .exec("assert(AccountDB.value == 91 and CharacterDB.value == 92)")
        .unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), account);
    assert_eq!(fs::read(fixture.character_path()).unwrap(), character);
    fixture
        .env
        .exec("ShutdownBadFrame:SetScript('OnEvent', function() end)")
        .unwrap();
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 91 and CharacterDB.value == 92)");
}

#[cfg(any(feature = "client-wrath", feature = "client-wowforever"))]
#[test]
fn reload_recovers_from_an_unowned_old_shutdown_error_without_saving_failed_values() {
    let mut fixture = Fixture::new();
    fixture.save();
    fixture
        .env
        .exec(
            r#"
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('PLAYER_LOGOUT')
        observer:SetScript('OnEvent', function()
            AccountDB.value = 99
            error('old shutdown handler failed')
        end)
    "#,
        )
        .unwrap();
    fixture
        .env
        .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    assert!(fixture.env.is_ui_environment_usable());
    fixture.env.exec("assert(AccountDB.value == 1)").unwrap();
    assert!(
        fixture
            .env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .any(|error| error.contains("old shutdown handler failed"))
    );
    fixture.finish().unwrap();
    fixture.read_back("assert(AccountDB.value == 1 and CharacterDB.value == 2)");
}

#[test]
fn unowned_initialization_errors_require_safe_reconstruction_and_keep_diagnostics() {
    let mut fixture = Fixture::new();
    fixture.save();
    fixture.env.state().borrow_mut().lua_error_records.push(
        crate::lua_api::state::LuaErrorRecord {
            message: "shared initialization failed".into(),
            addon_name: None,
        },
    );
    fixture.env.record_addon_initialization_errors_since(0);
    assert!(!fixture.env.is_ui_environment_usable());
    fixture.env.exec("AccountDB.value = 99").unwrap();
    assert!(fixture.finish().is_err());
    fixture
        .env
        .set_reload_loader(|env, manager| register(env, manager.as_mut().unwrap()));
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    assert!(fixture.env.is_ui_environment_usable());
    fixture.env.exec("assert(AccountDB.value == 1)").unwrap();
    assert_eq!(
        fixture.env.state().borrow().lua_error_records[0].message,
        "shared initialization failed"
    );
    fixture.finish().unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_reload_and_finish_use_distinct_unloading_payloads_without_repeated_world_departure() {
    let mut fixture = Fixture::new();
    let install = |env: &WowLuaEnv| {
        env.exec(
            r#"
        AccountDB.shutdowns = AccountDB.shutdowns or {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('PLAYER_LEAVING_WORLD')
        observer:RegisterEvent('ADDONS_UNLOADING')
        observer:SetScript('OnEvent', function(_, event, closing)
            if event == 'PLAYER_LEAVING_WORLD' then error('not in world') end
            AccountDB.shutdowns[#AccountDB.shutdowns + 1] = closing
        end)
    "#,
        )
    };
    install(&fixture.env).unwrap();
    fixture.env.set_reload_loader(move |env, manager| {
        register(env, manager.as_mut().unwrap())?;
        install(env)
    });
    fixture.env.exec("ReloadUI()").unwrap();
    assert!(fixture.env.service_reload(&mut fixture.manager).unwrap());
    fixture.finish().unwrap();
    fixture.read_back("assert(#AccountDB.shutdowns == 2 and AccountDB.shutdowns[1] == false and AccountDB.shutdowns[2] == true)");
}
