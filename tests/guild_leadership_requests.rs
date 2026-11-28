//! C_GuildInfo Demote/SetLeader/Disband request submission on Retail 12.1.5 and Forever.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use wow_ui_sim::c_api::c_guild_info::GuildRequest;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::GuildMember;

fn guild_master_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.world.guild_id = 42;
        sim.world.guild_player_rank = 0;
        sim.world.guild_members = vec![GuildMember {
            name: "Alice".into(),
            guid: None,
            rank_index: 2,
            online: true,
        }];
        sim.guild_request_inputs.transport_available = true;
    }
    env
}

#[test]
fn leadership_requests_submit_resolved_names_without_mutating_roster() {
    let env = guild_master_env();
    env.exec(
        r#"
        C_GuildInfo.Demote('aLiCe')
        C_GuildInfo.Demote('Nobody')
        C_GuildInfo.SetLeader('Bob')
        C_GuildInfo.Disband()
    "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    assert_eq!(
        sim.guild_request_inputs.submitted,
        vec![
            GuildRequest::Demote("Alice".into()),
            GuildRequest::SetLeader("Bob".into()),
            GuildRequest::Disband,
        ]
    );
    assert_eq!(sim.world.guild_members[0].rank_index, 2);
    assert_eq!(sim.world.guild_id, 42);
}

#[test]
fn leadership_requests_validate_names_and_respect_gates() {
    let env = guild_master_env();
    let (missing, too_long): (bool, bool) = env
        .eval(
            "return not pcall(C_GuildInfo.Demote), \
             not pcall(C_GuildInfo.Demote, string.rep('x', 49))",
        )
        .unwrap();
    assert!(missing && too_long);
    env.state().borrow_mut().world.guild_player_rank = 1;
    env.exec("C_GuildInfo.SetLeader('Bob')").unwrap();
    env.state().borrow_mut().guild_request_inputs.action_disabled = true;
    env.exec("C_GuildInfo.Disband()").unwrap();
    assert!(env.state().borrow().guild_request_inputs.submitted.is_empty());
}
