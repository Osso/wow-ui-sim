use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn talent_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TalentUI")
}

fn talent_ui_mists_toc() -> PathBuf {
    talent_ui_dir().join("Blizzard_TalentUI_Mists.toc")
}

const ALL_FOUR_SCREENS: &[ScreenKind] = &[
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

#[test]
fn find_toc_file_returns_none_on_mainline_target() {
    let resolved = find_toc_file(&talent_ui_dir());
    assert!(
        resolved.is_none(),
        "find_toc_file must return None — addon ships ONLY \
         `Blizzard_TalentUI_Mists.toc`, no `_Mainline.toc` and no bare \
         `Blizzard_TalentUI.toc`. The fallback at src/loader/mod.rs:78-92 \
         explicitly excludes `_Mists` filenames so the simulator's \
         mainline target finds no eligible TOC. Got: {resolved:?}"
    );
}

#[test]
fn mists_toc_file_exists_at_expected_path() {
    assert!(
        talent_ui_mists_toc().exists(),
        "Blizzard_TalentUI_Mists.toc must exist on disk — direct parse \
         path for verifying the legacy talent-tree addon's body even \
         though the mainline target won't load it"
    );
}

#[test]
fn flavor_subdirs_present_classic_subdir_absent() {
    assert!(
        talent_ui_dir().join("Mists").is_dir(),
        "Mists/ flavor subdir must exist — holds the Mists-tier \
         StaticPopupDialogs + PlayerTalentFrame logic"
    );
    assert!(
        talent_ui_dir().join("Cata").is_dir(),
        "Cata/ flavor subdir must exist — holds the Cataclysm-era \
         talent-tree variant (referenced by no TOC body in our \
         checkout, kept only for source-archive completeness)"
    );
    assert!(
        !talent_ui_dir().join("Classic").exists(),
        "Classic/ subdir must be ABSENT in the mainline-flavor sparse \
         checkout — the Mists TOC body references \
         Classic\\Blizzard_TalentUI_Shared.lua/.xml which would be \
         pulled from the upstream classic-flavor source distribution \
         (not mirrored here). Confirms the addon is unrunnable in our \
         checkout regardless of TOC selection"
    );
}

#[test]
fn mists_toc_parses_with_load_on_demand_and_help_plate_dep() {
    let toc = TocFile::from_file(&talent_ui_mists_toc()).expect("Mists TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — talent panel only loads when the player \
         opens it via the talent micro-button or N keybind (legacy \
         Mists semantics; mainline retail uses Blizzard_PlayerSpells \
         instead)"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HelpPlate".to_string()],
        "`## Dependencies: Blizzard_HelpPlate` — the talent panel's \
         tutorial overlay depends on Blizzard_HelpPlate's HelpPlate \
         framework. Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_game_type_restricted());
    assert!(toc.default_enabled());
}

#[test]
fn mists_toc_raw_bytes_pin_three_metadata_directives() {
    let raw = std::fs::read_to_string(talent_ui_mists_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard Talent UI",
        "## LoadOnDemand: 1",
        "## Dependencies: Blizzard_HelpPlate",
        "Blizzard_TalentUI_Bootstrap.lua [Bootstrap]",
        "Mists\\Blizzard_TalentUI_Bootstrap.lua [Bootstrap]",
        "Classic\\Blizzard_TalentUI_Shared.lua",
        "Classic\\Blizzard_TalentUI_Shared.xml",
        "Mists\\Blizzard_TalentUI.lua",
        "Mists\\Blizzard_TalentUI.xml",
        "Classic\\Localization.lua",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — current body includes root and Mists bootstraps \
             before the five legacy source entries"
        );
    }

    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## RequiredDep"));
}

#[test]
fn body_resolves_two_bootstraps_and_five_entries_with_normalized_slashes() {
    let toc = TocFile::from_file(&talent_ui_mists_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = vec![
        "Blizzard_TalentUI_Bootstrap.lua".to_string(),
        "Mists/Blizzard_TalentUI_Bootstrap.lua".to_string(),
        "Classic/Blizzard_TalentUI_Shared.lua".to_string(),
        "Classic/Blizzard_TalentUI_Shared.xml".to_string(),
        "Mists/Blizzard_TalentUI.lua".to_string(),
        "Mists/Blizzard_TalentUI.xml".to_string(),
        "Classic/Localization.lua".to_string(),
    ];

    assert_eq!(
        body, expected,
        "Body must resolve to two bootstraps followed by five legacy source entries in declared \
         order. Got: {body:?}"
    );
}

#[test]
fn allow_load_absent_restricts_to_game_screen_only() {
    let toc = TocFile::from_file(&talent_ui_mists_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only — talent tree only renders in-world"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must be excluded — talent tree \
             needs PLAYER_LOGIN's spec/talent state which only exists \
             in-world"
        );
    }
}

#[test]
fn absent_from_every_screen_eager_discovery() {
    for screen in ALL_FOUR_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_TalentUI");
        assert!(
            !found,
            "Blizzard_TalentUI must be absent from {screen:?} eager \
             discovery — find_toc_file returns None on the mainline \
             target (only `_Mists.toc` exists, fallback excludes it), \
             so discover_blizzard_addons_for_screen at \
             src/loader/mod.rs:521 short-circuits with `continue`"
        );
    }
}

#[test]
fn no_mainline_addon_declares_talent_ui_as_dependency() {
    let entries = std::fs::read_dir(blizzard_ui_dir()).expect("BlizzardUI dir reads");
    let mut declarers: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let addon_dir = entry.path();
        if !addon_dir.is_dir() {
            continue;
        }
        let Some(toc_path) = find_toc_file(&addon_dir) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        if toc.dependencies().iter().any(|d| d == "Blizzard_TalentUI")
            || toc.optional_deps().iter().any(|d| d == "Blizzard_TalentUI")
        {
            let name = addon_dir.file_name().unwrap().to_string_lossy().to_string();
            declarers.push(name);
        }
    }

    assert!(
        declarers.is_empty(),
        "No mainline-discoverable Blizzard addon may declare \
         Blizzard_TalentUI as a Dependency or OptionalDep — mainline \
         retail talents live in Blizzard_PlayerSpells/Talents/, not \
         here. Found declarers: {declarers:?}"
    );
}
