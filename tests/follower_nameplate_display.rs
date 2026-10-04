#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

const FOLLOWER_GUID: &str = "Creature-0-0-0-0-211937-000001";
const OTHER_GUID: &str = "Creature-0-0-0-0-211938-000002";

fn npc_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create follower display environment");
    env.exec("TargetUnit('enemy1')")
        .expect("create existing target fixture");
    {
        let mut state = env.state().borrow_mut();
        let target = state.current_target.as_mut().expect("target exists");
        target.guid = FOLLOWER_GUID.into();
        target.name = "Follower Mage".into();
        target.class_index = 8;
        target.is_player = false;
        target.is_enemy = false;
        target.reaction = 5;
        target.health = target.health_max;
    }
    env
}

fn mark_follower(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .npc_follower_guids
        .insert(FOLLOWER_GUID.into());
}

#[test]
fn follower_display_classification_preserves_npc_identity_and_can_be_removed() {
    let env = npc_env();
    env.exec(
        r#"
        assert(not UnitTreatAsPlayerForDisplay('target'))
        assert(not UnitIsPlayer('target'))
        assert(not UnitIsHumanPlayer('target'))
        "#,
    )
    .expect("ordinary NPC begins unclassified");
    mark_follower(&env);
    env.exec(
        r#"
        assert(UnitTreatAsPlayerForDisplay('target'), 'host-marked NPC is display-player')
        assert(select('#', UnitTreatAsPlayerForDisplay('target')) == 1)
        assert(not issecretvalue(UnitTreatAsPlayerForDisplay('target')))
        assert(not UnitIsPlayer('target'))
        assert(not UnitIsHumanPlayer('target'))
        assert(UnitIsFriend('player', 'target'))
        local _, class = UnitClass('target')
        assert(class == 'MAGE')
        "#,
    )
    .expect("follower classification does not rewrite identity/class/friendliness");
    env.state()
        .borrow_mut()
        .npc_follower_guids
        .remove(FOLLOWER_GUID);
    env.exec("assert(not UnitTreatAsPlayerForDisplay('target'))")
        .expect("host removal changes the next observable query");
}

#[test]
fn follower_display_classification_follows_guid_not_target_slot() {
    let env = npc_env();
    mark_follower(&env);
    {
        let mut state = env.state().borrow_mut();
        state.current_focus = state.current_target.clone();
        state.current_target.as_mut().unwrap().guid = OTHER_GUID.into();
    }
    env.exec(
        r#"
        assert(not UnitTreatAsPlayerForDisplay('target'))
        assert(UnitTreatAsPlayerForDisplay('focus'), 'focus still resolves follower GUID')
        "#,
    )
    .expect("new target does not inherit the old target classification");
    env.state().borrow_mut().current_focus = None;
    env.exec("assert(not UnitTreatAsPlayerForDisplay('focus'))")
        .expect("retained classification never creates an absent unit");
}

#[test]
fn follower_display_classification_is_environment_local() {
    let first = npc_env();
    let second = npc_env();
    mark_follower(&first);
    first
        .exec("assert(UnitTreatAsPlayerForDisplay('target'))")
        .expect("first environment has follower input");
    second
        .exec("assert(not UnitTreatAsPlayerForDisplay('target'))")
        .expect("same GUID in second environment remains an ordinary NPC");
}

#[test]
fn follower_display_flag_does_not_imply_friendliness() {
    let env = npc_env();
    mark_follower(&env);
    {
        let mut state = env.state().borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.is_enemy = true;
        target.reaction = 2;
    }
    env.exec(
        r#"
        assert(UnitTreatAsPlayerForDisplay('target'))
        assert(not UnitIsFriend('player', 'target'))
        assert(not UnitIsPlayer('target'))
        "#,
    )
    .expect("classification is independent of hostility; vendor still gates friendly CVars");
}

#[test]
fn follower_display_query_preserves_secure_and_addon_stack_taint() {
    let env = npc_env();
    mark_follower(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        assert(UnitTreatAsPlayerForDisplay('target'))
        assert(debug.getstacktaint() == nil)
        local function addonQuery()
            assert(debug.getstacktaint() == 'FollowerDisplayAddon')
            assert(UnitTreatAsPlayerForDisplay('target'))
            assert(debug.getstacktaint() == 'FollowerDisplayAddon')
        end
        debug.setobjecttaint(addonQuery, 'FollowerDisplayAddon')
        addonQuery()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("public host classification does not elevate callers or taint outputs");
}

#[test]
fn follower_display_authenticates_host_secret_token_and_restores_addon_context() {
    let env = npc_env();
    mark_follower(&env);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let token = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "target");
        lua.state_mut().push(token);
        let published = lua.set_global_val("FollowerSecretToken", token);
        lua.state_mut().pop();
        published.expect("root authentic VM secret token");
    }
    env.exec(
        r#"
        assert(issecretvalue(FollowerSecretToken))
        assert(UnitTreatAsPlayerForDisplay(FollowerSecretToken))
        assert(issecretvalue(FollowerSecretToken))
        local function addonQuery()
            assert(debug.getstacktaint() == 'FollowerSecretAddon')
            assert(not pcall(UnitTreatAsPlayerForDisplay, FollowerSecretToken))
            assert(debug.getstacktaint() == 'FollowerSecretAddon')
            assert(UnitTreatAsPlayerForDisplay('target'))
            assert(issecretvalue(FollowerSecretToken))
        end
        debug.setobjecttaint(addonQuery, 'FollowerSecretAddon')
        addonQuery()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        assert(issecretvalue(FollowerSecretToken))
        assert(UnitTreatAsPlayerForDisplay(FollowerSecretToken))
        "#,
    )
    .expect("AllowedWhenUntainted preserves rooted argument and caller trust");
}

#[test]
fn follower_display_query_does_not_fabricate_pet_vehicle_or_missing_units() {
    let env = npc_env();
    mark_follower(&env);
    env.exec(
        r#"
        assert(not UnitTreatAsPlayerForDisplay(nil))
        assert(not UnitTreatAsPlayerForDisplay('nameplate999'))
        assert(not UnitTreatAsPlayerForDisplay('pet'))
        assert(not UnitTreatAsPlayerForDisplay('vehicle'))
        assert(UnitTreatAsPlayerForDisplay('player'))
        assert(not pcall(UnitTreatAsPlayerForDisplay, {}))
        "#,
    )
    .expect("inferred missing-unit/type policies; remove blanket pet/vehicle defaults");
}
