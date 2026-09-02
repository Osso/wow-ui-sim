use super::*;

#[test]
fn edit_mode_layout_api_persists_active_layout_and_saved_layouts() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (initial_active, initial_count): (i32, i32) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            return info.activeLayout, #info.layouts
            "#,
        )
        .expect("read initial edit mode layouts");

    assert_eq!(initial_active, 1);
    assert_eq!(initial_count, 0);

    env.exec(
        r#"
        C_EditMode.SaveLayouts({
            activeLayout = 3,
            layouts = {
                {
                    layoutName = "Custom",
                    layoutType = Enum.EditModeLayoutType.Account,
                    systems = {
                        {
                            system = Enum.EditModeSystem.ActionBar,
                            systemIndex = Enum.EditModeActionBarSystemIndices.MainBar,
                            isInDefaultPosition = false,
                            anchorInfo = {
                                point = "CENTER",
                                relativeTo = "UIParent",
                                relativePoint = "CENTER",
                                offsetX = 42,
                                offsetY = -7,
                            },
                            settings = {
                                {
                                    setting = Enum.EditModeActionBarSetting.IconSize,
                                    value = 4,
                                },
                            },
                        },
                    },
                },
            },
        })
        C_EditMode.SetActiveLayout(4)
        "#,
    )
    .expect("save edit mode layout state");

    let (active, count, name, offset_x, icon_size): (i32, i32, String, i32, i32) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            local system = info.layouts[1].systems[1]
            return info.activeLayout,
                #info.layouts,
                info.layouts[1].layoutName,
                system.anchorInfo.offsetX,
                system.settings[1].value
            "#,
        )
        .expect("read saved edit mode layout state");

    assert_eq!(active, 4);
    assert_eq!(count, 1);
    assert_eq!(name, "Custom");
    assert_eq!(offset_x, 42);
    assert_eq!(icon_size, 4);
}

#[test]
fn edit_mode_layout_api_loads_wtf_cache_files() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "1 2 1 100 ",
            "7 Custom 1 ",
            "0 0 0 4 4 UIParent 12.5 -34.0 -1 ##$$",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "3 3 3 3 3 3\0",
    )
    .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, None))
        .expect("load edit mode cache");

    let (
        active,
        layout_count,
        grid_spacing,
        damage_meter_default,
        external_defensives_default,
        totem_action_bar_default,
        name,
        system,
        system_index,
        point,
        offset_x,
        setting_value,
    ): (i32, i32, i32, i32, i32, i32, String, i32, i32, String, f64, i32) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            local settings = C_EditMode.GetAccountSettings()
            local accountSettingMap = {}
            for _, settingInfo in ipairs(settings) do
                accountSettingMap[settingInfo.setting] = settingInfo.value
            end
            local system = info.layouts[1].systems[1]
            return info.activeLayout,
                #info.layouts,
                accountSettingMap[Enum.EditModeAccountSetting.GridSpacing],
                accountSettingMap[Enum.EditModeAccountSetting.ShowDamageMeter],
                accountSettingMap[Enum.EditModeAccountSetting.ShowExternalDefensives],
                accountSettingMap[Enum.EditModeAccountSetting.ShowTotemActionBar],
                info.layouts[1].layoutName,
                system.system,
                system.systemIndex,
                system.anchorInfo.point,
                system.anchorInfo.offsetX,
                system.settings[2].value
            "#,
        )
        .expect("read imported edit mode cache");

    assert_eq!(active, 3);
    assert_eq!(layout_count, 1);
    assert_eq!(grid_spacing, 100);
    assert_eq!(
        damage_meter_default, 1,
        "missing newer account settings should be filled from defaults"
    );
    assert_eq!(
        external_defensives_default, 1,
        "missing newer account settings should be filled from defaults"
    );
    assert_eq!(
        totem_action_bar_default, 1,
        "missing latest account settings should be filled from defaults"
    );
    assert_eq!(name, "Custom");
    assert_eq!(system, 0);
    assert_eq!(
        system_index, 1,
        "WTF cache stores system indices zero-based, but Lua layout state must use EditMode enum values"
    );
    assert_eq!(point, "CENTER");
    assert_eq!(offset_x, 12.5);
    assert_eq!(setting_value, 1);
}

