# Intl number and currency formatting

The PTR `C_Intl` namespace and locale contexts expose number/currency formatting and parsing through ICU4C. Contracts come from the [pinned register](../../data/patch-api/sources/12.1.5-register.json); [native linking](intl-native-linking.md) describes required build/runtime support. Existing ICU4X operations remain unchanged.

## What it must do

- [x] Publish `FormatNumber`, `ParseNumber`, `FormatCurrency`, and `ParseCurrency` globally and on locale-context userdata for the PTR API epoch. Earlier-retail absence remains pending independent profile proof.
- [x] Global calls use `GetCurrentLocale`; context calls use the context's current identifier without mutating it. BCP-47 extensions pass to ICU4C locale conversion.
- [x] Support Decimal, Integer, Percent, and Currency styles through locale-specific ICU4C symbols, grouping, and patterns.
- [x] `FormatCurrency(number, currencyCode)` uses its explicit code; currency-style `FormatNumber` uses ICU's locale default. That default is simulator policy, not verified native WoW behavior.
- [x] Return one string for successful formatting, one finite number for successful number parsing, and a fresh `{ amount, currencyCode }` table for successful currency parsing.
- [x] Parsing consumes complete input; malformed or incomplete parses return zero Lua results. `CurrencyParseResult` is not a global table.
- [x] Reject malformed UTF-8, invalid required argument types/styles/currency codes, and nonfinite format operands. Failed calls do not mutate locale contexts.
- [x] Keep native allocation, formatter lifetime, encoding conversion, and error handling inside the bounded C bridge; pass neither C++ objects nor Rust references across the ABI.

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

- [x] Native bridge and Lua registration have focused proof: 8 Linux wrapper tests at `8b1648770` and 3 PTR Lua API tests at `0f25b13b7`.
- [ ] Verify retail profile exclusion and supported native build/packaging paths; macOS and Windows workflow configuration exists but was not executed.
- [ ] Native WoW precision, parsing grammar, failure conditions, currency selection, ICU-version/data equivalence, and `AllowedWhenUntainted` enforcement remain unverified.

## Out of scope

Existing ICU4X replacement, date/time formatting, display names, custom precision controls, speculative formatter caches, native WoW security enforcement, deployment, and publishing are not part of this slice.
