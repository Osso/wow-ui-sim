#![cfg(feature = "client-mists")]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::toc::TocFile;

const INSTALLED_ADDON_ROOT: &str = "/syncthing/World of Warcraft/_classic_/Interface/AddOns";

#[derive(Clone, Copy, Debug)]
enum AddonShape {
    LargeDatabase,
    AuctionHouse,
    FrameMover,
    PanelBehavior,
    QuestDialog,
    MapUi,
    QualityOfLife,
    Nameplates,
    ItemOverlay,
}

#[derive(Clone, Copy, Debug)]
struct PandariaAddonTarget {
    name: &'static str,
    shape: AddonShape,
    toc_file: &'static str,
    min_files: usize,
    saved_variable_prefix: &'static str,
}

const PANDARIA_ADDON_TARGETS: &[PandariaAddonTarget] = &[
    PandariaAddonTarget {
        name: "AllTheThings",
        shape: AddonShape::LargeDatabase,
        toc_file: "AllTheThings.toc",
        min_files: 90,
        saved_variable_prefix: "ATT",
    },
    PandariaAddonTarget {
        name: "Auctionator",
        shape: AddonShape::AuctionHouse,
        toc_file: "Auctionator.toc",
        min_files: 8,
        saved_variable_prefix: "AUCTIONATOR",
    },
    PandariaAddonTarget {
        name: "BlizzMove",
        shape: AddonShape::FrameMover,
        toc_file: "BlizzMove.toc",
        min_files: 4,
        saved_variable_prefix: "BlizzMove",
    },
    PandariaAddonTarget {
        name: "DeModal",
        shape: AddonShape::PanelBehavior,
        toc_file: "DeModal.toc",
        min_files: 4,
        saved_variable_prefix: "DEMODAL",
    },
    PandariaAddonTarget {
        name: "DialogueUI",
        shape: AddonShape::QuestDialog,
        // Source: /syncthing/World of Warcraft/_classic_/Interface/AddOns/DialogueUI/DialogueUI.toc
        // Version: 1.0.7 b; SHA-256: f0b455dced85c5a3b8ee5818233da60bb2c03a7aa6062af87105f5e8190a8022
        // The generic TOC declares Mists via its X-GameType header.
        toc_file: "DialogueUI.toc",
        min_files: 5,
        saved_variable_prefix: "DialogueUI",
    },
    PandariaAddonTarget {
        name: "Leatrix_Maps",
        shape: AddonShape::MapUi,
        toc_file: "Leatrix_Maps_Mists.toc",
        min_files: 3,
        saved_variable_prefix: "LeaMaps",
    },
    PandariaAddonTarget {
        name: "Leatrix_Plus",
        shape: AddonShape::QualityOfLife,
        toc_file: "Leatrix_Plus_Mists.toc",
        min_files: 3,
        saved_variable_prefix: "LeaPlus",
    },
    PandariaAddonTarget {
        name: "Plater",
        shape: AddonShape::Nameplates,
        toc_file: "Plater_Mists.toc",
        min_files: 40,
        saved_variable_prefix: "Plater",
    },
    PandariaAddonTarget {
        name: "SimpleItemLevel",
        shape: AddonShape::ItemOverlay,
        toc_file: "SimpleItemLevel.toc",
        min_files: 4,
        saved_variable_prefix: "SimpleItemLevel",
    },
];

fn addon_root() -> PathBuf {
    std::env::var_os("WOW_SIM_PANDARIA_ADDONS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(INSTALLED_ADDON_ROOT))
}

fn target(name: &str) -> &'static PandariaAddonTarget {
    PANDARIA_ADDON_TARGETS
        .iter()
        .find(|target| target.name == name)
        .unwrap_or_else(|| panic!("missing Pandaria addon target {name}"))
}

fn toc_basename(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .expect("TOC path should have a UTF-8 file name")
}

