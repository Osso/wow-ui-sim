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
    let missing_reason = if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        "DEP_MISSING"
    } else {
        "AbsentRequiredProbe"
    };
    assert!(reason.unwrap().contains(missing_reason));
    assert!(
        fixture
            .env
            .eval::<bool>("return RuntimeProbeCodeRan == nil")
            .unwrap()
    );
    fixture.assert_failed_owner_recovers(missing_reason);
}

#[test]
fn runtime_optional_dependency_loads_before_required_dependency() {
    let root = tempdir().unwrap();
    for (name, metadata, source) in [
        (
            "RuntimeOrderParent",
            "## OptionalDeps: RuntimeOrderOptional\n## Dependencies: RuntimeOrderRequired\n",
            "table.insert(runtimeDependencyOrder, 'parent')",
        ),
        (
            "RuntimeOrderRequired",
            "",
            "table.insert(runtimeDependencyOrder, 'required')",
        ),
        (
            "RuntimeOrderOptional",
            "",
            "table.insert(runtimeDependencyOrder, 'optional')",
        ),
    ] {
        let directory = root.path().join(name);
        fs::create_dir(&directory).unwrap();
        fs::write(
            directory.join(format!("{name}.toc")),
            format!("## Title: {name}\n{metadata}{name}.lua\n"),
        )
        .unwrap();
        fs::write(directory.join(format!("{name}.lua")), source).unwrap();
    }

    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
    env.exec("runtimeDependencyOrder = {}").unwrap();
    env.exec("assert(C_AddOns.LoadAddOn('RuntimeOrderParent'))")
        .unwrap();
    env.exec("assert(table.concat(runtimeDependencyOrder, ',') == 'optional,required,parent')")
        .unwrap();
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

struct ScreenDependencyFixture {
    _root: TempDir,
    env: WowLuaEnv,
}

impl ScreenDependencyFixture {
    fn new(probe_toc_extra: &str) -> Self {
        let root = tempdir().unwrap();
        let probe = root.path().join("RuntimeScreenProbe");
        fs::create_dir(&probe).unwrap();
        fs::write(
            probe.join("RuntimeScreenProbe.toc"),
            format!("## Title: RuntimeScreenProbe\n{probe_toc_extra}\nRuntimeScreenProbe.lua\n"),
        )
        .unwrap();
        fs::write(
            probe.join("RuntimeScreenProbe.lua"),
            "RuntimeScreenProbeRan = true",
        )
        .unwrap();

        let glue_only = root.path().join("Blizzard_GlueOnlyProbe");
        fs::create_dir(&glue_only).unwrap();
        fs::write(
            glue_only.join("Blizzard_GlueOnlyProbe.toc"),
            "## Title: Blizzard_GlueOnlyProbe\n## AllowLoad: Glue\nBlizzard_GlueOnlyProbe.lua\n",
        )
        .unwrap();
        fs::write(
            glue_only.join("Blizzard_GlueOnlyProbe.lua"),
            "Blizzard_GlueOnlyProbeRan = true",
        )
        .unwrap();

        let env = WowLuaEnv::new().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
        Self { _root: root, env }
    }

    fn load(&self) -> (bool, Option<String>) {
        self.env
            .eval("return C_AddOns.LoadAddOn('RuntimeScreenProbe')")
            .unwrap()
    }

    fn probe_ran(&self) -> bool {
        self.env
            .eval("return RuntimeScreenProbeRan == true")
            .unwrap()
    }

    fn glue_only_loaded(&self) -> bool {
        self.env
            .eval("return Blizzard_GlueOnlyProbeRan == true")
            .unwrap()
    }
}

#[test]
fn runtime_required_dependency_for_another_screen_still_blocks_load() {
    let fixture = ScreenDependencyFixture::new("## Dependencies: Blizzard_GlueOnlyProbe");
    let (loaded, reason) = fixture.load();
    assert!(
        !loaded,
        "a required Glue-only dependency must still block a Game-screen runtime load"
    );
    let reason = reason.expect("a blocked load should report a reason");
    assert_eq!(reason, "DEP_WRONG_ACTIVE_INTERFACE");
    assert!(!fixture.probe_ran(), "blocked addon files must not run");
    assert!(!fixture.glue_only_loaded());
}

#[test]
fn runtime_optional_dependency_for_another_screen_is_skipped() {
    let fixture = ScreenDependencyFixture::new("## OptionalDeps: Blizzard_GlueOnlyProbe");
    assert_eq!(fixture.load(), (true, None));
    assert!(fixture.probe_ran());
    assert!(!fixture.glue_only_loaded());
}

#[test]
fn runtime_screen_annotated_dependency_is_skipped_outside_its_screen() {
    let fixture = ScreenDependencyFixture::new("## Dep: Blizzard_GlueOnlyProbe [AllowLoad glue]");
    assert_eq!(fixture.load(), (true, None));
    assert!(fixture.probe_ran());
    assert!(!fixture.glue_only_loaded());
}
