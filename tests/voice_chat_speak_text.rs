#![cfg(feature = "retail-12-0-5")]
//! Row409 request recording and NeverSecret boundary; not native speech proof.
//! INFERRED strict representations and explicit-nil overlap default.

use rilua::LuaApiMut;
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string,
};
use wow_ui_sim::c_api::c_voice_chat_speak::SpeakTextRequest;
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create speech environment");
    env.exec(
        r#"
        function RejectSpeech(...)
            assert(type(rawget(C_VoiceChat, 'SpeakText')) == 'function',
                'explicit provider required; namespace proxy is not proof')
            local ok, err = pcall(C_VoiceChat.SpeakText, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
            return err
        end
        "#,
    )
    .expect("install rejection assertion without replacing API");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    let number = wrap_host_secret_number(lua.state_mut(), 7.0);
    lua.state_mut().push(number);
    let inserted = lua.set_global_val("SpeechSecretNumber", number);
    lua.state_mut().pop();
    inserted.expect("root authentic secret NUMBER");
    let text = wrap_host_secret_string(lua.state_mut(), "Secret speech");
    lua.state_mut().push(text);
    let inserted = lua.set_global_val("SpeechSecretText", text);
    lua.state_mut().pop();
    inserted.expect("root authentic secret STRING");
    let overlap = wrap_host_secret_bool(lua.state_mut(), false);
    lua.state_mut().push(overlap);
    let inserted = lua.set_global_val("SpeechSecretOverlap", overlap);
    lua.state_mut().pop();
    inserted.expect("root authentic secret BOOLEAN");
}

fn requests(env: &WowLuaEnv) -> Vec<SpeakTextRequest> {
    env.state().borrow().voice_chat_speak_requests.clone()
}

fn seed_request(env: &WowLuaEnv) -> Vec<SpeakTextRequest> {
    env.exec("C_VoiceChat.SpeakText(9, 'Existing request', -1, 65, true)")
        .expect("seed queue through API");
    requests(env)
}

#[test]
fn ordinary_request_records_all_declared_fields_and_returns_zero_values() {
    let env = fixture_env();
    assert!(requests(&env).is_empty());
    assert_eq!(
        env.eval::<f64>(
            r#"return select('#', C_VoiceChat.SpeakText(4, 'Hello <bookmark mark="one"/>', -2, 81.5, true))"#
        )
        .expect("zero results"),
        0.0
    );
    assert_eq!(
        requests(&env),
        vec![SpeakTextRequest {
            voice_id: 4.0,
            text: "Hello <bookmark mark=\"one\"/>".into(),
            rate: -2.0,
            volume: 81.5,
            overlap: true,
        }]
    );
}

#[test]
fn multiple_requests_keep_order_and_default_overlap_false() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(select('#', C_VoiceChat.SpeakText(1, 'First', 2, 30, true)) == 0)
        assert(select('#', C_VoiceChat.SpeakText(2.5, 'Second', -3.5, 40)) == 0)
        assert(select('#', C_VoiceChat.SpeakText(3, '', 0, 0, nil)) == 0)
        "#,
    )
    .expect("ordered requests, declared default and INFERRED explicit-nil default");
    assert_eq!(
        requests(&env),
        vec![
            SpeakTextRequest {
                voice_id: 1.0,
                text: "First".into(),
                rate: 2.0,
                volume: 30.0,
                overlap: true,
            },
            SpeakTextRequest {
                voice_id: 2.5,
                text: "Second".into(),
                rate: -3.5,
                volume: 40.0,
                overlap: false,
            },
            SpeakTextRequest {
                voice_id: 3.0,
                text: String::new(),
                rate: 0.0,
                volume: 0.0,
                overlap: false,
            },
        ]
    );
}

fn assert_never_secret_rejections(env: &WowLuaEnv, tainted: bool) {
    env.exec(
        r#"
        function ProbeNeverSecretSpeech()
            local before = debug.getstacktaint()
            for _, value in ipairs({SpeechSecretNumber, SpeechSecretText, SpeechSecretOverlap}) do
                for _, position in ipairs({1, 3, 4, 5}) do
                    local args = {4, 'Ordinary', 2, 50, false}
                    args[position] = value
                    local denial = RejectSpeech(unpack(args))
                    assert(string.find(denial, 'must not be secret', 1, true))
                    -- Original wrappers are classified before other argument conversions.
                    args[2] = {}
                    assert(RejectSpeech(unpack(args)) == denial)
                    if position ~= 1 then
                        args[1] = {}
                        assert(RejectSpeech(unpack(args)) == denial)
                    end
                    assert(issecretvalue(value))
                    assert(debug.getstacktaint() == before)
                end
            end
            if before then
                assert(before == 'SpeechProbe')
                assert(not pcall(secretunwrap, SpeechSecretNumber))
                assert(not pcall(secretunwrap, SpeechSecretText))
                assert(not pcall(secretunwrap, SpeechSecretOverlap))
            end
            assert(debug.getstacktaint() == before)
        end
        "#,
    )
    .expect("install actual caller probe");
    if tainted {
        env.exec("debug.setobjecttaint(ProbeNeverSecretSpeech, 'SpeechProbe')")
            .expect("taint probe closure");
    }
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        ProbeNeverSecretSpeech()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SpeechSecretNumber) and secretunwrap(SpeechSecretNumber) == 7)
        assert(issecretvalue(SpeechSecretText) and secretunwrap(SpeechSecretText) == 'Secret speech')
        assert(issecretvalue(SpeechSecretOverlap) and secretunwrap(SpeechSecretOverlap) == false)
        "#,
    )
    .expect("NeverSecret denial preserves caller taint and every original secret");
}

