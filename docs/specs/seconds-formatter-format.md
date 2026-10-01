# SecondsFormatter handles and duration formatting

`src/c_api/seconds_formatter.rs` owns `C_StringUtil.CreateSecondsFormatter`. Retail 12.0.5+ (including default retail), PTR, and Forever use the existing ICU duration-unit backend through a private `native-duration-formatting` capability. Other profiles retain their previous formatting output; no failed native operation falls back to that output. This is a bounded simulator model, not native-client conformance.

## What it must do

- [ ] Return opaque userdata with protected, read-only methods and independent configuration. `securecopy` preserves the same handle, including configuration and retained curve references through GC. Borrowed methods and cloned/foreign userdata cannot impersonate a registered instance.
- [ ] Support the `NumericFormatter.FormatNumber` interface required by `ProcessCustomAuraButtonDurationTextOptions`, without changing that validator or accepting a generic table in place of the handle.
- [ ] On Retail 12.0.5+, PTR, and Forever, publish `None=0`, `Truncate=1`, `OneLetter=2`, with abbreviation metadata `0/2/3`; reject value `3`. The prior default-retail four-member publication was simulator drift, not native evidence. Profiles outside this capability are not upgraded by this slice.
- [ ] Format real localized second/minute/hour/day units using the existing interval-selection/rounding Lua code and ICU `unumf`/`ulistfmt`. Consume min/max interval, maximum curve, desired count, abbreviation, rounding, approximation, millisecond threshold, and whitespace mode.
- [ ] Strip Unicode whitespace inside individual formatted unit strings for mode `Strip`, except documented German/Russian locale families; `Preserve` leaves units unchanged and `StripIgnoreLocale` strips even those families. Keep ICU's inter-unit list separators.
- [ ] Accept only authenticated wrapped numeric format inputs from untainted callers. Return wrapped text for wrapped input; tainted calls cannot decode it through the formatter. Keep wrapped time at configurable curve callbacks and never pass decoded time to addon conversion overrides.
- [ ] Native AuraContainer option copying and `SetDurationText` retain the formatter. A real binding/FontString follows remaining time through one minute, five seconds, and sub-second values; secret-derived text retains its existing read guard.
- [ ] Reject invalid format numbers and receivers without changing configuration or visible text; reject invalid whitespace-mode writes atomically. Preserve existing deferred validation for older interval/count setters. Curve and ICU failures propagate; there is no placeholder/error-to-zero rendering path when the native-duration capability is enabled.
- [ ] Share only the private native backend capability. Retail 12.0.5/default retail and Forever must not publish `C_Intl` or enable the complete `retail-12-1-5` API epoch.

## Evidence and explicit simulator guesses

Pinned Forever `SecondsFormatterSharedDocumentation.lua` declares the three abbreviation values and whitespace values `Preserve=0`, `Strip=1`, `StripIgnoreLocale=2`. Its Strip description names `deDE` and `ruRU` as examples of locale overrides. The generated SecondsFormatter documentation declares a userdata script object. Unchanged ActionBarAuras file `8920553`, `Core.lua:94–120`, constructs this object and passes it as `textFormatter`; the native AuraContainer field requires `NumericFormatter` with `FormatNumber`.

Retail 12.0.5 source `data/patch-api/sources/12.0.5-api-changes.txt:19` explicitly names SecondsFormatter among the three duration-formatting objects. Cached `NumericFormatterAPIDocumentation.lua` supplies the common `FormatNumber(number) -> string` contract; this is source-backed support, not native-probe evidence or a claim about declared inheritance. Active retail `SecondsFormatterSharedDocumentation.lua` declares `None=0`, `Truncate=1`, `OneLetter=2` and metadata `0/2/3`, matching the shared native backend. Promoting that private capability into cumulative `retail-12-0-5` uses the retained 12.0.5 introduction as epoch evidence; exact historical enum wording and ICU output are not native-probed. At `9a50d8a5c`, `/tmp/patch-12.0.5-batch4-seconds-native-red.log` records `seconds_formatter_format_retail_epoch_formats_numeric_duration_units` failing with `expected duration units, got 93`. Post-promotion GREEN is pending parent compilation. User-run Forever probes are unavailable. The following remain explicit simulator guesses:

- Applying the documented `deDE`/`ruRU` exceptions to their language families, and stripping for other languages, is not a complete native exception list. Stripping applies to Unicode whitespace within unit strings, not ICU's list separator.
- Authenticated wrapped format input produces wrapped text. An authenticated wrapped maximum-curve result also keeps the final text wrapped. Scalar evaluator reads require authorization when decoding wrapped inputs/results. Secret configuration values are rejected rather than silently decoded into ordinary settings; full native secret-configuration semantics are not implemented.
- Existing interval/default/rounding/negative/approximation policies below are retained, not claimed native behavior.

