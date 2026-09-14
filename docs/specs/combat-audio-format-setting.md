# Combat-audio format-setting state

Pinned `data/patch-api/sources/12.0.0-register.json` declares `GetFormatSetting(unit, alertType) -> number` and `SetFormatSetting(unit, alertType, newVal:number) -> success:boolean`. Numeric enum IDs are Player=0, Target=1; Health=0, Cast=1.

## What it must do

- [ ] Preserve explicit and repeated numeric writes by unit/type pair independently per environment.
- [ ] Return one number from the getter and one boolean `true` for accepted writes (simulator policy).
- [ ] Preserve other pairs and independent speaker speed/volume state.

Numeric arguments are required. IDs use integer conversion; invalid/fractional/out-of-domain input behavior is not native-validated. An empty map and missing-key value `0` are simulator policies, not established native defaults.

## How it works

- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_combat_audio_alert.rs`: keyed setting accessors under existing registration.
- `src/lua_api/state/sim_state.rs`: per-environment numeric pair map.
- `src/lua_api/state.rs`: empty-map initialization.

## Tests asserting this spec

`tests/c_namespace_noop_replacements.rs`: three `audio_format_setting_*` tests at `70979713d`, RED 0/3 because the setter returned one nil. GREEN proof is recorded separately after the runtime commit.

## Known gaps (current cycle)

- [ ] Independent verification beyond the focused development run.

## Out of scope

Native defaults, enum/range validation, coercion/errors, success semantics, CVar aliasing, actual formatting, callbacks, persistence, playback and security remain unresolved; this slice models ordinary stored settings only.
