use std::path::{Path, PathBuf};
use wow_ui_sim::toc::TocFile;

fn blizzard_shared_xml_base_toc() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Interface/BlizzardUI/Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc")
}

const ACE3_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/Ace3/Ace3.toc";

#[test]
fn test_parse_blizzard_shared_xml_base() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path)
        .expect("Failed to read TOC file");

    assert_eq!(toc.name, "Blizzard_SharedXMLBase");
    assert!(toc.is_blizzard_addon());

    // Should have many files
    assert!(toc.files.len() > 20, "Expected many files, got {}", toc.files.len());

    // First file should be Compat.lua
    assert_eq!(toc.files[0].to_str().unwrap(), "Compat.lua");

    // Should contain Mixin.lua
    assert!(
        toc.files.iter().any(|f| f.to_str() == Some("Mixin.lua")),
        "Expected Mixin.lua in file list"
    );
}

#[test]
fn test_parse_ace3() {
    let path = Path::new(ACE3_TOC);
    if !path.exists() {
        eprintln!("Skipping test_parse_ace3: {} not found", ACE3_TOC);
        return;
    }

    let toc = TocFile::from_file(path)
        .expect("Failed to read TOC file");

    assert_eq!(toc.name, "Lib: Ace3");
    assert!(!toc.is_blizzard_addon());

    let versions = toc.interface_versions();
    assert!(!versions.is_empty(), "Expected interface version");

    assert!(
        toc.files[0].to_str().unwrap().contains("LibStub"),
        "Expected LibStub as first file"
    );
}

#[test]
fn test_file_paths_absolute() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path)
        .expect("Failed to read TOC file");

    let paths = toc.file_paths();

    for path in &paths {
        assert!(path.is_absolute(), "Expected absolute path: {:?}", path);
        assert!(path.exists(), "File should exist: {:?}", path);
    }
}

#[test]
fn test_lua_and_xml_files() {
    let toc_path = blizzard_shared_xml_base_toc();
    let toc = TocFile::from_file(&toc_path)
        .expect("Failed to read TOC file");

    let lua_count = toc.files.iter().filter(|f| {
        f.extension().map(|e| e == "lua").unwrap_or(false)
    }).count();

    let xml_count = toc.files.iter().filter(|f| {
        f.extension().map(|e| e == "xml").unwrap_or(false)
    }).count();

    assert!(lua_count > 0, "Expected Lua files");
    assert!(xml_count > 0, "Expected XML files");
}
