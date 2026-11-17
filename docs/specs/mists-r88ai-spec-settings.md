# Mists R88ai combat-audio spec settings

This spec covers Mists-only `C_CombatAudioAlert.GetSpecSetting` and `SetSpecSetting` in `src/c_api/mists_r88ai.rs`. The native answer is `pr46-49-decomp-answers.md` (untracked decomp handoff), section `R88ai` (Mists 5.5.4.70631).

## What it must do

- [x] **VERIFIED (R88ai):** `GetSpecSetting(setting)` returns exactly one number. `setting` is required; numbers and number-like strings are accepted, truncated toward zero, and only the low 8 bits are checked against 0–8 (`264` selects SayIfTargeted; `9` raises the usage error).
- [x] **VERIFIED (R88ai):** a nil, non-numeric or rejected `setting` raises an error containing the exact usage text; a value outside the signed 32-bit range raises an error containing `integer overflow attempting to store`. The caller-name prefix and `%f` rendering are not asserted because they are supplied at runtime.
- [x] **VERIFIED (R88ai):** all nine per-spec CVars default to empty, so every loaded position holds its absent value: Percent 0, Format 1, Voice 0, Volume 100, SayIfTargeted 1.
- [x] **VERIFIED (R88ai):** while the alert settings are not loaded, every setting reads 0 and `SetSpecSetting` returns false. The settings-loaded state is the same gate that makes `IsEnabled` false.
- [x] **VERIFIED (R88ai):** `SetSpecSetting(setting, newVal)` returns one boolean. `newVal` is required, must be a signed 32-bit number or number-like string, and is truncated. On range failure, tests assert the verified range text and usage text without asserting the caller-name wrapper.
- [x] **VERIFIED (R88ai):** Percent and Format accept a low byte of 0–5; Volume accepts 0–100; Voice accepts 0 up to the voice-list length; SayIfTargeted accepts a low byte of 0–3. On success the full 32-bit value is stored (`SetSpecSetting(8, 256)` then reads 256).
- [x] **VERIFIED (R88ai):** neither function fires a Lua event directly. The simulator emits no event.
- [x] **VERIFIED consumer path:** with `CAAEnabled=1`, the original `CombatAudioAlertManagerMixin:RefreshEvents` registers `UNIT_TARGET` when SayIfTargeted > 0 and `UNIT_POWER_UPDATE` when either Resource Percent > 0.
- [x] **INFERRED (R88ai, 70631 applied to the 70268 cache):** the Mists registration is enabled because the cached declarations, enums and default constants match.

## How it works

- [C API boundary and Lua API](../lua-api.md)
- [Mists R88ah C APIs](mists-r88ah-apis.md) (the shared alert-settings gate)

## Implementation inventory

- `src/c_api/mists_r88ai.rs` — registration, argument reading, validation and the in-memory per-spec lists.
- `src/c_api/helpers.rs` — low-byte enum argument reader shared with R88ah.
- `src/lua_api/state/sim_state.rs` — in-memory lists, the settings-loaded gate and the voice-list length host input.

## Tests asserting this spec

- `tests/mists_r88ai.rs` — original manager `RefreshEvents` registrations driven by spec settings, defaults, conversions, per-setting validation, unloaded settings and argument errors.

## Known gaps (current cycle)

- [ ] **VERIFIED (R88ai), not modelled:** on success `SetSpecSetting` writes the list back to the matching CVar. The simulator does not write the CVar, so `GetCVar` keeps its old text; values live in memory only. **UNKNOWN (R88ai):** the text encoding of the per-spec CVar lists, so the simulator also does not decode non-empty CVar text.
- [ ] **UNKNOWN (R88ai):** whether the CVar write reaches Lua as `CVAR_UPDATE`. No event is emitted.
- [ ] **UNKNOWN (R88ai):** the engine lifecycle that loads and unloads the alert settings; the gate is a neutral host input, on by default.
- [ ] **UNKNOWN (R88ai):** the voice list's count and contents; the host input `mists_combat_audio_voice_count` is empty (0) by default.
- [ ] **INFERRED (R88ai):** the slot field is ChrSpecialization OrderIndex; the simulator maps active specialization index 1–4 to slots 1–4 and anything else to the fifth slot. The internal flag that forces the fifth slot is not modelled. Tests do not assert slot selection.
- [ ] **UNKNOWN (R88ai):** whether the pending script-error state can be set by the getter's lookups; not modelled. Secret-argument access mode for `setting` is not established; the shared reader uses the untainted-execution text, and tests do not assert it.
- [x] **Resolved by R88aj:** table-form `RegisterUnitEvent` (`RegisterForUnitHealth`, `Blizzard_CombatAudioAlertManager.lua:438`) now registers `UNIT_HEALTH` with an empty filter; see [RegisterUnitEvent unit arguments](register-unit-event-units.md).
