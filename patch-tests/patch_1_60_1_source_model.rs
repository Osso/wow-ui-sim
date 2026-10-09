//! Frozen 1.60.1 line 12: a configured player name before PLAYER_LOGIN.
//! No tuple, unavailable-name, token-alias, native, security or loaded-UI credit.

#[test]
fn forever_source_player_name_reads_configured_state_before_login() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let other = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    other.state().borrow_mut().player.name = "Linus".to_owned();

    for name in ["Ada", "Grace"] {
        env.state().borrow_mut().player.name = name.to_owned();
        assert_eq!(
            env.eval::<String>("return UnitName('player')").unwrap(),
            name
        );
        assert_eq!(
            other.eval::<String>("return UnitName('player')").unwrap(),
            "Linus"
        );
    }
}