#[test]
fn edit_mode_account_cache_without_layout_rows_yields_no_layouts() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    // A 12.1 client writes the layout count and the account settings only:
    // `<layoutCount> <settingCount> <settingCount values>` with no layout rows.
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "2 36 ",
            "0 100 1 1 0 0 0 0 1 0 0 0 1 1 0 0 1 1 0 0 0 0 1 1 1 0 0 0 0 0 0 1 1 0 0 0",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "2 0 0 1 1 1\0",
    )
    .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 1, None))
        .expect("load edit mode cache");

    let (layout_count, active, setting_count): (i32, i32, i32) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            return #info.layouts, info.activeLayout, #C_EditMode.GetAccountSettings()
            "#,
        )
        .expect("read imported edit mode cache");

    assert_eq!(
        layout_count, 0,
        "a layout count without layout rows must not fabricate empty layouts"
    );
    assert_eq!(active, 2);
    assert_eq!(setting_count, 36);
}

#[test]
fn edit_mode_wtf_cache_can_override_active_layout_by_name() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "2 0 ",
            "9 Ultrawide 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "10 Widescreen 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ##",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "2 2 2 2 0 0\0",
    )
    .expect("write character edit mode cache");

    let _guard = EnvVarGuard::set(EDIT_MODE_LAYOUT_ENV, "Ultrawide");
    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, None))
        .expect("load edit mode cache");

    let (active, active_name): (i32, String) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            return info.activeLayout, info.layouts[info.activeLayout].layoutName
            "#,
        )
        .expect("read overridden active edit mode layout");

    assert_eq!(active, 1);
    assert_eq!(active_name, "Ultrawide");
}

#[test]
fn edit_mode_snapshot_layout_selects_active_layout_by_name() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "2 0 ",
            "9 Ultrawide 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "10 Widescreen 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ##",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    // Character cache resolves to layout index 2 (Widescreen); the snapshot
    // name must win and select Ultrawide instead.
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "2 2 2 2 0 0\0",
    )
    .expect("write character edit mode cache");

    let _guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, Some("Ultrawide")))
        .expect("load edit mode cache");

    let (active, active_name): (i32, String) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            return info.activeLayout, info.layouts[info.activeLayout].layoutName
            "#,
        )
        .expect("read snapshot-selected active edit mode layout");

    assert_eq!(active, 1);
    assert_eq!(active_name, "Ultrawide");
}

#[test]
fn edit_mode_env_override_beats_snapshot_layout() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "2 0 ",
            "9 Ultrawide 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "10 Widescreen 1 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ##",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "2 2 2 2 0 0\0",
    )
    .expect("write character edit mode cache");

    let _guard = EnvVarGuard::set(EDIT_MODE_LAYOUT_ENV, "Widescreen");
    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    // Env var (Widescreen) is the explicit manual override and must beat the
    // snapshot-captured layout (Ultrawide).
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, Some("Ultrawide")))
        .expect("load edit mode cache");

    let active_name: String = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            return info.layouts[info.activeLayout].layoutName
            "#,
        )
        .expect("read active edit mode layout");

    assert_eq!(active_name, "Widescreen");
}

#[test]
fn edit_mode_wtf_cache_decodes_frame_point_numbers_like_wow() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "1 0 ",
            "9 Ultrawide 3 ",
            "2 -1 0 2 2 UIParent 0.0 0.0 -1 ## ",
            "0 0 0 8 8 UIParent 0.0 0.0 -1 ## ",
            "1 -1 0 4 4 UIParent 0.0 0.0 -1 ##",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "1 1 1 1 1 1\0",
    )
    .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 1, None))
        .expect("load edit mode cache");

    let (minimap_point, action_bar_point, cast_bar_point): (String, String, String) = env
        .eval(
            r#"
            local systems = C_EditMode.GetLayouts().layouts[1].systems
            return systems[1].anchorInfo.point,
                systems[2].anchorInfo.point,
                systems[3].anchorInfo.point
            "#,
        )
        .expect("read decoded frame points");

    assert_eq!(minimap_point, "TOPRIGHT");
    assert_eq!(action_bar_point, "BOTTOMRIGHT");
    assert_eq!(cast_bar_point, "CENTER");
}

#[test]
fn edit_mode_account_cache_preserves_all_legacy_values_and_fills_new_defaults() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "0 29 ",
            "1 20 1 1 0 0 0 1 0 1 0 0 1 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(character_path.join("edit-mode-cache-character.txt"), "1\0")
        .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 1, None))
        .expect("load edit mode cache");

    let saved_values: String = env
        .eval(
            r#"
            local settingMap = {}
            for _, settingInfo in ipairs(C_EditMode.GetAccountSettings()) do
                settingMap[settingInfo.setting] = settingInfo.value
            end

            local values = {}
            for setting = Enum.EditModeAccountSetting.ShowGrid, Enum.EditModeAccountSetting.ShowTotemActionBar do
                table.insert(values, tostring(setting) .. "=" .. tostring(settingMap[setting]))
            end
            return table.concat(values, ",")
            "#,
        )
        .expect("read account settings");

    assert_eq!(
        saved_values,
        "0=1,1=20,2=1,3=1,4=0,5=0,6=0,7=1,8=0,9=1,10=0,11=0,12=1,13=0,14=0,15=0,16=0,17=0,18=0,19=0,20=0,21=0,22=0,23=1,24=0,25=0,26=0,27=0,28=1,29=1,30=1,31=1,32=1,33=1,34=1",
        "legacy account cache values should be preserved and newer profile options should be default-filled"
    );
}

