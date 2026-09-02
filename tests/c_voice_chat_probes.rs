//! Tests for `C_VoiceChat` probes backed by `SimState.voice_chat`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::VoiceChannel;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn channel(channel_id: i32, name: &str, channel_type: i32) -> VoiceChannel {
    VoiceChannel {
        channel_id,
        name: name.into(),
        channel_type,
        members: vec![],
    }
}

// ── GetActiveChannelType ──────────────────────────────────────────────────────

#[test]
fn get_active_channel_type_returns_the_active_channel_type() {
    let env = env();
    env.state().borrow_mut().voice_chat.active_channel_id = Some(1);
    let (value_type, channel_type): (String, i32) = env
        .eval(
            r#"
            local channelType = C_VoiceChat.GetActiveChannelType()
            return type(channelType), channelType
            "#,
        )
        .unwrap();

    assert_eq!(value_type, "number");
    assert_eq!(channel_type, 1);
}

#[test]
fn get_active_channel_type_tracks_active_channel_changes() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.voice_chat.channels = vec![channel(1, "Party", 2), channel(2, "Community", 4)];
        sim.voice_chat.active_channel_id = Some(1);
    }

    let initial: i32 = env
        .eval("return C_VoiceChat.GetActiveChannelType()")
        .unwrap();
    assert_eq!(initial, 2);

    env.state().borrow_mut().voice_chat.active_channel_id = Some(2);
    let updated: i32 = env
        .eval("return C_VoiceChat.GetActiveChannelType()")
        .unwrap();
    assert_eq!(updated, 4);
}

#[test]
fn get_active_channel_type_returns_one_nil_without_active_channel() {
    let env = env();
    env.state().borrow_mut().voice_chat.active_channel_id = None;

    let (count, is_nil): (i32, bool) = env
        .eval(
            r#"
            local function inspect(value, ...)
                return 1 + select('#', ...), value == nil
            end
            return inspect(C_VoiceChat.GetActiveChannelType())
            "#,
        )
        .unwrap();

    assert_eq!(count, 1);
    assert!(is_nil);
}

#[test]
fn get_active_channel_type_returns_one_nil_for_stale_active_channel() {
    let env = env();
    env.state().borrow_mut().voice_chat.active_channel_id = Some(999);

    let (count, is_nil): (i32, bool) = env
        .eval(
            r#"
            local function inspect(value, ...)
                return 1 + select('#', ...), value == nil
            end
            return inspect(C_VoiceChat.GetActiveChannelType())
            "#,
        )
        .unwrap();

    assert_eq!(count, 1);
    assert!(is_nil);
}

// ── GetActiveChannelID ────────────────────────────────────────────────────────

#[test]
fn get_active_channel_id_is_nil_by_default() {
    // A client that is not in a voice channel: the chat channel button shows
    // the speaker icon, not the headset (ChannelFrameButtonMixin:OnLoad).
    let env = env();
    let is_nil: bool = env
        .eval("return C_VoiceChat.GetActiveChannelID() == nil")
        .unwrap();
    assert!(is_nil, "no active channel by default, as for a client outside voice chat");
}

#[test]
fn get_active_channel_id_returns_the_active_channel() {
    let env = env();
    env.state().borrow_mut().voice_chat.active_channel_id = Some(1);
    let id: i32 = env.eval("return C_VoiceChat.GetActiveChannelID()").unwrap();
    assert_eq!(id, 1);
}

#[test]
fn get_active_channel_id_returns_nil_when_none() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.voice_chat.active_channel_id = None;
    }
    let count: i32 = env
        .eval(
            r#"
            local function count(...)
                return select('#', ...)
            end
            return count(C_VoiceChat.GetActiveChannelID())
            "#,
        )
        .unwrap();
    assert_eq!(count, 1, "returns nil, not nothing");
}

// ── GetChannels ───────────────────────────────────────────────────────────────

#[test]
fn get_channels_returns_seeded_channel() {
    let env = env();
    let (count, channel_id): (i32, i32) = env
        .eval(
            r#"
            local ch = C_VoiceChat.GetChannels()
            return #ch, ch[1].channelID
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(channel_id, 1);
}

#[test]
fn get_channels_reflects_mutation() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.voice_chat.channels = vec![channel(1, "Party", 1), channel(2, "Guild", 2)];
    }
    let count: i32 = env.eval("return #C_VoiceChat.GetChannels()").unwrap();
    assert_eq!(count, 2);
}

// ── GetChannel ────────────────────────────────────────────────────────────────

