//! Author-only B39–B44 fixtures. INFERRED host snapshot policies, not native parity.
#![cfg(feature = "retail-12-0-7")]

use rilua::LuaApiMut;
use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::host_chat_inputs::{HostChatArgument, HostChatKind, HostChatMessage};

// Test-support only: inputs-only RED has no inherent host publisher yet.
// Rust selects the inherent producer in GREEN. This trait never ships in src/.
trait InputsOnlyChatPublisher {
    fn publish_next_host_chat(&self) -> wow_ui_sim::Result<bool>;
}
impl InputsOnlyChatPublisher for WowLuaEnv {
    fn publish_next_host_chat(&self) -> wow_ui_sim::Result<bool> {
        Ok(false)
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create B39–B44 environment");
    let _red_entry_signature = <WowLuaEnv as InputsOnlyChatPublisher>::publish_next_host_chat;
    env.exec(
        r#"
        B39Events = {}
        B39Frame = CreateFrame('Frame', 'B39Frame')
        B39Frame:RegisterAllEvents()
        B39Frame:SetScript('OnEvent', function(self, event, ...)
            if string.sub(event, 1, 9) == 'CHAT_MSG_' then
                B39Events[#B39Events + 1] = {event=event, count=select('#', ...), ...}
            end
        end)
        function B39Tainted(probe)
            local function addon()
                assert(debug.getstacktaint() == 'B39Audit')
                probe()
                assert(debug.getstacktaint() == 'B39Audit')
            end
            debug.setobjecttaint(addon, 'B39Audit')
            addon()
        end
        "#,
    )
    .expect("install public listeners");
    env
}

fn enqueue(env: &WowLuaEnv, kind: HostChatKind, secret: bool) {
    env.state()
        .borrow_mut()
        .host_chat_inputs
        .pending
        .push_back(HostChatMessage {
            kind,
            arguments: vec![
                HostChatArgument {
                    value: EventArg::String("Gold +17".into()),
                    secret,
                },
                HostChatArgument {
                    value: EventArg::Number(17.0),
                    secret: false,
                },
                HostChatArgument {
                    value: EventArg::Boolean(true),
                    secret: false,
                },
                HostChatArgument {
                    value: EventArg::Nil,
                    secret: false,
                },
            ],
            discord_info: Default::default(),
        });
}

fn install_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("native VM helpers");
    for (name, text) in [
        ("B39SecretToken", "target"),
        ("B39SecretExtra", "audit secret"),
    ] {
        let value = rilua::table_security::wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root real secret in secure host code");
    }
}

#[test]
fn b39_empty_ingress_and_environment_isolation() {
    let first = fixture_env();
    let second = fixture_env();
    assert!(!first.publish_next_host_chat().unwrap());
    enqueue(&first, HostChatKind::Money, false);
    assert!(!second.publish_next_host_chat().unwrap());
    assert!(first.publish_next_host_chat().unwrap());
    assert!(!first.publish_next_host_chat().unwrap());
    first.exec("assert(#B39Events == 1)").unwrap();
    second.exec("assert(#B39Events == 0)").unwrap();
}

#[test]
fn b39_all_nine_chat_rows_deliver_public_exact_payload_in_both_lockdown_states() {
    for lockdown in [false, true] {
        let env = fixture_env();
        env.state().borrow_mut().chat_messaging_lockdown = lockdown;
        for kind in [
            HostChatKind::CombatFactionChange,
            HostChatKind::CombatHonorGain,
            HostChatKind::CombatMiscInfo,
            HostChatKind::CombatXpGain,
            HostChatKind::Currency,
            HostChatKind::Filtered,
            HostChatKind::Loot,
            HostChatKind::Money,
            HostChatKind::Restricted,
        ] {
            enqueue(&env, kind, false);
            assert!(env.publish_next_host_chat().unwrap());
            env.exec(&format!(
                "local e=B39Events[#B39Events]; assert(e.event=='{}'); assert(e.count==18); assert(e[1]=='Gold +17' and e[2]==17 and e[3]==true and e[4]==nil); assert(e[17]==nil and e[18].userID==0); for i=1,3 do assert(not issecretvalue(e[i])) end",
                kind.event_name(),
            )).unwrap();
        }
        env.exec("assert(#B39Events == 9)").unwrap();
    }
}

#[test]
fn b39_lockdown_is_read_at_delivery_and_restricted_control_remains_secret() {
    let env = fixture_env();
    enqueue(&env, HostChatKind::Say, false);
    env.state().borrow_mut().chat_messaging_lockdown = true;
    assert!(env.publish_next_host_chat().unwrap());
    env.exec("local e=B39Events[1]; assert(e.count==18); assert(issecretvalue(e[1]) and issecretvalue(e[2])); assert(not issecretvalue(e[3])); assert(not issecretvalue(e[4]))").unwrap();
    env.state().borrow_mut().chat_messaging_lockdown = false;
    enqueue(&env, HostChatKind::Say, false);
    assert!(env.publish_next_host_chat().unwrap());
    env.exec("assert(B39Events[2][1]=='Gold +17'); assert(issecretvalue(B39Events[1][1]))")
        .unwrap();
}

#[test]
fn b39_independent_source_secret_survives_exemption_and_addon_taint() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local handler = B39Frame:GetScript('OnEvent')
        debug.setobjecttaint(handler, 'B39Audit')
        "#,
    )
    .unwrap();
    env.state().borrow_mut().chat_messaging_lockdown = true;
    enqueue(&env, HostChatKind::Loot, true);
    assert!(env.publish_next_host_chat().unwrap());
    env.exec(
        r#"
        B39Tainted(function()
            local e = B39Events[1]
            assert(e.count == 18 and issecretvalue(e[1]))
            assert(not pcall(secretunwrap, e[1]))
            assert(e[2] == 17 and not issecretvalue(e[2]))
        end)
        assert(not issecretvalue('Gold +17'))
        "#,
    )
    .unwrap();
}

