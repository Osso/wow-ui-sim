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

fn active_flavor() -> &'static str {
    use crate::client_profile::{ACTIVE, ClientProfile};
    match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => "Mainline",
        ClientProfile::Wrath => "Wrath",
        ClientProfile::Mists => "Mists",
        ClientProfile::Era | ClientProfile::Anniversary => "Vanilla",
        ClientProfile::WowForever => "Camelot",
    }
}

#[test]
fn find_toc_file_accepts_exact_match() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["MyAddon.toc"]);
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon.toc")));
}

#[test]
fn find_toc_file_accepts_case_mismatched_active_flavor() {
    let tmp = tempfile::tempdir().unwrap();
    let filename = format!("myaddon_{}.toc", active_flavor().to_ascii_lowercase());
    let dir = make_addon_dir(tmp.path(), "MyAddon", &[&filename]);
    assert_eq!(find_toc_file(&dir), Some(dir.join(filename)));
}

#[test]
fn find_toc_file_scan_rejects_unrelated_toc_names() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["SomethingElse.toc"]);
    assert_eq!(find_toc_file(&dir), None);
}

#[test]
fn find_toc_file_prioritizes_dash_flavor_before_generic() {
    let tmp = tempfile::tempdir().unwrap();
    let filename = format!("MyAddon-{}.toc", active_flavor());
    let dir = make_addon_dir(tmp.path(), "MyAddon", &["MyAddon.toc", &filename]);
    assert_eq!(find_toc_file(&dir), Some(dir.join(filename)));
}

#[test]
fn find_toc_file_retains_underscore_priority_before_dash() {
    let tmp = tempfile::tempdir().unwrap();
    let underscore = format!("MyAddon_{}.toc", active_flavor());
    let dash = format!("MyAddon-{}.toc", active_flavor());
    let dir = make_addon_dir(tmp.path(), "MyAddon", &[&dash, "MyAddon.toc", &underscore]);
    assert_eq!(find_toc_file(&dir), Some(dir.join(&underscore)));
    std::fs::remove_file(dir.join(underscore)).unwrap();
    assert_eq!(find_toc_file(&dir), Some(dir.join(&dash)));
    std::fs::remove_file(dir.join(dash)).unwrap();
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon.toc")));
}

#[test]
fn find_toc_file_case_matching_preserves_flavor_and_exact_name_priority() {
    let tmp = tempfile::tempdir().unwrap();
    let lower = format!("myaddon_{}.ToC", active_flavor().to_ascii_lowercase());
    let dash = format!("MyAddon-{}.toc", active_flavor());
    let canonical = format!("MyAddon_{}.toc", active_flavor());
    let dir = make_addon_dir(tmp.path(), "MyAddon", &[&dash, &lower, "MyAddon.toc"]);
    assert_eq!(find_toc_file(&dir), Some(dir.join(&lower)));
    std::fs::write(dir.join(&canonical), "Core.lua\n").unwrap();
    assert_eq!(find_toc_file(&dir), Some(dir.join(canonical)));
}

#[test]
fn find_toc_file_case_collisions_ignore_directory_creation_order() {
    let tmp = tempfile::tempdir().unwrap();
    let upper = format!("MYADDON_{}.TOC", active_flavor().to_ascii_uppercase());
    let lower = format!("myaddon_{}.toc", active_flavor().to_ascii_lowercase());
    for (label, names) in [
        ("forward", [lower.as_str(), upper.as_str()]),
        ("reverse", [upper.as_str(), lower.as_str()]),
    ] {
        let dir = make_addon_dir(&tmp.path().join(label), "MyAddon", &names);
        assert_eq!(find_toc_file(&dir), Some(dir.join(&upper)));
    }
}

#[test]
fn find_toc_file_rejects_foreign_flavors_in_both_separators_and_cases() {
    use crate::client_profile::{ACTIVE, ClientProfile};
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(tmp.path(), "MyAddon", &[]);
    let compatible = match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => ["Mainline", "Standard"],
        ClientProfile::Wrath => ["Wrath", "Classic"],
        ClientProfile::Mists => ["Mists", "Classic"],
        ClientProfile::Era | ClientProfile::Anniversary => ["Vanilla", "Classic"],
        ClientProfile::WowForever => ["Camelot", "Mainline"],
    };
    for flavor in [
        "Mainline", "Standard", "Wrath", "Mists", "Vanilla", "Camelot", "Classic", "Cata", "TBC",
    ] {
        if compatible.contains(&flavor) {
            continue;
        }
        for separator in ["_", "-"] {
            let filename = format!("myaddon{separator}{}.toc", flavor.to_ascii_lowercase());
            std::fs::write(dir.join(filename), "Foreign.lua\n").unwrap();
        }
    }
    assert_eq!(find_toc_file(&dir), None);
}

#[test]
fn find_toc_file_does_not_guess_unknown_flavor_suffixes() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(
        tmp.path(),
        "MyAddon",
        &["MyAddon-backup.toc", "MyAddon_future.toc"],
    );
    assert_eq!(find_toc_file(&dir), None);
}

#[test]
#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
fn find_toc_file_retains_standard_alias_after_generic() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(
        tmp.path(),
        "MyAddon",
        &["myaddon_standard.toc", "MyAddon.toc"],
    );
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon.toc")));
    std::fs::remove_file(dir.join("MyAddon.toc")).unwrap();
    assert_eq!(find_toc_file(&dir), Some(dir.join("myaddon_standard.toc")));
}

#[test]
#[cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
fn find_toc_file_retains_classic_family_alias_after_generic() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = make_addon_dir(
        tmp.path(),
        "MyAddon",
        &["MyAddon-Classic.toc", "MyAddon.toc"],
    );
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon.toc")));
    std::fs::remove_file(dir.join("MyAddon.toc")).unwrap();
    assert_eq!(find_toc_file(&dir), Some(dir.join("MyAddon-Classic.toc")));
}

#[test]
#[cfg(feature = "client-mists")]
fn find_toc_file_preserves_named_mists_compatibility_variants() {
    let tmp = tempfile::tempdir().unwrap();
    for (folder, flavor) in [
        ("Blizzard_GameMenu", "Mainline"),
        ("Blizzard_UIParentPanelManager", "Classic"),
    ] {
        let underscore = format!("{folder}_{flavor}.toc");
        let dash = format!("{folder}-{flavor}.toc");
        let generic = format!("{folder}.toc");
        let dir = make_addon_dir(tmp.path(), folder, &[&dash, &generic, &underscore]);
        assert_eq!(find_toc_file(&dir), Some(dir.join(&underscore)));
        std::fs::remove_file(dir.join(underscore)).unwrap();
        assert_eq!(find_toc_file(&dir), Some(dir.join(dash)));
    }
}
