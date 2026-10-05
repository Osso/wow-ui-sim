#![cfg(feature = "retail-12-0-7")]
//! Retail 12.0.7 world tier: `GetInstanceInfo` ret11 and the active player's tier.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn instance_info_appends_has_world_tier_from_instance_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(select('#', GetInstanceInfo()) == 11)
        assert(select(11, GetInstanceInfo()) == false)
        "#,
    )
    .expect("open world reports hasWorldTier=false");
    {
        let mut state = env.state().borrow_mut();
        state.world.in_instance = true;
        state.world.instance_type = "scenario".into();
        state.world.instance_lfg_dungeon_id = None;
        state.world.instance_has_world_tier = true;
    }
    env.exec(
        r#"
        assert(select('#', GetInstanceInfo()) == 11)
        assert(select(10, GetInstanceInfo()) == nil)
        assert(select(11, GetInstanceInfo()) == true)
        "#,
    )
    .expect("world-tier instance reports hasWorldTier=true after a nil lfgDungeonID");
}

#[test]
fn active_player_world_tier_follows_selected_difficulty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "assert(C_DelvesUI.GetWorldTierDifficultyForActivePlayer() == Enum.WorldTierDifficulty.Normal)",
    )
    .expect("no selection reports Normal");
    env.state().borrow_mut().world.world_tier_difficulty = Some(3);
    env.exec(
        "assert(C_DelvesUI.GetWorldTierDifficultyForActivePlayer() == Enum.WorldTierDifficulty.Mythic)",
    )
    .expect("selected Mythic tier is reported");
    env.state().borrow_mut().world.world_tier_difficulty = Some(2);
    env.exec(
        "assert(C_DelvesUI.GetWorldTierDifficultyForActivePlayer() == Enum.WorldTierDifficulty.Heroic)",
    )
    .expect("tier changes are live");
}
