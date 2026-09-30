//! Explicit retail input, not inferred combat/aura activation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn stat_restriction_predicate_defaults_plain() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_Secrets.ShouldUnitStatsBeSecret) == 'function')
        assert(select('#', C_Secrets.ShouldUnitStatsBeSecret()) == 1)
        assert(C_Secrets.ShouldUnitStatsBeSecret() == false)
        assert(not issecretvalue(GetCombatRating(9)))
        assert(not issecretvalue(GetBlockChance()))
        "#,
    )
    .unwrap();
}
