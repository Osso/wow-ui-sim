# Locale context storage

PTR locale contexts are opaque userdata that store caller-provided byte identifiers independently. They do not implement locale interpretation.

## Content

`C_Intl.CreateLocaleContext(locale)` creates a context. `GetLocale()` returns its stored identifier. `SetLocale(locale)` changes that identifier only after simulator validation. `C_Intl.GetCurrentLocale()` delegates to existing `GetLocale()` and is unaffected by context mutation.

The simulator accepts nonempty, NUL-free identifiers and returns `false` without mutation for invalid setter input. Those rules, success semantics, and identifier spelling are simulator choices. No canonicalization, BCP validation, ICU behavior, formatting, Unicode processing, or secret/taint semantics are modeled.

`LuaLocaleContext` is userdata returned by the constructor, not a global Lua type table. The PTR surface is absent on earlier retail.

## Sources

- [locale context spec](../../specs/intl-locale-context.md) — tested contract and stated assumptions
- [C API implementation](../../../src/c_api/c_intl.rs) — userdata storage and profile registration
- [locale-context tests](../../../src/loader/tests/wow_api_globals/patch_12_1_5_locale_context.rs) — focused profile behavior

## See Also

- [[patch-12-1-5-api-audit]] — occurrence-level credit and remaining locale work
- [[lua-api]] — C API and userdata conventions
