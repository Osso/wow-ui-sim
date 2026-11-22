#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
fn zone_story_supplied_relations_drive_blizzard_header_and_tooltip() {
    let env = keybinding_panel_fixtures::setup_env();
    env.state()
        .borrow_mut()
        .world
        .earned_achievements
        .insert(513);
    env.send_key_press("M", None).expect("open world map");
    env.exec(
        r#"
        local mapID = QuestMapFrame:GetParent():GetMapID()
        local otherMapID = mapID + 1
        A_Admin.SetZoneStoryInfo(mapID, 513, 84)
        A_Admin.SetZoneStoryInfo(otherMapID, 948, 85)

        local achievementID, storyMapID = C_QuestLog.GetZoneStoryInfo(mapID)
        assert(achievementID == 513 and storyMapID == 84)
        achievementID, storyMapID = C_QuestLog.GetZoneStoryInfo(otherMapID)
        assert(achievementID == 948 and storyMapID == 85)
        assert(GetAchievementNumCriteria(513) > 0)
        assert(select(3, GetAchievementCriteriaInfo(513, 1)) == true)

        assert(debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
            :find('Mainline/QuestMapFrame.lua', 1, true))
        QuestLogQuests_Update()
        local header = QuestScrollFrame.Contents.StoryHeader
        assert(header:IsShown())
        assert(header.Text:GetText() == C_Map.GetMapInfo(84).name)
        header:ShowTooltip()
        assert(QuestScrollFrame.StoryTooltip:IsShown())
        assert(QuestScrollFrame.StoryTooltip.Title:GetText() == C_Map.GetMapInfo(84).name)
        "#,
    )
    .expect("supplied map-story relations drive the original Blizzard consumer");
}
