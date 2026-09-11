# Intl currency metadata

PTR `C_Intl.GetCurrencyName`, `LuaLocaleContext:GetCurrencyName`, and `C_Intl.GetCurrencyFractionDigits` query ICU4C currency data. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) requires currency strings and name-style selectors, permits no result, and marks arguments `AllowedWhenUntainted`. The [native linking contract](intl-native-linking.md) remains unchanged.

## What it must do

- [x] Publish the three documented APIs on PTR; keep the earlier-retail namespace absent. Do not invent a context fraction-digits method.
- [x] Map `CurrencyNameStyle` explicitly: Symbol (0), NarrowSymbol (1), Long (2), FormalSymbol (3), VariantSymbol (4). ICU's Long/Narrow enum order differs from WoW's.
- [x] Global names use the current locale; context names use the stored locale. Preserve existing strict UTF-8 validation, WoW-tag translation, and BCP-47 extensions without mutating contexts.
- [x] Reuse three-ASCII-letter currency validation and uppercase normalization. Malformed codes and styles raise errors.
- [x] Check ICU's known currency catalogue before requesting names or fraction digits. Well-formed unknown codes return zero Lua results, not an ISO-code display or generic fraction-digit default.
- [x] Return one localized string for a known currency name and one number for known fraction digits. Distinguish JPY's zero digits from unknown/no-result and native failure through an explicit native status and output parameter.
- [x] Keep returned Rust/Lua strings independent of borrowed ICU name data. Free allocated locale/output buffers on success and error paths; unknown-currency paths allocate no native lookup buffers.
- [x] Preserve existing number/currency formatting, parsing, and ICU4X APIs; add no dependencies or platform provisioning changes.

## Model policy and evidence limits

Known means `ucurr_isAvailable(code, U_DATE_MIN, U_DATE_MAX)` succeeds: the ICU catalogue across all dates, not only currently circulating currencies. Names come from `ucurr_getName`; standard fraction digits come from `ucurr_getDefaultFractionDigits`. Names, catalogue membership, and minor units can change with the installed ICU/CLDR version. Tests use USD/CAD names and USD=2, JPY=0, KWD=3 as concrete data, not an exhaustive currency table.

Unknown-code no-result, uppercase normalization, all-date catalogue membership, invalid-input errors, and ICU locale-data selection are simulator policies. They do not establish native WoW failure conditions, data-version equality, coercion, or secret/taint enforcement. These APIs do not change amounts or apply currency rounding. No native formatting fallback is added.

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

- `tests/intl_currency_metadata.rs`: all name styles, localized names, current/context state, zero-versus-unknown digits, errors, and profile absence.
- `src/c_api/intl_native/currency_metadata_tests.rs`: direct native results and repeated success/no-match/error cleanup paths with retained output.
- Existing `intl_number_formatting` integration and `intl_` library tests provide bounded regressions.

## Focused development proof

- RED: three PTR metadata tests failed on the missing APIs.
- Native wrapper tests: 11 passed, including three metadata tests exercising known/unknown currency, all styles, zero digits, retained outputs, and repeated success/no-match/error cleanup paths.
- Number/metadata integration tests: 6 PTR and 2 retail passed.
- Existing Intl/context library regressions: 21 PTR and 11 retail passed.
- Root `cargo fmt` completed. ICU runtime: `78.3.0.0` on Linux. No Windows/macOS execution is claimed.

Commands (each profile executed separately):

```text
cargo test --lib --offline --no-default-features --features sound,gui,client-ptr intl_native:: -- --nocapture
cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- intl_currency_metadata:: intl_number_formatting:: --nocapture
cargo test --lib --offline --no-default-features --features sound,gui,client-<profile> -- patch_12_1_5_intl_ patch_12_1_5_locale_context:: --nocapture
```

## Known gaps (current cycle)

- [ ] Windows/macOS execution remains pending as accepted by the user; this slice changes no provisioning.
- [ ] Native WoW catalogue/name-style equivalence, Unicode/CLDR version equality, and `AllowedWhenUntainted` enforcement remain unverified.

## Out of scope

Audit-manifest credit, broad verification/readability gates, new dependencies, native platform provisioning, titlecase/date/transliteration features, deployment, and publication.
