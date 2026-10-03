#![cfg(feature = "retail-12-0-5")]
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn create_identity_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create instanced identity environment");
    env.exec(
        r#"
        A_Admin.SetPlayerName('Instance Local Player')
        A_Admin.SetTarget('Instance Visitor', 63, 1, true)
        assert(UnitExists('target'))
        SavedVisitorGUID = UnitGUID('target')
        assert(SavedVisitorGUID ~= nil)
        "#,
    )
    .expect("create concrete player and target identities");
    env
}

fn assert_identity_outputs(env: &WowLuaEnv, token: &str, secret: bool) {
    let check = format!(
        r#"
        local token = '{token}'
        local expected = {secret}
        assert(UnitExists(token))
        local getters = {{UnitName, UnitNameUnmodified, GetUnitName, UnitPVPName, UnitGUID}}
        for _, getter in ipairs(getters) do
            local value = getter(token)
            assert(value ~= nil, 'fixture getter must resolve a value')
            assert(select('#', getter(token)) == 1, 'existing single-return arity')
            assert(issecretvalue(value) == expected, 'identity output classification')
        end
        local name, realm = UnitFullName(token)
        assert(select('#', UnitFullName(token)) == 2)
        assert(issecretvalue(name) == expected, 'full-name classification')
        assert(issecretvalue(realm) == expected, 'realm classification')
        "#,
    );
    env.exec(&check).expect("observe real name/GUID getters");
}

#[test]
fn instanced_identity_friendly_and_attackable_visitors_share_current_policy() {
    for is_enemy in [false, true] {
        let env = create_identity_env();
        {
            let mut state = env.state().borrow_mut();
            let target = state.current_target.as_mut().unwrap();
            target.is_enemy = is_enemy;
            target.reaction = if is_enemy { 1 } else { 5 };
        }
        env.exec(&format!(
            "assert(UnitCanAttack('player', 'target') == {is_enemy})"
        ))
        .expect("fixture exposes actual attackability contrast");
        assert_identity_outputs(&env, "target", false);
        env.state().borrow_mut().instance_identity.on_instanced_map = true;
        assert_identity_outputs(&env, "target", true);
        assert_identity_outputs(&env, "player", false);
        env.exec("assert(not issecretvalue('Instance Visitor'))")
            .expect("secret getter output must not poison equal public literals");
    }
}

#[test]
fn instanced_identity_map_exit_changes_new_outputs_not_retained_values() {
    let env = create_identity_env();
    env.state().borrow_mut().instance_identity.on_instanced_map = true;
    assert_identity_outputs(&env, "target", true);
    env.exec("RetainedVisitorName = UnitName('target'); RetainedVisitorGUID = UnitGUID('target')")
        .expect("retain genuinely restricted outputs");
    env.state().borrow_mut().instance_identity.on_instanced_map = false;
    assert_identity_outputs(&env, "target", false);
    env.exec(
        r#"
        assert(UnitName('target') == 'Instance Visitor')
        assert(UnitGUID('target') == SavedVisitorGUID)
        assert(issecretvalue(RetainedVisitorName))
        assert(issecretvalue(RetainedVisitorGUID))
        "#,
    )
    .expect("new public results preserve payload without declassifying retained outputs");
}

#[test]
fn instanced_identity_group_exemption_follows_guid_alias_and_mind_control() {
    let env = create_identity_env();
    let member_guid: String = {
        let mut state = env.state().borrow_mut();
        assert!(
            !state.party_members.is_empty(),
            "fixture needs a real member"
        );
        state.party_members.truncate(1);
        state.party_group_active = true;
        state.party_members[0].name = "Instance Ally".to_string();
        // Existing party GUID provider; capture before enabling instance restriction.
        state.current_target.as_mut().unwrap().name = "Instance Ally".to_string();
        drop(state);
        env.eval("return UnitGUID('party1')")
            .expect("capture actual member GUID before querying restricted outputs")
    };
    {
        let mut state = env.state().borrow_mut();
        state.instance_identity.on_instanced_map = true;
        state.current_target.as_mut().unwrap().guid = member_guid.clone();
        state.current_target.as_mut().unwrap().is_enemy = false;
        state.current_target.as_mut().unwrap().reaction = 5;
        state.current_focus = state.current_target.clone();
    }
    assert_identity_outputs(&env, "party1", false);
    assert_identity_outputs(&env, "target", false);
    assert_identity_outputs(&env, "focus", false);
    env.exec("assert(not UnitCanAttack('player', 'target'))")
        .expect("ally initially nonattackable");
    {
        let mut state = env.state().borrow_mut();
        state
            .instance_identity
            .mind_controlled_guids
            .insert(member_guid.clone());
        state.current_target.as_mut().unwrap().is_enemy = true;
    }
    env.exec("assert(UnitCanAttack('player', 'target'))")
        .expect("host control fixture changes attackability, not identity membership");
    assert_identity_outputs(&env, "party1", false);
    assert_identity_outputs(&env, "target", false);
    assert_identity_outputs(&env, "focus", false);
    env.state().borrow_mut().party_group_active = false;
    assert_identity_outputs(&env, "target", true);
}

#[test]
fn instanced_identity_control_alone_does_not_exempt_a_visitor() {
    let env = create_identity_env();
    let guid: String = env.eval("return SavedVisitorGUID").expect("visitor GUID");
    {
        let mut state = env.state().borrow_mut();
        state.instance_identity.on_instanced_map = true;
        state.instance_identity.mind_controlled_guids.insert(guid);
        state.current_target.as_mut().unwrap().is_enemy = false;
        state.current_target.as_mut().unwrap().reaction = 5;
    }
    assert_identity_outputs(&env, "target", true);
}

#[test]
fn instanced_identity_explicit_secret_override_is_independent_of_map_exemption() {
    let env = create_identity_env();
    let guid: String = env.eval("return SavedVisitorGUID").expect("visitor GUID");
    env.state().borrow_mut().instance_identity.on_instanced_map = true;
    env.state()
        .borrow_mut()
        .instance_identity
        .player_owned_guids
        .insert(guid.clone());
    assert_identity_outputs(&env, "target", false);
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(guid.clone());
    assert_identity_outputs(&env, "target", true);
    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    assert_identity_outputs(&env, "target", false);
}

#[test]
fn instanced_identity_tainted_caller_observes_context_without_losing_taint() {
    let env = create_identity_env();
    env.state().borrow_mut().instance_identity.on_instanced_map = true;
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        local function addonCaller()
            assert(debug.getstacktaint() == 'InstanceIdentityAddon')
            assert(issecretvalue(UnitName('target')))
            assert(issecretvalue(UnitGUID('target')))
            assert(not issecretvalue(UnitName('player')))
            assert(debug.getstacktaint() == 'InstanceIdentityAddon')
        end
        debug.setobjecttaint(addonCaller, 'InstanceIdentityAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("getter producers preserve secure and tainted caller contexts");
}
