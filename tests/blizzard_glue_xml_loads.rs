#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn glue_xml_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GlueXML")
}

fn glue_xml_mainline_toc() -> PathBuf {
    glue_xml_dir().join("Blizzard_GlueXML_Mainline.toc")
}

fn glue_xml_mists_toc() -> PathBuf {
    glue_xml_dir().join("Blizzard_GlueXML_Mists.toc")
}

fn load_character_select_screen() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::CharacterSelect);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[test]
fn blizzard_glue_xml_find_toc_resolves_mainline_variant() {
    let resolved = find_toc_file(&glue_xml_dir()).expect("Blizzard_GlueXML TOC should resolve");
    assert_eq!(
        resolved,
        glue_xml_mainline_toc(),
        "Blizzard_GlueXML ships two flavor TOC variants (`_Mainline.toc` for retail and \
         `_Mists.toc` for the Mists of Pandaria classic flavor) — `find_toc_file` \
         (src/loader/mod.rs:65) prefers `_Mainline.toc` on the first lookup so the simulator \
         (which targets retail mainline) ignores the Mists TOC. The Mists TOC additionally \
         declares `## AllowLoadGameType: mists` which `is_game_type_restricted()` filters out \
         of glue-screen auto-discovery, so even in the fallback scan it would not load on \
         mainline"
    );
}

#[test]
fn blizzard_glue_xml_mainline_toc_declares_load_first_glue_with_eleven_deps() {
    let toc = TocFile::from_file(&glue_xml_mainline_toc())
        .expect("Blizzard_GlueXML_Mainline TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GlueXML is non-LoadOnDemand — the bulk of the glue-screen UI surface \
         (AccountLogin, RealmList, CharacterSelect, CharacterServices, ServerAlert, MovieFrame, \
         CinematicsMenu, CreditsFrame, PhotosensitivityWarningFrame, SocialContract, \
         PromotionFrame) must auto-load on the glue-screen discovery pass"
    );
    assert!(
        toc.is_load_first(),
        "Blizzard_GlueXML declares `## LoadFirst: 1` — the glue-screen UI surface must install \
         before downstream glue-screen addons that reference its globals (Blizzard_GlueStubs's \
         dep, Blizzard_DeclensionFrameGlue's `Mainline\\DeclensionFrame.lua` overlay)"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GlueXML does not declare `## UseSecureEnvironment` — the glue-screen surface \
         is not in the secure environment (the `addToSecureEnv` flag is on the inner \
         ScopedModifier XML element of Blizzard_GlueParent's Frame, not a TOC-level flag here)"
    );
    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_StaticPopup_Glue".to_string(),
            "Blizzard_LoginWarningDialogs".to_string(),
            "Blizzard_TimerunningUtil".to_string(),
            "Blizzard_Menu".to_string(),
            "Blizzard_MoneyFrame".to_string(),
            "Blizzard_MatchmakingQueueDisplay".to_string(),
            "Blizzard_GlueCollections".to_string(),
            "Blizzard_HelpPlate".to_string(),
            "Blizzard_GlueMenuFrame".to_string(),
            "Blizzard_CharacterSelectNavBar".to_string(),
            "Blizzard_Narration".to_string(),
        ],
        "Retail 12.1.0.69497 declares eleven GlueXML dependencies in published order, \
         including Blizzard_Narration last"
    );
}

#[test]
fn blizzard_glue_xml_mainline_toc_declares_glue_screen_mainline_only() {
    let toc_text = std::fs::read_to_string(glue_xml_mainline_toc())
        .expect("Blizzard_GlueXML_Mainline TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: Glue"),
        "Blizzard_GlueXML_Mainline declares `## AllowLoad: Glue` (capital G — glue-screen-only). \
         Loads on Login + CharacterSelect + CharacterCreate, absent from Game"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_GlueXML_Mainline declares `## AllowLoadGameType: mainline` so retail-only"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_GlueXML declares `## DefaultState: enabled` — the glue-screen surface must \
         always be active"
    );
}

