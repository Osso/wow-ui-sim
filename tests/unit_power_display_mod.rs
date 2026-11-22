#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec("A_Admin.SetPlayerClass(9); A_Admin.SetSpec(3)")
        .expect("Set the player to a Destruction Warlock before UI loading");
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
        r#"
        local frame = WarlockPowerFrame
        assert(frame, "WarlockPowerFrame was not created")
        assert(frame:IsShown(), "WarlockPowerFrame is hidden")
        assert(frame.powerType == Enum.PowerType.SoulShards, "wrong WarlockPowerFrame power type")
        A_Admin.SetPlayerPower(35, 5, frame.powerType)
        A_Admin.SetUnitPowerDisplayMod(frame.powerType, 10)
        frame:UpdatePower()

        local shards = {}
        for shard in frame.classResourceButtonPool:EnumerateActive() do
            shards[#shards + 1] = shard
        end
        assert(#shards == 5, "expected 5 active shards, got " .. #shards)
        for index = 1, 3 do
            assert(shards[index].fillAmount == 1, "full shard has wrong fill amount")
            assert(shards[index].Shard_Soul:IsShown(), "full shard soul is hidden")
            assert(not shards[index].FillIncrement:IsShown(), "full shard increment is shown")
        end
        assert(shards[4].fillAmount == 0.5, "10x modifier did not show a half shard")
        assert(not shards[4].Shard_Soul:IsShown(), "half shard soul is shown")
        assert(shards[4].FillIncrement:IsShown(), "half shard increment is hidden")

        A_Admin.SetUnitPowerDisplayMod(frame.powerType, 100)
        frame:UpdatePower()
        assert(shards[1].fillAmount == 0.3, "100x modifier did not show a 0.3 shard")
        assert(not shards[1].Shard_Soul:IsShown(), "partial shard soul is shown")
        assert(shards[1].FillIncrement:IsShown(), "partial shard increment is hidden")
        for index = 2, 5 do
            assert(shards[index].fillAmount == 0, "later shard has a nonzero fill")
            assert(not shards[index].Shard_Soul:IsShown(), "later shard soul is shown")
            assert(not shards[index].FillIncrement:IsShown(), "later shard increment is shown")
        end
        "#,
    )
    .expect("the visible Warlock shard frame should use supplied display modifiers");
}
