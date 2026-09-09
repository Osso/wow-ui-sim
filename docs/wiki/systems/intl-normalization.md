# PTR Unicode normalization

PTR `C_Intl.Normalize` and `C_Intl.IsNormalized` provide four-form Unicode normalization through ICU 2.1.1 after validating Lua string bytes as UTF-8.

## Content

`NormalizationForm` values select NFC, NFD, NFKC, or NFKD. Valid UTF-8 input returns one normalized string from `Normalize`; `IsNormalized` returns the matching predicate. The implementation uses ICU compiled data, preserving canonical combining-order, Hangul, and compatibility behavior covered by focused tests.

The simulator rejects invalid UTF-8, nonstring input, and invalid form selectors. `Normalize` returns one result for accepted input despite its source `MayReturnNothing` annotation. Those invalid-input and return choices are simulator policy, not native conformance. ICU data-version equivalence, embedded-NUL cstring handling, secret/taint enforcement, coercion, and native edge behavior remain unmodeled.

The API is registered only on PTR. Earlier retail keeps `C_Intl` absent; no global normalization type is published.

## Sources

- [normalization spec](../../specs/intl-normalization.md) — supported forms, assumptions, and exclusions
- [C API implementation](../../../src/c_api/c_intl/normalization.rs) — ICU dispatch and validation
- [normalization tests](../../../src/loader/tests/wow_api_globals/patch_12_1_5_intl_normalization.rs) — focused profile behavior
- [ICU dependency](../../../Cargo.toml) — pinned `icu_normalizer` 2.1.1 with compiled data

## See Also

- [[locale-context-storage]] — opaque PTR locale contexts
- [[patch-12-1-5-api-audit]] — occurrence credit and remaining locale work
- [[lua-api]] — C API registration conventions
