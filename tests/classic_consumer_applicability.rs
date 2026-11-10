//! The classic fixtures must load their native consumers, not require Retail roots.
#![cfg(any(
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-mists"
))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn classic_panel_fixture_loads_native_action_buttons_without_timerunning() {
    let env = WowLuaEnv::new().expect("create classic panel environment");
    env.set_screen_size(1024.0, 768.0);
    common::panel_fixtures::seed_addon_search_paths(&env);
    common::panel_fixtures::load_panel_addons(&env);
    env.exec(
        r#"
        assert(UIParent and UIParent:GetScript('OnEvent'), 'native UIParent consumer')
        assert(ActionButton1 and ActionButton1:GetScript('OnEvent'), 'native action button consumer')
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_TimerunningUtil'), 'mainline-only root')
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_POIButton'), 'absent Retail root')
        "#,
    )
    .expect("classic panel fixture must load its actual native consumers");
}

#[test]
fn classic_game_menu_fixture_opens_and_closes_native_menu_without_poi_button() {
    let env = common::game_menu_fixture::setup_game_menu_env();
    env.exec(
        r#"
        assert(GameMenuFrame and GameMenuFrame:GetScript('OnShow'), 'native classic menu')
        HideUIPanel(GameMenuFrame)
        ToggleGameMenu()
        assert(GameMenuFrame:IsShown(), 'native menu opens')
        ToggleGameMenu()
        assert(not GameMenuFrame:IsShown(), 'native menu closes')
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_POIButton'), 'absent Retail root')
        "#,
    )
    .expect("classic game-menu fixture must execute its native toggle");
}
