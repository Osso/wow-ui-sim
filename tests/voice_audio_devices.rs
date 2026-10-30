//! Supplied device catalogs through the pinned Audio.lua InitVoiceSettings, not a UI rewrite.
use wow_ui_sim::c_api::c_voice_chat_devices::VoiceAudioDevice;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

use crate::voice_settings_access::original_audio_env;

fn publishes_devices() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever)
}

fn device(id: &str, name: &str, active: bool, system_default: bool) -> VoiceAudioDevice {
    VoiceAudioDevice {
        device_id: id.into(),
        display_name: name.into(),
        is_active: active,
        is_system_default: system_default,
        is_comms_default: false,
    }
}

fn initialized_audio_env(
    output: Option<Vec<VoiceAudioDevice>>,
    input: Option<Vec<VoiceAudioDevice>>,
) -> WowLuaEnv {
    let env = original_audio_env();
    {
        let mut state = env.state().borrow_mut();
        state.voice_chat.can_access_settings = Some(true);
        state.voice_chat.output_devices = output;
        state.voice_chat.input_devices = input;
    }
    env.exec(include_str!("voice_audio_devices/observe.lua")).unwrap();
    env.eval::<bool>("return RunOriginalVoiceInitialization()")
        .expect("original InitVoiceSettings must consume the supplied device catalogs");
    env
}

fn observe(env: &WowLuaEnv, kind: &str) -> (String, String, String) {
    env.eval(&format!("return ObserveOriginalDeviceSetting('{kind}')"))
        .unwrap()
}

#[test]
fn original_voice_initialization_accepts_chosen_empty_catalogs() {
    if !publishes_devices() {
        return;
    }
    let env = original_audio_env();
    let defaults = env.state().borrow().voice_chat.clone();
    assert_eq!(defaults.output_devices, Some(Vec::new()));
    assert_eq!(defaults.input_devices, Some(Vec::new()));
    env.exec(include_str!("voice_audio_devices/observe.lua")).unwrap();
    env.eval::<bool>("return RunOriginalVoiceInitialization()")
        .expect("original InitVoiceSettings must complete with the chosen empty catalogs");
    for kind in ["OUTPUT", "INPUT"] {
        assert_eq!(observe(&env, kind), (String::new(), String::new(), String::new()));
    }
}

#[test]
fn original_voice_dropdowns_read_supplied_devices() {
    if !publishes_devices() {
        return;
    }
    let output = vec![
        device("headset-1", "Headset", true, false),
        device("speakers-2", "Speakers", false, true),
    ];
    let input = vec![device("mic-1", "Microphone", false, true)];
    let env = initialized_audio_env(Some(output.clone()), Some(input));
    let (default_label, rendered): (String, String) = env
        .eval("return VOICE_CHAT_OUTPUT_DEVICE_DEFAULT, 'speakers-2=' .. VOICE_CHAT_OUTPUT_DEVICE_DEFAULT .. ';headset-1=Headset'")
        .unwrap();
    assert!(!default_label.is_empty());
    assert_eq!(
        observe(&env, "OUTPUT"),
        ("headset-1".into(), "speakers-2".into(), rendered)
    );
    let mic_label: String = env.eval("return VOICE_CHAT_INPUT_DEVICE_DEFAULT").unwrap();
    assert_eq!(
        observe(&env, "INPUT"),
        (String::new(), "mic-1".into(), format!("mic-1={mic_label}"))
    );
    // The consumer sorted its copy in place; the supplied catalog keeps its order.
    assert_eq!(env.state().borrow().voice_chat.output_devices, Some(output));

    env.state().borrow_mut().voice_chat.output_devices = Some(vec![device("usb-3", "USB", true, true)]);
    let (active, _, options) = observe(&env, "OUTPUT");
    assert_eq!(active, "usb-3", "setting reads the live catalog");
    assert!(options.starts_with("usb-3="), "{options}");
}

#[test]
fn original_voice_dropdowns_mark_unsupplied_catalog_invalid() {
    if !publishes_devices() {
        return;
    }
    let env = initialized_audio_env(None, Some(Vec::new()));
    let invalid: String = env.eval("return VOICE_CHAT_INVALID_DEVICE").unwrap();
    assert_eq!(
        observe(&env, "OUTPUT"),
        (String::new(), String::new(), format!("{invalid}={invalid}"))
    );
    assert_eq!(observe(&env, "INPUT"), (String::new(), String::new(), String::new()));
}

#[test]
fn device_queries_match_documented_shape() {
    let env = WowLuaEnv::new().unwrap();
    if !publishes_devices() {
        assert!(
            env.eval::<bool>(
                "return rawget(C_VoiceChat, 'GetAvailableInputDevices') == nil \
                 and rawget(C_VoiceChat, 'GetAvailableOutputDevices') == nil"
            )
            .unwrap()
        );
        return;
    }
    env.state().borrow_mut().voice_chat.input_devices =
        Some(vec![device("mic-1", "Microphone", true, false)]);
    env.exec(
        r#"
        local function pack(...) return select('#', ...), ... end
        local n, devices = pack(C_VoiceChat.GetAvailableInputDevices())
        assert(n == 1 and #devices == 1)
        local row = devices[1]
        assert(row.deviceID == 'mic-1' and row.displayName == 'Microphone')
        assert(row.isActive == true and row.isSystemDefault == false and row.isCommsDefault == false)
        assert(C_VoiceChat.GetAvailableInputDevices() ~= devices, 'fresh table per call')
        n, devices = pack(C_VoiceChat.GetAvailableOutputDevices())
        assert(n == 1 and type(devices) == 'table' and #devices == 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().voice_chat.output_devices = None;
    let (arity, is_nil): (i32, bool) = env
        .eval("local function pack(...) return select('#', ...), (...) == nil end; return pack(C_VoiceChat.GetAvailableOutputDevices())")
        .unwrap();
    assert_eq!((arity, is_nil), (1, true));
}
