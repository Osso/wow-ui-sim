# SecondsFormatter numeric configuration

`C_StringUtil.CreateSecondsFormatter()` creates opaque C API-owned formatter handles with independent approximation and milliseconds-threshold configuration. The model lives in `src/c_api/seconds_formatter.rs`; `930726316` replaces the former temporary SecondsFormatter factory with private formatter state and shared ICU duration rendering where the native-duration feature is enabled. The four evaluators expose configured decisions; formatter rendering is specified separately.

## What it must do

- [x] Initialize each formatter's `GetApproximationSeconds()` and `GetMillisecondsThreshold()` results to numeric `0` (simulator assumption, not a documented native default).
- [x] `SetApproximationSeconds(seconds)` and `SetMillisecondsThreshold(threshold)` store finite Lua numbers independently, preserving zero, fractions, negative values, and repeated updates without clamping or conversion.
- [x] Each getter returns exactly one number; each setter returns no values.
- [x] Missing, nil, nonnumeric, NaN, and infinite setter arguments raise an error without changing either stored value. This validation policy is a simulator choice; numeric strings are not coerced.
- [x] Configuration remains intact while a formatter survives garbage collection and is isolated from other formatter instances.
- [x] Formatter identity and stored setter values remain intact; maximum-mode switching is specified below. Retail 12.0.5+, PTR, and Forever formatting use the shared native-duration backend; profiles without that feature retain their existing placeholder behavior. Formatting policy is specified separately in [duration formatting](seconds-formatter-format.md).
- [x] Both current PTR and earlier retail expose these methods through the existing proxy lookup path.
- [x] Re-registering `C_StringUtil` during post-EnvironmentCleanup restoration preserves the existing namespace and formatter factory, keeping public/secure references consistent and existing/new formatters usable. This preserves existing profile formatting behavior; it does not add a fallback or upgrade formatting semantics.

### Interval whitespace modes

Pinned current Retail, PTR, and Forever `SecondsFormatterSharedDocumentation.lua` declare `Preserve=0`, `Strip=1`, and `StripIgnoreLocale=2`. `SecondsFormatterAPIDocumentation.lua` defines the proxy setter/getter argument/result as this enum, not a boolean. Native `SecondsFormatterMixin` is a separate Lua utility with a boolean setting; it is unchanged.

- [x] Publish `Enum.SecondsFormatterIntervalWhitespace` and its `EnumMeta` bounds `0/2/3` from `c_api::seconds_formatter`, registered with `C_StringUtil`, for Retail 12.1+, PTR, and Forever. Preserve the historical Retail and classic publication surface.
- [x] Store validated numeric modes independently per formatter; setters return no values and getters return exactly one number. Preserve mode `0` as numeric zero, not Lua truthiness.
- [x] Reject nil, booleans, strings, fractional values, nonfinite values, and numbers outside `0..2` without changing the stored mode. This validation policy and initial `Preserve` mode are simulator guesses, not native-tested defaults/coercion.

The real ActionBarAuras RED in `/tmp/forever-addon-runtime/main-batch-000-032.json` reaches `Core.lua:99` with the enum absent. The earlier slice added publication and configuration state only. At `55ee20d7`, the isolated replay in `/tmp/forever-addon-runtime/actionbarauras-native-formatter-malg6_wg/ledger.json` advances past the `NumericFormatter` validation and prior `pairs(nil)` boundary, then fails during `Core.lua:37` because `C_Spell.GetBaseSpell` produces a nil spell ID. It remains a failed startup, not an interaction acceptance. The integrating caller must separately prove observable unit-string whitespace behavior, including locale override versus `StripIgnoreLocale`.

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

