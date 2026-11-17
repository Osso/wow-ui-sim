use super::*;
use std::path::Path;
use std::process::Command;

const CHILD_ROOT: &str = "WOW_SIM_EXCLUSIVE_PATH_TEST_ROOT";

fn write_fixture(root: &Path, name: &str) {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{name}.toc")),
        format!(
            "## Interface: {}\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
    .unwrap();
    std::fs::write(dir.join("main.lua"), format!("{name}Loaded = true")).unwrap();
}

#[test]
fn exclusive_environment_limits_real_loader_candidates() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        assert_loader_candidates(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let selected = temp.path().join("selected");
    let ambient = temp.path().join("Interface/AddOns");
    write_fixture(&selected, "ExclusiveSelectedFixture");
    write_fixture(&ambient, "ExclusiveAmbientFixture");
    for exclusive in [false, true] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "exclusive_path_tests::exclusive_environment_limits_real_loader_candidates",
                "--nocapture",
            ])
            .current_dir(temp.path())
            .env(CHILD_ROOT, temp.path())
            .env("WOW_SIM_ADDONS_PATH", &selected)
            .env(
                "WOW_SIM_ADDONS_PATH_EXCLUSIVE",
                if exclusive { "1" } else { "0" },
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "exclusive={exclusive}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn assert_loader_candidates(root: &Path) {
    let exclusive = std::env::var("WOW_SIM_ADDONS_PATH_EXCLUSIVE").unwrap() == "1";
    let paths = wow_ui_sim::paths::default_addons_paths();
    assert!(paths.contains(&root.join("selected")));
    let mut addons = addon_loading::scan_addon_paths(
        &wow_ui_sim::toc::TocCache::default(),
        &paths,
        &[],
        ScreenKind::Game,
    );
    assert_eq!(
        addons
            .iter()
            .any(|(name, _)| name == "ExclusiveAmbientFixture"),
        !exclusive,
        "ambient addon discovery must follow the selected mode"
    );
    // Keep this loader integration fixture independent of installed addon APIs.
    addons.retain(|(name, _)| name.starts_with("Exclusive"));
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = paths;
    for (_, toc) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    }
    env.exec("assert(ExclusiveSelectedFixtureLoaded == true)")
        .unwrap();
    let loaded: bool = env
        .eval("return ExclusiveAmbientFixtureLoaded == true")
        .unwrap();
    assert_eq!(loaded, !exclusive);
    if exclusive {
        env.exec("assert(not C_AddOns.LoadAddOn('ExclusiveAmbientFixture'))")
            .unwrap();
    }
}

#[test]
fn exclusive_run_tests_finds_selected_tests_and_bundled_framework() {
    if let Some(root) = std::env::var_os("WOW_SIM_FRAMEWORK_TEST_ROOT") {
        let env = WowLuaEnv::new().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![PathBuf::from(root).join("selected")];
        wow_ui_sim::addon_tests::run_addon_tests(&env, "FrameworkFixture", None, false);
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let tests = temp.path().join("selected/FrameworkFixture/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(
        tests.join("probe.lua"),
        "test('bundled assertions', function() assertEquals(42, 6 * 7) end)",
    )
    .unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "exclusive_path_tests::exclusive_run_tests_finds_selected_tests_and_bundled_framework",
            "--nocapture",
        ])
        .current_dir(temp.path())
        .env("WOW_SIM_FRAMEWORK_TEST_ROOT", temp.path())
        .env("WOW_SIM_ADDONS_PATH_EXCLUSIVE", "1")
        .env("WOW_SIM_ADDONS_PATH", temp.path().join("selected"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("1 passed"));
}

#[test]
fn exclusive_environment_without_a_root_does_not_restore_candidates() {
    if std::env::var_os("WOW_SIM_EMPTY_EXCLUSIVE_CHILD").is_some() {
        assert!(wow_ui_sim::paths::default_addons_paths().is_empty());
        let args = Args::try_parse_from(["wow-sim", "run-tests", "Fixture"]).unwrap();
        assert!(args.read_addon_paths().is_empty());
        return;
    }
    for value in ["1", "true"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "exclusive_path_tests::exclusive_environment_without_a_root_does_not_restore_candidates", "--nocapture"])
            .env("WOW_SIM_EMPTY_EXCLUSIVE_CHILD", "1")
            .env("WOW_SIM_ADDONS_PATH_EXCLUSIVE", value)
            .env_remove("WOW_SIM_ADDONS_PATH")
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn repeated_cli_addon_paths_are_accepted() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir(&first).unwrap();
    std::fs::create_dir(&second).unwrap();
    let args = Args::try_parse_from([
        "wow-sim",
        "--addons-path",
        first.to_str().unwrap(),
        "--addons-path",
        second.to_str().unwrap(),
        "run-tests",
        "Fixture",
    ])
    .expect("repeatable exclusive roots");
    assert_eq!(args.read_addon_paths(), vec![first, second]);
}
