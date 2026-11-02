use std::process::Command;

#[test]
fn no_saved_vars_still_loads_edit_mode_cache() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_wow-sim"))
        .arg("--session-wtf-path")
        .arg(temp.path().join("data/wow-sim"))
        .arg("--no-saved-vars")
        .arg("--no-addons")
        .arg("--exec-lua")
        .arg(
            r#"local system = C_EditMode.GetLayouts().layouts[1].systems[1]
error("CACHE_HIDDEN=" .. tostring(system and system.hidden))"#,
        )
        .arg("dump-tree")
        .arg("--filter")
        .arg("NoSuchFrame")
        .env("WOW_SIM_WTF_PATH", &wtf_path)
        .env("WOW_SIM_WTF_ACCOUNT", "TestAccount")
        .env("WOW_SIM_WTF_REALM", "Test Realm")
        .env("WOW_SIM_WTF_CHARACTER", "Testchar")
        .output()
        .expect("run wow-sim");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("CACHE_HIDDEN=true"),
        "--no-saved-vars should still import the non-Lua EditMode cache: {stderr}"
    );
}
