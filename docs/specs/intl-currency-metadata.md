# Intl currency metadata

PTR `C_Intl.GetCurrencyName`, `LuaLocaleContext:GetCurrencyName`, and `C_Intl.GetCurrencyFractionDigits` query ICU4C currency data. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) requires currency strings and name-style selectors, permits no result, and marks arguments `AllowedWhenUntainted`. Audit credit is bounded in the [PTR occurrence inventory](../wiki/investigations/patch-12-1-5-occurrence-inventory.md); the [native linking contract](intl-native-linking.md) remains unchanged.

## What it must do

- [x] Publish the three documented APIs on PTR; keep the earlier-retail namespace absent. Do not invent a context fraction-digits method.
- [x] Map `CurrencyNameStyle` explicitly: Symbol (0), NarrowSymbol (1), Long (2), FormalSymbol (3), VariantSymbol (4). ICU's Long/Narrow enum order differs from WoW's.
- [x] Global names use the current locale; context names use the stored locale. Preserve existing strict UTF-8 validation, WoW-tag translation, and BCP-47 extensions without mutating contexts.
- [x] Check the ICU catalogue before requesting names or fraction digits. Committed evidence covers known/miss distinction and USD=2, JPY=0, KWD=3 precision.
- [x] Return one localized string for tested known names and one number for tested known fraction digits. JPY's zero remains distinguishable from a catalogue miss through a separate native status and output parameter.
- [ ] Do not credit malformed-code errors, unknown-code zero results, ICU naming/catalogue/version behavior, or security semantics as native behavior.
- [x] Keep returned Rust/Lua strings independent of borrowed ICU name data. Free allocated locale/output buffers on success and error paths; unknown-currency paths allocate no native lookup buffers.
- [x] Preserve existing number/currency formatting, parsing, and ICU4X APIs; add no dependencies or platform provisioning changes.

## Model policy and evidence limits

Known means `ucurr_isAvailable(code, U_DATE_MIN, U_DATE_MAX)` succeeds: the ICU catalogue across all dates, not only currently circulating currencies. Names come from `ucurr_getName`; standard fraction digits come from `ucurr_getDefaultFractionDigits`. Names, catalogue membership, and minor units can change with the installed ICU/CLDR version. Tests use USD/CAD names and USD=2, JPY=0, KWD=3 as concrete data, not an exhaustive currency table.

Invalid-code errors and unknown-code zero results are selected simulator policies, not audit credit. Uppercase normalization, all-date catalogue membership, and ICU locale-data selection do not establish native WoW failure conditions, naming/version equality, coercion, or secret/taint enforcement. These APIs do not change amounts or apply currency rounding. No native formatting fallback is added.

## How it works

- [ICU4C build/provisioning](../ptr-icu-build.md)
- [Native linking contract](intl-native-linking.md)
- [Number and currency formatting](intl-number-formatting.md)
- [Locale context storage](intl-locale-context.md)

## Implementation inventory

- `native/intl/currency_metadata.c`: catalogue checks, explicit style mapping, borrowed-name conversion, and fraction-digit status/output handling.
- `native/intl/bridge.h`: fixed-width currency metadata ABI.
- `build/intl_native.rs`: compiles the new C source under the existing PTR gate.
- `src/c_api/intl_native.rs`: typed safe wrappers and reused currency validation.
- `src/c_api/intl_native/ffi.rs`: native status conversion and output ownership.
- `src/c_api/c_intl/currency_metadata.rs`: Lua arguments, result arity, and global/context metadata registration.
- `src/c_api/c_intl.rs`: existing PTR registration path.
- `src/c_api/c_intl/number_formatting.rs`: shared locale/error/text helpers used without changing formatting semantics.

## Tests asserting this spec

- `tests/intl_currency_metadata.rs`: all name styles, localized names, global/context locale selection, known/miss catalogue behavior, precision, and profile absence.
- `src/c_api/intl_native/currency_metadata_tests.rs`: direct style/locale, catalogue, and fraction-precision behavior.
- Existing `intl_number_formatting` integration and `intl_` library tests are regression context, not metadata audit credit.

## Focused development proof

Committed focused proof at `2170f394a` covers five style mappings, global/current and context/stored locale selection, known/miss catalogue behavior, USD/KWD/JPY precision, and PTR/earlier-retail profile boundaries. This audit refresh does not rerun those tests or any native build.

## Known gaps (current cycle)

- [ ] Invalid-code errors and unknown-code zero results remain selected simulator policy, not native credit.
- [ ] ICU naming/catalogue/version behavior, native WoW catalogue/name-style equivalence, Unicode/CLDR equality, and `AllowedWhenUntainted` enforcement remain unverified.
- [ ] Windows/macOS execution remains accepted pending; this slice changes no provisioning.

## Out of scope

Broad verification/readability gates, new dependencies, native platform provisioning, titlecase/date/transliteration features, deployment, and publication.
