//! Integration tests for `src/lua_api/globals/real/voice_chat_probes.rs`.

use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn source_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── Defaults ──────────────────────────────────────────────────────────────────

#[test]
fn voice_enabled_defaults_true() {
    let env = env();
    let b: bool = env.eval("return IsVoiceEnabled()").unwrap();
    assert!(b, "retail ships with voice chat enabled by default");
}

#[test]
fn other_probes_default_false() {
    let env = env();
    let tuple: (bool, bool, bool, bool, bool) = env
        .eval(
            "return IsUsingVoiceChat(),
                    VoiceChat_IsConnecting(),
                    VoiceChat_IsMuted(),
                    VoiceChat_IsDeafened(),
                    VoiceChat_IsTalking()",
        )
        .unwrap();
    assert_eq!(tuple, (false, false, false, false, false));
}

// ── Individual field round-trip ───────────────────────────────────────────────

#[test]
fn is_using_voice_chat_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.using = true;
    let b: bool = env.eval("return IsUsingVoiceChat()").unwrap();
    assert!(b);
}

#[test]
fn is_voice_enabled_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.enabled = false;
    let b: bool = env.eval("return IsVoiceEnabled()").unwrap();
    assert!(!b);
}

#[test]
fn voice_chat_is_connecting_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.connecting = true;
    let b: bool = env.eval("return VoiceChat_IsConnecting()").unwrap();
    assert!(b);
}

#[test]
fn voice_chat_is_muted_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.muted = true;
    let b: bool = env.eval("return VoiceChat_IsMuted()").unwrap();
    assert!(b);
}

#[test]
fn voice_chat_is_deafened_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.deafened = true;
    let b: bool = env.eval("return VoiceChat_IsDeafened()").unwrap();
    assert!(b);
}

#[test]
fn voice_chat_is_talking_reads_state_field() {
    let env = env();
    env.state().borrow_mut().voice_chat.talking = true;
    let b: bool = env.eval("return VoiceChat_IsTalking()").unwrap();
    assert!(b);
}

#[test]
fn voice_chat_probe_globals_live_under_real_globals_boundary() {
    assert!(
        !source_path("src/lua_api/globals/voice_chat_probes.rs").exists(),
        "voice chat probe globals are modeled through SimState and belong under globals::real",
    );
    assert!(
        source_path("src/lua_api/globals/real/voice_chat_probes.rs").exists(),
        "voice chat probe globals should stay classified as real modeled Lua globals",
    );
}