- `src/c_api/seconds_formatter.rs` — opaque formatter factory, private configuration, validation, configuration/evaluation methods, and native-duration rendering hookup.
- `src/c_api/seconds_formatter/{configuration.lua,format.lua,render.rs,units.rs}` and `src/c_api/native_icu.rs` — private handle behavior and shared ICU duration rendering.
- `src/lua_api/env_init/mod.rs` — formatter bootstrap wiring.
- `src/c_api/c_string_util.rs` — namespace and formatter-owned enum registration; it does not own the formatter factory.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs` — unrelated temporary proxy factories remain there; it no longer implements SecondsFormatter.

## Tests asserting this spec

- `tests/seconds_formatter_configuration.rs` — independent storage, arity, updates, atomic validation, GC retention, evaluation boundaries, and curve dispatch/errors; grouped `integration` target.
- `tests/seconds_formatter_native.rs` — opaque identity, `FormatNumber`, actual AuraContainer binding, modeled duration text, locale whitespace, validation, and bounded secret/curve handoffs.
- `src/lua_api/workarounds/temporary/environment_cleanup_restore.rs::tests::post_cleanup_restore_preserves_seconds_formatter_namespace_and_factory` — initial availability, public/secure identity, and existing/new formatter behavior across the actual restoration entry point.

Focused development proof at `89fced131`: three new tests failed before implementation; the complete `seconds_formatter_configuration::` group passes six tests per profile with `--test integration --offline --no-default-features --features sound,gui,client-<ptr|retail>`. This includes the three existing configuration regressions. No broad, check, readability, or audit-artifact gates were run.

At `c1e830ffa`, before the opaque-handle implementation, the isolated Forever integration build and `seconds_formatter_whitespace` filter passed 1/1. The selected Forever filter did not execute the cfg-excluded older-profile publication test. That evidence remains valid for the earlier enum/configuration slice only.

At `55ee20d7`, isolated Forever focused integration filters recorded `seconds_formatter_native::` 7/7, `seconds_formatter_configuration::` 7/7, `duration_text_binding_copy::` 11/11, and `numeric_rule_formatter::` 8/8 passing in `/tmp/forever-addon-audit/verify-930726316-focused-integration-ledger.json` and its linked stdout files. After the rilua pin change, the same Forever groups passed again at `9e20a29d`; the profile ledger additionally compiled the PTR integration target and passed PTR native formatter 3/3 plus configuration 7/7. This is GREEN evidence for the modeled opaque-handle, formatter binding, secret-handoff, and numeric-regression behaviors; it is not native conformance or ActionBarAuras major-interaction acceptance. Evidence: `/tmp/forever-addon-audit/verify-b8-{focused,profile}-ledger.json`.

After the import-only `0e23609d6`, the isolated library restoration test passed 1/1 and `installs_proxy_factories` passed 1/1. At `350f5444a`, the integration, library, and `wow-sim` targets compiled; third-party LoD passed 3/3 and all `addon_loading::tests::` passed 8/8. Forever `cargo check --offline --no-default-features --features gui,client-wowforever` and `cargo fmt --check` passed. Evidence: `/tmp/forever-addon-audit/verify-0e23609d6-{integration-no-run,lib-no-run,wow-sim-no-run}-ledger.json`, `/tmp/forever-addon-audit/verify-0e23609d6-cleanup-direct.stdout`, `/tmp/forever-addon-audit/verify-350f5444-direct-tests-ledger.json`, and `/tmp/forever-addon-audit/verify-350f5444-cargo-check-ledger.json`.

## Known gaps (current cycle)

- [x] Compile the PTR integration target and run focused PTR native formatter/configuration checks; default `cargo check` also passes. These are profile-preservation checks, not complete PTR/default behavioral coverage.
- [x] Diagnose ActionBarAuras's post-formatter `C_Spell.GetBaseSpell` nil spell-ID failure. Its `b8f0982be` startup replay is clean, but a real aura-duration workflow remains unproven.
- [ ] Native defaults, validation/coercion, exact locale/unit formatting, `Seconds` representation, opaque-handle identity, and secret/taint enforcement remain unverified despite bounded simulator tests. In particular, the modeled `FormatNumber` secret-input route rejects tainted callers; this is an explicit simulator limitation, not a claim to match the API's allowed-tainted native contract.
- [ ] Native defaults, time-unit selection, curve-output rounding, and desired-count policy remain unverified. PTR `Format` and millisecond display use the separate [modeled formatting policy](seconds-formatter-format.md); only profiles outside the native-duration capability retain placeholder output; the default retail profile now inherits the 12.0.5 promotion, pending parent GREEN.
- [ ] Existing numeric curves currently interpolate linearly even when configured as Step. These evaluators call that existing engine unchanged and explicitly reject a fractional interval result. Vendor AuraContainer's Step curve therefore still requires a separate curve-engine correction; no broader redesign was attempted.

## Out of scope

Vendor or `SecondsFormatterMixin` changes, native-userdata conversion, additional curve setters, native Step-curve correction, remaining formatter methods, audit-artifact credit, and broad/final verification gates.
