# Supplied voice audio devices

`C_VoiceChat.GetAvailableInputDevices()` and `C_VoiceChat.GetAvailableOutputDevices()` expose environment-local catalogs from `VoiceChatState.input_devices` and `VoiceChatState.output_devices`. Each starts with one selected system-default device. That is a researched best guess at a normal machine, not native evidence (see [Default device evidence](#default-device-evidence)). The simulator has no audio-device enumeration or voice proxy; a host overrides either catalog on the state.

The original consumer is `Blizzard_SettingsDefinitions_Shared/Audio.lua` `InitVoiceSettings`, entered once [settings access](voice-settings-access.md) answers true. Without these members it fails at Retail Audio.lua:179 (`GetDefaultOutputDevice`) with `attempt to call field 'GetAvailableOutputDevices' (a nil value)`.

## What it must do

- [x] Publish both members on Retail, PTR and Forever, whose pinned generated documentation declares them.
- [x] Return exactly one value: a fresh array of `VoiceAudioDevice` rows (`deviceID`, `displayName`, `isActive`, `isSystemDefault`, `isCommsDefault`), or nil when the supplied catalog is `None` (the result is declared `Nilable = true`).
- [x] Return a new table per call. Audio.lua sorts the result in place; the supplied catalog keeps its order.
- [x] The original `InitVoiceSettings` completes with the default catalogs and selects the "System Default" option on both dropdowns. Its device dropdowns, active-device getters and default-device values read supplied catalogs, and a `None` catalog selects the original `VOICE_CHAT_INVALID_DEVICE` option.

### Pinned declarations

Paths are relative to the active profile's `AddOns/Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua`. Each function has no arguments and one result `table`, `InnerType = "VoiceAudioDevice"`, `Nilable = true`.

| Profile (Gethe revision) | GetAvailableInputDevices | GetAvailableOutputDevices | VoiceAudioDevice |
|---|---|---|---|
| Retail `b9be937` | :116-123 | :125-132 | :1427-1437 |
| PTR `2cdbc7f` | :125-132 | :134-141 | :1512-1522 |
| Forever `ca8368f` | :125-132 | :134-141 | :1512-1522 |

## Default device evidence

No generated documentation, aso finding or in-game capture states the native lists, so the default follows the AGENTS.md best-guess rule. The warcraft.wiki.gg pages `API:C_VoiceChat.GetAvailableOutputDevices`, `API:C_VoiceChat.GetAvailableInputDevices` and `Structure_VoiceAudioDevice` (read 2026-10-30) give only the shape: no field descriptions or example values, and `isCommsDefault` added in 9.1.0.

- **Consumer code.** Retail Audio.lua:120-129 sorts `isSystemDefault` rows first and `isCommsDefault` second, and labels the system-default row with `VOICE_CHAT_OUTPUT_DEVICE_DEFAULT`/`VOICE_CHAT_INPUT_DEVICE_DEFAULT` (:208-219). Retail and Forever GlobalStrings give both as "System Default" (Forever GlobalStrings.csv IDs 36064-36065). The setting default is the system-default ID (:238, :307) and the current value is the `isActive` row (:175, :159).
- **Player reports.** In US forum topic 49806 "Voice chat output device" (2018-12-16), a player had voice-chat input and output both set to "system default" and relied on Windows' default communication device. In topic 2192152 "No Sound on WOW" (reply of 2025-11-26), a player reports that the voice-chat device list shows other speakers besides the game's System Default and NoSound outputs. So real lists hold the system default plus machine-specific physical devices. Troubleshooting guides found by search also recommend System Default for both voice devices; those were seen only as search summaries.
- **Voice backend.** Vivox SDK documentation describes virtual "Default System Device" and "Default Communication Device" entries in the available-device lists, matching `isSystemDefault` and `isCommsDefault`. That WoW's voice proxy uses Vivox is outside citation here, and this too was reached only through search summaries.

The default row uses `deviceID`/`displayName` "Default System Device" and `isActive`/`isSystemDefault` true. The display name is never shown, because Audio.lua substitutes the "System Default" label. Physical devices and a communication-default row are machine-specific (the latter is Windows-specific) and are not invented.

## Implementation inventory

- `src/c_api/c_voice_chat_devices.rs`: `VoiceAudioDevice`, profile-gated registration and the two queries.
- `src/lua_api/sim_substates/mod.rs`: catalogs on `VoiceChatState`, initially the best-guess system default.
- `src/c_api/registration.rs`, `src/c_api/mod.rs`: registration wiring.

## Tests asserting this spec

- `tests/voice_audio_devices.rs`: runs the selected original Audio polling callback unhooked, so the real `InitVoiceSettings` consumes the catalogs; checks the original settings' values, defaults and dropdown options for the default, supplied, empty and `None` catalogs; checks the documented result shape and absence on other profiles.
- `tests/voice_audio_devices/observe.lua`: reads the original settings and dropdown initializers. No vendor callee is replaced.
- `tests/voice_settings_access/original_callback.lua`: exposes the located original callback to the device test.

## Known gaps (current cycle)

- [ ] Replace the best-guess default with aso or in-game evidence of a native list.
- [ ] Forever: the shape test passes, but the three consumer tests stop before the device code. Forever Audio.lua:270-271 registers the `voiceProvider` dropdown with `VOICE_CHAT_SERVICE`, which the Forever global strings (build 69913) lack, so `Blizzard_Setting.lua:34` rejects the nil name. That is a separate data gap; Forever `upstream/master` already has six unrelated `voice_` failures.
- [ ] Run the consumer tests under PTR; its declaration is cited but its runtime was not executed in the proving environment.
- [ ] Mists, Era and Anniversary ship Audio.lua and VoiceChatDocumentation.lua, but their pinned contents were not checked; the members stay unpublished there.

## Out of scope

Native device enumeration, `SetInputDevice`/`SetOutputDevice`, device-change events, admin inputs and voice transport.
