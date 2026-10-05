#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_removed_plain_globals_are_not_simulator_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({
            'CombatLogAddFilter', 'CombatLogAdvanceEntry', 'CombatLogClearEntries',
            'CombatLogGetCurrentEntry', 'CombatLogGetCurrentEventInfo',
            'CombatLogGetNumEntries', 'CombatLogGetRetentionTime', 'CombatLogResetFilter',
            'CombatLogSetCurrentEntry', 'CombatLogSetRetentionTime', 'CombatLogShowCurrentEntry',
            'CombatLog_Object_IsA', 'CombatTextSetActiveUnit', 'DeathRecap_GetEvents',
            'DeathRecap_HasEvents', 'GetBattlegroundInfo', 'GetCurrentCombatTextEventInfo',
            'GetDeathRecapLink', 'SetPortraitToTexture'
        }) do
            assert(rawget(_G, name) == nil, name .. ' must not be simulator-published')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_cloak_helm_transitions_are_independent_and_reversible() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local cloak, helm = ShowingCloak(), ShowingHelm()
        ShowCloak(false); ShowHelm(true)
        assert(not ShowingCloak() and ShowingHelm())
        ShowCloak(true); ShowHelm(false)
        assert(ShowingCloak() and not ShowingHelm())
        ShowCloak(cloak); ShowHelm(helm)
        assert(ShowingCloak() == cloak and ShowingHelm() == helm)
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_player_queries_read_host_inputs() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.plain_global_inputs.collapsing_star_cost = 125.5;
        sim.plain_global_inputs.raid_marker_system_enabled = false;
    }
    assert_eq!(
        env.eval::<(f64, bool)>("return GetCollapsingStarCost(), IsRaidMarkerSystemEnabled()")
            .unwrap(),
        (125.5, false)
    );
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .raid_marker_system_enabled = true;
    assert!(
        env.eval::<bool>("return IsRaidMarkerSystemEnabled()")
            .unwrap()
    );
}

#[test]
fn p1200_secret_helpers_use_vm_wrappers_and_calling_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(canaccesssecrets())
        local secret = secretwrap(42)
        assert(not hasanysecretvalues())
        assert(not hasanysecretvalues(nil, 42, 'public', {secret}))
        assert(hasanysecretvalues(nil, secret, 'public'))
        assert(not issecrettable(secret))
        local plain = {secret}
        assert(not issecrettable(plain))
        assert(issecrettable(secretwrap(plain)))
        local f = function() return canaccesssecrets(), hasanysecretvalues(secret) end
        debug.setobjecttaint(f, 'PlainGlobalAddon')
        local access, contains = f()
        assert(not access and contains)
        assert(canaccesssecrets())
    "#,
    )
    .unwrap();
}
