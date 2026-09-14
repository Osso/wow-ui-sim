# Combat-audio speaker-volume state

Historical retail 12.0.0 contracts declare `C_CombatAudioAlert.GetSpeakerVolume() -> number` and `SetSpeakerVolume(newVal: number) -> success: boolean`, without a category argument. The pinned declarations live in `data/patch-api/sources/12.0.0-register.json`.

## What it must do

- Store explicit and repeated numeric writes independently per simulator environment.
- Return one number from the getter and one boolean `true` from accepted numeric writes.
- Preserve speaker-speed state when volume changes and volume state when speed changes.

Initial volume `100.0` and accepted-write `true` are simulator policies, not verified native defaults or changed-versus-accepted semantics. Numeric input is required; no native range or coercion contract is claimed.

## How it works

- [Related setting-state boundaries](combat-audio-speaker-speed.md).

## Implementation inventory

- `src/c_api/c_combat_audio_alert.rs`: getter/setter under existing retail-12-0-0 registration.
- `src/lua_api/state/sim_state.rs`: independent numeric volume field.
- `src/lua_api/state.rs`: per-environment initialization.

## Tests asserting this spec

Three grouped `audio_speaker_volume_*` tests in `tests/c_namespace_noop_replacements.rs` cover explicit/repeated writes, return types/arity, speed independence and distinct environments. At tests-only revision `4b5ac2a4a`, all three failed because the setter returned nil instead of boolean success. At runtime revision `cfad799bb`, targeted retail 12.0.0 GREEN passed 3/3 (exit 0); ledger: `/tmp/audio-speaker-volume-green-ledger.json`. Independent profile verification remains pending.

## Known gaps (current cycle)

- [ ] Independent profile verification and audit evidence integration remain outside this runtime slice.

## Out of scope

Native defaults, units, ranges, coercion/error details and success semantics remain unproven. No category APIs, CVar coupling, callbacks, persistence, playback or security behavior is modeled or credited. Source declarations do not establish loaded consumer execution or earlier-profile availability.
