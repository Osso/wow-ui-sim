//! Original world-map consumer for the user-approved quest difficulty behavior.

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
fn user_defaults_batch2_world_map_quest_consumer() {
    let env = keybinding_panel_fixtures::setup_env();
    env.send_key_press("M", None).expect("open world map");
    env.exec(
        r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(type(QuestMapFrame:GetParent():GetMapID()) == 'number')
        local header = QuestScrollFrame.Contents.StoryHeader
        header:Show()
        QuestLogQuests_Update()
        assert(not header:IsShown(), 'no story result must hide the header')
    "#,
    )
    .expect("real quest list consumes the approved no-story default");
}
