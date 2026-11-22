#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn shard_bar_uses_supplied_power_display_modifiers() {
    let env = load_full_game_ui();
    env.exec(
        "local powerType = Enum.PowerType.SoulShards; \
         local bar = { powerType = powerType }; \
         A_Admin.SetPlayerPower(100, 500, powerType); \
         A_Admin.SetUnitPowerDisplayMod(powerType, 10); \
         assert(WarlockPowerBar.UnitPower(bar, 'player') == 10); \
         A_Admin.SetUnitPowerDisplayMod(powerType, 100); \
         assert(WarlockPowerBar.UnitPower(bar, 'player') == 1)",
    )
    .expect("original ShardBar should scale power by the supplied display modifier");
}