#[test]
fn secure_caller_rejects_every_never_secret_position_without_state_change() {
    let env = fixture_env();
    install_host_secrets(&env);
    let before = seed_request(&env);
    assert_never_secret_rejections(&env, false);
    assert_eq!(requests(&env), before);
}

#[test]
fn tainted_caller_rejects_every_never_secret_position_without_state_change() {
    let env = fixture_env();
    install_host_secrets(&env);
    let before = seed_request(&env);
    assert_never_secret_rejections(&env, true);
    assert_eq!(requests(&env), before);
}

#[test]
fn secret_text_is_conservatively_rejected_secure_and_tainted() {
    let env = fixture_env();
    install_host_secrets(&env);
    let before = seed_request(&env);
    env.exec(
        r#"
        function ProbeSecretSpeechText()
            local before = debug.getstacktaint()
            for _, value in ipairs({SpeechSecretText, SpeechSecretNumber, SpeechSecretOverlap}) do
                RejectSpeech(4, value, 2, 50, false)
                assert(issecretvalue(value))
            end
            assert(debug.getstacktaint() == before)
        end
        assert(debug.getstacktaint() == nil)
        ProbeSecretSpeechText()
        debug.setobjecttaint(ProbeSecretSpeechText, 'SpeechTextProbe')
        ProbeSecretSpeechText()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SpeechSecretText) and secretunwrap(SpeechSecretText) == 'Secret speech')
        "#,
    )
    .expect("AllowedWhenTainted secret text remains explicitly unmodeled");
    assert_eq!(requests(&env), before);
}

#[test]
fn malformed_public_arguments_reject_atomically_and_public_calls_recover() {
    let env = fixture_env();
    let before = seed_request(&env);
    env.exec(
        r#"
        RejectSpeech()
        RejectSpeech(nil, 'Text', 2, 50, false)
        RejectSpeech(4, nil, 2, 50, false)
        RejectSpeech(4, 'Text', nil, 50, false)
        RejectSpeech(4, 'Text', 2, nil, false)
        for _, value in ipairs({'4', false, {}, function() end, 0/0, math.huge, -math.huge}) do
            RejectSpeech(value, 'Text', 2, 50, false)
            RejectSpeech(4, 'Text', value, 50, false)
            RejectSpeech(4, 'Text', 2, value, false)
        end
        for _, value in ipairs({4, false, {}, function() end, string.char(255)}) do
            RejectSpeech(4, value, 2, 50, false)
        end
        for _, value in ipairs({0, 100, 'false', {}, function() end}) do
            RejectSpeech(4, 'Text', 2, 50, value)
        end
        "#,
    )
    .expect("INFERRED strict finite numbers, UTF-8 text, boolean overlap");
    assert_eq!(requests(&env), before);
    env.exec(
        r#"
        local function recovery()
            assert(debug.getstacktaint() == 'SpeechRecoveryProbe')
            assert(select('#', C_VoiceChat.SpeakText(6, 'Recovered', 1, 75, false)) == 0)
            assert(debug.getstacktaint() == 'SpeechRecoveryProbe')
        end
        debug.setobjecttaint(recovery, 'SpeechRecoveryProbe')
        recovery()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("public recovery accepts tainted callers without clearing taint");
    let mut expected = before;
    expected.push(SpeakTextRequest {
        voice_id: 6.0,
        text: "Recovered".into(),
        rate: 1.0,
        volume: 75.0,
        overlap: false,
    });
    assert_eq!(requests(&env), expected);
}

#[test]
fn rooted_secret_inputs_survive_gc_and_denial_without_declassification() {
    let env = fixture_env();
    install_host_secrets(&env);
    let before = seed_request(&env);
    env.exec(
        r#"
        SpeechNumberBeforeGC = SpeechSecretNumber
        SpeechTextBeforeGC = SpeechSecretText
        SpeechOverlapBeforeGC = SpeechSecretOverlap
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(SpeechNumberBeforeGC, SpeechSecretNumber))
        assert(rawequal(SpeechTextBeforeGC, SpeechSecretText))
        assert(rawequal(SpeechOverlapBeforeGC, SpeechSecretOverlap))
        "#,
    )
    .expect("authentic wrappers rooted across full GC");
    assert_never_secret_rejections(&env, true);
    assert_eq!(requests(&env), before);
}

#[test]
fn environments_keep_separate_request_queues() {
    let first = fixture_env();
    let second = fixture_env();
    let first_requests = seed_request(&first);
    assert!(requests(&second).is_empty());
    second
        .exec("C_VoiceChat.SpeakText(2, 'Second environment', 3, 40, false)")
        .expect("independent environment request");
    assert_eq!(requests(&first), first_requests);
    assert_eq!(
        requests(&second),
        vec![SpeakTextRequest {
            voice_id: 2.0,
            text: "Second environment".into(),
            rate: 3.0,
            volume: 40.0,
            overlap: false,
        }]
    );
}
