# Voice service connection

Retail and PTR `C_VoiceChat.IsVoiceChatConnected` returns the supplied service-connection state as one boolean. Retail `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua:604-610` declares the return shape. The initial false value is inferred from zero-initialized connection storage, not proof of every pre-login state.

Retail and WoW Forever `C_VoiceChat.IsLoggedIn` returns a separately supplied login state. The Retail and Forever `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua` declarations (Retail :443-452; Forever :452-461) specify one nonnil boolean. INFERRED (R88eb (native evidence pending)): the simulator starts logged out, and login is not derived from connection or settings readiness. `A_Admin.SetVoiceChatLoggedIn` supplies the state. Login changes do not synthesize connection or readiness transitions.

WM35-B (Retail 12.1.5 build 70688) verifies that a success producer sets the native source true before optional `VOICE_CHAT_CONNECTION_SUCCESS`, selected error producers set it false before optional `VOICE_CHAT_ERROR`, and teardown clears validity. Login and logout producers do not directly write the source. Full lifecycle and callback ordering remain unknown; supplying this simulator state does not synthesize these transitions or events.
