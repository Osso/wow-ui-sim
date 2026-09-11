# PTR SecondsFormatter.Format

PTR `C_StringUtil.CreateSecondsFormatter():Format(seconds, abbreviation?)` consumes the existing formatter configuration and renders duration units through the ICU4C C APIs. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) specifies a string result and changes the seconds argument's type alias; it does not establish the algorithm below. This document defines simulator assumptions, not native WoW conformance.

## What it must do

- [x] PTR returns exactly one localized duration string; earlier retail retains its existing `tostring(seconds or 0)` output, including ignored settings. That earlier output is a known baseline gap, not formatting conformance.
- [x] Consume only registered settings: minimum/maximum interval, maximum curve, desired unit count, default/explicit abbreviation, rounding, final-unit round-up permission, approximation seconds, and millisecond threshold. Existing configuration accessors/evaluators remain unchanged.
- [x] Use a private bootstrap argument captured by the formatter closure for native rendering. Do not expose a public helper global or C namespace method. Keep the callback alive through collection without pinning instances.
- [x] Render actual localized plural-sensitive second/minute/hour/day units with `unumf` measure-unit skeletons and join them with `ulistfmt` units. Reuse checked native buffers, explicit lengths, locale parsing, and cleanup; no C++ ABI or new dependency.
- [x] Validate input/settings and propagate configured curve/ICU errors without falling back to static settings or placeholder output.
- [x] Preserve per-instance state and the six existing configuration/evaluation tests, with only profile-specific placeholder assertions updated for the new PTR contract.

## Simulator policies

- Intervals are seconds/minutes/hours/days (`0..3`) with lengths `1/60/3600/86400`. Select the largest allowed interval fitting the magnitude, or the minimum if none fits. Desired count selects a contiguous interval window, capped at four. Omit zero components, but always render one unit for zero.
- Evaluate min/max/count at the selected magnitude; configured maximum curves are consulted on every call. A minimum larger than maximum is an error.
- Default rounding is Truncate (`1`). RoundUp (`0`) rounds the last selected unit upward only when `canRoundUpLastUnit` is true (default true). Earlier components use integer decomposition. Normalize carries into allowed larger units and reselect the window; do not exceed configured maximum.
- Abbreviation defaults to `None=0`. `None=0` and `Full=3` use ICU full unit names and wide unit lists; `OneLetter=1` uses ICU narrow width, `TwoLetters=2` uses ICU short width. These are width mappings, not guarantees of one/two literal letters in every locale. An explicit argument overrides the stored abbreviation.
- For `0 < seconds < approximationSeconds`, format the threshold magnitude with the literal prefix `< `. Equality is not approximate. Zero and negative inputs are not approximated.
- If the formatted magnitude is positive and below `millisecondsThreshold`, seconds in the last position retain up to three fractional digits (trailing zeros removed). Other intervals remain integral. The threshold is tested against the entire magnitude, not a residual component.
- Negative inputs format their absolute magnitude with a single literal `-` prefix, including a truncated `-0 seconds`. No special negative unit arithmetic or locale-specific prefix is claimed.
- Rendering follows current `GetLocale()` each call, including supported locale extensions. Numeric precision overflow and malformed settings fail explicitly. ICU/CLDR versions can change wording and spacing.

## How it works

- [Numeric configuration](seconds-formatter-configuration.md)
- [Native ICU boundary](intl-native-linking.md)

## Implementation inventory

- `src/c_api/seconds_formatter/format.lua`: private selection, decomposition, rounding, and prefix policy.
- `src/c_api/seconds_formatter/render.rs`: private Rust callback, part validation, and current locale lookup.
- `src/c_api/seconds_formatter.rs`: configuration integration and feature-scoped callback construction.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs`: bootstrap argument/rooting and existing factory.
- `src/c_api/intl_native.rs`, `src/c_api/intl_native/ffi.rs`: checked native duration-part call and output ownership.
- `native/intl/duration_units.c`, `native/intl/bridge.h`, `build/intl_native.rs`: ICU >=72 C formatter/list APIs and build inputs.

## Tests asserting this spec

- `tests/seconds_formatter_format.rs`: boundary values, window selection, settings, locale/width/plurals, validation, private lifetime, and retail baseline.
- `tests/seconds_formatter_configuration.rs`: six existing configuration/evaluation regressions.

Focused development proof at `59833ba76`: four new PTR tests failed against the prior placeholder. The grouped filters `seconds_formatter_format:: seconds_formatter_configuration::` then passed **11 PTR** and **8 retail** tests with `--test integration --offline --no-default-features --features sound,gui,client-<profile>`. Counts include four new PTR tests or one retail baseline test, all six existing configuration tests, and one matching existing garden-format test. Compiler output had no warnings. The matching garden test logs partial-addon Lua diagnostics also present in RED; this is not a clean full-startup claim. Logs: `/tmp/seconds-formatter-format-{red,59833ba76-ptr,59833ba76-retail}.log`. No final check/readability/smoke gates or audit artifact updates were run.

## Known gaps (current cycle)
- [ ] Native defaults, selection/rounding, approximation text, millisecond precision, abbreviation semantics, secret/taint behavior, and exact ICU data equivalence remain unverified.
- [ ] Numeric Step curves retain the existing engine limitation; Format propagates invalid fractional interval results rather than changing curve behavior.

## Out of scope

Vendor mixins, unregistered formatter methods, native-userdata conversion, Step-curve engine changes, native security enforcement, final check/readability/smoke gates, audit artifacts, deployment, and publishing.