#[test]
fn b41_default_snapshot_matches_declared_return_arity() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(select('#', GetEventCPUUsage()) == 2)
        assert(select('#', GetFunctionCPUUsage()) == 2)
        assert(select('#', GetScriptCPUUsage()) == 1)
        local time, count = GetEventCPUUsage()
        assert(time == 0 and count == 0)
        time, count = GetFunctionCPUUsage()
        assert(time == 0 and count == 0 and GetScriptCPUUsage() == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn b41_snapshots_read_host_updates_live_without_cross_input_coupling() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.performance_inputs.event_time = 12.5;
        state.performance_inputs.event_count = 7.0;
        state.performance_inputs.function_time = 3.25;
        state.performance_inputs.function_count = 2.0;
        state.performance_inputs.script_usage = 9.75;
    }
    env.exec("local t,n=GetEventCPUUsage(); assert(t==12.5 and n==7); t,n=GetFunctionCPUUsage(); assert(t==3.25 and n==2); assert(GetScriptCPUUsage()==9.75)").unwrap();
    env.state().borrow_mut().performance_inputs.event_time = 19.0;
    env.exec("local t,n=GetEventCPUUsage(); assert(t==19 and n==7); t,n=GetFunctionCPUUsage(); assert(t==3.25 and n==2); assert(GetScriptCPUUsage()==9.75)").unwrap();
}

#[test]
fn b41_snapshots_are_environment_local() {
    let first = fixture_env();
    let second = fixture_env();
    first.state().borrow_mut().performance_inputs.script_usage = 41.0;
    first.exec("assert(GetScriptCPUUsage()==41)").unwrap();
    second.exec("assert(GetScriptCPUUsage()==0)").unwrap();
}

#[test]
fn b41_inferred_never_secret_checks_every_extra_for_every_caller() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local queries = {GetEventCPUUsage, GetFunctionCPUUsage, GetScriptCPUUsage}
        for _, query in ipairs(queries) do
            assert(not pcall(query, B39SecretExtra))
            assert(not pcall(query, nil, {}, B39SecretExtra))
            B39Tainted(function()
                assert(not pcall(query, B39SecretExtra))
                assert(not pcall(query, nil, {}, B39SecretExtra))
                assert(pcall(query, 'public extra'))
            end)
        end
        assert(issecretvalue(B39SecretExtra))
        "#,
    )
    .unwrap();
}

