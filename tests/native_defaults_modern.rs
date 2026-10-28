//! Approved defaults through selected modern Blizzard consumers.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
fn approved_native_defaults_quest_story_return_shape() {
    let ui = blizzard_ui_dir();
    let toc =
        TocFile::from_file(&find_toc_file(&ui.join("Blizzard_UIPanels_Game")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|p| p.to_string_lossy().contains("QuestMapFrame.lua"))
    );
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_UIPanels_Game"],
        &[],
    );
    assert!(loaded.iter().any(|n| n == "Blizzard_UIPanels_Game"));
    // A prior substitute-pool fixture failed before the story lookup. This
    // independent check covers only the declared return shape.
    env.exec(r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(select('#', C_QuestLog.GetZoneStoryInfo(84)) == 0)
        assert(select('#', C_QuestLog.GetZoneStoryInfo(999999)) == 0)
    "#).expect("unknown zone story returns no values");
}

#[test]
fn approved_native_defaults_real_quest_story_consumer() {
    let env = keybinding_panel_fixtures::setup_env();
    env.send_key_press("M", None).expect("open world map");
    env.exec(r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(type(QuestMapFrame:GetParent():GetMapID()) == 'number')
        local header = QuestScrollFrame.Contents.StoryHeader
        header:Show()
        QuestLogQuests_Update()
        assert(not header:IsShown(), 'no story result must hide the header')
    "#).expect("real quest list consumes the approved no-story default");
}

#[test]
fn approved_native_defaults_game_menu_external_event_consumer() {
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_GameMenu")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|p| p.to_string_lossy().contains("GameMenuFrame.lua"))
    );
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_GameMenu"],
        &[],
    );
    assert!(loaded.iter().any(|n| n == "Blizzard_GameMenu"));
    env.exec(
        r#"
        local f = GameMenuFrameMixin.ExternalEventClickCallback
        local source = debug.getinfo(f, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/GameMenuFrame.lua', 1, true))
        f({})
        assert(C_ExternalEventURL.HasURL() == false)
        assert(C_ExternalEventURL.IsNew() == false)
        assert(select('#', C_ExternalEventURL.LaunchURL()) == 0)
    "#,
    )
    .expect("original event click consumes no URL default");
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Ptr {
        env.exec("assert(C_ExternalEventURL.CanShow() == false)")
            .unwrap();
    }
}
