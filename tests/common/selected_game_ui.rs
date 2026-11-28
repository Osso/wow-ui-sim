//! Full game UI selected by the profile's real TOCs, with supplied Lua inputs.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

pub fn load_selected_game_ui(inputs: &str, required_addons: &[&str]) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let screen = ScreenKind::Game;
    env.set_screen_mode(screen);
    env.set_screen_size(1024.0, 768.0);
    wow_ui_sim::xml::register_intrinsic_templates();
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, screen);
    env.exec(inputs).unwrap();
    for (name, toc) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    for required in required_addons {
        assert!(addons.iter().any(|(name, _)| name.as_str() == *required), "{required} must be selected");
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, screen);
    env
}
