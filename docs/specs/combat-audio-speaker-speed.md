# Combat-audio speaker-speed state

Pinned retail 12.0.0 contracts declare `GetSpeakerSpeed() -> number` and `SetSpeakerSpeed(number) -> success:boolean`. Unmodified settings proxies and the combat-audio manager query the getter and call the setter for changed inputs.

## Modeled contract

- Store explicit numeric writes independently per simulator environment, including repeated writes.
- Getter returns one number; an accepted numeric write returns one boolean `true`.
- Simulator initial value is `0`. Neither this initial policy nor accepted-versus-changed success semantics is established as native behavior.
- Numeric input is required; native bounds, coercion, non-finite behavior and exact errors remain unproven. Proof is limited to the ordinary finite values exercised by tests.
- Publication begins at the retail 12.0.0 feature boundary.

## State boundary

`SimState.combat_audio_speaker_speed` is a numeric field accessed by `src/c_api/c_combat_audio_alert.rs`. It does not alias the `CAASpeed` CVar or invoke audio playback. The CVar name/default and generic sample callback registration do not prove API/CVar state identity.

## Evidence

Three grouped `audio_speaker_speed_*` tests at `87adb4882` reached RED 0/3. The observed boundary was the initial setter returning a value that did not satisfy boolean/accepted-write expectations, not a missing-method exception. Getter, repeated-write and later isolation assertions were blocked behind that failure. Post-implementation verification pending.

## Gaps

Native defaults/ranges/success semantics, CVar coupling, callbacks, persistence, reset/lifecycle, category settings, actual audio, throttling and secret/taint enforcement remain unproven. Source consumer callsites are not proof of loaded consumer execution.
