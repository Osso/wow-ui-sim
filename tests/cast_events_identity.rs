#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::{CastSuccess, WowLuaEnv};

fn observe_casts() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("cast observer environment");
    env.exec(
        r#"
        CastEvents = {}
        local function record(event, ...)
            table.insert(CastEvents, {event = event, count = select('#', ...), ...})
        end
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('UNIT_SPELLCAST_SUCCEEDED')
        frame:SetScript('OnEvent', function(self, event, ...) record(event, ...) end)
        CallbackCasts = 0
        UnitCallbackCasts = 0
        RegisterEventCallback('UNIT_SPELLCAST_SUCCEEDED', function()
            CallbackCasts = CallbackCasts + 1
        end)
        RegisterUnitEventCallback('UNIT_SPELLCAST_SUCCEEDED', function()
            UnitCallbackCasts = UnitCallbackCasts + 1
        end, 'party1')
        "#,
    )
    .expect("register real frame and global callbacks");
    env
}

fn success(
    unit: &str,
    caster_is_player: bool,
    instant: bool,
    spell_is_secret: bool,
) -> CastSuccess {
    CastSuccess {
        unit: unit.to_owned(),
        caster_is_player,
        cast_guid: "Cast-Host-193-17".to_owned(),
        spell_id: 20473,
        cast_bar_id: Some(17),
        instant,
        spell_is_secret,
    }
}

