# Intl collation search

PTR `C_Intl.FindStringMatches` and `LuaLocaleContext:FindStringMatches` search through ICU4C collation. The [pinned register](../../data/patch-api/sources/12.1.5-register.json) requires `text: cstring`, `pattern: cstring`, and `strength: CollationStrength`, returning a table of numeric `byteOffsets` with `MayReturnNothing`. Source lives in `src/c_api/c_intl/string_matches.rs` and the existing native bridge.

## What it must do

- [x] Publish global and context methods with all five strengths: Primary, Secondary, Tertiary, Quaternary, Identical.
- [x] Global calls use the current locale; context calls use that object's current stored locale. Reuse locale parsing without modifying identifiers or other contexts.
- [x] Return a fresh numeric array of zero-based UTF-8 match starts. Search forward with overlap disabled; `banana` searched for `ana` returns only offset `1`.
- [x] Enable collator normalization explicitly. Support canonical equivalence, strength-sensitive accents/case, German expansion, and Turkish locale differences through ICU rather than literal/regex substitutes.
- [x] Return one empty table for empty text, empty pattern, or no matches; validate locale, strength, and required inputs even for empty searches.
- [x] Reject malformed UTF-8, invalid strengths, invalid locale identifiers, and invalid receivers explicitly. Preserve valid embedded NULs through length-based native conversion.
- [x] Keep offsets correct after supplementary characters and multibyte text. Validate both ends of native ranges as scalar boundaries and reject nonprogressing/overlapping results.
- [x] Close each search iterator before its borrowed collator; release converted strings and result allocations on success and failure. Rust owns the returned range allocation through a drop guard.
- [x] Build a UTF-16-scalar-boundary-to-UTF-8-byte map once, then translate matches in O(n + m) time and O(n + m) space outside ICU's own search. Never repeatedly convert prefixes for each result.
- [ ] Preserve earlier-retail namespace absence and all existing Intl/native regressions in the bounded profile test run.

## Simulator assumptions

Zero-based offsets, forward non-overlap, normalization enabled, and empty-input tables are explicit simulator policies, not confirmed WoW conventions. Zero-length ICU matches from collation-ignorable patterns are omitted; iterator progress is still checked. Argument/native operational errors raise Lua errors rather than using an approximate search fallback. Successful calls always return one table; native `MayReturnNothing` conditions remain unknown.

Search uses installed ICU4C data. `CompareStrings` and `GetSortKey` still use ICU4X 2.1.1. Cross-backend Unicode/CLDR-version agreement, native WoW match boundaries, locale tailoring, and secret/untainted behavior are unverified. No claim equates ICU4C search results to a sequence of ICU4X comparisons.

## How it works

- [Native linking](intl-native-linking.md)
- [Locale context storage](intl-locale-context.md)
- [ICU4X collation](intl-collation.md)

## Implementation inventory

- `src/c_api/c_intl/string_matches.rs`: global/context registration, strict argument decoding, and Lua array construction.
- `src/c_api/intl_native/search.rs`: safe native call, range-allocation ownership, and linear offset translation.
- `native/intl/string_search.c`: owned collator/search resources, normalization/strength selection, bounded forward iteration, and range buffer growth.
- `native/intl/bridge.h`: fixed-width range ABI and deallocation declaration.
- `build/intl_native.rs`: compiles the separate search C source using existing ICU provisioning.
- `src/c_api/intl_native/ffi.rs`: shared native diagnostic conversion; no new dependency or provisioning path.

## Tests asserting this spec

- `tests/intl_string_matches.rs`: strengths, canonical forms, expansion, locale changes, non-overlap, multibyte/NUL offsets, 2,048 matches, validation, and profile absence.
- `src/c_api/intl_native/search_tests.rs`: direct native expansion/canonical ranges, supplementary matches, empty/ignorable patterns, invalid locales, and repeated allocations.

## Known gaps (current cycle)

- [ ] Execute bounded existing Intl/native regressions and earlier-retail absence test.
- [ ] Native WoW offsets, locale-data versions, `MayReturnNothing`, and `AllowedWhenUntainted` enforcement remain unverified.
- [ ] Allocation-failure paths and ICU internal invariant failures are handled but have no fault-injection proof. Windows/macOS execution is not claimed by Linux tests.

## Out of scope

Regex search, handcrafted matching fallback, new dependencies/provisioning, other locale algorithms, audit-artifact credit, broad verification gates, security enforcement, deployment, and publishing.
