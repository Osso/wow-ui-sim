//! Shared inferred recast policy on Forever; native formula/security remain unproven.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::aura_duration::SpellAuraDuration;
use wow_ui_sim::lua_api::WowLuaEnv;

fn seed_durations(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let template = state.player.buffs[0].clone();
    let expiration = state.start_time.elapsed().as_secs_f64() + 600.0;
    state.player.buffs = [(101, 99001), (102, 99002)]
        .into_iter()
        .map(|(instance_id, spell_id)| {
            let mut aura = template.clone();
            aura.aura_instance_id = instance_id;
            aura.spell_id = spell_id;
            aura.duration = 99.0;
            aura.expiration_time = expiration;
            aura
        })
        .collect();
    for (spell_id, cap) in [(99001, 6.0), (99002, 3.0)] {
        state.spell_aura_durations.insert(
            spell_id,
            SpellAuraDuration {
                base_duration_seconds: 20.0,
                max_carryover_seconds: cap,
            },
        );
    }
    state
        .spell_id_aliases
        .insert("carryover override".into(), 99002);
}

#[test]
fn forever_aura_carryover_uses_distinct_caps_and_optional_spell_override() {
    let env = WowLuaEnv::new().unwrap();
    seed_durations(&env);
    env.exec(
        r#"
        local query = rawget(C_UnitAuras, 'GetRefreshCarryOverDuration')
        assert(type(query) == 'function')
        assert(query('player', 101) == 6)
        assert(query('player', 102) == 3)
        assert(query('player', 101, nil) == 6)
        assert(query('player', 101, 99002) == 3)
        assert(query('player', 101, 'CARRYOVER Override') == 3)
        assert(query('player', 101, 'unknown override') == nil)
        assert(select('#', query('player', 101)) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_carryover_clamps_remaining_clock_time_at_zero() {
    let env = WowLuaEnv::new().unwrap();
    seed_durations(&env);
    let expiration = {
        let mut state = env.state().borrow_mut();
        let expiration = state.start_time.elapsed().as_secs_f64() + 60.0;
        state.player.buffs[0].expiration_time = expiration;
        state
            .spell_aura_durations
            .get_mut(&99001)
            .unwrap()
            .max_carryover_seconds = 100.0;
        expiration
    };
    let before = env.state().borrow().start_time.elapsed().as_secs_f64();
    let actual: f64 = env
        .eval("return C_UnitAuras.GetRefreshCarryOverDuration('player', 101)")
        .unwrap();
    let after = env.state().borrow().start_time.elapsed().as_secs_f64();
    assert!(actual >= (expiration - after).max(0.0) - 0.001);
    assert!(actual <= (expiration - before).max(0.0) + 0.001);
    assert!(actual > 0.0 && actual < 100.0);
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs[0].expiration_time = state.start_time.elapsed().as_secs_f64() - 1.0;
    }
    assert_eq!(
        env.eval::<f64>("return C_UnitAuras.GetRefreshCarryOverDuration('player', 101)")
            .unwrap(),
        0.0
    );
}

#[test]
fn forever_aura_carryover_keeps_unknown_and_permanent_auras_nil_without_mutation() {
    let env = WowLuaEnv::new().unwrap();
    seed_durations(&env);
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs[0].duration = 0.0;
        state.spell_aura_durations.remove(&99002);
    }
    let (auras, metadata, aliases, event_count) = {
        let state = env.state().borrow();
        (
            format!("{:?}", state.player.buffs),
            state.spell_aura_durations.clone(),
            state.spell_id_aliases.clone(),
            state.events.pending().len(),
        )
    };
    env.exec(
        r#"
        local query = C_UnitAuras.GetRefreshCarryOverDuration
        for _, id in ipairs({101, 102, 999}) do
            assert(query('player', id) == nil)
            assert(select('#', query('player', id)) == 1)
        end
        assert(query('missing', 101, 99001) == nil)
        collectgarbage('collect')
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(format!("{:?}", state.player.buffs), auras);
    assert_eq!(state.spell_aura_durations, metadata);
    assert_eq!(state.spell_id_aliases, aliases);
    assert_eq!(state.events.pending().len(), event_count);
}

#[test]
fn forever_aura_carryover_preserves_argument_validation() {
    let env = WowLuaEnv::new().unwrap();
    seed_durations(&env);
    env.exec(
        r#"
        local query = C_UnitAuras.GetRefreshCarryOverDuration
        assert(not pcall(query))
        assert(not pcall(query, 'player'))
        assert(not pcall(query, {}, 101))
        assert(not pcall(query, 'player', {}))
        assert(not pcall(query, 'missing', 101, {}))
        assert(query('player', 101) == 6)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_carryover_selected_original_pandemic_window_uses_actual_caps() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_AuraContainer"));
    seed_durations(&env);
    let probe = r#"
        local util = assert(__secureenv.AuraContainerUtil)
        for _, case in ipairs({{101, 6}, {102, 3}}) do
            local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', case[1])
            data.auraType = __secureenv.AuraContainerAuraDataType.Aura
            local startTime, endTime = util.GetPandemicWindow('player', data)
            assert(endTime == data.expirationTime)
            assert(endTime - startTime == case[2])
        end
    "#;
    env.exec(probe)
        .expect("selected unchanged GetPandemicWindow must consume actual carryover values");
    env.apply_post_load_workarounds();
    env.exec(probe)
        .expect("post-load cleanup must preserve the original numeric consumer");
    env.state().borrow_mut().spell_aura_durations.clear();
    env.exec(
        r#"
        local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 101)
        data.auraType = __secureenv.AuraContainerAuraDataType.Aura
        assert(__secureenv.AuraContainerUtil.GetPandemicWindow('player', data) == nil)
        "#,
    )
    .expect("unknown metadata must leave the original pandemic window unknown");
}