fn parse_target_toc(target: &PandariaAddonTarget) -> (PathBuf, TocFile) {
    let addon_dir = addon_root().join(target.name);
    assert!(
        addon_dir.is_dir(),
        "{} should exist in installed Pandaria addon root {}",
        target.name,
        addon_root().display()
    );

    let toc_path = find_toc_file(&addon_dir)
        .unwrap_or_else(|| panic!("{} should have a Mists-compatible TOC", target.name));
    assert_eq!(
        toc_basename(&toc_path),
        target.toc_file,
        "{} should resolve the expected Mists TOC",
        target.name
    );

    let toc = TocFile::from_file(&toc_path)
        .unwrap_or_else(|error| panic!("{} TOC should parse: {error}", target.name));
    (toc_path, toc)
}

fn assert_source_contract(target: &PandariaAddonTarget) {
    let (_toc_path, toc) = parse_target_toc(target);
    let interfaces = toc.interface_versions();
    assert!(
        interfaces
            .iter()
            .any(|version| *version == 50503 || *version == 50504),
        "{} should declare Pandaria Classic interface 50503/50504; got {interfaces:?}",
        target.name
    );
    assert!(
        toc.files.len() >= target.min_files,
        "{} should keep a substantive TOC file list for {:?}; got {} files",
        target.name,
        target.shape,
        toc.files.len()
    );
    assert!(
        toc.dependencies().is_empty(),
        "{} target is expected to be standalone for the first Pandaria pass; required deps={:?}",
        target.name,
        toc.dependencies()
    );
    assert!(
        toc.saved_variables()
            .iter()
            .chain(toc.saved_variables_per_character().iter())
            .any(|name| name.starts_with(target.saved_variable_prefix)),
        "{} should expose saved-variable ownership for harness persistence checks",
        target.name
    );
}

fn assert_manifest_row_matches_target(target: &PandariaAddonTarget, row: &[&str]) {
    assert_eq!(row[1], "mists", "{} profile should be mists", target.name);
    assert_eq!(row[3], "-", "{} local ref should be '-'", target.name);
    assert_eq!(row[4], ".", "{} local subpath should be root", target.name);
    assert_eq!(
        row[2],
        format!("mists-addon:{}", target.name),
        "{} manifest URL should use the installed-or-fixture Mists resolver",
        target.name
    );
}

#[test]
fn pandaria_manifest_matches_the_installed_addon_targets() {
    let manifest = include_str!("../tools/classic-addon-manifest.tsv");
    let target_names: HashSet<_> = PANDARIA_ADDON_TARGETS
        .iter()
        .map(|target| target.name)
        .collect();
    let mut seen = HashSet::new();

    for line in manifest.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let row: Vec<_> = line.split('\t').collect();
        assert_eq!(
            row.len(),
            5,
            "manifest row should have five columns: {line}"
        );
        if row[1] != "mists" {
            continue;
        }

        let target = target(row[0]);
        assert_manifest_row_matches_target(target, &row);
        seen.insert(row[0]);
    }

    assert_eq!(
        seen, target_names,
        "Mists manifest rows should exactly match the analyzed installed Pandaria addons"
    );
}

#[test]
fn all_the_things_installed_source_contract() {
    assert_source_contract(target("AllTheThings"));
}

#[test]
fn auctionator_installed_source_contract() {
    assert_source_contract(target("Auctionator"));
}

#[test]
fn blizzmove_installed_source_contract() {
    assert_source_contract(target("BlizzMove"));
}

#[test]
fn demodal_installed_source_contract() {
    assert_source_contract(target("DeModal"));
}

#[test]
fn dialogue_ui_installed_source_contract() {
    assert_source_contract(target("DialogueUI"));
}

#[test]
fn leatrix_maps_installed_source_contract() {
    assert_source_contract(target("Leatrix_Maps"));
}

#[test]
fn leatrix_plus_installed_source_contract() {
    assert_source_contract(target("Leatrix_Plus"));
}

#[test]
fn plater_installed_source_contract() {
    assert_source_contract(target("Plater"));
}

#[test]
fn simple_item_level_installed_source_contract() {
    assert_source_contract(target("SimpleItemLevel"));
}
