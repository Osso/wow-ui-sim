#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::reputation_model::ReputationFaction;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_reputation_panel() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env
}

fn supply_rows(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.reputation.factions = [
        (899_101, "Fixture faction Alpha", 1500),
        (899_102, "Fixture faction Beta", 2400),
    ]
    .into_iter()
    .map(|(faction_id, name, standing)| ReputationFaction {
        faction_id,
        name: Some(name.to_owned()),
        reaction: 5,
        standing,
        top: 6000,
        ..Default::default()
    })
    .collect();
}

#[test]
fn original_reputation_panel_rebuilds_from_host_rows() {
    let env = load_reputation_panel();
    env.exec(
        r#"
        local panel = assert(ReputationFrame)
        assert(debug.getinfo(panel.Update, "S").source:find("Blizzard_UIPanels_Game", 1, true))
        local character = assert(CharacterFrame, "original CharacterFrame missing")
        character.activeSubframe = "ReputationFrame"
        ShowUIPanel(character)
        panel:Show()
        assert(panel:IsVisible(), "original panel is not visible")
        panel:Update()
        assert(panel.ScrollBox:GetDataProvider():GetSize() == 0, "host rows must start empty")
    "#,
    )
    .expect("original panel should render empty history");
    supply_rows(&env);
    env.exec(r#"
        local panel = ReputationFrame
        panel:Update()
        assert(panel.ScrollBox:GetDataProvider():GetSize() == 2, "both host rows must be presented")
        local function check(id, label, standing)
            local row = assert(panel.ScrollBox:FindFrameByPredicate(
                function(_, entry) return entry.factionID == id end), "original row not materialized: " .. id)
            assert(row.Content.Name:GetText() == label, "original row must display supplied name")
            assert(row.elementData.currentStanding == standing, "original row must consume standing")
        end
        check(899101, "Fixture faction Alpha", 1500)
        check(899102, "Fixture faction Beta", 2400)
    "#).expect("original panel should present distinct host rows");
}
