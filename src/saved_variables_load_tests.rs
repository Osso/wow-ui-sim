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
        assert!(fs::symlink_metadata(&input).unwrap().file_type().is_symlink());
    }
}
