//! Addon-local file queries must use the TOC directory selected by the real loader.

use std::fs;
use std::path::{Path, PathBuf};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn write_addon(root: &Path, name: &str, source: &str) -> PathBuf {
    let addon = root.join("Interface/AddOns").join(name);
    fs::create_dir_all(&addon).unwrap();
    fs::write(addon.join("Query.lua"), source).unwrap();
    let toc = addon.join(format!("{name}.toc"));
    fs::write(&toc, "## Title: Asset probe\nQuery.lua\n").unwrap();
    toc
}

fn write_asset(root: &Path, name: &str, path: &str) {
    let asset = root.join("Interface/AddOns").join(name).join(path);
    fs::create_dir_all(asset.parent().unwrap()).unwrap();
    fs::write(asset, b"fixture contents").unwrap();
}

#[test]
fn loose_sound_is_known_during_load_and_afterward_without_synthetic_id() {
    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(
        root.path(),
        "SoundProbe",
        r#"
        local path = "Interface\\AddOns\\sOuNdPrObE\\Media\\Sounds\\LONG.ogg"
        AssetAtLoad = {
            C_UIFileAsset.IsKnownFile(path),
            C_UIFileAsset.IsLooseFile(path),
            C_UIFileAsset.GetFileID(path),
        }
        "#,
    );
    write_asset(root.path(), "SoundProbe", "Media/Sounds/Long.ogg");
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(AssetAtLoad[1] == true and AssetAtLoad[2] == true and AssetAtLoad[3] == nil)
        local path = "interface/addons/SOUNDPROBE/media/sounds/long.ogg"
        assert(C_UIFileAsset.IsKnownFile(path) == true)
        assert(C_UIFileAsset.IsLooseFile(path) == true)
        assert(C_UIFileAsset.GetFileID(path) == nil)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SoundProbe/Media/Sounds/Long") == false)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SoundProbe/Media/Sounds/Long") == false)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SoundProbe/Media/Sounds/absent.ogg") == false)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SoundProbe/Media/Sounds/absent.ogg") == false)
        assert(C_UIFileAsset.GetFileID("Interface/AddOns/SoundProbe/Media/Sounds/absent.ogg") == nil)
        assert(C_UIFileAsset.GetFileID("Interface\\Icons\\Trade_Engineering.blp") == 136243)
        assert(C_UIFileAsset.IsKnownFile(123) == true)
        assert(C_UIFileAsset.IsLooseFile(123) == false)
        assert(C_UIFileAsset.GetFileID(123) == 123)
        "#,
    )
    .unwrap();
}

#[test]
fn only_selected_addon_root_supplies_loose_files_and_texture_extensions() {
    let selected = tempfile::tempdir().unwrap();
    let alternate = tempfile::tempdir().unwrap();
    let toc = write_addon(selected.path(), "SharedName", "AssetQueryLoaded = true");
    write_asset(selected.path(), "SharedName", "Media/Icon.blp");
    write_addon(alternate.path(), "SharedName", "AssetQueryLoaded = false");
    write_asset(alternate.path(), "SharedName", "Media/Missing.ogg");
    let env = WowLuaEnv::new().unwrap();
    env.scan_and_register_addons(&alternate.path().join("Interface/AddOns"));
    env.exec("assert(C_UIFileAsset.IsKnownFile('Interface/AddOns/SharedName/Media/Missing.ogg'))")
        .unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(AssetQueryLoaded == true)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SharedName/Media/Icon") == true)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SharedName/Media/Icon") == true)
        assert(C_UIFileAsset.GetFileID("Interface/AddOns/SharedName/Media/Icon") == nil)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SharedName/Media/Missing.ogg") == false)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SharedName/Media/Missing.ogg") == false)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SharedName/Media/Missing") == false)
        "#,
    )
    .unwrap();
}

#[cfg(unix)]
#[test]
fn loose_file_queries_reject_traversal_absolute_paths_and_symlink_escapes() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(root.path(), "SafeProbe", "AssetQueryLoaded = true");
    write_asset(root.path(), "SafeProbe", "Media/valid.ogg");
    write_asset(root.path(), "Outside", "secret.ogg");
    let addon = toc.parent().unwrap();
    symlink(
        root.path().join("Interface/AddOns/Outside"),
        addon.join("Escape"),
    )
    .unwrap();
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SafeProbe/Media/valid.ogg") == true)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SafeProbe/Media/valid.ogg") == true)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SafeProbe/../Outside/secret.ogg") == false)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SafeProbe/../Outside/secret.ogg") == false)
        assert(C_UIFileAsset.IsKnownFile("Interface/AddOns/SafeProbe/Escape/secret.ogg") == false)
        assert(C_UIFileAsset.IsLooseFile("Interface/AddOns/SafeProbe/Escape/secret.ogg") == false)
        assert(C_UIFileAsset.IsKnownFile("/Interface/AddOns/SafeProbe/secret.ogg") == false)
        "#,
    )
    .unwrap();
}
