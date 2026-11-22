#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

const PREPARE_STORY_HEADER: &str = r#"
    local mapID = QuestMapFrame:GetParent():GetMapID()
    local otherMapID = mapID + 1
    A_Admin.SetZoneStoryInfo(mapID, 513, 84)
    A_Admin.SetZoneStoryInfo(otherMapID, 948, 85)

    local achievementID, storyMapID = C_QuestLog.GetZoneStoryInfo(mapID)
    assert(achievementID == 513 and storyMapID == 84)
    achievementID, storyMapID = C_QuestLog.GetZoneStoryInfo(otherMapID)
    assert(achievementID == 948 and storyMapID == 85)
    assert(select('#', C_QuestLog.GetZoneStoryInfo(otherMapID + 1)) == 0)
    assert(GetAchievementNumCriteria(513) > 0)
    assert(select(3, GetAchievementCriteriaInfo(513, 1)) == true)

    QuestLogQuests_Update()
    local header = QuestScrollFrame.Contents.StoryHeader
    assert(header:IsShown())
    assert(header.Text:GetText() == C_Map.GetMapInfo(84).name)
    assert(header.Progress:GetText() == string.format(QUEST_STORY_STATUS, 1, GetAchievementNumCriteria(513)))
"#;

const VERIFY_STORY_TOOLTIP: &str = r#"
    local tooltip = QuestScrollFrame.StoryTooltip
    local numCriteria = GetAchievementNumCriteria(513)
    assert(tooltip:IsShown())
    assert(tooltip.Title:GetText() == C_Map.GetMapInfo(84).name)
    assert(tooltip.ProgressCount:GetText() == string.format(STORY_CHAPTERS, 1, numCriteria))
"#;

#[test]
fn zone_story_supplied_relations_drive_blizzard_header_and_tooltip() {
    let env = keybinding_panel_fixtures::setup_env();
    env.exec("A_Admin.SetAchievementEarned(513, true)")
        .expect("supply achievement completion through the admin API");
    env.send_key_press("M", None).expect("open world map");
    env.exec(PREPARE_STORY_HEADER)
        .expect("supplied relations drive the original zone-story header");

    let geometry: String = env
        .eval(
            r#"
            local map = QuestMapFrame:GetParent()
            local uiRight = select('#', UIParent:GetRight())
            local mapRight = select('#', map:GetRight())
            local mapRect = select('#', map:GetRect())
            local mapSize = select('#', map:GetSize())
            return table.concat({uiRight, mapRight, mapRect, mapSize}, '|')
        "#,
        )
        .expect("read original tooltip anchor geometry");
    eprintln!(
        "tooltip geometry result counts: UIParent|WorldMapFrame right, rect, size={geometry}"
    );
    env.exec("QuestScrollFrame.Contents.StoryHeader:ShowTooltip()")
        .expect("original story tooltip should display");
    env.exec(VERIFY_STORY_TOOLTIP)
        .expect("original tooltip reads the selected story map and achievement progress");
}
