//! SavedVariables imports, diagnostics and ordinary persistence through addon loading.

use std::fs;
use std::path::PathBuf;

use rilua::LuaApiMut;

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::{SavedVariablesManager, WtfConfig};

use super::{StartupAddonLoadKind, load_addon_with_saved_vars, load_startup_addon};

const ACCOUNT_INPUT: &[u8] = b"FixtureDB = { input = true }\nFixtureSecond = 42\n";
const ADDON_CODE: &str = r#"
    FixtureCodeRuns = (FixtureCodeRuns or 0) + 1
    FixtureDB = FixtureDB or {}
    FixtureDB.edited = true
    FixtureSecond = false
    FixtureChar = { character = "Hero" }
"#;

struct Fixture {
    directory: tempfile::TempDir,
    env: WowLuaEnv,
    manager: SavedVariablesManager,
    toc: PathBuf,
    account_file: PathBuf,
}

impl Fixture {
    fn new(code: &str, xml: Option<&str>, account: &[u8]) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let addon = root.join("AddOns/PersistenceFixture");
        let wtf = root.join("WTF");
        fs::create_dir_all(&addon).unwrap();
        fs::create_dir_all(wtf.join("Account/ACCOUNT/SavedVariables")).unwrap();
        fs::create_dir_all(wtf.join("Account/ACCOUNT/Realm/Hero/SavedVariables")).unwrap();
        let toc = addon.join("PersistenceFixture.toc");
        fs::write(
            &toc,
            format!(
                "## Interface: {}\n## SavedVariables: FixtureDB, FixtureSecond\n## SavedVariablesPerCharacter: FixtureChar\nFixture.lua\n{}",
                crate::client_profile::ACTIVE_INTERFACE_VERSION,
                if xml.is_some() { "Fixture.xml\n" } else { "" },
            ),
        ).unwrap();
        fs::write(addon.join("Fixture.lua"), code).unwrap();
        if let Some(xml) = xml {
            fs::write(addon.join("Fixture.xml"), xml).unwrap();
        }
        let account_file = wtf.join("Account/ACCOUNT/SavedVariables/PersistenceFixture.lua");
        fs::write(&account_file, account).unwrap();
        let mut manager = SavedVariablesManager::with_storage_dir(root.join("private"));
        manager.set_wtf_config(WtfConfig::new(wtf, "ACCOUNT", "Realm", "Hero"));
        Self {
            directory,
            env: WowLuaEnv::new().unwrap(),
            manager,
            toc,
            account_file,
        }
    }
}

#[test]
fn saved_variable_loader_imports_scopes_and_saves_to_simulator_storage() {
    let mut fixture = Fixture::new(ADDON_CODE, None, ACCOUNT_INPUT);
    let result = load_addon_with_saved_vars(
        &fixture.env.loader_env(),
        &fixture.toc,
        &mut fixture.manager,
    )
    .unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert_eq!(result.lua_files, 1);
    assert!(
        fixture
            .env
            .eval::<bool>("return FixtureDB.input == true")
            .unwrap()
    );
    assert_eq!(
        fixture.env.eval::<i32>("return FixtureCodeRuns").unwrap(),
        1
    );
    assert_eq!(
        fixture
            .env
            .eval::<String>("return FixtureChar.character")
            .unwrap(),
        "Hero"
    );
    fixture
        .env
        .exec("FixtureDB.edited = true; FixtureSecond = false")
        .unwrap();
    fixture
        .manager
        .save_all(fixture.env.rilua_mut().state_mut())
        .unwrap();

    let fresh = WowLuaEnv::new().unwrap();
    for relative in [
        "PersistenceFixture.lua",
        "Realm/Hero/PersistenceFixture.lua",
    ] {
        let output =
            fs::read_to_string(fixture.directory.path().join("private").join(relative)).unwrap();
        fresh.exec(&output).unwrap();
    }
    fresh
        .exec(
            "assert(FixtureDB.input == true and FixtureDB.edited == true); \
         assert(FixtureSecond == false and FixtureChar.character == 'Hero')",
        )
        .unwrap();
    assert_eq!(fs::read(&fixture.account_file).unwrap(), ACCOUNT_INPUT);
}

