//! Real-addon payloads reach the unchanged Retail Settings keybinding consumer.
use super::*;
use clap::Parser;

const PAYLOAD_CHILD_ENV: &str = "WOW_SIM_RELOAD_PAYLOAD_CHILD";
const PAYLOAD_TEST_NAME: &str = "addon_loading::reload_payload_tests::reload_real_addon_payload_retains_original_settings_consumer";

fn assert_payload_child_process() {
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            PAYLOAD_TEST_NAME,
            "--nocapture",
            "--test-threads=1",
        ])
        .env(PAYLOAD_CHILD_ENV, "1")
        .status()
        .unwrap();
    assert!(status.success(), "original Settings payload child failed");
}

#[derive(Debug, PartialEq)]
struct PayloadObservation {
    name: String,
    argument_count: i32,
    value_type: String,
    contains_bindings: bool,
    displayed_label: bool,
}

fn observe(env: &WowLuaEnv, target: &str, label: &str) {
    env.exec(&format!(
        r#"
        ReloadPayloadTarget = {target:?}
        ReloadPayloadCount = -1
        ReloadPayloadType = '<not-seen>'
        ReloadPayloadBindings = nil
        BINDING_HEADER_OTHER = {label:?}
        if not ReloadPayloadObserver then
            ReloadPayloadObserver = CreateFrame('Frame')
            ReloadPayloadObserver:RegisterEvent('ADDON_LOADED')
            ReloadPayloadObserver:SetScript('OnEvent', function(self, event, ...)
                local name, containsBindings = ...
                if name == ReloadPayloadTarget then
                    ReloadPayloadCount = select('#', ...)
                    ReloadPayloadType = type(containsBindings)
                    ReloadPayloadBindings = containsBindings
                end
            end)
        end
    "#
    ))
    .unwrap();
}

fn read_payload_and_original_label(env: &WowLuaEnv, name: &str, label: &str) -> PayloadObservation {
    env.exec("SettingsPanel:Hide(); Settings.OpenToCategory(Settings.KEYBINDINGS_CATEGORY_ID)")
        .unwrap();
    let displayed_label = {
        let state = env.state().borrow();
        let found = state.widgets.iter_ids().any(|id| {
            state
                .widgets
                .get(id)
                .is_some_and(|frame| frame.text.as_deref() == Some(label))
        });
        found
    };
    PayloadObservation {
        name: name.into(),
        argument_count: env.eval("return ReloadPayloadCount").unwrap(),
        value_type: env.eval("return ReloadPayloadType").unwrap(),
        contains_bindings: env.eval("return ReloadPayloadBindings == true").unwrap(),
        displayed_label,
    }
}

#[test]
fn reload_real_addon_payload_retains_original_settings_consumer() {
    // Real startup installs a process-global tracing subscriber exactly once.
    if std::env::var_os(PAYLOAD_CHILD_ENV).is_none() {
        assert_payload_child_process();
        return;
    }
    let args = crate::Args::try_parse_from(["wow-sim", "--no-addons", "--no-saved-vars"]).unwrap();
    let session = tempfile::tempdir().unwrap();
    let (env, _fonts, manager) =
        crate::startup::init_and_load(&args, ScreenKind::Game, session.path().to_path_buf())
            .unwrap();
    assert!(manager.is_none());
    wow_ui_sim::startup::fire_startup_events(&env);
    env.exec("Settings.OpenToCategory(Settings.KEYBINDINGS_CATEGORY_ID)")
        .unwrap();
    let mut actual = Vec::new();
    for (name, label) in [
        ("Blizzard_PingUI", "Reload startup binding section"),
        ("Blizzard_Colors", "Reload startup no-binding section"),
    ] {
        observe(&env, name, label);
        fire_addon_loaded(&env, name);
        actual.push(read_payload_and_original_label(&env, name, label));
    }

    let fixture = tempfile::tempdir().unwrap();
    for (name, with_bindings) in [("ReloadBindingsLoD", true), ("ReloadNoBindingsLoD", false)] {
        let dir = fixture.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        let toc = format!(
            "## Interface: {}\n## LoadOnDemand: 1\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        );
        std::fs::write(dir.join(format!("{name}.toc")), toc).unwrap();
        if with_bindings {
            let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
            std::fs::copy(
                root.join("Blizzard_PingUI/Bindings.xml"),
                dir.join("Bindings.xml"),
            )
            .unwrap();
        }
    }
    env.state()
        .borrow_mut()
        .addon_base_paths
        .push(fixture.path().to_path_buf());
    for (name, label) in [
        ("ReloadBindingsLoD", "Reload LoD binding section"),
        ("ReloadNoBindingsLoD", "Reload LoD no-binding section"),
    ] {
        observe(&env, name, label);
        env.exec(&format!("assert(C_AddOns.LoadAddOn({name:?}))"))
            .unwrap();
        actual.push(read_payload_and_original_label(&env, name, label));
    }
    let expected: Vec<_> = [
        ("Blizzard_PingUI", true),
        ("Blizzard_Colors", false),
        ("ReloadBindingsLoD", true),
        ("ReloadNoBindingsLoD", false),
    ]
    .into_iter()
    .map(|(name, contains_bindings)| PayloadObservation {
        name: name.into(),
        argument_count: 2,
        value_type: "boolean".into(),
        contains_bindings,
        displayed_label: contains_bindings,
    })
    .collect();
    assert_eq!(
        actual, expected,
        "both real-addon emitters and the original Settings consumer"
    );
}
