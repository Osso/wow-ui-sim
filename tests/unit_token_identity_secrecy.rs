#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn create_identity_lookup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create identity lookup environment");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.party_members.len() >= 2, "fixture needs two members");
        state.party_members.truncate(2);
        state.party_group_active = true;
    }
    env.exec(
        r#"
        IdentityLookupFirstGUID = UnitGUID('party1')
        IdentityLookupSecondGUID = UnitGUID('party2')
        assert(IdentityLookupFirstGUID ~= nil)
        assert(IdentityLookupSecondGUID ~= nil)
        assert(IdentityLookupFirstGUID ~= IdentityLookupSecondGUID)
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == 'party1')
        assert(UnitTokenFromGUID(IdentityLookupSecondGUID) == 'party2')
        "#,
    )
    .expect("resolve two actual party identities before classification");
    env
}

fn classify_first_identity_secret(env: &WowLuaEnv) -> String {
    let guid: String = env
        .eval("return IdentityLookupFirstGUID")
        .expect("read public fixture GUID before classifying its identity");
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(guid.clone());
    guid
}

#[test]
fn identity_lookup_secret_party_suppression_recovers_after_host_clear() {
    let env = create_identity_lookup_env();
    let guid = classify_first_identity_secret(&env);
    env.exec(
        r#"
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)
        assert(select('#', UnitTokenFromGUID(IdentityLookupFirstGUID)) == 1)
        assert(not issecretvalue(UnitTokenFromGUID(IdentityLookupFirstGUID)))
        assert(UnitTokenFromGUID(IdentityLookupSecondGUID) == 'party2')
        "#,
    )
    .expect("secret first identity is suppressed, public second identity remains");
    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    env.exec("assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == 'party1')")
        .expect("host clearing restores the existing party mapping");
}

#[test]
fn identity_lookup_suppression_preserves_secure_and_addon_callers() {
    let env = create_identity_lookup_env();
    classify_first_identity_secret(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)
        assert(debug.getstacktaint() == nil)
        local function addonCaller()
            assert(debug.getstacktaint() == 'IdentityLookupFixtureAddon')
            assert(not issecretvalue(IdentityLookupFirstGUID))
            assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)
            assert(UnitTokenFromGUID(IdentityLookupSecondGUID) == 'party2')
            assert(debug.getstacktaint() == 'IdentityLookupFixtureAddon')
        end
        debug.setobjecttaint(addonCaller, 'IdentityLookupFixtureAddon')
        addonCaller()
        "#,
    )
    .expect("host identity classification differs from argument secrecy and taint");
}

#[test]
fn identity_lookup_classification_does_not_create_missing_units() {
    let env = create_identity_lookup_env();
    classify_first_identity_secret(&env);
    env.state().borrow_mut().party_group_active = false;
    env.exec(
        r#"
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)
        assert(UnitTokenFromGUID(IdentityLookupSecondGUID) == nil)
        assert(UnitTokenFromGUID('Player-Missing-IdentityLookupFixture') == nil)
        assert(select('#', UnitTokenFromGUID('Player-Missing-IdentityLookupFixture')) == 1)
        "#,
    )
    .expect("classification does not fabricate inactive or unknown unit mappings");
    env.state().borrow_mut().party_group_active = true;
    env.exec(
        r#"
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)
        assert(UnitTokenFromGUID(IdentityLookupSecondGUID) == 'party2')
        "#,
    )
    .expect("roster activation does not erase host classification");
}

#[test]
fn identity_lookup_player_token_remains_outside_party_suppression() {
    let env = create_identity_lookup_env();
    let player_guid: String = env.eval("return UnitGUID('player')").expect("player GUID");
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(player_guid);
    env.exec(
        r#"
        assert(UnitTokenFromGUID(UnitGUID('player')) == 'player')
        assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == 'party1')
        "#,
    )
    .expect("inferred bounded policy leaves player resolution unchanged");
}

#[test]
fn identity_lookup_host_classification_is_environment_local() {
    let first = create_identity_lookup_env();
    let second = create_identity_lookup_env();
    classify_first_identity_secret(&first);
    first
        .exec("assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == nil)")
        .expect("first environment suppresses classified party identity");
    second
        .exec("assert(UnitTokenFromGUID(IdentityLookupFirstGUID) == 'party1')")
        .expect("second environment retains empty-default public classification");
}
