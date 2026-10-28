//! Approved defaults through selected modern Blizzard consumers.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[test]
fn approved_native_defaults_quest_story_consumer() {
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
    env.exec(r#"
        local f = QuestLogQuests_Update
        local source = debug.getinfo(f, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        local pool = {ReleaseAll=function() end}
        local header = {shown=true, Hide=function(self) self.shown=false end}
        local separator = {layoutIndex=1, SetShown=function() end}
        QuestScrollFrame = {titleFramePool=pool, objectiveFramePool=pool, headerFramePool=pool,
            campaignHeaderFramePool=pool, campaignHeaderMinimalFramePool=pool, covenantCallingsHeaderFramePool=pool,
            Contents={StoryHeader=header, Separator=separator, ResetUsage=function() end,
                SelectButtonByQuestID=function() end, Layout=function() end},
            SearchBox={UpdateState=function() end}, UpdateBackground=function() end}
        QuestMapFrame = {ResetLayoutIndex=function() end, GetParent=function() return {GetMapID=function() return 84 end} end,
            SetFrameLayoutIndex=function() end, GetLastLayoutIndex=function() return 1 end}
        QuestLogQuests_BuildQuestInfoContainer=function() return {} end
        QuestLogQuests_GetCampaignInfos=function() return {} end
        QuestLogQuests_GetCovenantCallingsInfos=function() return {} end
        QuestLogQuests_GetQuestInfos=function() return {} end
        QuestLogQuests_BuildInitialDisplayState=function() return {} end
        QuestLogQuests_DisplayQuestsFromIndices=function() end
        QuestLogQuests_ShowQuestCount=function() end
        f()
        assert(not header.shown)
        assert(select('#', C_QuestLog.GetZoneStoryInfo(84)) == 0)
        assert(select('#', C_QuestLog.GetZoneStoryInfo(999999)) == 0)
    "#).expect("original quest list hides story for unknown map");
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
