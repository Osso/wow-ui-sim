#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_presence_setters_are_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(type(rawget(C_BattleNet, 'SetAFK')) == 'function')
        assert(type(rawget(C_BattleNet, 'SetDND')) == 'function')
        C_BattleNet.SetAFK()
        C_BattleNet.SetDND(true)
    "#).unwrap();
    assert!(env.state().borrow().bnet_presence.is_afk);
    assert!(env.state().borrow().bnet_presence.is_dnd);
    env.exec("C_BattleNet.SetAFK(false); C_BattleNet.SetDND(false)").unwrap();
    assert!(!env.state().borrow().bnet_presence.is_afk);
    assert!(!env.state().borrow().bnet_presence.is_dnd);
}

#[test]
fn patch_12_0_1_voice_channel_unknown_lookup_returns_nothing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(type(rawget(C_VoiceChat, 'GetChannelForChannelType')) == 'function')
        assert(type(rawget(C_VoiceChat, 'GetChannelForCommunityStream')) == 'function')
        assert(select('#', C_VoiceChat.GetChannelForChannelType(999)) == 0)
        assert(select('#', C_VoiceChat.GetChannelForCommunityStream('10', '20')) == 0)
    "#).unwrap();
    use wow_ui_sim::c_api::c_voice_chat_channels::{VoiceChannel, VoiceMember};
    env.state().borrow_mut().voice_channels.push(VoiceChannel {
        name: "Raid Voice".into(), channel_id: 71, channel_type: 2,
        club_id: "9007199254740993".into(), stream_id: "20".into(), volume: 0.75,
        is_active: true, is_muted: false, is_transmitting: true, is_transcribing: false,
        members: vec![VoiceMember { energy: 0.25, member_id: 9, is_active: true,
            is_speaking: true, is_muted_for_all: false, is_silenced: false }],
    });
    env.exec(r#"
        local c = C_VoiceChat.GetChannelForChannelType(2)
        assert(c.channelID == 71 and c.name == 'Raid Voice' and c.volume == 0.75)
        assert(c.isActive and c.isTransmitting and not c.isMuted)
        assert(#c.members == 1 and c.members[1].memberID == 9 and c.members[1].isSpeaking)
        c.members[1].memberID = 99
        c.name = 'Changed'
        local same = C_VoiceChat.GetChannelForCommunityStream('9007199254740993', '20')
        assert(same.channelID == 71 and same.name == 'Raid Voice' and same.members[1].memberID == 9)
        assert(select('#', C_VoiceChat.GetChannelForCommunityStream('9007199254740992', '20')) == 0)
    "#).unwrap();
}
