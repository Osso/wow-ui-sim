# SecondsFormatter numeric configuration

`C_StringUtil.CreateSecondsFormatter()` proxies retain independent approximation and milliseconds-threshold configuration. The model lives in `src/c_api/seconds_formatter.rs`; the existing simulator factory remains temporary. This slice does not implement time formatting.

## What it must do

- [x] Initialize each formatter's `GetApproximationSeconds()` and `GetMillisecondsThreshold()` results to numeric `0` (simulator assumption, not a documented native default).
- [x] `SetApproximationSeconds(seconds)` and `SetMillisecondsThreshold(threshold)` store finite Lua numbers independently, preserving zero, fractions, negative values, and repeated updates without clamping or conversion.
- [x] Each getter returns exactly one number; each setter returns no values.
- [x] Missing, nil, nonnumeric, NaN, and infinite setter arguments raise an error without changing either stored value. This validation policy is a simulator choice; numeric strings are not coerced.
- [x] Configuration remains intact while a formatter survives garbage collection and is isolated from other formatter instances.
- [x] Existing factory identity and other setter behavior remain intact. `Format` retains its existing placeholder behavior and does not consume these fields.
- [x] Both current PTR and earlier retail expose these methods through the existing proxy lookup path.

The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) changes all four numeric parameter/return types from `DurationSecondsDouble` to `Seconds`; it does not add the methods or specify default values or value bounds. Both setters retain `SecretArguments = AllowedWhenUntainted`. Publication follows the existing cross-profile factory, not a new PTR-only gate.

## How it works

- [C API architecture](../lua-api.md) — simulator API surface.
- [Frame data flow](../frame-data-flow.md) — Lua method lookup context.

## Implementation inventory

- `src/c_api/seconds_formatter.rs` — private per-proxy numeric configuration and four methods.
- `src/c_api/mod.rs` — internal module wiring.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs` — temporary factory integration; retire this connection when the modeled formatter owns the complete factory.

## Tests asserting this spec

- `tests/seconds_formatter_configuration.rs` — independent storage, arity, updates, atomic validation, GC retention, and unchanged existing methods; grouped `integration` target.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs::tests::installs_proxy_factories` — existing factory regression.

## Known gaps (current cycle)

- [ ] Native defaults, validation/coercion, `Seconds` representation, and secret/taint enforcement remain unverified.
- [ ] Native formatting, approximation decisions, milliseconds display, and output effects remain unmodeled by this slice.

## Out of scope

Vendor or `SecondsFormatterMixin` changes, native-userdata conversion, remaining formatter methods, audit-artifact credit, and broad/final verification gates.
