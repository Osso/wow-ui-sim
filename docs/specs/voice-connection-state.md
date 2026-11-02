# Voice service connection

Retail and PTR `C_VoiceChat.IsVoiceChatConnected` returns the supplied service-connection state as one boolean. Retail `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua:604-610` declares the return shape. The initial false value is inferred from zero-initialized connection storage, not proof of every pre-login state.

WM35-B (Retail 12.1.5 build 70688) verifies that a success producer sets the native source true before optional `VOICE_CHAT_CONNECTION_SUCCESS`, selected error producers set it false before optional `VOICE_CHAT_ERROR`, and teardown clears validity. Login and logout producers do not directly write the source. Full lifecycle and callback ordering remain unknown; supplying this simulator state does not synthesize these transitions or events.
