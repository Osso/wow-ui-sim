# Combat-audio speech requests

`C_CombatAudioAlert.SpeakText` records Blizzard combat-audio speech requests and publishes completion through `VOICE_CHAT_TTS_PLAYBACK_FINISHED`. Its current behavior is INFERRED (R88fc, native evidence pending); see the [event system](../event-system.md) for event dispatch details.

## What it must do

### API and request queue

- [x] Retail declares `SpeakText(text, category, allowOverlap = true) -> utteranceID:number`, non-nil (VERIFIED: `Blizzard_APIDocumentationGenerated/CombatAudioAlertDocumentation.lua:233-247`).
- [x] An accepted call records `text`, `category`, and `allowOverlap` in order. An omitted `allowOverlap` uses `true` (INFERRED, R88fc, native evidence pending).
- [x] Each environment returns increasing utterance IDs starting at `1` (INFERRED, R88fc, native evidence pending).
- [x] A completion event is sent on a later timer tick with exactly one numeric `utteranceID` payload (VERIFIED event shape: `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua:1298-1306`; timing INFERRED, R88fc, native evidence pending).
- [x] The provider records a request without synthesizing speech; the selected consumer plays its queued combat sound after the completion event.

### Profile scope

- [x] The Rust registration and replacement of the temporary `SpeakText` fallback apply only to Retail and WoW Forever.
- [ ] Other sound-driver defaults and other profile fallbacks remain unchanged at runtime on profiles outside Retail and WoW Forever.

## How it works

- [Event dispatch](../event-system.md)
- [C API boundary](../lua-api.md)

## Implementation inventory

- `src/c_api/c_voice_chat_speak.rs` — CAA request data, `SpeakText` registration, and request acceptance.
- `src/lua_api/state/sim_state.rs` — per-environment request IDs, request records, and pending completion IDs.
- `src/lua_api/env_runtime.rs` — deferred completion-event dispatch.
- `src/lua_api/globals/register.rs` — Retail and Forever registration.
- `src/lua_api/workarounds/temporary/sound_driver_defaults.rs` — profile-gated retirement of only the CAA `SpeakText` fallback.

## Tests asserting this spec

- `tests/combat_audio_manager_consumer.rs::combat_audio_speak_text_completes_two_play_sound_slashes` — original chat slash command, distinct IDs, deferred completion, and two different sound kits.

## Known gaps (current cycle)

- [ ] Native execution evidence is pending (R88fc); argument validation and completion timing remain inferred.
- [ ] Non-Retail and non-Forever fallback runtime behavior was not exercised; lane proof is compile-only on Mists and Wrath.

## Out of scope

- Audio synthesis, voice selection, speech duration, failure/cancellation events, and native overlap behavior.
- `C_Sound.IsPlaying`, `C_Sound.GetSoundScaledVolume`, and file muting have no selected consumers in the Retail or Forever caches and remain unchanged.
- `C_Sound.PlayItemSound` has two selected call sites, but remains audit-only because this lane is narrowed to the separate CAA speech lifecycle.