#[test]
fn b43_unknown_tokens_have_defaults_and_supported_target_is_live() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(UnitGUID('audit-absent') == nil)
        assert(select('#', UnitGUID('audit-absent')) == 1)
        for _, query in ipairs({UnitHealth, UnitHealthMax, UnitPower, UnitPowerMax}) do
            assert(select('#', query('audit-absent')) == 1)
            assert(query('audit-absent') == 0)
        end
        assert(UnitAura('audit-absent', 1) == nil)
        A_Admin.SetTarget('Audit Visitor', 63, 1, true)
        assert(UnitGUID('target') ~= nil and UnitHealth('target') > 0)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .health = 321;
    env.exec("assert(UnitHealth('target')==321)").unwrap();
}

#[test]
fn b43_explicit_unsupported_state_overrides_existing_identity_and_vitals_live() {
    let env = fixture_env();
    env.exec("A_Admin.SetTarget('Audit Visitor', 63, 1, true); B39TargetGUID=UnitGUID('target')")
        .unwrap();
    env.state()
        .borrow_mut()
        .unsupported_unit_tokens
        .insert("target".into());
    env.exec("assert(UnitGUID('target')==nil); assert(UnitHealth('target')==0 and UnitHealthMax('target')==0); assert(UnitPower('target')==0 and UnitPowerMax('target')==0); assert(UnitAura('target',1)==nil)").unwrap();
    env.state().borrow_mut().unsupported_unit_tokens.clear();
    env.exec("assert(UnitGUID('target')==B39TargetGUID); assert(UnitHealth('target')>0)")
        .unwrap();
}

#[test]
fn b43_unsupported_state_is_environment_local() {
    let first = fixture_env();
    let second = fixture_env();
    first
        .state()
        .borrow_mut()
        .unsupported_unit_tokens
        .insert("player".into());
    first
        .exec("assert(UnitGUID('player')==nil and UnitHealth('player')==0)")
        .unwrap();
    second
        .exec("assert(UnitGUID('player')~=nil and UnitHealth('player')>0)")
        .unwrap();
}

#[test]
fn b43_authentication_precedes_unsupported_defaults_and_argument_validation() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec("A_Admin.SetTarget('Audit Visitor',63,1,true)")
        .unwrap();
    env.state()
        .borrow_mut()
        .unsupported_unit_tokens
        .insert("target".into());
    env.exec(
        r#"
        local queries = {UnitGUID, UnitHealth, UnitHealthMax, UnitPower, UnitPowerMax}
        for _, query in ipairs(queries) do
            assert(pcall(query, B39SecretToken))
            B39Tainted(function()
                assert(not pcall(query, B39SecretToken))
                assert(not pcall(query, 'target', nil, nil, B39SecretExtra))
                local ok, err = pcall(query, {}, nil, nil, B39SecretExtra)
                assert(not ok)
                assert(not string.find(tostring(err), 'unit token must be a string', 1, true))
                assert(pcall(query, 'target'))
            end)
            assert(not pcall(query, {}))
        end
        assert(pcall(UnitAura, B39SecretToken, 1))
        B39Tainted(function()
            assert(not pcall(UnitAura, B39SecretToken, 1))
            assert(not pcall(UnitAura, 'target', 1, nil, B39SecretExtra))
        end)
        assert(issecretvalue(B39SecretToken))
        "#,
    )
    .unwrap();
}

// Test-support only; the inherent host producer replaces this RED entry point.
trait InputsOnlyUrlPublisher {
    fn publish_next_url_texture_result(&self) -> wow_ui_sim::Result<bool>;
}
impl InputsOnlyUrlPublisher for WowLuaEnv {
    fn publish_next_url_texture_result(&self) -> wow_ui_sim::Result<bool> {
        Ok(false)
    }
}

