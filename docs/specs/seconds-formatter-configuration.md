# SecondsFormatter numeric configuration

`C_StringUtil.CreateSecondsFormatter()` proxies retain independent approximation and milliseconds-threshold configuration. The model lives in `src/c_api/seconds_formatter.rs`; the existing simulator factory remains temporary. The four evaluators expose configured decisions without implementing time formatting.

## What it must do

- [x] Initialize each formatter's `GetApproximationSeconds()` and `GetMillisecondsThreshold()` results to numeric `0` (simulator assumption, not a documented native default).
- [x] `SetApproximationSeconds(seconds)` and `SetMillisecondsThreshold(threshold)` store finite Lua numbers independently, preserving zero, fractions, negative values, and repeated updates without clamping or conversion.
- [x] Each getter returns exactly one number; each setter returns no values.
- [x] Missing, nil, nonnumeric, NaN, and infinite setter arguments raise an error without changing either stored value. This validation policy is a simulator choice; numeric strings are not coerced.
- [x] Configuration remains intact while a formatter survives garbage collection and is isolated from other formatter instances.
- [x] Existing factory identity and stored setter values remain intact; maximum-mode switching is specified below. PTR `Format` is modeled separately in [duration formatting](seconds-formatter-format.md); earlier retail retains placeholder output.
- [x] Both current PTR and earlier retail expose these methods through the existing proxy lookup path.
- [ ] Re-registering `C_StringUtil` during post-EnvironmentCleanup restoration preserves the existing namespace and formatter factory, keeping public/secure references consistent and existing/new formatters usable. This preserves existing profile formatting behavior; it does not add a fallback or upgrade formatting semantics.

### Evaluation model (simulator assumptions)

- [x] `CanApproximate(s)` returns `s > 0 and s < approximationSeconds`. Equality at either boundary is false. This agrees with the separate Blizzard Lua mixin's ordinary predicate, but is not native-object conformance evidence.
- [x] `EvaluateMinInterval(s)` returns the configured minimum enum value; default `Seconds = 0`.
- [x] `EvaluateMaxInterval(s)` returns the configured maximum enum value; default `Days = 3`. A configured curve instead receives the exact `s` through normal `curve:Evaluate(s)` lookup on every call, including mutations after installation.
- [x] Static intervals are configured bounds, not the largest unit fitting `s`: no automatic 60/3600/86400 promotion, clamping, or rounding is modeled. Curve output must itself be an integral enum value `0..3`; fractional, nonfinite, missing, or out-of-range results raise errors. Curve lookup/call errors propagate; no static fallback is used.
- [x] `SetMaxInterval` selects static mode and clears a previously configured curve. `SetMaxIntervalCurve(nil)` selects the retained static maximum. These mode-switch choices, including the existing simulator setter's acceptance of nil, remain native-unverified.
- [x] `EvaluateDesiredUnitCount(s)` returns the configured positive integral count; default `1`, independent of `s`. Existing setters still store their inputs; invalid stored interval/count values fail at evaluation without mutation.
- [x] All four evaluators accept finite numeric seconds (including negative values) and return exactly one value. Invalid seconds/receivers raise errors. Milliseconds threshold, abbreviation, rounding, and final-unit flags do not affect these configuration queries.
- [x] Evaluators share existing approximation state and proxy fields; instances remain independent. Existing accessors remain unchanged; profile-specific `Format` behavior is specified separately.

The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) changes the four configuration accessor types and four evaluator argument types from `DurationSecondsDouble` to `Seconds`; it does not add the methods or specify default values or value bounds. Both setters retain `SecretArguments = AllowedWhenUntainted`; the four evaluators retain it and `ConstSecretAccessor = true`. Publication follows the existing cross-profile factory, not a new PTR-only gate.

## How it works

- [C API architecture](../lua-api.md) — simulator API surface.
- [Frame data flow](../frame-data-flow.md) — Lua method lookup context.

## Implementation inventory

- `src/c_api/seconds_formatter.rs` — private per-proxy numeric configuration, validation, and eight configuration/evaluation methods.
- `src/c_api/mod.rs` — internal module wiring.
- `src/c_api/c_string_util.rs` — namespace registration must retain the already-installed formatter factory.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs` — temporary factory integration; retire this connection when the modeled formatter owns the complete factory.

## Tests asserting this spec

- `tests/seconds_formatter_configuration.rs` — independent storage, arity, updates, atomic validation, GC retention, unchanged existing methods, evaluation boundaries, and real/proxy curve dispatch/errors; grouped `integration` target.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs::tests::installs_proxy_factories` — existing factory regression.
- `src/lua_api/workarounds/temporary/environment_cleanup_restore.rs::tests::post_cleanup_restore_preserves_seconds_formatter_namespace_and_factory` — initial availability, public/secure identity, and numeric formatting before/after the actual restoration entry point. External RED confirms the bootstrap snapshot formats `12` while post-cleanup public factory is nil and namespace identity differs; compiled GREEN remains pending.

Focused development proof at `89fced131`: three new tests failed before implementation; the complete `seconds_formatter_configuration::` group passes six tests per profile with `--test integration --offline --no-default-features --features sound,gui,client-<ptr|retail>`. This includes the three existing configuration regressions. No broad, check, readability, or audit-artifact gates were run.

## Known gaps (current cycle)

- [ ] Native defaults, validation/coercion, `Seconds` representation, and secret/taint enforcement remain unverified.
- [ ] Native defaults, time-unit selection, curve-output rounding, and desired-count policy remain unverified. PTR `Format` and millisecond display use the separate [modeled formatting policy](seconds-formatter-format.md); earlier retail still has placeholder output.
- [ ] Existing numeric curves currently interpolate linearly even when configured as Step. These evaluators call that existing engine unchanged and explicitly reject a fractional interval result. Vendor AuraContainer's Step curve therefore still requires a separate curve-engine correction; no broader redesign was attempted.

## Out of scope

Vendor or `SecondsFormatterMixin` changes, native-userdata conversion, additional curve setters, native Step-curve correction, remaining formatter methods, audit-artifact credit, and broad/final verification gates.
