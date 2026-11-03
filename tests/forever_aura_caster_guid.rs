//! Forever's selected public aura source shares the modeled live caster query.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const TARGET_GUID: &str = "Creature-0-1-2-3-19750-000000ABCD";

fn seed_sources(env: &WowLuaEnv) {
    env.exec("A_Admin.SetTarget('Aura caster', 60, 1, false)")
        .unwrap();
    let mut sim = env.state().borrow_mut();
    sim.current_target.as_mut().unwrap().guid = TARGET_GUID.into();
    let template = sim.player.buffs[0].clone();
    sim.player.buffs = [
        (701, "player"),
        (702, "party1"),
        (703, "target"),
        (704, "missing"),
    ]
    .into_iter()
    .map(|(id, source)| {
        let mut aura = template.clone();
        aura.aura_instance_id = id;
        aura.source_unit = source.into();
        aura.is_helpful = id != 702;
        aura
    })
    .collect();
    let mut party_aura = template;
    party_aura.aura_instance_id = 701;
    party_aura.source_unit = "party1".into();
    sim.party_group_active = true;
    sim.party_members[0].buffs = vec![party_aura];
    sim.party_members[0].debuffs.clear();
}

#[test]
fn forever_aura_caster_guid_resolves_sources_and_isolates_unit_instances() {
    let env = WowLuaEnv::new().unwrap();
    seed_sources(&env);
    env.exec(
        r#"
        local query = rawget(C_UnitAuras, 'GetAuraCasterGUID')
        assert(type(query) == 'function')
        local playerGUID, partyGUID = UnitGUID('player'), UnitGUID('party1')
        assert(type(playerGUID) == 'string' and type(partyGUID) == 'string')
        assert(playerGUID ~= partyGUID)
        assert(query('player', 701) == playerGUID)
        assert(query('player', 702) == partyGUID, 'harmful aura caster')
        assert(query('player', 703) == 'Creature-0-1-2-3-19750-000000ABCD')
        assert(query('party1', 701) == partyGUID, 'instance IDs are scoped to their owner')
        assert(query('party1', 702) == nil, 'do not find another unit aura')
        assert(query('missing', 701) == nil, 'missing aura owner')
        assert(query('player', 704) == nil, 'missing caster')
        assert(query('player', 999) == nil, 'missing aura')
        assert(select('#', query('player', 701)) == 1)
        assert(select('#', query('player', 999)) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_caster_guid_observes_guid_changes_and_caster_or_aura_removal() {
    let env = WowLuaEnv::new().unwrap();
    seed_sources(&env);
    assert_eq!(
        env.eval::<String>("return C_UnitAuras.GetAuraCasterGUID('player', 703)")
            .unwrap(),
        TARGET_GUID
    );
    let replacement = "Creature-0-1-2-3-19750-000000DCBA";
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .guid = replacement.into();
    assert_eq!(
        env.eval::<String>("return C_UnitAuras.GetAuraCasterGUID('player', 703)")
            .unwrap(),
        replacement
    );
    {
        let mut sim = env.state().borrow_mut();
        sim.current_target = None;
        sim.player.buffs.retain(|aura| aura.aura_instance_id != 701);
        sim.party_group_active = false;
    }
    env.exec(
        r#"
        for _, id in ipairs({701, 702, 703}) do
            assert(C_UnitAuras.GetAuraCasterGUID('player', id) == nil)
            assert(select('#', C_UnitAuras.GetAuraCasterGUID('player', id)) == 1)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_caster_guid_validates_required_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = C_UnitAuras.GetAuraCasterGUID
        assert(not pcall(query))
        assert(not pcall(query, nil, 701))
        assert(not pcall(query, 'player'))
        assert(not pcall(query, 'player', nil))
        assert(not pcall(query, {}, 701))
        assert(not pcall(query, 'player', {}))
        "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_caster_guid_selected_original_source_survives_post_load_cleanup() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_AuraContainer"));
    seed_sources(&env);
    let probe = r#"
        local source = assert(__secureenv.AuraContainerPublicAuraSource)
        assert(source:GetAuraCasterGUID('player', 701) == UnitGUID('player'))
        assert(source:GetAuraCasterGUID('player', 702) == UnitGUID('party1'))
        assert(source:GetAuraCasterGUID('player', 703) == 'Creature-0-1-2-3-19750-000000ABCD')
        assert(source:GetAuraCasterGUID('party1', 701) == UnitGUID('party1'))
        assert(source:GetAuraCasterGUID('player', 999) == nil)
        assert(select('#', source:GetAuraCasterGUID('player', 999)) == 1)
    "#;
    env.exec(probe)
        .expect("selected unchanged original public aura source must return actual GUIDs");
    env.apply_post_load_workarounds();
    env.exec(probe)
        .expect("post-load cleanup must preserve the original consumer's modeled query");
}
