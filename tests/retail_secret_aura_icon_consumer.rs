#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, load_env, set_player_auras, settle,
};

#[test]
fn retail_tainted_aura_container_sets_secret_icon_through_original_button() {
    let env = load_env();
    env.state().borrow_mut().restricted_actions[0] = 2;
    set_player_auras(&env, vec![aura(987_654)]);

    env.exec("ProbeContainer = AuditContainer({{'icons', 'HELPFUL'}}); ProbeContainer:Show()")
        .expect("tainted addon callback creates and shows the original aura container");
    settle(&env);

    assert_no_lua_errors(&env);
}
