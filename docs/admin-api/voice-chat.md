# Voice chat

`A_Admin` can supply whether the player and client can use voice chat. This input drives `C_VoiceChat.CanPlayerUseVoiceChat()` on Retail and WoW Forever. It defaults to `true`.

```lua
A_Admin.SetCanPlayerUseVoiceChat(false)
assert(C_VoiceChat.CanPlayerUseVoiceChat() == false)
```

`CanPlayerUseVoiceChat` is INFERRED R88fv (native evidence pending). The setter supplies the result directly; it does not calculate parental-control, Kiosk, server, or CVar policy.
