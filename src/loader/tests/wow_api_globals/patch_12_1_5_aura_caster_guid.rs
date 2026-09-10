use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let template = sim.player.buffs[0].clone();
        sim.player.buffs = [
            (701, "player"),
            (702, "party1"),
            (703, "target"),
            (704, "missing"),
            (705, ""),
            (706, "pet"),
        ]
        .into_iter()
        .map(|(id, source)| {
            let mut aura = template.clone();
            aura.aura_instance_id = id;
            aura.source_unit = source.into();
            aura
        })
        .collect();
        sim.party_group_active = true;
        sim.party_members[0].buffs = vec![sim.player.buffs[0].clone()];
        sim.party_members[0].buffs[0].source_unit = "party1".into();
        sim.party_members[0].debuffs.clear();
        sim.current_target = None;
    }
    env
}

#[cfg(feature = "client-ptr")]
#[test]
fn aura_caster_guid_resolves_live_sources_and_isolates_units() {
    let env = seeded_env();
    env.exec(
        r#"
        local query = C_UnitAuras.GetAuraCasterGUID
        assert(query('player', 701) == UnitGUID('player'), 'player caster')
        assert(query('player', 702) == UnitGUID('party1'), 'party caster')
        assert(query('party1', 701) == UnitGUID('party1'), 'same ID belongs to queried unit')
        assert(query('party1', 702) == nil, 'wrong unit')
        assert(query('player', 99999) == nil)
        for _, id in ipairs({703, 704, 705, 706}) do
            assert(query('player', id) == nil, 'unresolved caster')
        end
        assert(query('missing', 701) == nil)
        assert(query('target', 1) == nil, 'missing owner must not expose target fixture')
        assert(select('#', query('player', 99999)) == 1)
        assert(select('#', query('player', 701)) == 1)
        A_Admin.SetTarget('Caster', 60, 1, false)
    "#,
    )
    .unwrap();
    for guid in ["Player-0000-AAA00001", "Player-0000-AAA00002"] {
        env.state()
            .borrow_mut()
            .current_target
            .as_mut()
            .unwrap()
            .guid = guid.into();
        assert_eq!(
            env.eval::<String>("return C_UnitAuras.GetAuraCasterGUID('player', 703)")
                .unwrap(),
            guid
        );
    }
    env.state().borrow_mut().current_target = None;
    env.state().borrow_mut().party_group_active = false;
    env.exec("assert(C_UnitAuras.GetAuraCasterGUID('player', 703) == nil); assert(C_UnitAuras.GetAuraCasterGUID('player', 702) == nil)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn aura_caster_guid_validates_required_arguments() {
    let env = seeded_env();
    env.exec(
        r#"
        local query = C_UnitAuras.GetAuraCasterGUID
        assert(not pcall(query))
        assert(not pcall(query, nil, 701))
        assert(not pcall(query, 'player'))
        assert(not pcall(query, 'player', nil))
        assert(not pcall(query, {}, 701))
        assert(not pcall(query, 'player', {}))
        assert(query('player', 701) == UnitGUID('player'))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn aura_caster_guid_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(rawget(C_UnitAuras, 'GetAuraCasterGUID') == nil)
            assert(C_UnitAuras.GetAuraCasterGUID == nil)
            assert(type(C_UnitAuras.GetAuraDataByAuraInstanceID) == 'function')
        "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
