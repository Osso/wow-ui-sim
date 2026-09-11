# Intl date and time formatting

PTR global/context `FormatDate`, `FormatTime`, and `FormatDateTime` use the existing ICU4C bridge. The [pinned API register](../../data/patch-api/sources/12.1.5-register.json) requires Unix seconds, date/time style selectors, and an explicit timezone string; all six declarations permit no result. Implementation policy returns one string for valid input and errors for invalid arguments.

## What it must do

- [ ] Expose all six APIs only on PTR; preserve existing Intl operations and earlier-retail absence.
- [ ] Convert finite Unix seconds to ICU milliseconds with overflow checking, preserving pre-epoch and fractional inputs without premature integer rounding.
- [ ] Map None/Short/Medium/Long/Full to ICU styles. Date-only and time-only calls disable the other component.
- [ ] Validate named and custom GMT-offset zones with ICU canonical timezone validation before opening a formatter; never silently use GMT for invalid IDs or the host timezone.
- [ ] Use UTC for an explicitly empty zone string. Return an empty string when both styles are None, but still validate time, locale, and zone. These are simulator choices.
- [ ] Preserve current global locale and context locale selection, including BCP-47 extensions; context mutation affects subsequent calls without changing other contexts.
- [ ] Reject malformed UTF-8, embedded-NUL zone IDs, invalid required arguments/styles/zones, and nonfinite or overflowed times.
- [ ] Reuse native allocation, UTF conversion, formatter close, and Rust output ownership/error paths; no C++ ABI or additional dependencies.

## How it works

- [Native ICU linking](intl-native-linking.md)
- [Locale context storage](intl-locale-context.md)
- [Number formatting bridge](intl-number-formatting.md)

## Implementation inventory

- `native/intl/date_format.c`: style mapping, canonical-zone validation, `udat_open`/`udat_format`, and resource cleanup.
- `native/intl/bridge.h`, `build/intl_native.rs`: bounded C ABI declaration and source build list.
- `src/c_api/intl_native.rs`, `intl_native/ffi.rs`: checked seconds conversion and safe native call boundary.
- `src/c_api/c_intl/date_formatting.rs`, `c_intl.rs`: Lua argument decoding and global/context registration.

## Tests asserting this spec

- `src/c_api/intl_native/date_tests.rs`: native epoch, fractions, all styles, explicit zones, DST, extensions, and failures.
- `tests/intl_date_formatting.rs`: six Lua methods, context changes, day boundaries, DST, style combinations, validation, and retail absence.

## Known gaps (current cycle)

- [ ] Run focused native/Lua tests and existing Intl regressions.
- [ ] Windows/macOS execution remains accepted pending; this slice does not provision or claim platform verification.
- [ ] Native WoW formatting patterns, calendar cutover, extreme date range/precision, timezone data version, locale data equivalence, no-result conditions, and security/taint semantics remain unverified.

## Out of scope

Artifact credit, final check/readability gates, new dependencies/provisioning, date parsing, host timezone inference, native WoW security enforcement, deployment, and publishing. Exact punctuation and zone display names can vary with ICU/CLDR/tzdata; stable numeric en-GB fixtures test epoch and DST arithmetic without asserting a universal textual format.