#[test]
fn blizzard_glue_xml_mainline_toc_lists_first_file_as_localization() {
    let toc = TocFile::from_file(&glue_xml_mainline_toc())
        .expect("Blizzard_GlueXML_Mainline TOC should parse");
    let first = toc
        .files
        .first()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .expect("Blizzard_GlueXML_Mainline TOC should list at least one file");
    assert_eq!(
        first, "Mainline/Localization.lua",
        "Blizzard_GlueXML_Mainline lists `Mainline\\Localization.lua` as the first file so \
         every downstream Lua / XML reference to the locale-specific strings (CHARACTER_SELECT_*, \
         ACCOUNT_LOGIN_*, etc.) resolves immediately"
    );
}

#[test]
fn blizzard_glue_xml_mainline_toc_enumerates_expected_top_level_files() {
    let toc = TocFile::from_file(&glue_xml_mainline_toc())
        .expect("Blizzard_GlueXML_Mainline TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    for expected in [
        "Mainline/Localization.lua",
        "ServerAlert.lua",
        "Mainline/AccountLogin.lua",
        "Mainline/RealmList.lua",
        "Mainline/CharacterSelect.lua",
        "Mainline/CharacterServices.lua",
        "MovieFrame.lua",
        "PhotosensitivityWarningFrame.lua",
        "Mainline/CinematicsMenu.lua",
        "Mainline/CreditsFrame.lua",
        "SocialContract.lua",
        "PromotionFrame.lua",
        "WoWLabs/PlunderstormLobby.lua",
    ] {
        assert!(
            files.iter().any(|f| f == expected),
            "Blizzard_GlueXML_Mainline TOC should enumerate `{expected}` — got: {files:?}"
        );
    }
}

#[test]
fn blizzard_glue_xml_mists_toc_declares_mists_game_type_only() {
    let toc =
        TocFile::from_file(&glue_xml_mists_toc()).expect("Blizzard_GlueXML_Mists TOC should parse");
    assert!(
        toc.is_load_first(),
        "Blizzard_GlueXML_Mists also declares `## LoadFirst: 1` — same load-order priority as \
         the mainline variant"
    );
    assert!(
        toc.is_game_type_restricted(),
        "Blizzard_GlueXML_Mists declares `## AllowLoadGameType: mists` which \
         `is_game_type_restricted()` (src/toc.rs:294) treats as a non-mainline restriction — so \
         this TOC is filtered out of mainline auto-discovery (only `mainline` and `standard` \
         are accepted as unrestricted game types)"
    );
    let mists_text = std::fs::read_to_string(glue_xml_mists_toc())
        .expect("Blizzard_GlueXML_Mists TOC should read");
    assert!(
        mists_text.contains("## AllowLoadGameType: mists"),
        "Blizzard_GlueXML_Mists raw TOC must contain `## AllowLoadGameType: mists`"
    );
    assert!(
        mists_text.contains("## AllowLoad: Glue"),
        "Blizzard_GlueXML_Mists also targets glue screens (the Mists classic flavor still has \
         a glue-screen flow), just gated by game type"
    );
    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_StaticPopup_Glue".to_string(),
            "Blizzard_LoginWarningDialogs".to_string(),
            "Blizzard_HelpPlate".to_string(),
            "Blizzard_GlueMenuFrame".to_string(),
            "Blizzard_Narration".to_string(),
        ],
        "Retail 12.1.0.69497 declares five Mists GlueXML dependencies, including \
         Blizzard_Narration last"
    );
}

#[test]
fn blizzard_glue_xml_directory_ships_bindings_and_subdirectories() {
    let dir = glue_xml_dir();
    assert!(
        dir.join("Bindings.xml").exists(),
        "Blizzard_GlueXML/Bindings.xml should exist — this is the auto-loaded keybinding \
         declaration file (TOGGLEGAMEMENU / TOGGLEMUSIC / TOGGLESOUND / TOGGLEAMBIENCE / \
         SCREENSHOT) that the WoW client picks up via the Bindings.xml convention regardless of \
         whether the TOC enumerates it"
    );
    for subdir in ["Mainline", "Mists", "Shared", "WoWLabs"] {
        assert!(
            dir.join(subdir).is_dir(),
            "Blizzard_GlueXML/{subdir} subdirectory should exist — ships the flavor-specific \
             ({subdir}) source files referenced by the matching TOC variant or the shared \
             cross-flavor surface"
        );
    }
}

