# Weather state

The PTR weather surface exposes simulator-owned weather state through `C_Weather.GetCurrentWeather` and `A_Admin.SetWeather`. Source lives in `src/c_api/c_weather.rs`; [weather system documentation](../wiki/systems/weather-state.md) is maintained separately.

## What it must do

### Pinned source contract

Source: `data/patch-api/sources/12.1.5-register.json`, Gethe base `a89e9d0ceb7f6cd31e8fc5ca7df1a338ac0b1b58` and target `49b69918fcdc77e109813281e4f537d45ec7dcbf`.

- [ ] PTR publishes `C_Weather.GetCurrentWeather()` returning a table with non-nil `type: WeatherType` and `intensity: number`; the structure documents `type` default `Clear`.
- [ ] Earlier retail leaves `C_Weather` absent, including normal lookup through namespace fallback.
- [ ] PTR accepts registration for the added `WEATHER_CHANGED` event.

### Chosen simulator behavior (not native conformance)

- [ ] Each simulator starts at Clear (`0`), intensity `0`. Zero intensity is an assumption; the source gives no intensity default.
- [ ] Each getter call returns an independent table snapshot. Editing returned fields cannot change backing state.
- [ ] PTR-only `A_Admin.SetWeather(type, intensity)` changes state without dispatching or queuing events.
- [ ] Admin input requires a numeric integer WeatherType value `0..4` and finite numeric intensity. Invalid input fails before mutation. This is simulator validation, not native coercion or range behavior; finite intensities outside `0..1` remain accepted.
- [ ] An explicit `A_Admin.FireEvent("WEATHER_CHANGED")` invokes a registered handler with zero injected arguments; the handler reads already-mutated state. No native event argument or timing claim follows.
- [ ] No global `WeatherInfo` constructor/table is fabricated: the documented structure describes returned fields.

## How it works

- [Weather model and registration](../wiki/systems/weather-state.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/c_api/c_weather.rs`: state, fresh-table getter, admin mutation, profile registration.
- `src/c_api/mod.rs`: namespace registration wiring.
- `src/lua_api/state/sim_state.rs`: per-simulator weather ownership.
- `src/lua_api/state.rs`: initial weather state.
- `src/lua_api/globals/admin.rs`: admin setter registration.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua`: preserves explicitly absent namespaces.
- `src/event/valid_events.rs`: PTR weather event registration.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_weather.rs`: defaults, mutation, independent snapshots, validation, explicit event observation, retail absence.
- `src/loader/tests/wow_api_globals/mod.rs`: existing grouped library test wiring.

## Known gaps (current cycle)

- [ ] Native initial intensity, supported intensity range, and returned-table identity remain unobserved.
- [ ] Native `WEATHER_CHANGED` arguments and timing remain unobserved; the pinned declaration supplies no payload.

## Out of scope

Automatic weather transitions, zone selection, rendering, event scheduling, native coercion, secret/taint/protected access, and gameplay effects require separate evidence or modeling. This slice supplies controlled state only.
