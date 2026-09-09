# Locale transforms

PTR `C_Intl.TransformLocale(transform)` and `LuaLocaleContext:TransformLocale(transform)` return transformed current/context locale identifiers without mutating storage. Pinned declarations are in `data/patch-api/sources/12.1.5-register.json`.

## What it must do

- [x] Support all eight LocaleTransform values using ICU canonicalization and likely-subtag expansion/minimization.
- [x] Preserve opaque context identifiers and extensions during canonicalization/expansion/minimization; emit BCP-47 serialization.
- [x] Return language/script/region/variants; missing optional components return empty strings.
- [x] Preserve earlier-retail absence and reject malformed identifiers/invalid operations without fallback.

### Simulator assumptions

Four-letter alphabetic WoW tags are translated to language-region only during transformation. Variant output joins ICU-ordered variants with hyphens. ParentLocale removes the last serialized extension group (private use first), otherwise the last variant, region, then script; language-only identifiers reduce to `und`, which remains `und`. This syntactic policy is not CLDR inheritance or confirmed native parent behavior. Valid calls return one string; native MayReturnNothing conditions remain unknown.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)

## Implementation inventory

- `src/c_api/c_intl/transform.rs`: return-only transforms.
- `src/c_api/c_intl.rs`: global/context registration.
- `Cargo.toml`: direct `icu_locale =2.1.1` with compiled data, required for canonicalizer/expander; reuses locked version without unrelated upgrades.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_transform.rs`
- Focused `intl_` library suite: 14 PTR and 7 retail tests passed after `15a52023b`; logs `/tmp/intl-transform-green-{ptr,retail}.log`.

## Known gaps (current cycle)

- [ ] Native tag handling, parent policy, failure returns, security and Unicode/CLDR version equivalence remain unverified.

## Out of scope

Formatting, collation, native access restrictions, and mutation of current/global locale.