#[test]
fn get_channel_returns_channel_for_valid_id() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            local ch = C_VoiceChat.GetChannel(1)
            return ch.channelName
            "#,
        )
        .unwrap();
    assert_eq!(name, "Party");
}

#[test]
fn get_channel_returns_nothing_for_invalid_id() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local function count(...)
                return select('#', ...)
            end
            return count(C_VoiceChat.GetChannel(999))
            "#,
        )
        .unwrap();
    assert_eq!(count, 0, "unknown channel = mayreturnnothing");
}

// ── GetCurrentVoiceChatConnectionStatusCode ───────────────────────────────────

#[test]
fn get_connection_status_returns_seeded_code() {
    let env = env();
    // Default seed: 2 = Connected
    let code: i32 = env
        .eval("return C_VoiceChat.GetCurrentVoiceChatConnectionStatusCode()")
        .unwrap();
    assert_eq!(code, 2);
}

#[test]
fn get_connection_status_reflects_mutation() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.voice_chat.connection_status = 0;
    }
    let code: i32 = env
        .eval("return C_VoiceChat.GetCurrentVoiceChatConnectionStatusCode()")
        .unwrap();
    assert_eq!(code, 0);
}

// ── GetMasterVolumeScale ──────────────────────────────────────────────────────

#[test]
fn get_master_volume_scale_returns_one_by_default() {
    let env = env();
    let scale: f64 = env
        .eval("return C_VoiceChat.GetMasterVolumeScale()")
        .unwrap();
    assert!((scale - 1.0).abs() < 1e-6);
}

// ── GetMicrophoneVolume / GetOutputVolume ─────────────────────────────────────

#[test]
fn get_microphone_volume_returns_seeded_value() {
    let env = env();
    let vol: f64 = env
        .eval("return C_VoiceChat.GetMicrophoneVolume()")
        .unwrap();
    assert!((vol - 1.0).abs() < 1e-6);
}

#[test]
fn get_output_volume_returns_seeded_value() {
    let env = env();
    let vol: f64 = env.eval("return C_VoiceChat.GetOutputVolume()").unwrap();
    assert!((vol - 1.0).abs() < 1e-6);
}

#[test]
fn transcription_and_tts_voices_default_to_unavailable() {
    let env = env();
    let (transcription_allowed, voice_count): (bool, i32) = env
        .eval(
            r#"
            local voices = C_VoiceChat.GetTtsVoices()
            return C_VoiceChat.IsTranscriptionAllowed(), #voices
            "#,
        )
        .unwrap();

    assert!(!transcription_allowed);
    assert_eq!(voice_count, 0);
}

// ── GetNumActiveChannels / GetNumMembers ──────────────────────────────────────

#[test]
fn get_num_active_channels_returns_one_by_default() {
    let env = env();
    let n: i32 = env
        .eval("return C_VoiceChat.GetNumActiveChannels()")
        .unwrap();
    assert_eq!(n, 1);
}

#[test]
fn get_num_members_returns_two_for_seeded_channel() {
    let env = env();
    let n: i32 = env.eval("return C_VoiceChat.GetNumMembers(1)").unwrap();
    assert_eq!(n, 2);
}

#[test]
fn get_num_members_returns_zero_for_unknown_channel() {
    let env = env();
    let n: i32 = env.eval("return C_VoiceChat.GetNumMembers(999)").unwrap();
    assert_eq!(n, 0);
}

// ── IsDeafened / IsEnabled / IsMuted / IsParentalDisabled / IsTalking ─────────

#[test]
fn is_deafened_false_by_default() {
    let env = env();
    let v: bool = env.eval("return C_VoiceChat.IsDeafened()").unwrap();
    assert!(!v);
}

#[test]
fn is_enabled_true_by_default() {
    let env = env();
    let v: bool = env.eval("return C_VoiceChat.IsEnabled()").unwrap();
    assert!(v);
}

#[test]
fn is_muted_false_by_default() {
    let env = env();
    let v: bool = env.eval("return C_VoiceChat.IsMuted()").unwrap();
    assert!(!v);
}

#[test]
fn is_parental_disabled_false_by_default() {
    let env = env();
    let v: bool = env.eval("return C_VoiceChat.IsParentalDisabled()").unwrap();
    assert!(!v);
}

#[test]
fn is_talking_false_by_default() {
    let env = env();
    let v: bool = env.eval("return C_VoiceChat.IsTalking()").unwrap();
    assert!(!v);
}

#[test]
fn is_muted_toggle() {
    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.voice_chat.muted = true;
    }
    let v: bool = env.eval("return C_VoiceChat.IsMuted()").unwrap();
    assert!(v);
}
