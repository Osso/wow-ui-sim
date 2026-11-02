use std::fs;
use std::path::PathBuf;

use rilua::{Lua, LuaApiMut};
use tempfile::{TempDir, tempdir};

use super::{SavedVariablesManager, WtfConfig};

struct Fixture {
    root: TempDir,
    manager: SavedVariablesManager,
}

impl Fixture {
    fn new() -> Self {
        let root = tempdir().unwrap();
        let mut manager = SavedVariablesManager::with_storage_dir(root.path().join("local"));
        manager.set_wtf_config(WtfConfig::new(
            root.path().join("WTF"),
            "Account",
            "Realm",
            "Hero",
        ));
        Self { root, manager }
    }

    fn account_path(&self) -> PathBuf {
        self.manager
            .wtf_config()
            .unwrap()
            .account_saved_vars_file("Addon")
    }

    fn character_path(&self) -> PathBuf {
        self.manager
            .wtf_config()
            .unwrap()
            .character_saved_vars_file("Addon")
    }

    fn write(path: &std::path::Path, source: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn init(&mut self, lua: &mut Lua, character: bool) -> crate::Result<()> {
        let character_vars = if character {
            vec!["CharacterDB".to_owned()]
        } else {
            Vec::new()
        };
        self.manager.init_for_addon(
            lua.state_mut(),
            "Addon",
            &["DB".to_owned()],
            &character_vars,
        )
    }
}

#[test]
fn failed_wtf_input_remains_failed_across_preload_and_registration() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { invalid");
    Fixture::write(&fixture.character_path(), b"CharacterDB = 42\n");
    Fixture::write(
        &fixture.manager.storage_dir.join("Addon.lua"),
        b"DB = 'local fallback'\n",
    );
    let mut lua = Lua::new().unwrap();
    let first_error = fixture
        .manager
        .load_wtf_for_addon(lua.state_mut(), "Addon")
        .unwrap_err()
        .to_string();
    assert!(!first_error.is_empty());
    lua.exec("assert(DB == nil and CharacterDB == 42)").unwrap();

    Fixture::write(&fixture.account_path(), b"DB = 'repaired input'\n");
    assert_eq!(
        fixture
            .manager
            .load_wtf_for_addon(lua.state_mut(), "Addon")
            .unwrap_err()
            .to_string(),
        first_error
    );
    assert_eq!(
        fixture.init(&mut lua, true).unwrap_err().to_string(),
        first_error
    );
    lua.exec("assert(DB == nil and CharacterDB == 42)").unwrap();
}

#[test]
fn unregistered_and_undeclared_failed_imports_do_not_block_declared_saves() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = 17\n");
    Fixture::write(&fixture.character_path(), b"CharacterDB = { invalid");
    let optional = fixture
        .manager
        .wtf_config()
        .unwrap()
        .account_saved_vars_file("ServerSnapshot");
    Fixture::write(&optional, b"ServerSnapshotDB = { invalid");
    let mut lua = Lua::new().unwrap();
    assert!(
        fixture
            .manager
            .load_wtf_for_addon(lua.state_mut(), "ServerSnapshot")
            .is_err()
    );
    fixture
        .manager
        .init_for_addon(lua.state_mut(), "ServerSnapshot", &[], &[])
        .unwrap();
    assert!(
        fixture
            .manager
            .load_wtf_for_addon(lua.state_mut(), "Addon")
            .is_err()
    );
    fixture.init(&mut lua, false).unwrap();
    lua.exec("DB = 23").unwrap();
    fixture
        .manager
        .retry_failed_inputs(lua.state_mut())
        .unwrap();
    fixture.manager.save_all(lua.state_mut()).unwrap();
    fixture
        .manager
        .save_addon(lua.state_mut(), "Addon")
        .unwrap();
    assert_eq!(
        fs::read(&optional).unwrap(),
        b"ServerSnapshotDB = { invalid"
    );
    assert_eq!(
        fs::read(fixture.character_path()).unwrap(),
        b"CharacterDB = { invalid"
    );
    assert!(
        !fixture
            .manager
            .storage_dir
            .join("ServerSnapshot.lua")
            .exists()
    );
    let output = fs::read_to_string(fixture.manager.storage_dir.join("Addon.lua")).unwrap();
    lua.exec(&output).unwrap();
    lua.exec("assert(DB == 23)").unwrap();
}

