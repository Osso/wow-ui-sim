# Voice chat

`A_Admin` can supply whether the player and client can use voice chat. This input drives `C_VoiceChat.CanPlayerUseVoiceChat()` on Retail and WoW Forever. It defaults to `true`.

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

`CanPlayerUseVoiceChat` is INFERRED R88fv (native evidence pending). Its setter supplies the result directly; it does not calculate parental-control, Kiosk, server, or CVar policy.
