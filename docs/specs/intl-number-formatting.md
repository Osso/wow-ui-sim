# Intl number and currency formatting

The PTR `C_Intl` namespace and locale contexts expose number/currency formatting and parsing through ICU4C. Contracts come from the [pinned register](../../data/patch-api/sources/12.1.5-register.json); [native linking](intl-native-linking.md) describes required build/runtime support. Existing ICU4X operations remain unchanged.

## What it must do

- [ ] Publish `FormatNumber`, `ParseNumber`, `FormatCurrency`, and `ParseCurrency` globally and on locale-context userdata only for the PTR API epoch; preserve earlier-retail namespace absence.
- [ ] Global calls use `GetCurrentLocale`; context calls use the context's current identifier without mutating it. Preserve BCP-47 extensions when selecting ICU4C locale behavior.
- [ ] Support all four `NumberStyle` values: Decimal, Integer, Percent, Currency. Use real locale-specific symbols, grouping, and patterns rather than English substitutions.
- [ ] `FormatCurrency(number, currencyCode)` uses its explicit currency code; currency-style `FormatNumber` uses ICU's locale-default currency. The latter is a simulator policy, not a verified native default.
- [ ] Return one string for successful formatting, one finite number for successful number parsing, and a fresh `{ amount, currencyCode }` table for successful currency parsing.
- [ ] Parsing consumes the complete input; malformed or incomplete parses return zero Lua results. Do not expose `CurrencyParseResult` as a global table.
- [ ] Reject malformed UTF-8, invalid required argument types/styles/currency codes, and nonfinite format operands explicitly. Failed calls do not alter locale contexts.
- [ ] Keep native allocation, formatter lifetime, encoding conversion, and error handling inside the bounded bridge; do not pass C++ objects or Rust references across the ABI.

## Simulator assumptions

ICU4C's default formatting precision, grouping, percent scaling, locale-default currency, currency minor units, and rounding are modeled behavior—not confirmation of WoW output. Integer style formats with zero fractional digits. Lua numbers are finite `f64` inputs; original decimal spelling and trailing precision are unavailable. Full-input parsing and zero-result parse failure are chosen policies. Output can differ with ICU/CLDR versions. No approximate fallback is installed when native ICU is missing.

## How it works

- [Native ICU build contract](intl-native-linking.md)
- [Locale context storage](intl-locale-context.md)

## Implementation inventory

- `src/c_api/c_intl/number_formatting.rs`: Lua argument/result conversion and global/context registration.
- `src/c_api/intl_native.rs`: safe Rust formatting/parsing boundary.
- `native/intl/`: ICU C API shim and encoding/lifetime management.

## Tests asserting this spec

- `tests/intl_number_formatting.rs`: all styles, locale differences, currency results, validation, full-input parsing, and profile preservation.
- `src/c_api/intl_native/tests.rs`: direct native behavior and linking proof.

## Known gaps (current cycle)

- [ ] Complete native bridge and Lua registration, then run focused tests and startup smoke.
- [ ] Verify supported native build/packaging paths; unexecuted platform checks must remain explicit.
- [ ] Native WoW precision, parsing grammar, failure conditions, currency selection, and `AllowedWhenUntainted` behavior remain unverified.

## Out of scope

Existing ICU4X replacement, date/time formatting, display names, custom precision controls, speculative formatter caches, native WoW security enforcement, deployment, and publishing are not part of this slice.
