use crate::lua_api::{AddonInfo, WowLuaEnv};
use crate::saved_variables::SavedVariablesManager;
use rilua::LuaApiMut;
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

struct Fixture {
    root: TempDir,
    env: WowLuaEnv,
    manager: Option<SavedVariablesManager>,
}

impl Fixture {
    fn new(toc_extra: &str, source: &str) -> Self {
        let root = tempdir().unwrap();
        let addon = root.path().join("RuntimeProbe");
        fs::create_dir(&addon).unwrap();
        fs::write(
            addon.join("RuntimeProbe.toc"),
            format!("## Title: RuntimeProbe\n{toc_extra}\nRuntimeProbe.lua\n"),
        )
        .unwrap();
        fs::write(addon.join("RuntimeProbe.lua"), source).unwrap();
        let env = WowLuaEnv::new().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
        let mut manager = SavedVariablesManager::with_storage_dir(root.path().join("saved"));
        manager
            .init_for_addon(
                env.rilua_mut().state_mut(),
                "RuntimeProbe",
                &["ProbeDB".into()],
                &[],
            )
            .unwrap();
        manager
            .init_for_addon(
                env.rilua_mut().state_mut(),
                "HealthyProbe",
                &["HealthyDB".into()],
                &[],
            )
            .unwrap();
        env.exec(
            "ProbeDB = { value = 'preceding save' }; HealthyDB = { value = 'healthy baseline' }",
        )
        .unwrap();
        manager.save_all(env.rilua_mut().state_mut()).unwrap();
        Self {
            root,
            env,
            manager: Some(manager),
        }
    }

    fn saved_path(&self) -> PathBuf {
        self.root.path().join("saved/RuntimeProbe.lua")
    }

    fn load(&self) -> (bool, Option<String>) {
        self.env
            .eval("return C_AddOns.LoadAddOn('RuntimeProbe')")
            .unwrap()
    }

    fn assert_failed_owner_recovers(&mut self, expected: &str) {
        let saved = fs::read(self.saved_path()).unwrap();
        self.env
            .exec("ProbeDB.value = 'must not save'; HealthyDB.value = 'healthy unsaved edit'")
            .unwrap();
        let error = self
            .env
            .finalize_session(&mut self.manager)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert_eq!(fs::read(self.saved_path()).unwrap(), saved);

        fs::write(
            self.root.path().join("RuntimeProbe/RuntimeProbe.toc"),
            "## Title: RuntimeProbe\nRuntimeProbe.lua\n",
        )
        .unwrap();
        fs::write(
            self.root.path().join("RuntimeProbe/RuntimeProbe.lua"),
            "ProbeDB.value = 'recovered edit'",
        )
        .unwrap();
        self.env.set_reload_loader(|env, manager| {
            let manager = manager.as_mut().unwrap();
            for (addon, variable) in [("RuntimeProbe", "ProbeDB"), ("HealthyProbe", "HealthyDB")] {
                manager.init_for_addon(
                    env.rilua_mut().state_mut(),
                    addon,
                    &[variable.into()],
                    &[],
                )?;
            }
            env.exec("assert(C_AddOns.LoadAddOn('RuntimeProbe'))")
        });
        self.env.exec("ReloadUI()").unwrap();
        assert!(self.env.service_reload(&mut self.manager).unwrap());
        self.env
            .exec("assert(ProbeDB.value == 'recovered edit' and HealthyDB.value == 'healthy unsaved edit')")
            .unwrap();
        assert_eq!(fs::read(self.saved_path()).unwrap(), saved);
        self.env.finalize_session(&mut self.manager).unwrap();
        let mut restored = rilua::Lua::new().unwrap();
        restored
            .exec(&fs::read_to_string(self.saved_path()).unwrap())
            .unwrap();
        restored
            .exec(&fs::read_to_string(self.root.path().join("saved/HealthyProbe.lua")).unwrap())
            .unwrap();
        restored.exec("assert(ProbeDB.value == 'recovered edit' and HealthyDB.value == 'healthy unsaved edit')").unwrap();
    }
}

#[test]
fn runtime_lua_file_failure_prevents_save_without_changing_load_return_values() {
    let mut fixture = Fixture::new("", "error('runtime file failure probe')");
    // The existing loader reports file failures as warnings while returning success.
    assert_eq!(fixture.load(), (true, None));
    fixture.assert_failed_owner_recovers("runtime file failure probe");
}

#[test]
fn runtime_addon_loaded_callback_failure_is_retained_with_a_custom_error_handler() {
    let mut fixture = Fixture::new(
        "",
        r#"
        seterrorhandler(function() end)
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('ADDON_LOADED')
        observer:SetScript('OnEvent', function(_, _, name)
            if name == 'RuntimeProbe' then error('runtime callback failure probe') end
        end)
        "#,
    );
    assert_eq!(fixture.load(), (true, None));
    fixture.assert_failed_owner_recovers("runtime callback failure probe");
}

