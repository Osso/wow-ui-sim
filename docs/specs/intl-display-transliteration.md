# Intl display names and transliteration

PTR exposes `C_Intl.GetDisplayName(displayLocale)`, `LuaLocaleContext:GetDisplayName(displayLocale)`, and global `C_Intl.Transliterate(text, transliteratorID)`. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) specifies required string arguments, string results, `MayReturnNothing`, and `AllowedWhenUntainted`. This implementation uses the existing ICU4C bridge; no locale-context transliteration method is added.

## What it must do

- [x] Display the global current locale or context's stored locale in the language specified by `displayLocale`, not the reverse. Context updates affect subsequent results without mutating other contexts.
- [x] Reuse existing strict UTF-8 and locale parsing, including WoW tag translation; convert both target and display BCP-47 tags for ICU4C.
- [x] Return ICU display names for English, French, and German display languages, preserving Unicode text.
- [x] Apply registered ICU transliterators through `utrans_openU` and `utrans_transUChars` using explicit UTF-16 lengths. Empty input is valid; embedded NUL and supplementary characters are preserved as input data.
- [x] Grow output buffers with checked arithmetic. On overflow, discard partially transformed output and retry from the original UTF-16 input, restoring length and limit.
- [ ] Close native transliterators and free temporary/output allocations on success, error, and retry paths.
- [x] Reject empty/NUL-containing/unavailable transliterator IDs and malformed UTF-8 explicitly; later calls remain usable after failure.
- [x] Preserve earlier-retail namespace absence and existing Intl behavior. Add no dependencies or platform provisioning changes.

## Simulator policies and native uncertainty

Display names and registered transliterator rules come from the installed ICU4C data. ICU's naming inheritance and data-selected names are retained; there is no alternate approximate implementation. Exact spelling, transliteration output, naming inheritance, and data versions may differ from WoW. Validation failures raise errors; successful calls return one string, including empty text. Native `MayReturnNothing` conditions and secret/untainted enforcement remain unverified. These APIs do not change locale-context storage or add native security enforcement.

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

- [x] Focused Linux RED/GREEN at `617d79e82`: three missing-API failures before implementation; 37 PTR library/native tests, 12 PTR Lua integration tests, 10 retail library tests, and four retail integration tests passed. Includes three new native tests, three new PTR Lua tests, and one retail absence test.
- [ ] Windows/macOS execution remains accepted pending; no new platform execution claim.
- [ ] Native naming, inheritance, transliteration data/version equivalence, error behavior, and security remain unverified.

## Out of scope

Context `Transliterate`, new dependencies/provisioning, alternative backends, general ICU replacement, final check/readability/broad gates, audit-artifact generation, deployment, and publishing.
