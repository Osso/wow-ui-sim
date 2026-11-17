# Combat audio spec-setting enum

This spec covers `Enum.CombatAudioAlertSpecSetting` and its metadata table. It records the profile-specific publication contract for the Blizzard Combat Audio Alerts consumer.

## What it must do

- [x] **VERIFIED (Retail 12.1.5.70783, PTR 12.1.7.70493, Forever 1.60.1.70707, Mists 5.5.4.70268, Era 1.15.9.70003, Anniversary 1.15.9.70003; cached `CombatAudioAlertSharedDocumentation.lua`):** the enum has `Resource1Percent=0`, `Resource1Format=1`, `Resource1Voice=2`, `Resource1Volume=3`, `Resource2Percent=4`, `Resource2Format=5`, `Resource2Voice=6`, `Resource2Volume=7`, and `SayIfTargeted=8`; metadata is `MinValue=0`, `MaxValue=8`, `NumValues=9`.
- [x] **VERIFIED (Mists 5.5.4.70631, R88ai):** the native enum publication has the same nine members and metadata.
- [x] Retail, PTR, Forever, and Mists publish the documented table; `tests/combat_audio_spec_setting_enum.rs` checks it in each runtime gate. Mists also checks the C API publication in `tests/mists_r88ai.rs`.
- [ ] **UNKNOWN (Wrath):** no cached Combat Audio Alerts API documentation is available. Leave both enum tables unpublished; do not infer values. The Wrath lane compiles only.

## How it works

- [Client profiles](client-profiles.md)
- [C API and Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/combat_audio_spec_setting_enum.rs` — publication for profiles with cached enum declarations.
- `src/c_api/mists_r88ai.rs` — Mists enum publication with the Mists API.
- `src/c_api/registration.rs` — registers the profile-backed enum before the Blizzard add-on load.
- `src/lua_api/globals/enum_data/missing_enums.lua` — no guessed Combat Audio Alerts fallback.

## Tests asserting this spec

- `tests/combat_audio_spec_setting_enum.rs` — exact enum members and metadata on the gated profiles that use the profile-scoped publisher.
- `tests/mists_r88ai.rs` — Mists enum and setting behavior.
- `tests/ptr_combat_audio_manager_consumer.rs` — original PTR manager behavior with a callable-wrong API overlay.

## Known gaps (current cycle)

- [ ] **UNKNOWN (Wrath):** keep the enum unpublished. The supplied Wrath cache has no Combat Audio Alerts documentation or manager file; the Wrath gate compiles only.
- [ ] **UNKNOWN (Retail API-epoch-only builds without `client-retail`):** no matching profile cache evidence is in scope, so keep the enum unpublished.
- [ ] Era and Anniversary caches declare the same table, but those profiles are not built or tested under the current gate policy.

## Out of scope

- `C_CombatAudioAlert` behavior outside the enum publication.