#[test]
fn cast_success_secret_instant_nonplayer_reaches_no_listener() {
    let env = observe_casts();
    env.fire_cast_success(&success("party1", false, true, true))
        .expect("suppressed completion is not an error");
    env.exec(
        r#"
        assert(#CastEvents == 0)
        assert(CallbackCasts == 0)
        assert(UnitCallbackCasts == 0)
        "#,
    )
    .expect("suppress before all listener families");
}

#[test]
fn cast_success_all_three_policy_axes_and_player_alias() {
    for caster_is_player in [false, true] {
        for instant in [false, true] {
            for spell_is_secret in [false, true] {
                let env = observe_casts();
                let unit = if caster_is_player { "player" } else { "party1" };
                env.fire_cast_success(&success(unit, caster_is_player, instant, spell_is_secret))
                    .expect("dispatch explicit host completion");
                let count: f64 = env.eval("return #CastEvents").expect("frame count");
                let callbacks: f64 = env.eval("return CallbackCasts").expect("callback count");
                let suppressed = !caster_is_player && instant && spell_is_secret;
                let expected = if suppressed { 0.0 } else { 1.0 };
                assert_eq!(
                    count, expected,
                    "player={caster_is_player} instant={instant} secret={spell_is_secret}"
                );
                assert_eq!(callbacks, expected);
            }
        }
    }
    let env = observe_casts();
    env.fire_cast_success(&success("target", true, true, true))
        .expect("host identifies target alias as player");
    assert_eq!(env.eval::<f64>("return #CastEvents").unwrap(), 1.0);
}

#[test]
fn cast_success_payload_and_host_policy_transition_are_observable() {
    let env = observe_casts();
    let mut input = success("party1", false, true, true);
    env.fire_cast_success(&input).unwrap();
    input.spell_is_secret = false;
    env.fire_cast_success(&input).unwrap();
    env.exec(
        r#"
        assert(#CastEvents == 1 and CallbackCasts == 1 and UnitCallbackCasts == 1)
        local event = CastEvents[1]
        assert(event.event == 'UNIT_SPELLCAST_SUCCEEDED' and event.count == 4)
        assert(event[1] == 'party1' and event[2] == 'Cast-Host-193-17')
        assert(event[3] == 20473 and event[4] == 17)
        "#,
    )
    .expect("actual dispatch retains input payload");
    input.instant = false;
    input.spell_is_secret = true;
    input.cast_bar_id = None;
    env.fire_cast_success(&input).unwrap();
    env.exec(
        r#"
        assert(#CastEvents == 2)
        assert(CastEvents[2].count == 4 and CastEvents[2][4] == nil)
        "#,
    )
    .expect("noninstant exception and explicit nil fourth payload");
}

#[test]
fn cast_success_pet_is_not_player_exception_and_inputs_are_isolated() {
    let first = observe_casts();
    let second = observe_casts();
    first
        .fire_cast_success(&success("pet", false, true, true))
        .unwrap();
    second
        .fire_cast_success(&success("pet", false, true, false))
        .unwrap();
    assert_eq!(first.eval::<f64>("return #CastEvents").unwrap(), 0.0);
    assert_eq!(second.eval::<f64>("return #CastEvents").unwrap(), 1.0);
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
mod model_identity {
    use super::*;

    fn fixture() -> WowLuaEnv {
        let env = WowLuaEnv::new().expect("model identity environment");
        {
            let mut sim = env.state().borrow_mut();
            assert!(sim.party_members.len() >= 2);
            sim.party_group_active = true;
        }
        env.exec(
            r#"
            BindingModel = CreateFrame('Model', 'BindingModel')
            local scene = CreateFrame('ModelScene', 'BindingScene')
            BindingActor = scene:CreateActor('BindingActor')
            FirstIdentity = UnitGUID('party1')
            SecondIdentity = UnitGUID('party2')
            assert(FirstIdentity ~= nil and SecondIdentity ~= nil, 'fixture resolves both roster GUIDs')
            assert(FirstIdentity ~= SecondIdentity, 'fixture roster GUIDs are distinct')
            "#,
        )
        .expect("create real model, actor and roster identities");
        env
    }

    fn stored_unit(env: &WowLuaEnv, name: &str) -> Option<String> {
        let sim = env.state().borrow();
        let id = sim.widgets.get_id_by_name(name).expect("named widget");
        sim.widgets
            .get(id)
            .unwrap()
            .model_state()
            .player_model_state
            .last_unit
            .clone()
    }

    fn classify_first(env: &WowLuaEnv) -> String {
        let guid: String = env.eval("return FirstIdentity").unwrap();
        env.state()
            .borrow_mut()
            .identity_secret_guids
            .insert(guid.clone());
        guid
    }

    fn assert_denial_and_binding_recovery(name: &str, method: &str) {
        let env = fixture();
        {
            // Seed prior host binding so RED isolates denial, not public success.
            let mut sim = env.state().borrow_mut();
            let id = sim.widgets.get_id_by_name(name).unwrap();
            sim.widgets
                .get_mut_visual(id)
                .unwrap()
                .model_state_mut()
                .player_model_state
                .last_unit = Some("party2".to_owned());
        }
        let guid = classify_first(&env);
        env.exec(&format!(
            r#"
            assert(not issecretvalue('party1'), 'unit token literal stays public')
            local function call() return {name}:{method}('party1') end
            local function check(...)
                assert(select('#', ...) == 2, '{name}:{method} pcall returns status and exactly one result')
                local ok, result = ...
                assert(ok == true and result == nil, '{name}:{method} denies secret identity without throwing')
                assert(not issecretvalue(result), '{name}:{method} denial nil stays public')
            end
            check(pcall(call))
            "#
        ))
        .expect("secret identity is denied without throwing, with exactly one nil");
        assert_eq!(stored_unit(&env, name).as_deref(), Some("party2"));
        env.state().borrow_mut().identity_secret_guids.remove(&guid);
        env.exec(&format!(
            "assert({name}:{method}('party1') == true, '{name}:{method} assigns after host secrecy clears')"
        ))
            .expect("host classification clearing restores assignment");
        assert_eq!(stored_unit(&env, name).as_deref(), Some("party1"));
    }

    #[test]
    fn model_actor_denial_returns_one_nil_and_preserves_prior_binding() {
        assert_denial_and_binding_recovery("BindingActor", "SetModelByUnit");
    }

    #[test]
    fn model_set_unit_denial_returns_one_nil_and_preserves_prior_binding() {
        assert_denial_and_binding_recovery("BindingModel", "SetUnit");
    }

    #[test]
    fn model_unit_binding_classification_is_identity_based_not_token_spelling() {
        let env = fixture();
        env.exec("TargetUnit('party1'); assert(UnitGUID('target') == FirstIdentity, 'target aliases first roster GUID')")
            .expect("actual target aliases first roster identity");
        classify_first(&env);
        env.exec(
            r#"
            assert(BindingActor:SetModelByUnit('target') == nil, 'actor denies alias of secret GUID')
            assert(BindingModel:SetUnit('target') == nil, 'model denies alias of secret GUID')
            assert(BindingActor:SetModelByUnit('party2') == true, 'actor assigns public roster identity')
            assert(BindingModel:SetUnit('party2') == true, 'model assigns public roster identity')
            "#,
        )
        .expect("denial follows existing GUID mapping, not party token spelling");
        assert_eq!(stored_unit(&env, "BindingActor").as_deref(), Some("party2"));
        assert_eq!(stored_unit(&env, "BindingModel").as_deref(), Some("party2"));
    }

    #[test]
    fn model_unit_binding_denial_preserves_real_addon_taint_and_env_isolation() {
        let restricted = fixture();
        let public = fixture();
        classify_first(&restricted);
        restricted
            .exec(
                r#"
            assert(debug.getstacktaint() == nil, 'secure caller starts untainted')
            local function addon()
                assert(debug.getstacktaint() == 'ModelIdentityFixture', 'actual addon caller starts tainted')
                assert(select('#', BindingActor:SetModelByUnit('party1')) == 1, 'tainted actor denial returns exactly one value')
                assert(BindingActor:SetModelByUnit('party1') == nil, 'tainted actor caller receives denial nil')
                assert(select('#', BindingModel:SetUnit('party1')) == 1, 'tainted model denial returns exactly one value')
                assert(BindingModel:SetUnit('party1') == nil, 'tainted model caller receives denial nil')
                assert(debug.getstacktaint() == 'ModelIdentityFixture', 'model calls preserve addon taint')
            end
            debug.setobjecttaint(addon, 'ModelIdentityFixture')
            addon()
            assert(debug.getstacktaint() == nil, 'return from addon preserves secure caller taint')
            "#,
            )
            .expect("ordinary token with secret identity returns nil even for actual addon caller");
        public.exec("assert(BindingActor:SetModelByUnit('party1') == true, 'independent environment actor assigns public identity'); assert(BindingModel:SetUnit('party1') == true, 'independent environment model assigns public identity')")
            .expect("second environment has no inherited classification");
        assert_eq!(stored_unit(&restricted, "BindingActor"), None);
        assert_eq!(stored_unit(&restricted, "BindingModel"), None);
        assert_eq!(
            stored_unit(&public, "BindingActor").as_deref(),
            Some("party1")
        );
        assert_eq!(
            stored_unit(&public, "BindingModel").as_deref(),
            Some("party1")
        );
    }

    #[test]
    fn model_unit_binding_uses_instanced_identity_context() {
        for (name, method) in [
            ("BindingActor", "SetModelByUnit"),
            ("BindingModel", "SetUnit"),
        ] {
            let env = fixture();
            env.exec(&format!(
                r#"
                assert({name}:{method}('party2') == true, '{name}:{method} establishes prior binding')
                A_Admin.SetTarget('Model Instance Visitor', 63, 1, true)
                assert(UnitExists('target'), 'visitor has an existing target identity')
                "#
            ))
            .expect("create public visitor and prior binding");
            let guid: String = env.eval("return UnitGUID('target')").unwrap();
            env.state().borrow_mut().instance_identity.on_instanced_map = true;
            env.exec(&format!(
                r#"
                assert(issecretvalue(UnitName('target')), 'instance visitor name is secret')
                assert(issecretvalue(UnitGUID('target')), 'instance visitor GUID is secret')
                local function check(...)
                    assert(select('#', ...) == 2, '{name}:{method} denial pcall has exactly one result')
                    local ok, result = ...
                    assert(ok == true and result == nil, '{name}:{method} denies instance visitor without throwing')
                    assert(not issecretvalue(result), '{name}:{method} denial nil is public')
                end
                check(pcall(function() return {name}:{method}('target') end))
                "#
            ))
            .expect("getter secrecy and model denial share instance policy");
            assert_eq!(stored_unit(&env, name).as_deref(), Some("party2"));
            env.state()
                .borrow_mut()
                .instance_identity
                .player_owned_guids
                .insert(guid.clone());
            env.exec(&format!(
                "assert(not issecretvalue(UnitGUID('target')), 'owned visitor GUID is public'); assert({name}:{method}('target') == true, '{name}:{method} assigns owned visitor')"
            ))
            .expect("host ownership exempts identity for getters and model binding");
            assert_eq!(stored_unit(&env, name).as_deref(), Some("target"));
            {
                let mut sim = env.state().borrow_mut();
                sim.instance_identity.player_owned_guids.remove(&guid);
                sim.instance_identity.on_instanced_map = false;
            }
            env.exec(&format!(
                "assert(not issecretvalue(UnitGUID('target')), 'map exit makes visitor GUID public'); assert({name}:{method}('target') == true, '{name}:{method} assigns visitor after map exit')"
            ))
            .expect("map exit restores public identity assignment");
            assert_eq!(stored_unit(&env, name).as_deref(), Some("target"));
        }
    }

    #[test]
    fn model_unit_binding_missing_identity_does_not_fabricate_success() {
        let env = fixture();
        env.exec(
            r#"
            assert(BindingActor:SetModelByUnit('party2') == true, 'actor establishes prior binding')
            assert(BindingModel:SetUnit('party2') == true, 'model establishes prior binding')
            "#,
        )
        .unwrap();
        env.state().borrow_mut().party_group_active = false;
        env.exec(
            r#"
            assert(BindingActor:SetModelByUnit('party1') == false, 'actor rejects missing identity without success')
            assert(BindingModel:SetUnit('party1') == false, 'model rejects missing identity without success')
            "#,
        )
        .expect("INFERRED failure policy requires an existing identity");
        assert_eq!(stored_unit(&env, "BindingActor").as_deref(), Some("party2"));
        assert_eq!(stored_unit(&env, "BindingModel").as_deref(), Some("party2"));
    }
}
