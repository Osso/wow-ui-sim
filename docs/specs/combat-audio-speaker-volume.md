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

Three grouped `audio_speaker_volume_*` tests in `tests/c_namespace_noop_replacements.rs` cover explicit/repeated writes, return types/arity, speed independence and distinct environments. Tests-only revision `4b5ac2a4a` reached RED 0/3 because the setter returned nil instead of boolean success. Runtime revision `cfad799bb` passed the focused retail 12.0.0 GREEN 3/3. Independent proof reuses exact 12.0.0 bytes and records fresh 12.0.5 and 12.0.7 runs, 3/3 each, plus fmt/check/build/startup (`[]`)/readability PASS: `/tmp/verify-audio-speaker-volume-ledger.json`.

Final metadata verification at `dcf344b01` records 14,964 fresh hashes, zero stale hashes, 31 renewals, 12 evidence additions, two bounded credits, and validator exit 0 with all 3,410 rows matching. Ledger: `/tmp/verify-audio-speaker-volume-metadata-ledger.json`.

## Out of scope

Native defaults, units, ranges, coercion/error details and success semantics remain unproven. No category APIs, CVar coupling, callbacks, persistence, playback or security behavior is modeled or credited. Source declarations do not establish loaded consumer execution or earlier-profile availability. Eight Mists AccountStore failures and the broad audit remain open.
