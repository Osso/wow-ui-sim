# Intl display names and transliteration

PTR exposes `C_Intl.GetDisplayName(displayLocale)`, `LuaLocaleContext:GetDisplayName(displayLocale)`, and global `C_Intl.Transliterate(text, transliteratorID)`. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) specifies required string arguments, string results, `MayReturnNothing`, and `AllowedWhenUntainted`. This implementation uses the existing ICU4C bridge; no locale-context transliteration method is added.

## What it must do

- [x] Display the global current locale or context's stored locale in the language specified by `displayLocale`, not the reverse. Context updates affect subsequent results without mutating other contexts.
- [x] Route supplied display-locale and transliteration text through the existing strict UTF-8 boundary.
- [ ] Establish exact display names, ICU data/version, locale inheritance, or native WoW output.
- [x] Apply registered ICU transliterators through `utrans_openU` and `utrans_transUChars` using explicit UTF-16 lengths. Empty input is valid; embedded NUL and supplementary characters are preserved as input data.
- [x] Grow output buffers with checked arithmetic. On overflow, discard partially transformed output and retry from the original UTF-16 input, restoring length and limit.
- [ ] Establish native allocation cleanup or error/no-result behavior.
- [ ] Establish validation-error behavior or post-failure recovery.
- [x] Preserve PTR publication and earlier-retail `C_Intl`/`LuaLocaleContext` absence. No context `Transliterate` is added.

## Simulator policies and native uncertainty

ICU data/version, naming inheritance, exact display/transliteration output, error/no-result behavior, and native WoW equivalence are unverified. `MayReturnNothing`, `AllowedWhenUntainted`, taint, secret, protected, coercion, and other security behavior are unverified. These APIs do not add context `Transliterate` or native security enforcement.

## How it works

- [Native ICU linking](intl-native-linking.md)
- [Locale-context storage](intl-locale-context.md)
- [Existing ICU4C provisioning](../ptr-icu-build.md)

## Implementation inventory

- `src/c_api/c_intl/display_transliteration.rs`: three Lua API adapters and argument direction.
- `src/c_api/intl_native.rs`, `src/c_api/intl_native/ffi.rs`: validated Rust interface and owned native output.
- `native/intl/display_transliteration.c`, `native/intl/bridge.h`: ICU C calls, locale conversion, resource cleanup, and original-input retries.
- `build/intl_native.rs`: compile the new shim using the existing feature-scoped ICU dependency.

## Tests asserting this spec

- `tests/intl_display_transliteration.rs`: Lua naming direction, context mutation, growth, repeated calls, errors, and retail absence.
- `src/c_api/intl_native/display_transliteration_tests.rs`: direct naming, expansion with supplementary characters, repeat/empty/NUL input, and error recovery.

## Known gaps (current cycle)

- [ ] Independently execute focused native/PTR/retail tests; this audit reviewed committed test sources only.
- [ ] Windows/macOS execution remains accepted pending.
- [ ] ICU data/version, naming inheritance, exact output, error/no-result behavior, native equivalence, and security remain unverified.

## Out of scope

Context `Transliterate`, new dependencies/provisioning, alternative backends, general ICU replacement, final check/readability/broad gates, audit-artifact generation, deployment, and publishing.
