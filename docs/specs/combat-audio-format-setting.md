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

## Evidence

Tests `70979713d` reached RED 0/3 because the setter returned one nil. Independent proof at `b36c27ce3` passes 3/3 on each retail target: 12.0.0 is exact-byte reused; 12.0.5 and 12.0.7 are fresh. Formatter, check, default build, startup (`[]`) and readability gates pass. Final metadata proof `/tmp/verify-audio-format-setting-metadata-ledger.json` at `19c423c5f` records 14,976 fresh hashes, zero stale, 43 renewals, 12 additions, validator exit 0 and 3,410 matching rows.

## Out of scope

Native defaults, enum/range validation, coercion/errors, success semantics, CVar aliasing, actual formatting, callbacks, persistence, playback and security remain unresolved; this slice models ordinary stored settings only.