#[test]
fn runtime_xml_onload_callback_failure_is_retained() {
    let mut fixture = Fixture::new("Frames.xml", "RuntimeProbeCodeRan = true");
    fs::write(
        fixture.root.path().join("RuntimeProbe/Frames.xml"),
        r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="RuntimeOnLoadProbe" parent="UIParent">
                <Scripts><OnLoad>
                    RuntimeOnLoadProbeRan = true
                    self.onLoadProbeRan = true
                    error('runtime OnLoad failure probe')
                </OnLoad></Scripts>
            </Frame>
        </Ui>"#,
    )
    .unwrap();
    assert_eq!(fixture.load(), (true, None));
    fixture
        .env
        .exec(
            "assert(RuntimeOnLoadProbe ~= nil and type(RuntimeOnLoadProbe:GetScript('OnLoad')) == 'function'); \
             assert(RuntimeOnLoadProbeRan == true and RuntimeOnLoadProbe.onLoadProbeRan == true)",
        )
        .unwrap();
    fixture.assert_failed_owner_recovers("runtime OnLoad failure probe");
}

#[test]
fn failed_required_dependency_of_an_enabled_runtime_addon_prevents_save() {
    let mut fixture = Fixture::new(
        "## Dependencies: AbsentRequiredProbe",
        "RuntimeProbeCodeRan = true",
    );
    let (loaded, reason) = fixture.load();
    assert!(!loaded);
    assert!(reason.unwrap().contains("AbsentRequiredProbe"));
    assert!(
        fixture
            .env
            .eval::<bool>("return RuntimeProbeCodeRan == nil")
            .unwrap()
    );
    fixture.assert_failed_owner_recovers("AbsentRequiredProbe");
}

#[test]
fn absent_disabled_optional_probes_and_nil_observations_do_not_reject_a_complete_save() {
    let mut fixture = Fixture::new(
        "## OptionalDeps: AbsentOptionalProbe",
        "local _ = OptionalRuntimeGlobalProbe; RuntimeProbeCodeRan = true",
    );
    fixture.env.register_addon(AddonInfo {
        folder_name: "DisabledRuntimeProbe".into(),
        enabled: false,
        ..Default::default()
    });
    fixture
        .env
        .exec(
            "local loaded, reason = C_AddOns.LoadAddOn('AbsentOptionalProbe'); \
             assert(loaded == false and reason == 'MISSING'); \
             loaded, reason = C_AddOns.LoadAddOn('DisabledRuntimeProbe'); \
             assert(loaded == false and reason == 'DISABLED')",
        )
        .unwrap();
    assert_eq!(fixture.load(), (true, None));
    assert!(
        fixture
            .env
            .eval::<bool>("return RuntimeProbeCodeRan == true")
            .unwrap()
    );
    fixture.env.exec("ProbeDB.value = 'complete save'").unwrap();
    fixture.env.finalize_session(&mut fixture.manager).unwrap();
    let saved = fs::read_to_string(fixture.saved_path()).unwrap();
    assert!(saved.contains("complete save"), "{saved}");
}

#[test]
fn glue_only_required_dependency_is_not_loaded_in_game() {
    let root = tempdir().unwrap();
    let game_addon = root.path().join("GameRuntimeProbe");
    fs::create_dir(&game_addon).unwrap();
    fs::write(
        game_addon.join("GameRuntimeProbe.toc"),
        "## Title: GameRuntimeProbe\n## Dep: GlueRuntimeProbe\nGameRuntimeProbe.lua\n",
    )
    .unwrap();
    fs::write(
        game_addon.join("GameRuntimeProbe.lua"),
        "GameRuntimeProbeRan = true",
    )
    .unwrap();
    let glue_addon = root.path().join("GlueRuntimeProbe");
    fs::create_dir(&glue_addon).unwrap();
    fs::write(
        glue_addon.join("GlueRuntimeProbe.toc"),
        "## Title: GlueRuntimeProbe\n## AllowLoad: Glue\nGlueRuntimeProbe.lua\n",
    )
    .unwrap();
    fs::write(
        glue_addon.join("GlueRuntimeProbe.lua"),
        "GlueRuntimeProbeRan = true",
    )
    .unwrap();

    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('GameRuntimeProbe')")
        .unwrap();
    assert!(loaded, "{reason:?}");
    let (game_ran, glue_ran, glue_loaded): (bool, bool, bool) = env
        .eval(
            "return GameRuntimeProbeRan == true, GlueRuntimeProbeRan == true, \
             C_AddOns.IsAddOnLoaded('GlueRuntimeProbe')",
        )
        .unwrap();
    assert!(game_ran, "the game addon's own files should run");
    assert!(!glue_ran, "a Glue-only dependency must not run in Game");
    assert!(!glue_loaded, "a Glue-only dependency must not be marked loaded");
}
