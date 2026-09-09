# Opaque locale contexts

PTR locale-context storage implements the pinned 12.1.5 declarations in `src/c_api/c_intl.rs`. Contexts store identifiers without interpreting them; see [C API architecture](../lua-api.md).

## What it must do

- [x] PTR exposes `C_Intl.CreateLocaleContext(locale)` and `GetCurrentLocale()`.
- [x] Each constructor call returns an independent userdata with `GetLocale()` and `SetLocale(locale)`.
- [x] Identifiers including `enUS`, `fr-FR`, and `zh-Hant-TW` are preserved verbatim.
- [x] Simulator assumption: identifiers must be nonempty and NUL-free. Invalid constructor identifiers error; invalid setter identifiers return false without mutation. Successful setters return true. Nonstring required arguments error.
- [x] Current locale follows existing `GetLocale()` without modifying its convention or being changed by context mutation.
- [x] Earlier retail omits `C_Intl`; neither profile publishes a global `LuaLocaleContext` type table.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_intl.rs` — opaque userdata storage, mutation, and client-locale query.
- `src/c_api/mod.rs` — registration.
- `src/loader/tests/wow_api_globals/patch_12_1_5_locale_context.rs` — profile and storage behavior.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_locale_context.rs`

## Known gaps (current cycle)

- [ ] Native identifier validation and setter failure semantics remain unverified; rules above are simulator choices.
- [ ] Secret/taint behavior remains unverified.

## Out of scope

Locale canonicalization, fixed locale catalogs, Unicode, formatting, parsing, translation, and other context methods are excluded from this storage-only slice. No BCP validation is claimed.
