use std::path::Path;
use wow_ui_sim::toc::TocFile;

const BLIZZARD_SHARED_XML_BASE: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc";

const ACE3_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/Ace3/Ace3.toc";

#[test]
fn test_parse_blizzard_shared_xml_base() {
    let toc = TocFile::from_file(Path::new(BLIZZARD_SHARED_XML_BASE))
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
    let toc = TocFile::from_file(Path::new(ACE3_TOC))
        .expect("Failed to read TOC file");

    assert_eq!(toc.name, "Lib: Ace3");
    assert!(!toc.is_blizzard_addon());

    // Should have interface version
    let versions = toc.interface_versions();
    assert!(!versions.is_empty(), "Expected interface version");

    // Should have LibStub as first file
    assert!(
        toc.files[0].to_str().unwrap().contains("LibStub"),
        "Expected LibStub as first file"
    );
}

#[test]
fn test_file_paths_absolute() {
    let toc = TocFile::from_file(Path::new(BLIZZARD_SHARED_XML_BASE))
        .expect("Failed to read TOC file");

    let paths = toc.file_paths();

    // All paths should be absolute and exist
    for path in &paths {
        assert!(path.is_absolute(), "Expected absolute path: {:?}", path);
        assert!(path.exists(), "File should exist: {:?}", path);
    }
}

#[test]
fn test_lua_and_xml_files() {
    let toc = TocFile::from_file(Path::new(BLIZZARD_SHARED_XML_BASE))
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