#[test]
fn both_save_apis_protect_failed_declared_inputs_while_healthy_addons_can_save() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { invalid");
    let mut lua = Lua::new().unwrap();
    assert!(fixture.init(&mut lua, false).is_err());
    fixture
        .manager
        .init_for_addon(lua.state_mut(), "Healthy", &["HealthyDB".into()], &[])
        .unwrap();
    lua.exec("DB = 'unsafe default'; HealthyDB = 'healthy edit'")
        .unwrap();
    assert!(
        fixture
            .manager
            .save_addon(lua.state_mut(), "Addon")
            .is_err()
    );
    assert!(fixture.manager.save_all(lua.state_mut()).is_err());
    assert!(!fixture.manager.storage_dir.join("Addon.lua").exists());
    assert!(!fixture.manager.storage_dir.join("Healthy.lua").exists());
    fixture
        .manager
        .retry_failed_inputs_excluding(
            lua.state_mut(),
            &std::collections::HashSet::from(["Addon".to_string()]),
        )
        .unwrap();
    fixture
        .manager
        .save_addon(lua.state_mut(), "Healthy")
        .unwrap();
    let output = fs::read_to_string(fixture.manager.storage_dir.join("Healthy.lua")).unwrap();
    assert!(output.contains("healthy edit"));
    assert_eq!(fs::read(fixture.account_path()).unwrap(), b"DB = { invalid");
}

#[test]
fn explicit_failed_input_retry_reads_repaired_source_and_keeps_healthy_edits() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { invalid");
    Fixture::write(&fixture.character_path(), b"CharacterDB = 42\n");
    let mut lua = Lua::new().unwrap();
    assert!(fixture.init(&mut lua, true).is_err());
    lua.exec("DB = 'unsafe default'; CharacterDB = 99").unwrap();
    Fixture::write(&fixture.account_path(), b"DB = 'repaired input'\n");
    assert!(fixture.init(&mut lua, true).is_err());
    fixture
        .manager
        .retry_failed_inputs(lua.state_mut())
        .unwrap();
    lua.exec("assert(DB == 'repaired input' and CharacterDB == 99)")
        .unwrap();
    fixture.manager.save_all(lua.state_mut()).unwrap();
    let mut restored = Lua::new().unwrap();
    restored
        .exec(&fs::read_to_string(fixture.manager.storage_dir.join("Addon.lua")).unwrap())
        .unwrap();
    restored
        .exec(
            &fs::read_to_string(fixture.manager.storage_dir.join("Realm/Hero/Addon.lua")).unwrap(),
        )
        .unwrap();
    restored
        .exec("assert(DB == 'repaired input' and CharacterDB == 99)")
        .unwrap();
}

#[test]
fn explicit_failed_input_retry_rejects_disappearance_and_keeps_the_import_cap() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { invalid");
    Fixture::write(&fixture.manager.storage_dir.join("Addon.lua"), b"DB = 37\n");
    fixture.manager.set_max_load_bytes(Some(8));
    let mut lua = Lua::new().unwrap();
    assert!(fixture.init(&mut lua, false).is_err());
    lua.exec("DB = 'unsafe default'").unwrap();
    fs::remove_file(fixture.account_path()).unwrap();
    let missing = fixture
        .manager
        .retry_failed_inputs(lua.state_mut())
        .unwrap_err()
        .to_string();
    assert!(missing.contains("input is missing"), "{missing}");
    lua.exec("assert(DB == 'unsafe default')").unwrap();
    Fixture::write(&fixture.account_path(), b"DB = 'still too large'\n");
    let oversized = fixture
        .manager
        .retry_failed_inputs(lua.state_mut())
        .unwrap_err()
        .to_string();
    assert!(oversized.contains("byte limit"), "{oversized}");
    Fixture::write(&fixture.account_path(), b"DB = 7\n");
    fixture
        .manager
        .retry_failed_inputs(lua.state_mut())
        .unwrap();
    lua.exec("assert(DB == 7)").unwrap();
    assert_eq!(
        fs::read(fixture.manager.storage_dir.join("Addon.lua")).unwrap(),
        b"DB = 37\n"
    );
}

