//! Tests for `find_toc_file`: the TOC must name the addon folder, matching
//! WoW's rule that renaming a folder (e.g. `MyAddon.disabled`) disables it.

use crate::loader::find_toc_file;

fn make_addon_dir(root: &std::path::Path, folder: &str, toc_names: &[&str]) -> std::path::PathBuf {
    let dir = root.join(folder);
    std::fs::create_dir_all(&dir).unwrap();
    for toc in toc_names {
        std::fs::write(dir.join(toc), "## Interface: 120005\n").unwrap();
    }
    dir
}

#[test]
fn find_toc_file_ignores_renamed_disabled_folder() {
    // CoreBehaviorProbe.disabled/CoreBehaviorProbe.toc loaded in the sim and
    // left full-screen mouse-blocker frames up; real WoW refuses to load a
    // folder whose TOC stem doesn't match the folder name.
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(
        tmp.path(),
        "CoreBehaviorProbe.disabled",
        &["CoreBehaviorProbe.toc"],
    );
    assert_eq!(find_toc_file(&dir), None);
}

#[test]
fn find_toc_file_accepts_exact_match() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["MyAddon.toc"]);
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon.toc")));
}

#[test]
fn find_toc_file_scan_accepts_case_mismatch_and_flavor_suffix() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["myaddon_standard.toc"]);
    assert_eq!(find_toc_file(&dir), Some(dir.join("myaddon_standard.toc")));
}

#[test]
fn find_toc_file_prefers_bare_toc_for_settings_definitions_frame() {
    // Blizzard_SettingsDefinitions_Frame.toc lists CombatAudioAlertConstants.lua
    // and drops the AudioAssist/Subtitles files that moved to
    // Blizzard_SettingsDefinitions_Shared; its _Mainline.toc predates that
    // split. Only retail-family profiles carry the split TOCs.
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(
        tmp.path(),
        "Blizzard_SettingsDefinitions_Frame",
        &[
            "Blizzard_SettingsDefinitions_Frame.toc",
            "Blizzard_SettingsDefinitions_Frame_Mainline.toc",
        ],
    );
    assert_eq!(
        find_toc_file(&dir),
        Some(dir.join("Blizzard_SettingsDefinitions_Frame.toc"))
    );

    // The preference is per addon, not a rule: any other addon keeps the
    // flavor variant.
    let other = make_addon_dir(
        tmp.path(),
        "Blizzard_UIParent",
        &["Blizzard_UIParent.toc", "Blizzard_UIParent_Mainline.toc"],
    );
    assert_eq!(
        find_toc_file(&other),
        Some(other.join("Blizzard_UIParent_Mainline.toc"))
    );
}

#[test]
fn find_toc_file_scan_rejects_unrelated_toc_names() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["SomethingElse.toc"]);
    assert_eq!(find_toc_file(&dir), None);
}