#[test]
fn edit_mode_wtf_cache_normalizes_indexed_system_rows() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "1 0 ",
            "10 Widescreen 6 ",
            "0 0 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "3 7 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "6 1 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "15 1 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "20 3 0 4 4 UIParent 0.0 0.0 -1 ## ",
            "1 -1 0 4 4 UIParent 0.0 0.0 -1 ##",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "1 1 1 1 1 1\0",
    )
    .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, None))
        .expect("load edit mode cache");

    let (layout_name, indices): (String, String) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            local layout = info.layouts[1]
            local values = {}
            for _, systemInfo in ipairs(layout.systems) do
                table.insert(values, tostring(systemInfo.systemIndex))
            end
            return layout.layoutName, table.concat(values, ",")
            "#,
        )
        .expect("read normalized edit mode indices");

    assert_eq!(layout_name, "Widescreen");
    assert_eq!(
        indices, "1,8,2,2,4,-1",
        "indexed WTF rows are zero-based, but singleton rows must remain -1"
    );
}

#[test]
fn edit_mode_cache_decodes_repeated_setting_chunks_as_large_value() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "1 0 ",
            "1 Custom 1 ",
            "20 0 0 0 0 UIParent 0.0 0.0 -1 (-($",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(
        character_path.join("edit-mode-cache-character.txt"),
        "1 1 1 1 1 1\0",
    )
    .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 2, None))
        .expect("load edit mode cache");

    let (settings_count, setting, value): (i32, i32, i32) = env
        .eval(
            r#"
            local info = C_EditMode.GetLayouts()
            local settings = info.layouts[1].systems[1].settings
            return #settings, settings[1].setting, settings[1].value
            "#,
        )
        .expect("read decoded edit mode settings");

    assert_eq!(settings_count, 1);
    assert_eq!(setting, 5);
    assert_eq!(value, 100);
}

#[test]
fn edit_mode_wtf_cache_marks_hidden_status_tracking_bar_systems() {
    let _env_guard = EnvVarGuard::unset(EDIT_MODE_LAYOUT_ENV);
    let temp = tempfile::tempdir().expect("create temp dir");
    let wtf_path = temp.path().join("WTF");
    let account_path = wtf_path.join("Account/TestAccount");
    let character_path = account_path.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character_path).expect("create WTF dirs");
    std::fs::write(
        account_path.join("edit-mode-cache-account.txt"),
        concat!(
            "1 0 ",
            "1 Hidden 1 ",
            "15 0 0 4 4 UIParent 0.0 0.0 -1 #",
            "\0"
        ),
    )
    .expect("write account edit mode cache");
    std::fs::write(character_path.join("edit-mode-cache-character.txt"), "1\0")
        .expect("write character edit mode cache");

    let env = WowLuaEnv::new().expect("create Lua environment");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local-sv"));
    saved_vars.set_wtf_config(WtfConfig::new(
        &wtf_path,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    env.loader_env()
        .with_state(|state| saved_vars.load_edit_mode_cache(state, 1, None))
        .expect("load edit mode cache");

    let (system, system_index, hidden): (i32, i32, bool) = env
        .eval(
            r#"
            local system = C_EditMode.GetLayouts().layouts[1].systems[1]
            return system.system, system.systemIndex, system.hidden == true
            "#,
        )
        .expect("read hidden status tracking system");

    assert_eq!(system, 15);
    assert_eq!(system_index, 1);
    assert!(
        hidden,
        "StatusTrackingBar1 should preserve the WTF profile hidden marker"
    );
}
/// `WOW_SIM_EDIT_MODE_LAYOUT` is process-global, but `load_edit_mode_cache`
/// reads it, so every test that loads the EditMode cache must serialize against
/// concurrent mutators. Holding this lock for the test body keeps a mutator's
/// transient env value invisible to other tests' cache loads.
fn edit_mode_env_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let guard = Self::acquire(key);
        unsafe {
            std::env::set_var(key, value);
        }
        guard
    }

    fn unset(key: &'static str) -> Self {
        let guard = Self::acquire(key);
        unsafe {
            std::env::remove_var(key);
        }
        guard
    }

    fn acquire(key: &'static str) -> Self {
        let lock = edit_mode_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self {
            key,
            previous: std::env::var_os(key),
            _lock: lock,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