#[test]
fn recovery_retirement_does_not_claim_unsaved_defaults_as_reload_storage() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = 'original input'\n");
    let mut first = Lua::new().unwrap();
    fixture.init(&mut first, false).unwrap();
    first.exec("DB = 'unsafe default'").unwrap();
    fixture
        .manager
        .retire_lua_environment_excluding(&std::collections::HashSet::from(["Addon".to_string()]));
    let mut second = Lua::new().unwrap();
    fixture.init(&mut second, false).unwrap();
    second.exec("assert(DB == 'original input')").unwrap();
    second.exec("DB = 'preceding valid save'").unwrap();
    fixture.manager.save_all(second.state_mut()).unwrap();
    fixture.manager.retire_lua_environment();
    let mut third = Lua::new().unwrap();
    fixture.init(&mut third, false).unwrap();
    third.exec("DB = 'unsafe default'").unwrap();
    fixture.manager.discard_lua_environment();
    let mut fourth = Lua::new().unwrap();
    fixture.init(&mut fourth, false).unwrap();
    fourth.exec("assert(DB == 'preceding valid save')").unwrap();
}

#[test]
fn discarded_partial_vm_retries_exact_failed_source_only_in_the_fresh_vm() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { invalid");
    Fixture::write(&fixture.manager.storage_dir.join("Addon.lua"), b"DB = 37\n");
    let mut first = Lua::new().unwrap();
    assert!(fixture.init(&mut first, false).is_err());
    fixture.manager.discard_lua_environment();
    drop(first);
    fs::remove_file(fixture.account_path()).unwrap();
    let mut second = Lua::new().unwrap();
    let missing = fixture.init(&mut second, false).unwrap_err().to_string();
    assert!(missing.contains("input is missing"), "{missing}");
    second.exec("assert(DB == nil)").unwrap();
    Fixture::write(&fixture.account_path(), b"DB = 'repaired original'\n");
    assert_eq!(
        fixture.init(&mut second, false).unwrap_err().to_string(),
        missing
    );
    fixture.manager.discard_lua_environment();
    drop(second);
    let mut third = Lua::new().unwrap();
    fixture.init(&mut third, false).unwrap();
    third.exec("assert(DB == 'repaired original')").unwrap();
    assert_eq!(
        fs::read(fixture.manager.storage_dir.join("Addon.lua")).unwrap(),
        b"DB = 37\n"
    );
}

#[test]
fn oversized_wtf_input_cannot_fall_back_or_succeed_on_retry() {
    let mut fixture = Fixture::new();
    let source = b"DB = { value = 'source must not load' }\n";
    Fixture::write(&fixture.account_path(), source);
    Fixture::write(&fixture.manager.storage_dir.join("Addon.lua"), b"DB = 37\n");
    fixture.manager.set_max_load_bytes(Some(8));
    let mut lua = Lua::new().unwrap();
    let first_error = fixture.init(&mut lua, false).unwrap_err().to_string();
    assert!(first_error.contains("byte limit"));
    fixture.manager.set_max_load_bytes(None);
    assert_eq!(
        fixture.init(&mut lua, false).unwrap_err().to_string(),
        first_error
    );
    lua.exec("assert(DB == nil)").unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), source);
}

#[test]
fn non_directory_input_ancestor_is_not_treated_as_absent() {
    let mut fixture = Fixture::new();
    fs::write(fixture.root.path().join("WTF"), b"not a directory").unwrap();
    Fixture::write(
        &fixture.manager.storage_dir.join("Addon.lua"),
        b"DB = 'local fallback'\n",
    );
    let mut lua = Lua::new().unwrap();
    let first_error = fixture.init(&mut lua, false).unwrap_err().to_string();
    assert!(!first_error.is_empty());
    assert_eq!(
        fixture.init(&mut lua, false).unwrap_err().to_string(),
        first_error
    );
    lua.exec("assert(DB == nil)").unwrap();
}

#[test]
fn preloaded_wtf_executes_once_before_registration() {
    let mut fixture = Fixture::new();
    let source = b"InputRuns = (InputRuns or 0) + 1\nDB = { value = InputRuns }\n";
    Fixture::write(&fixture.account_path(), source);
    let mut lua = Lua::new().unwrap();
    assert_eq!(
        fixture
            .manager
            .load_wtf_for_addon(lua.state_mut(), "Addon")
            .unwrap(),
        1
    );
    lua.exec("DB.value = 37").unwrap();
    fixture.init(&mut lua, false).unwrap();
    assert_eq!(
        fixture
            .manager
            .load_wtf_for_addon(lua.state_mut(), "Addon")
            .unwrap(),
        0
    );
    lua.exec("assert(InputRuns == 1 and DB.value == 37)")
        .unwrap();
    fixture.manager.save_all(lua.state_mut()).unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), source);
}

