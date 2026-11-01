#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn end_of_match_ui_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_EndOfMatchUI/Blizzard_EndOfMatchUI.toc")
}

fn end_match_ui_alt_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_EndOfMatchUI/Blizzard_EndMatchUI.toc")
}

#[test]
fn blizzard_end_of_match_ui_toc_declares_plunderstorm_only_with_no_deps() {
    let toc =
        TocFile::from_file(&end_of_match_ui_toc()).expect("Blizzard_EndOfMatchUI TOC should parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_EndOfMatchUI omits `## LoadOnDemand:` — when the Plunderstorm client loads \
         it auto-discovers on the Game screen, no explicit LoadAddOn() needed"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_EndOfMatchUI does not declare `## UseSecureEnvironment` — runs in the \
         standard taint environment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_EndOfMatchUI declares zero dependencies — the addon's three mixins reference \
         only globally-registered templates (BigRedThreeSliceButtonTemplate / \
         BigGoldRedThreeSliceButtonTemplate / HorizontalLayoutFrame) and a handful of \
         `WOW_LABS_*` localization strings, no other addon needs to load first"
    );

    assert!(
        toc.is_game_type_restricted(),
        "Blizzard_EndOfMatchUI declares `## AllowLoadGameType: plunderstorm` — `plunderstorm` \
         is NOT one of the mainline tokens (src/toc.rs:298-299), so the addon IS \
         game-type-restricted on standard retail. discover_blizzard_addons_for_screen \
         filters it out at src/loader/mod.rs:527"
    );

    let toc_text = std::fs::read_to_string(end_of_match_ui_toc())
        .expect("Blizzard_EndOfMatchUI TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_EndOfMatchUI declares `## AllowLoad: Game` — confines the post-match summary \
         frame to the in-game screen (no glue-screen end-of-match overlay)"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_EndOfMatchUI declares `## DefaultState: enabled` — auto-enabled on first \
         install, no user opt-in required for the Plunderstorm post-match recap"
    );
}

#[test]
fn blizzard_end_match_ui_alternate_toc_mirrors_main_toc_flags() {
    let toc = TocFile::from_file(&end_match_ui_alt_toc())
        .expect("Blizzard_EndMatchUI alternate TOC should parse");

    assert!(
        toc.is_game_type_restricted(),
        "The alternate `Blizzard_EndMatchUI.toc` (Title: 'Blizzard End Match UI', no `Of`) \
         shares the same `## AllowLoadGameType: plunderstorm` restriction as the canonical \
         `Blizzard_EndOfMatchUI.toc`. Both TOCs reference the same Lua + XML pair, so the \
         flags must match"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Alternate TOC declares zero dependencies — same as the canonical TOC"
    );
    assert!(
        !toc.is_load_on_demand(),
        "Alternate TOC omits `## LoadOnDemand:` — same as the canonical TOC"
    );
}

#[test]
fn blizzard_end_of_match_ui_is_excluded_from_standard_game_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_EndOfMatchUI");
    assert!(
        !in_game,
        "Blizzard_EndOfMatchUI must be filtered out of Game-screen auto-discovery on standard \
         mainline retail. The TOC declares `## AllowLoadGameType: plunderstorm`, and \
         discover_blizzard_addons_for_screen skips game-type-restricted addons at \
         src/loader/mod.rs:527 unless the active game type matches"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_EndOfMatchUI");
    assert!(
        !in_login,
        "Blizzard_EndOfMatchUI must NOT appear on Login / glue screens — `## AllowLoad: Game` \
         restricts it to the Game screen, and `## AllowLoadGameType: plunderstorm` restricts \
         it to the Plunderstorm client"
    );
}

