# Intl date and time formatting

PTR global/context `FormatDate`, `FormatTime`, and `FormatDateTime` use the ICU4C bridge. The [pinned API register](../../data/patch-api/sources/12.1.5-register.json) declares Unix seconds, date/time styles, an explicit timezone string, and `MayReturnNothing`; generated documentation does not establish native runtime semantics.

## Credited behavior

- [x] PTR-only global and locale-context publication; earlier retail keeps `C_Intl` absent.
- [x] Finite Unix seconds cross the native boundary as seconds × 1000, including tested pre-epoch and fractional values.
- [x] Date, time, and date-time paths accept all five `DateTimeStyle` values, including `None`.
- [x] Supplied valid UTC, fixed-offset, and named zones produce tested epoch/day-boundary and New York DST results.
- [x] Global/current and independent locale-context selection affect formatting; tested BCP-47 extensions are preserved.

## Explicit policy or unverified behavior

- Empty timezone means UTC; both styles `None` return an empty string; argument and zone validation errors are simulator policy, not native credit.
- Exact rendered text, ICU/CLDR/tzdata version, calendar cutover, extreme precision/range, locale-data equivalence, and `MayReturnNothing` conditions are unverified.
- Native WoW equivalence, `AllowedWhenUntainted`, taint, secret, protected, coercion, and security semantics are unverified.
- Windows, macOS, and Docker execution remain accepted pending.

## Implementation inventory

- `native/intl/date_format.c`: ICU style mapping, supplied-zone formatter creation, formatting, and cleanup.
- `native/intl/bridge.h`, `build/intl_native.rs`: fixed-width C ABI and PTR-native build scope.
- `src/c_api/intl_native.rs`, `intl_native/ffi.rs`: checked Unix-seconds-to-milliseconds boundary.
- `src/c_api/c_intl/date_formatting.rs`, `c_intl.rs`: Lua decoding and global/context registration.

## Proof

- `ca312b672`: implementation and focused native/Lua tests.
- `3259e3a76`: canonical-zone validation repair.
- `ec09c4037`: focused explicit-zone proof record.
- `src/c_api/intl_native/date_tests.rs`: Unix seconds, styles including `None`, explicit zones, day boundary, DST, and locale extension.
- `tests/intl_date_formatting.rs`: six APIs, global/context locale selection, profile absence, and focused PTR behavior.

## Related specs

- [Native ICU linking](intl-native-linking.md)
- [Locale context storage](intl-locale-context.md)
- [Number formatting bridge](intl-number-formatting.md)