#[test]
fn missing_wtf_scopes_fall_back_to_local_persistence() {
    let mut fixture = Fixture::new();
    Fixture::write(
        &fixture.manager.storage_dir.join("Addon.lua"),
        b"DB = { value = 37 }\n",
    );
    Fixture::write(
        &fixture.manager.storage_dir.join("Realm/Hero/Addon.lua"),
        b"CharacterDB = 11\n",
    );
    let mut lua = Lua::new().unwrap();
    fixture.init(&mut lua, true).unwrap();
    lua.exec("assert(DB.value == 37 and CharacterDB == 11)")
        .unwrap();
    assert!(!fixture.account_path().exists());
    assert!(!fixture.character_path().exists());
}

#[test]
fn explicit_session_restores_saved_scopes_before_import_on_restart() {
    let mut fixture = Fixture::new();
    let account = b"DB = { value = 1 }\n";
    let character = b"CharacterDB = 2\n";
    Fixture::write(&fixture.account_path(), account);
    Fixture::write(&fixture.character_path(), character);
    let source = fixture.manager.wtf_config().unwrap().clone();
    let session = fixture.root.path().join("session");
    fixture.manager = SavedVariablesManager::with_session_wtf_path(&session);
    fixture.manager.set_wtf_config(source.clone());
    let mut first = Lua::new().unwrap();
    fixture.init(&mut first, true).unwrap();
    first.exec("DB.value = 37; CharacterDB = false").unwrap();
    fixture.manager.save_all(first.state_mut()).unwrap();
    drop(first);

    fixture.manager = SavedVariablesManager::with_session_wtf_path(&session);
    fixture.manager.set_wtf_config(source);
    let mut restarted = Lua::new().unwrap();
    assert_eq!(
        fixture
            .manager
            .load_wtf_for_addon(restarted.state_mut(), "Addon")
            .unwrap(),
        2
    );
    fixture.init(&mut restarted, true).unwrap();
    restarted
        .exec("assert(DB.value == 37 and CharacterDB == false)")
        .unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), account);
    assert_eq!(fs::read(fixture.character_path()).unwrap(), character);
}

#[test]
fn explicit_session_imports_only_missing_account_or_character_scopes() {
    let mut fixture = Fixture::new();
    let source = fixture.manager.wtf_config().unwrap().clone();
    fixture.manager =
        SavedVariablesManager::with_session_wtf_path(&fixture.root.path().join("session"));
    fixture.manager.set_wtf_config(source);
    Fixture::write(
        &fixture.account_path(),
        b"DB = 1; UnselectedImport = true\n",
    );
    Fixture::write(&fixture.character_path(), b"CharacterDB = 2\n");
    Fixture::write(&fixture.manager.storage_dir.join("Addon.lua"), b"DB = 37\n");
    Fixture::write(
        &fixture
            .manager
            .storage_dir
            .join("OtherRealm/Hero/Addon.lua"),
        b"CharacterDB = 'wrong realm'\n",
    );
    let mut lua = Lua::new().unwrap();
    fixture
        .manager
        .load_wtf_for_addon(lua.state_mut(), "Addon")
        .unwrap();
    fixture.init(&mut lua, true).unwrap();
    lua.exec("assert(DB == 37 and CharacterDB == 2 and UnselectedImport == nil)")
        .unwrap();
}

#[test]
fn corrupt_explicit_session_scope_does_not_fall_back_to_import() {
    let mut fixture = Fixture::new();
    let source = fixture.manager.wtf_config().unwrap().clone();
    fixture.manager =
        SavedVariablesManager::with_session_wtf_path(&fixture.root.path().join("session"));
    fixture.manager.set_wtf_config(source);
    Fixture::write(&fixture.account_path(), b"DB = 'import fallback'\n");
    Fixture::write(
        &fixture.manager.storage_dir.join("Addon.lua"),
        b"DB = { invalid\n",
    );
    let mut lua = Lua::new().unwrap();
    let error = fixture
        .manager
        .load_wtf_for_addon(lua.state_mut(), "Addon")
        .unwrap_err()
        .to_string();
    assert!(!error.is_empty());
    assert_eq!(
        fixture.init(&mut lua, false).unwrap_err().to_string(),
        error
    );
    lua.exec("assert(DB == nil)").unwrap();
}

