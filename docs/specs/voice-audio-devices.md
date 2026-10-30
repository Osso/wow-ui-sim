# Supplied voice audio devices

`C_VoiceChat.GetAvailableInputDevices()` and `C_VoiceChat.GetAvailableOutputDevices()` expose environment-local supplied catalogs from `VoiceChatState.input_devices` and `VoiceChatState.output_devices`. Both start as an empty list. That is a chosen default, not native evidence: the simulator has no audio-device enumeration or voice proxy. A host that needs devices supplies them on the state.

The original consumer is `Blizzard_SettingsDefinitions_Shared/Audio.lua` `InitVoiceSettings`, entered once [settings access](voice-settings-access.md) answers true. Without these members it fails at Retail Audio.lua:179 (`GetDefaultOutputDevice`) with `attempt to call field 'GetAvailableOutputDevices' (a nil value)`.

## What it must do

- [x] Publish both members on Retail, PTR and Forever, whose pinned generated documentation declares them.
- [x] Return exactly one value: a fresh array of `VoiceAudioDevice` rows (`deviceID`, `displayName`, `isActive`, `isSystemDefault`, `isCommsDefault`), or nil when the supplied catalog is `None` (the result is declared `Nilable = true`).
- [x] Return a new table per call. Audio.lua sorts the result in place; the supplied catalog keeps its order.
- [x] The original `InitVoiceSettings` completes with the chosen empty catalogs. Its device dropdowns, active-device getters and default-device values read supplied catalogs, and a `None` catalog selects the original `VOICE_CHAT_INVALID_DEVICE` option.

### Pinned declarations

Paths are relative to the active profile's `AddOns/Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua`. Each function has no arguments and one result `table`, `InnerType = "VoiceAudioDevice"`, `Nilable = true`.

| Profile (Gethe revision) | GetAvailableInputDevices | GetAvailableOutputDevices | VoiceAudioDevice |
|---|---|---|---|
| Retail `b9be937` | :116-123 | :125-132 | :1427-1437 |
| PTR `2cdbc7f` | :125-132 | :134-141 | :1512-1522 |
| Forever `ca8368f` | :125-132 | :134-141 | :1512-1522 |

## Implementation inventory

- `src/c_api/c_voice_chat_devices.rs`: `VoiceAudioDevice`, profile-gated registration and the two queries.
- `src/lua_api/sim_substates/mod.rs`: optional supplied catalogs on `VoiceChatState`, initially empty.
- `src/c_api/registration.rs`, `src/c_api/mod.rs`: registration wiring.

## Tests asserting this spec

- `tests/voice_audio_devices.rs`: runs the selected original Audio polling callback unhooked, so the real `InitVoiceSettings` consumes the catalogs; checks the original settings' values, defaults and dropdown options for empty, supplied and `None` catalogs; checks the documented result shape and absence on other profiles.
- `tests/voice_audio_devices/observe.lua`: reads the original settings and dropdown initializers. No vendor callee is replaced.
- `tests/voice_settings_access/original_callback.lua`: exposes the located original callback to the device test.

## Known gaps (current cycle)

- [ ] Run the consumer tests under PTR; its declaration is cited but its runtime was not executed in the proving environment.
- [ ] Mists, Era and Anniversary ship Audio.lua and VoiceChatDocumentation.lua, but their pinned contents were not checked; the members stay unpublished there.

## Out of scope

Native device enumeration and defaults, `SetInputDevice`/`SetOutputDevice`, device-change events, admin inputs and voice transport.