#[test]
fn saved_variable_loader_keeps_lua_xml_and_xml_onload_diagnostics() {
    let cases = [
        (
            "FixtureDB = { partial = true }; error('fixture Lua failure')",
            None,
        ),
        (ADDON_CODE, Some("<Ui><Frame name=\"BrokenFixture\"></Ui>")),
        (
            ADDON_CODE,
            Some(
                r#"<Ui><Frame name="BrokenFixture"><Scripts>
            <OnLoad>error("fixture XML OnLoad failure")</OnLoad>
        </Scripts></Frame></Ui>"#,
            ),
        ),
    ];
    for (code, xml) in cases {
        let mut fixture = Fixture::new(code, xml, ACCOUNT_INPUT);
        let result = load_addon_with_saved_vars(
            &fixture.env.loader_env(),
            &fixture.toc,
            &mut fixture.manager,
        )
        .unwrap();
        assert!(
            !result.warnings.is_empty()
                || !fixture.env.state().borrow().lua_error_records.is_empty(),
            "failed addon produced no warning or Lua error record: {code}"
        );
        assert_eq!(fs::read(&fixture.account_file).unwrap(), ACCOUNT_INPUT);
    }
}

#[test]
fn saved_variable_loader_reports_unreadable_input_without_local_fallback() {
    let mut fixture = Fixture::new(ADDON_CODE, None, ACCOUNT_INPUT);
    fs::remove_file(&fixture.account_file).unwrap();
    fs::create_dir(&fixture.account_file).unwrap();
    let local = fixture
        .directory
        .path()
        .join("private/PersistenceFixture.lua");
    fs::create_dir_all(local.parent().unwrap()).unwrap();
    fs::write(&local, "FixtureDB = { localFallback = true }").unwrap();

    let result = load_addon_with_saved_vars(
        &fixture.env.loader_env(),
        &fixture.toc,
        &mut fixture.manager,
    )
    .unwrap();
    assert!(!result.warnings.is_empty());
    assert!(
        fixture
            .env
            .eval::<bool>("return FixtureDB.localFallback == nil")
            .unwrap()
    );
    assert!(fixture.account_file.is_dir());
}

#[test]
fn saved_variable_loader_reports_bootstrap_failure_without_reexecuting_it() {
    let mut fixture = Fixture::new(ADDON_CODE, None, ACCOUNT_INPUT);
    let addon = fixture.toc.parent().unwrap();
    fs::write(
        &fixture.toc,
        format!(
            "## Interface: {}\n## LoadOnDemand: 1\n## SavedVariables: FixtureDB, FixtureSecond\n## SavedVariablesPerCharacter: FixtureChar\nBootstrap.lua [Bootstrap]\nFixture.lua\n",
            crate::client_profile::ACTIVE_INTERFACE_VERSION,
        ),
    ).unwrap();
    fs::write(
        addon.join("Bootstrap.lua"),
        "BootstrapRuns = (BootstrapRuns or 0) + 1; error('fixture bootstrap failure')",
    )
    .unwrap();
    let bootstrap = load_startup_addon(
        &fixture.env.loader_env(),
        &fixture.toc,
        StartupAddonLoadKind::BootstrapOnly,
        Some(&mut fixture.manager),
    )
    .unwrap();
    assert!(!bootstrap.warnings.is_empty());

    let full = load_startup_addon(
        &fixture.env.loader_env(),
        &fixture.toc,
        StartupAddonLoadKind::Full,
        Some(&mut fixture.manager),
    )
    .unwrap();
    assert!(full.warnings.is_empty(), "{:?}", full.warnings);
    assert_eq!(full.lua_files, 1, "bootstrap must not be executed again");
    assert_eq!(fixture.env.eval::<i32>("return BootstrapRuns").unwrap(), 1);
    assert!(
        fixture
            .env
            .eval::<bool>("return FixtureDB.input == true")
            .unwrap()
    );
    assert_eq!(fs::read(&fixture.account_file).unwrap(), ACCOUNT_INPUT);
}
