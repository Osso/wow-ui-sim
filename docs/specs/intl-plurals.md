# Intl plural selection

PTR `C_Intl.SelectPlural` and `LuaLocaleContext:SelectPlural` select CLDR plural categories. Source contracts are pinned in `data/patch-api/sources/12.1.5-register.json`; implementation lives in `src/c_api/c_intl/plurals.rs`.

## What it must do

- [ ] Accept a required Lua number and `PluralType.Cardinal=0` or `Ordinal=1`, returning one category string for valid input.
- [ ] Use current `GetLocale()` globally and each context's stored locale independently; locale changes affect later selection without mutating other contexts.
- [ ] Apply CLDR cardinal rules for English, French, Russian, and Arabic and English ordinal rules, including fractional, negative, and zero inputs.
- [ ] Reject nonfinite/non-number operands, invalid plural types, invalid locales and incompatible receivers without changing context state.
- [ ] Preserve earlier-retail namespace absence and existing Intl APIs.

### Simulator operand policy

These choices are modeled behavior, not native WoW guarantees:

- Convert the absolute finite Lua `f64` with `fixed_decimal::Decimal::try_from_f64` and `FloatPrecision::RoundTrip`: shortest decimal retaining the represented binary value. Precision already lost when Lua parsed/calculated the number cannot be recovered.
- Numeric `1`, `1.0`, and `1.00` are indistinguishable. Original textual trailing zeros are not preserved; decimal fractional digits come from round-trip conversion.
- Negative operands use absolute value, including negative zero. NaN and infinities raise errors. ICU's `PluralOperands` conversion governs extreme decimal magnitudes and significant operand limits; native behavior there remains unknown.
- Return lowercase CLDR categories `zero`, `one`, `two`, `few`, `many`, `other`. Do not silently substitute a category after parsing, conversion, or provider errors.

### Dependency contract

Use exact `icu_plurals =2.1.1` with compiled CLDR data, matching the existing ICU line, rather than handcrafted locale rules. Direct `fixed_decimal =0.7.0` with `ryu` is required to construct the public decimal operand accepted by `PluralRules::category_for` with supported `f64` round-trip conversion. No unrelated package upgrades are part of this slice.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)

## Implementation inventory

- `src/c_api/c_intl/plurals.rs`: CLDR selection, numeric validation and category publication.
- `src/c_api/c_intl.rs`: global/context registration and locale access.
- `Cargo.toml`, `Cargo.lock`: explicit ICU plural and decimal dependencies.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_plurals.rs`: categories, locale independence, validation and profile absence.

## Known gaps (current cycle)

- [ ] Native operand precision, trailing-zero interpretation, negative/nonfinite policy, CLDR-version equivalence and `MayReturnNothing` conditions remain unverified.
- [ ] `AllowedWhenUntainted` enforcement is not established by these ordinary-value tests.

## Out of scope

Locale negotiation redesign, plural message formatting, secret/taint enforcement, and native-invalid-input compatibility require separate evidence.
