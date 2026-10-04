#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn guid_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("model GUID environment");
    env.exec(
        r#"
        GuidScene = CreateFrame('ModelScene', 'GuidScene')
        GuidActor = GuidScene:CreateActor('GuidActor')
        ReadActorGuid = GuidActor.GetModelUnitGUID
        "#,
    )
    .expect("create genuine actor and retain native getter");
    env
}

#[test]
fn p1207_model_guid_default_is_one_public_empty_string() {
    let env = guid_environment();
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 1)
            local guid = ...
            assert(guid == '' and not issecretvalue(guid))
        end
        check(GuidActor:GetModelUnitGUID())
        "#,
    )
    .expect("INFERRED unbound default honors nonnil WOWGUID return");
}

#[test]
fn p1207_model_guid_reads_live_host_identity_not_replaceable_lua() {
    let env = guid_environment();
    env.exec(
        r#"
        A_Admin.SetTarget('GUID target', 63, 1, true)
        assert(GuidActor:SetModelByUnit('target') == true)
        UnitGUID = function() error('replaceable Lua must not be queried') end
        GuidActor.GetObjectType = function() error('no Lua type protocol') end
        "#,
    )
    .expect("bind a real host target");
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .expect("host target")
        .guid = "Creature-Host-First".to_owned();
    assert_eq!(
        env.eval::<String>("return ReadActorGuid(GuidActor)").unwrap(),
        "Creature-Host-First"
    );
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .guid = "Creature-Host-Second".to_owned();
    assert_eq!(
        env.eval::<String>("return ReadActorGuid(GuidActor)").unwrap(),
        "Creature-Host-Second"
    );
    env.state().borrow_mut().current_target = None;
    assert_eq!(
        env.eval::<String>("return ReadActorGuid(GuidActor)").unwrap(),
        ""
    );
}

#[test]
fn p1207_model_guid_binding_changes_are_local_to_actor_and_environment() {
    let first = guid_environment();
    let second = guid_environment();
    first.exec(
        r#"
        A_Admin.SetTarget('First target', 63, 1, true)
        A_Admin.SetFocus('First focus', 63, 1, true)
        assert(GuidActor:SetModelByUnit('target') == true)
        OtherGuidActor = GuidScene:CreateActor('OtherGuidActor')
        assert(OtherGuidActor:GetModelUnitGUID() == '')
        "#,
    )
    .expect("configure one actor only");
    {
        let mut sim = first.state().borrow_mut();
        sim.current_target.as_mut().unwrap().guid = "Creature-Host-Target".to_owned();
        sim.current_focus.as_mut().unwrap().guid = "Creature-Host-Focus".to_owned();
    }
    assert_eq!(
        first.eval::<String>("return GuidActor:GetModelUnitGUID()").unwrap(),
        "Creature-Host-Target"
    );
    first.exec("assert(GuidActor:SetModelByUnit('focus') == true)").unwrap();
    assert_eq!(
        first.eval::<String>("return GuidActor:GetModelUnitGUID()").unwrap(),
        "Creature-Host-Focus"
    );
    assert_eq!(
        second.eval::<String>("return GuidActor:GetModelUnitGUID()").unwrap(),
        ""
    );
}

#[test]
fn p1207_model_guid_removed_conditional_secret_is_public_for_both_callers() {
    let env = guid_environment();
    env.exec(
        r#"
        A_Admin.SetTarget('GUID visitor', 63, 1, true)
        assert(GuidActor:SetModelByUnit('target') == true)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().current_target.as_mut().unwrap().guid =
        "Creature-Host-Visitor".to_owned();
    env.state().borrow_mut().instance_identity.on_instanced_map = true;
    env.exec(
        r#"
        assert(issecretvalue(UnitGUID('target')))
        local function check()
            local before = debug.getstacktaint()
            local value = GuidActor:GetModelUnitGUID()
            assert(value == 'Creature-Host-Visitor')
            assert(not issecretvalue(value))
            assert(debug.getstacktaint() == before)
        end
        check()
        debug.setobjecttaint(check, 'GUIDAddon')
        check()
        "#,
    )
    .expect("source removes ConditionalSecret; no secret result even when identity is restricted");
}

#[test]
fn p1207_model_guid_requires_native_actor_not_forgeable_type_protocol() {
    let env = guid_environment();
    env.exec(
        r#"
        local fake = {GetObjectType = function() return 'ModelSceneActor' end}
        assert(not pcall(ReadActorGuid, fake))
        fake[0] = GuidActor[0]
        assert(not pcall(ReadActorGuid, fake))
        assert(not pcall(ReadActorGuid, CreateFrame('Frame')))
        assert(not pcall(ReadActorGuid, GuidScene))
        GuidActor.GetObjectType = function() return 'Frame' end
        assert(ReadActorGuid(GuidActor) == '')
        "#,
    )
    .expect("native backing and host actor metadata establish identity");
}

#[test]
fn p1207_model_guid_rejects_secret_receiver_and_every_extra_for_all_callers() {
    let env = guid_environment();
    env.exec(
        r#"
        local hiddenActor = secretwrap(GuidActor)
        local hiddenExtra = secretwrap(19)
        local function check()
            local before = debug.getstacktaint()
            for _, arguments in ipairs({
                {hiddenActor},
                {GuidActor, hiddenExtra},
                {GuidActor, 'ignored', hiddenExtra},
                {{}, 'ignored', hiddenExtra},
            }) do
                local ok, message = pcall(ReadActorGuid, unpack(arguments))
                assert(not ok)
                assert(string.find(message, 'does not accept secret arguments', 1, true))
            end
            assert(debug.getstacktaint() == before)
            assert(ReadActorGuid(GuidActor, 'public extra') == '')
        end
        check()
        debug.setobjecttaint(check, 'GUIDAddon')
        check()
        "#,
    )
    .expect("INFERRED NotAllowed checks all supplied values before receiver validation");
}
