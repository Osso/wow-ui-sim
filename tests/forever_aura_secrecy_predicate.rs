//! Publication/preservation of the existing public Forever AuraData policy.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_aura_secrecy_predicate_agrees_with_current_aura_outputs() {
    let env = WowLuaEnv::new().unwrap();
    for restricted_input in [false, true, false] {
        env.state().borrow_mut().restricted_actions[0] = if restricted_input { 2 } else { 0 };
        env.exec(
            r#"
            local query = rawget(C_Secrets, 'ShouldAurasBeSecret')
            assert(type(query) == 'function')
            assert(type(query()) == 'boolean' and select('#', query()) == 1)
            local data = C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')
            assert(data ~= nil)
            assert(query() == issecretvalue(data))
            assert(query() == false, 'Forever keeps its existing public AuraData policy')
            assert(not issecretvalue(query()))
            "#,
        )
        .unwrap();
    }
    env.apply_post_load_workarounds();
    env.exec("assert(type(rawget(C_Secrets, 'ShouldAurasBeSecret')) == 'function')")
        .unwrap();
}

#[test]
fn forever_aura_secrecy_predicate_preserves_addon_taint_and_separate_object_context() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().auras_secret_in_context = true;
    env.exec(
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'AuraPredicateProbe')
            assert(C_Secrets.ShouldAurasBeSecret() == false)
            assert(not issecretvalue(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL')))
            assert(debug.getstacktaint() == 'AuraPredicateProbe')
        end
        debug.setobjecttaint(addon, 'AuraPredicateProbe')
        addon()
        "#,
    )
    .unwrap();
}