Configuration resides in a private weak-key Lua table keyed by userdata. Captured host primitives allocate and write this native-owned storage without turning settings into addon-owned Lua slots. These primitives do not unwrap configuration values, clear stack taint, or alter callback closure taint. Format inputs use a separate authenticated numeric decoder. The factory no longer resides in the temporary proxy-factory bootstrap. Duration consumers retain only the private identity-validator and FormatNumber closures, not a configuration table or public method lookup; derived secret time stays wrapped at entry and at curve callbacks. See [common duration formatting](duration-core.md#common-numeric-formatting). Profiles outside Retail 12.0.5+, PTR, and Forever retain a checked FormatNumber entry with primitive numeric text; its private host conversion authenticates wrappers and cannot invoke a number `__tostring` metamethod. Existing Format behavior is unchanged.

## Formatting policies retained from the PTR model

- Intervals `0..3` have lengths `1/60/3600/86400`. Select the largest allowed interval fitting the magnitude, or the minimum. Desired count selects a contiguous window capped at four; omit zero components but render one unit for zero.
- Evaluate min/max/count at the selected magnitude; consult configured maximum curves on every call. A minimum exceeding maximum is an error. Secret input remains wrapped when passed to a configurable curve; a curve that cannot consume it fails explicitly.
- Default rounding is Truncate (`1`). RoundUp (`0`) rounds the last selected unit upward when `canRoundUpLastUnit` is true (default true). Normalize carries into allowed larger units and reselect the window.
- Abbreviation defaults to `None=0`. ICU widths map to wide/full-name, short, and narrow for `0/1/2`. These choices do not guarantee literal string length or exact native wording.
- For `0 < seconds < approximationSeconds`, use the threshold magnitude with `< `; equality, zero, and negative inputs are not approximated.
- A positive magnitude below `millisecondsThreshold` retains up to three fractional digits for seconds in the last position. Other intervals are integral. The threshold applies to the entire magnitude.
- Negative values use absolute magnitude plus `-`, including a truncated negative zero. Locale-specific sign/approximation prefixes are not modeled.
- Rendering reads current `GetLocale()` without passing it any timing argument. Captured numeric/conversion functions cannot be replaced by later addon overrides. Malformed state and precision overflow fail explicitly; ICU/CLDR versions may change text.

## How it works

- [Numeric configuration](seconds-formatter-configuration.md)
- [Native ICU boundary](intl-native-linking.md)
- [Duration text binding](duration-text-binding.md)
- [Secret display policy](aura-secret-display.md)

## Implementation inventory

- `src/c_api/seconds_formatter.rs`, `seconds_formatter/configuration.lua`: authentic handle membership, private configuration, enum publication, captured callbacks, and factory.
- `seconds_formatter/format.lua`: shared interval/rounding/prefix selection and NumericFormatter interface.
- `seconds_formatter/render.rs`, `seconds_formatter/units.rs`: private validated duration rendering and C ABI.
- `src/c_api/native_icu.rs`: shared checked buffers, diagnostics, and input validation, also reused by PTR Intl.
- `native/intl/duration_units.c`, `native/intl/bridge.h`: localized units, per-unit whitespace, and list joining.
- `Cargo.toml`, `build.rs`, `build/intl_native.rs`: the shared duration capability builds only `text.c` and `duration_units.c` on Retail 12.0.5/default retail and Forever; remaining Intl shims stay PTR-only.
- `src/lua_api/env_init/mod.rs`: model installation; the replaced SecondsFormatter block is removed from temporary proxy factories.

### Build dependency

No crate versions are added. Retail 12.0.5+ and Forever activate the existing optional `cc`, `pkg-config`, and `vcpkg` build dependencies and requires ICU4C >=72 for real duration formatting. Unix uses `icu-uc`/`icu-i18n` through pkg-config. MSVC retains the existing explicit static `icu:x64-windows-static-md` requirement. No alternate backend is provided. Local read-only pkg-config observation: ICU 78.3; cross-target build/link verification remains with the integrating caller.

## Tests asserting this spec

- `tests/seconds_formatter_native.rs`: userdata/securecopy/type acceptance, live binding/FontString output, locale whitespace, tainted configuration, secret conversion/curve boundaries, and the actual CustomAuraButton consumer.
- `tests/seconds_formatter_format.rs`: shared-capability formatting, locale, validation, GC, exact enum before/after bootstrap, and Retail 12.0.5+ numeric/duration/curve units without `C_Intl`. Placeholder coverage applies only outside the capability.
- `tests/seconds_formatter_configuration.rs`: existing state/arity/evaluation regressions with shared-capability output expectations.
- `environment_cleanup_restore.rs` and `proxy_object_factories.rs` existing library tests retain factory/namespace and native-handle assertions.

## Known gaps (current cycle)

- [ ] New tests and code are unexecuted in this implementation slice: Cargo/runtime execution was prohibited. Parent must compile, run relevant grouped tests and the actual ActionBarAuras path, then perform independent verification before checking the requirements above.
- [ ] Retail epochs before 12.0.5 and classic profiles remain outside this promotion; their prior placeholder behavior is not native compatibility evidence.
- [ ] Native inheritance, complete locale exception lists, exact native wording/defaults/rounding, arbitrary secret configuration, and general VM/debug secrecy are unverified.
- [ ] Numeric curves retain existing engine limitations; errors propagate rather than changing their behavior or declassifying secret callback inputs.

## Out of scope

Addon/vendor/native-cache edits, validator loosening, generic conversion changes, unrelated C_Intl namespace publication, new formatting backends, broader curve/VM/security redesign, downloads, deployment, or an overall ActionBarAuras/addon compatibility claim.
