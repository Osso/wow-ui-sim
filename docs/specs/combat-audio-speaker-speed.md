# Combat-audio speaker-speed state

Pinned retail 12.0.0 contracts declare `GetSpeakerSpeed() -> number` and `SetSpeakerSpeed(number) -> success:boolean`. Unmodified settings proxies and the combat-audio manager query the getter and call the setter for changed inputs.

## Modeled contract

- Store explicit numeric writes independently per simulator environment, including repeated writes.
- Getter returns one number; an accepted numeric write returns one boolean `true`.
- Simulator initial value is `0`. Neither this initial policy nor accepted-versus-changed success semantics is established as native behavior.
- Numeric input is required; native bounds, coercion, non-finite behavior and exact errors remain unproven. Proof is limited to the ordinary finite values exercised by tests.
- Modeled Rust registration is gated at `retail-12-0-0`. Earlier-profile generic fallback callability or absence is unproven.

## State boundary

`SimState.combat_audio_speaker_speed` is a numeric field accessed by `src/c_api/c_combat_audio_alert.rs`. It does not alias the `CAASpeed` CVar or invoke audio playback. The CVar name/default and generic sample callback registration do not prove API/CVar state identity.

## Evidence

Three grouped `audio_speaker_speed_*` tests at `87adb4882` reached RED 0/3. The observed boundary was the initial setter returning a value that did not satisfy boolean/accepted-write expectations, not a missing-method exception. Getter, repeated-write and later isolation assertions were blocked behind that failure. Independent `/tmp/verify-audio-speaker-speed-ledger.json` at `df3137a23` records fresh 3/3 each on retail 12.0.0/12.0.5/12.0.7, plus fmt/check/build/startup (`[]`)/readability PASS. Exact current runtime/test hashes match that proof; no tests or runtime gates were rerun for metadata. Two bounded credits yield 2322 best-effort / 1086 evidence-required / 2 exceptions. Committed metadata inspection at `7f8ecd69c` records 14,952 fresh hashes, zero stale, 21 renewals and 16 additions. Validator ran once: exit 0, all 3,410 rows match the committed checklist. Ledger: `/tmp/audio-speaker-speed-metadata-ledger.json`.

## Gaps

Native defaults/ranges/success semantics, CVar coupling, callbacks, persistence, reset/lifecycle, category settings, actual audio, throttling and secret/taint enforcement remain unproven. Source consumer callsites are not proof of loaded consumer execution.
