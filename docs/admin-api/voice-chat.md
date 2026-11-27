# Voice chat

### `A_Admin.SetCanPlayerUseVoiceChat(canUse)`

Supplies whether the player and client can use voice chat. This input drives `C_VoiceChat.CanPlayerUseVoiceChat()` on Retail and WoW Forever. It defaults to `true`. The setter supplies the result directly; it does not calculate parental-control, Kiosk, server, or CVar policy.

### `A_Admin.SetVoiceChatConnected(connected)`

Supplies the voice-service connection that `C_VoiceChat.IsVoiceChatConnected()` returns on Retail and WoW Forever. It defaults to `false`. The setter does not fire `VOICE_CHAT_CONNECTION_SUCCESS` or `VOICE_CHAT_ERROR`, and it does not change login state.

### `A_Admin.SetRemoteTtsVoices(voices)`

Supplies the list that `C_VoiceChat.GetRemoteTtsVoices()` returns on Retail and WoW Forever. Each entry needs an integer `voiceID` and a string `name`. The list is empty by default. The setter does not fire `VOICE_CHAT_TTS_VOICES_UPDATE`. The empty default is INFERRED R88g9 (native evidence pending).

```lua
A_Admin.SetVoiceChatConnected(true)
A_Admin.SetRemoteTtsVoices({ { voiceID = 1, name = "Remote One" } })
assert(C_VoiceChat.IsVoiceChatConnected() and C_VoiceChat.GetRemoteTtsVoices()[1].voiceID == 1)
```

### `A_Admin.SetJoinClubVoiceChannelError(clubId, reason)`

Supply or clear the error reason returned by `C_VoiceChat.GetJoinClubVoiceChannelError(clubId)`. Set `reason` to a `VoiceChannelErrorReason` value or `nil` to clear it. No error is supplied by default.

This per-club input is INFERRED R88fv (native evidence pending).

### `A_Admin.SetChannelJoinPending(channelType, clubId, streamId, pending)`

Set whether a voice-channel join is pending for the specified chat-channel type and optional club/stream IDs. The getter returns `false` when no matching pending join exists.

This input is INFERRED R88fv (native evidence pending).

```lua
A_Admin.SetCanPlayerUseVoiceChat(false)
assert(C_VoiceChat.CanPlayerUseVoiceChat() == false)
```

This result is INFERRED R88fv (native evidence pending).
