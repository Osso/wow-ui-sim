#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::TargetInfo;

fn npc(guid: &str) -> TargetInfo {
    TargetInfo {
        unit_id: "target".into(),
        name: "Lieutenant".into(),
        class_index: 1,
        level: 80,
        health: 100,
        health_max: 100,
        power: 0,
        power_max: 0,
        power_type: 1,
        power_type_name: "RAGE".into(),
        is_player: false,
        is_enemy: true,
        guid: guid.into(),
        classification: "elite".into(),
        creature_type: "Humanoid".into(),
        reaction: 2,
        interaction: Default::default(),
    }
}

#[test]
fn p1200_unit_role_snapshots_follow_guid_not_selected_token() {
    let env = WowLuaEnv::new().unwrap();
    let original = "Creature-0-0-0-0-123-1";
    {
        let mut sim = env.state().borrow_mut();
        sim.current_target = Some(npc(original));
        sim.plain_global_inputs
            .lieutenant_guids
            .insert(original.into());
        sim.plain_global_inputs.minion_guids.insert(original.into());
        sim.plain_global_inputs
            .npc_as_player_guids
            .insert(original.into());
    }
    env.exec("assert(UnitIsLieutenant('target')); assert(UnitIsMinion('target')); assert(UnitIsNPCAsPlayer('target'))").unwrap();
    env.state().borrow_mut().current_focus = Some(npc(original));
    env.state().borrow_mut().current_target = Some(npc("Creature-0-0-0-0-124-2"));
    env.exec(
        r#"
        assert(not UnitIsLieutenant('target') and UnitIsLieutenant('focus'))
        assert(not UnitIsMinion('target') and UnitIsMinion('focus'))
        assert(not UnitIsNPCAsPlayer('target') and UnitIsNPCAsPlayer('focus'))
    "#,
    )
    .unwrap();
    env.state().borrow_mut().current_focus = None;
    env.exec("assert(not UnitIsLieutenant('focus')); assert(not UnitIsMinion('focus')); assert(not UnitIsNPCAsPlayer('focus'))").unwrap();
}