#[test]
fn reload_uses_local_saved_state_without_the_external_size_limit() {
    let mut fixture = Fixture::new();
    let source = b"DB = { value = 1 }\n";
    Fixture::write(&fixture.account_path(), source);
    let mut first = Lua::new().unwrap();
    fixture.init(&mut first, false).unwrap();
    first
        .exec("DB.value = 2; DB.payload = string.rep('x', 128)")
        .unwrap();
    fixture.manager.save_all(first.state_mut()).unwrap();
    fixture.manager.set_max_load_bytes(Some(8));
    fixture.manager.retire_lua_environment();
    drop(first);

    let mut reloaded = Lua::new().unwrap();
    assert_eq!(
        fixture
            .manager
            .load_wtf_for_addon(reloaded.state_mut(), "Addon")
            .unwrap(),
        0
    );
    fixture.init(&mut reloaded, false).unwrap();
    reloaded
        .exec("assert(DB.value == 2 and DB.payload == string.rep('x', 128))")
        .unwrap();
    assert_eq!(fs::read(fixture.account_path()).unwrap(), source);
}

#[test]
fn missing_reload_output_is_an_error_instead_of_reimporting_wtf() {
    let mut fixture = Fixture::new();
    Fixture::write(&fixture.account_path(), b"DB = { value = 1 }\n");
    let mut first = Lua::new().unwrap();
    fixture.init(&mut first, false).unwrap();
    fixture.manager.save_all(first.state_mut()).unwrap();
    fs::remove_file(fixture.manager.storage_dir.join("Addon.lua")).unwrap();
    fixture.manager.retire_lua_environment();
    drop(first);

    let mut reloaded = Lua::new().unwrap();
    let error = fixture.init(&mut reloaded, false).unwrap_err();
    assert!(!error.to_string().is_empty());
    reloaded.exec("assert(DB == nil)").unwrap();
    fixture.manager.set_max_load_bytes(Some(8));
    Fixture::write(
        &fixture.manager.storage_dir.join("Addon.lua"),
        b"DB = { value = 'repaired simulator output exceeds the external import cap' }\n",
    );
    fixture
        .manager
        .retry_failed_inputs(reloaded.state_mut())
        .unwrap();
    reloaded
        .exec("assert(DB.value == 'repaired simulator output exceeds the external import cap')")
        .unwrap();
    assert_eq!(
        fs::read(fixture.account_path()).unwrap(),
        b"DB = { value = 1 }\n"
    );
}

#[test]
fn ordinary_loading_preserves_binary_strings_through_vm_fallback() {
    let mut fixture = Fixture::new();
    Fixture::write(
        &fixture.account_path(),
        b"DB = { [\"\xff\"] = \"\xfe\x80\" }\n",
    );
    let mut lua = Lua::new().unwrap();
    fixture.init(&mut lua, false).unwrap();
    lua.exec("assert(DB[string.char(255)] == string.char(254,128))")
        .unwrap();
}

#[cfg(unix)]
#[test]
fn file_symlinks_load_regular_targets_and_retain_invalid_target_errors() {
    use std::os::unix::fs::symlink;

    let mut fixture = Fixture::new();
    let target = fixture.root.path().join("linked-input.lua");
    Fixture::write(&target, b"DB = { value = 'linked file' }\n");
    let input = fixture.account_path();
    fs::create_dir_all(input.parent().unwrap()).unwrap();
    symlink(&target, &input).unwrap();
    let mut lua = Lua::new().unwrap();
    fixture.init(&mut lua, false).unwrap();
    lua.exec("assert(DB.value == 'linked file')").unwrap();

    for directory_target in [false, true] {
        let mut fixture = Fixture::new();
        let target = fixture.root.path().join("invalid-target");
        if directory_target {
            fs::create_dir(&target).unwrap();
        }
        let input = fixture.account_path();
        fs::create_dir_all(input.parent().unwrap()).unwrap();
        symlink(&target, &input).unwrap();
        Fixture::write(
            &fixture.manager.storage_dir.join("Addon.lua"),
            b"DB = 'local fallback'\n",
        );
        let mut lua = Lua::new().unwrap();
        let error = fixture.init(&mut lua, false).unwrap_err().to_string();
        let expected = if directory_target {
            "not a regular file"
        } else {
            "Resolve SavedVariables link"
        };
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            fixture.init(&mut lua, false).unwrap_err().to_string(),
            error
        );
        lua.exec("assert(DB == nil)").unwrap();
        assert!(
            fs::symlink_metadata(&input)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}