#[test]
fn blizzard_glue_xml_appears_in_all_three_glue_screen_discoveries() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let entries: Vec<&(String, PathBuf)> = addons
            .iter()
            .filter(|(name, _)| name == "Blizzard_GlueXML")
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "Blizzard_GlueXML should appear exactly once in {screen:?} auto-discovery — \
             `find_toc_file` resolves to the `_Mainline.toc` variant on retail; the `_Mists.toc` \
             variant is filtered out by `is_game_type_restricted()`. Got entries: {entries:?}"
        );
        assert_eq!(
            entries[0].1,
            glue_xml_mainline_toc(),
            "Blizzard_GlueXML on {screen:?} should resolve to the `_Mainline.toc` variant \
             (filtered by game-type restriction logic)"
        );
    }
}

#[test]
fn blizzard_glue_xml_absent_from_game_screen_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let discovered = addons.iter().any(|(name, _)| name == "Blizzard_GlueXML");
    assert!(
        !discovered,
        "Blizzard_GlueXML MUST NOT appear in Game-screen auto-discovery — `## AllowLoad: Glue` \
         is glue-only. The in-game UI surface (UIParent / FrameXML / etc.) is loaded by \
         separate Game-screen addons"
    );
}

#[test]
fn blizzard_glue_xml_loads_without_addon_specific_lua_errors() {
    let env = load_character_select_screen();

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| e.contains("Blizzard_GlueXML/") || e.contains("Blizzard_GlueXML\\"))
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_GlueXML emitted addon-specific Lua errors during CharacterSelect-screen \
         load:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn blizzard_glue_xml_publishes_top_level_screen_mixins() {
    let env = load_character_select_screen();

    for mixin in [
        "ServerAlertMixin",
        "ServerAlertBoxMixin",
        "CollapsibleServerAlertMixin",
        "CharacterSelectFrameMixin",
        "PhotosensitivityWarningFrameMixin",
        "CinematicsMenuMixin",
        "CreditsFrameMixin",
        "AccountNameMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("mixin existence query should succeed");
        assert!(
            exists,
            "After CharacterSelect-screen load, `{mixin}` should be published as a `_G` table — \
             this is one of the top-level mixins owned by Blizzard_GlueXML's Mainline source \
             files (ServerAlert.lua, AccountLogin.lua, CharacterSelect.lua, MovieFrame.lua, \
             PhotosensitivityWarningFrame.lua, CinematicsMenu.lua, CreditsFrame.lua)"
        );
    }
}

#[test]
fn blizzard_glue_xml_publishes_movie_frame_lifecycle_helpers() {
    let env = load_character_select_screen();

    for helper in [
        "MovieFrame_OnLoad",
        "MovieFrame_OnShow",
        "MovieFrame_OnHide",
        "MovieFrame_OnUpdate",
        "MovieFrame_OnKeyUp",
        "MovieFrame_PlayMovie",
        "MovieFrame_PlayNextMovie",
        "MovieFrame_OnMovieFinished",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{helper}']) == 'function'"))
            .expect("helper existence query should succeed");
        assert!(
            exists,
            "After CharacterSelect-screen load, `{helper}` should be published as a `_G` \
             function — MovieFrame.lua publishes 8 lifecycle / playback helpers as plain \
             globals (not a mixin) since the MovieFrame XML uses `OnLoad function=\"...\"` \
             script bindings instead of `mixin=\"\"` attribute"
        );
    }
}

#[test]
fn blizzard_glue_xml_publishes_top_level_glue_screen_frames() {
    let env = load_character_select_screen();

    for frame_name in [
        "CharacterSelect",
        "CreditsFrame",
        "CinematicsMenu",
        "PhotosensitivityWarningFrame",
        "PromotionFrame",
        "SocialContractFrame",
        "StarterEditionPopUp",
    ] {
        let exists: bool = env
            .eval(&format!(
                "local f = _G['{frame_name}']; return type(f) == 'table' and type(f.GetName) == 'function'"
            ))
            .expect("frame existence query should succeed");
        assert!(
            exists,
            "After CharacterSelect-screen load, `{frame_name}` should be published as a global \
             frame (table with the `GetName` method) — these are the top-level glue-screen \
             frames declared in Blizzard_GlueXML's XML files"
        );
    }
}
