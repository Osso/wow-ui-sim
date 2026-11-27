# Voice service connection

Retail, PTR and WoW Forever `C_VoiceChat.IsVoiceChatConnected` returns the supplied service-connection state as one boolean. Retail `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua:604-610` and Forever `:613-620` declare the return shape. `A_Admin.SetVoiceChatConnected` supplies the state; it does not fire connection events. The initial false value is inferred from zero-initialized connection storage, not proof of every pre-login state.

Retail and WoW Forever `C_VoiceChat.IsLoggedIn` returns a separately supplied login state. The Retail and Forever `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua` declarations (Retail :443-452; Forever :452-461) specify one nonnil boolean. INFERRED (R88eb (native evidence pending)): the simulator starts logged out, and login is not derived from connection or settings readiness. `A_Admin.SetVoiceChatLoggedIn` supplies the state. Login changes do not synthesize connection or readiness transitions.

WM35-B (Retail 12.1.5 build 70688) verifies that a success producer sets the native source true before optional `VOICE_CHAT_CONNECTION_SUCCESS`, selected error producers set it false before optional `VOICE_CHAT_ERROR`, and teardown clears validity. Login and logout producers do not directly write the source. Full lifecycle and callback ordering remain unknown; supplying this simulator state does not synthesize these transitions or events.

Retail and WoW Forever `C_VoiceChat.GetRemoteTtsVoices` returns the supplied remote voice list (`VoiceTtsVoiceType`: `voiceID`, `name`; Retail `VoiceChatDocumentation.lua:381-388`, Forever `:390-397`). `A_Admin.SetRemoteTtsVoices` supplies it. INFERRED R88g9 (native evidence pending): the list is empty until supplied.

The original AudioAssist settings registrant (`Blizzard_SettingsDefinitions_Shared/AudioAssist.lua:1340-1355`) registers the voice and combat audio settings only when the connection is supplied and the remote voice list is not empty. Otherwise it waits for `VOICE_CHAT_CONNECTION_SUCCESS` or `VOICE_CHAT_TTS_VOICES_UPDATE`. Test: `combat_audio_forever_settings::*` (Forever).
