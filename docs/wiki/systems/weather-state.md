# Weather State

PTR weather uses simulator-owned state: `C_Weather.GetCurrentWeather()` returns a fresh `{ type, intensity }` snapshot, while `A_Admin.SetWeather` is a controlled test/admin mutation surface. `WEATHER_CHANGED` is registerable on PTR; explicit admin event injection is tested separately from mutation.

## Contract boundary

Pinned PTR documentation adds `C_Weather.GetCurrentWeather`, `WeatherInfo.type`, `WeatherInfo.intensity`, and `WEATHER_CHANGED`. It documents `type` defaulting to `Clear`, but does not establish returned initial intensity, transitions, event payloads, or timing.

## Simulator behavior

- PTR starts at Clear with intensity `0`; this is a simulator assumption.
- Each getter returns an independent table snapshot.
- `A_Admin.SetWeather(type, intensity)` accepts an integer `WeatherType` value and finite intensity, mutates only simulator state, and does not dispatch events.
- `A_Admin.FireEvent("WEATHER_CHANGED")` injects the registered event with zero arguments after mutation.
- Earlier retail explicitly keeps `C_Weather` absent despite namespace fallback.

## Limits

No native conformance claim covers weather selection, intensity defaults or range, automatic transitions, event timing or payload, rendering, coercion, taint, protected access, or gameplay effects.

## Sources

- [weather-state spec](../../specs/weather-state.md) — bounded simulator contract and assumptions.
- [PTR occurrence register](../../../data/patch-api/sources/12.1.5-register.json) — declared PTR surface.
- [weather implementation](../../../src/c_api/c_weather.rs) — state, snapshot, registration, and admin mutation.
- [focused weather tests](../../../src/loader/tests/wow_api_globals/patch_12_1_5_weather.rs) — observed simulator behavior.

## See Also

- [[patch-12-1-5-api-audit]] — occurrence-level disposition.
- [[event-system]] — event registration and dispatch.