fn url_fixture() -> (WowLuaEnv, u64) {
    let env = fixture_env();
    let _red_entry_signature =
        <WowLuaEnv as InputsOnlyUrlPublisher>::publish_next_url_texture_result;
    env.exec(
        r#"
        B42Texture = B39Frame:CreateTexture('B42Texture', 'ARTWORK')
        B42Events = {}
        B42Frame = CreateFrame('Frame')
        B42Frame:RegisterEvent('URL_TEXTURE_REQUEST_RESULT')
        B42Frame:SetScript('OnEvent', function(self, event, ...)
            assert(event == 'URL_TEXTURE_REQUEST_RESULT')
            assert(select('#', ...) == 2)
            local texture, result = ...
            assert(texture == B42Texture)
            assert(not issecretvalue(texture) and not issecretvalue(result))
            B42Events[#B42Events + 1] = result
        end)
    "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("B42Texture")
        .unwrap();
    (env, id)
}

#[test]
fn b42_notifications_use_actual_texture_identity_and_cached_enum_values_once() {
    use wow_ui_sim::c_api::url_texture_inputs::{UrlTextureNotification, UrlTextureResult};
    let (env, id) = url_fixture();
    assert!(!env.publish_next_url_texture_result().unwrap());
    for (result, expected) in [
        (UrlTextureResult::Requested, 3),
        (UrlTextureResult::Found, 1),
        (UrlTextureResult::NotFound, 2),
        (UrlTextureResult::NotAllowed, 4),
    ] {
        env.state()
            .borrow_mut()
            .url_texture_inputs
            .pending
            .push_back(UrlTextureNotification {
                texture_id: id,
                result,
            });
        assert!(env.publish_next_url_texture_result().unwrap());
        env.exec(&format!("assert(B42Events[#B42Events]=={expected})"))
            .unwrap();
        assert!(!env.publish_next_url_texture_result().unwrap());
    }
    env.exec("assert(#B42Events==4)").unwrap();
}

#[test]
fn b42_notifications_are_environment_local() {
    use wow_ui_sim::c_api::url_texture_inputs::{UrlTextureNotification, UrlTextureResult};
    let (first, id) = url_fixture();
    let (second, _) = url_fixture();
    first
        .state()
        .borrow_mut()
        .url_texture_inputs
        .pending
        .push_back(UrlTextureNotification {
            texture_id: id,
            result: UrlTextureResult::Found,
        });
    assert!(!second.publish_next_url_texture_result().unwrap());
    assert!(first.publish_next_url_texture_result().unwrap());
    second.exec("assert(#B42Events==0)").unwrap();
}

#[test]
fn b42_invalid_host_receiver_fails_without_notification() {
    use wow_ui_sim::c_api::url_texture_inputs::{UrlTextureNotification, UrlTextureResult};
    let (env, _) = url_fixture();
    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("B39Frame")
        .unwrap();
    for id in [frame_id, u64::MAX] {
        env.state()
            .borrow_mut()
            .url_texture_inputs
            .pending
            .push_back(UrlTextureNotification {
                texture_id: id,
                result: UrlTextureResult::Found,
            });
        assert!(env.publish_next_url_texture_result().is_err());
        env.exec("assert(#B42Events==0)").unwrap();
    }
}

#[test]
fn b39_never_secret_host_position_rejects_before_listener_dispatch() {
    let env = fixture_env();
    enqueue(&env, HostChatKind::Loot, false);
    env.state()
        .borrow_mut()
        .host_chat_inputs
        .pending
        .front_mut()
        .unwrap()
        .arguments[2]
        .secret = true;
    assert!(env.publish_next_host_chat().is_err());
    env.exec("assert(#B39Events==0)").unwrap();
}

#[test]
fn b43_legacy_aura_defaults_do_not_delete_populated_host_aura() {
    use wow_ui_sim::lua_api::state::AuraInfo;
    let env = fixture_env();
    env.state().borrow_mut().player.buffs = vec![AuraInfo {
        name: "Audit Blessing".into(),
        spell_id: 99039,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 3,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: Some("Magic".into()),
        aura_instance_id: 3901,
    }];
    env.exec("assert(UnitAura('player',1,'HELPFUL')=='Audit Blessing'); assert(select('#',UnitAura('player',1,'HELPFUL'))==10)").unwrap();
    env.state()
        .borrow_mut()
        .unsupported_unit_tokens
        .insert("player".into());
    env.exec("assert(UnitAura('player',1,'HELPFUL')==nil); assert(select('#',UnitAura('player',1,'HELPFUL'))==1)").unwrap();
    env.state().borrow_mut().unsupported_unit_tokens.clear();
    env.exec("assert(UnitAura('player',1,'HELPFUL')=='Audit Blessing')")
        .unwrap();
}
