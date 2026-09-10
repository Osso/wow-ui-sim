# Native Intl linking

The PTR number-formatting bridge uses ICU4C's stable C `UNumberFormat` API behind a narrow Rust wrapper. The contract covers native formatting/parsing, not Lua registration or packaging. See [Intl locale contexts](intl-locale-context.md) for the adapter's locale state.

## What it must do

- [ ] Compile and link the shim only with `retail-12-1-5`; earlier retail and Mists must not probe or link ICU4C.
- [ ] On Unix, discover both `icu-i18n` and `icu-uc` through `pkg-config`, requiring ICU >=72. Reject missing/older development libraries explicitly.
- [ ] On `x86_64-pc-windows-msvc`, require explicit `VCPKG_ROOT` and installed `icu:x64-windows-static-md`. Reject dynamic ICU selection, other triplets, and static-CRT Rust targets. No DLL/path guessing or alternate discovery route.
- [ ] Accept complete BCP-47 locale tags, preserving Unicode extensions through `uloc_forLanguageTag`. Also accept explicit ICU underscore identifiers by strict conversion to a language tag. Reject partially parsed or NUL-containing identifiers.
- [ ] Format finite numbers with Decimal, Integer, Percent, or Currency style. Currency style without a code uses ICU's locale default currency.
- [ ] Format explicit three-ASCII-letter currency codes case-insensitively using ICU currency data, not handwritten maps.
- [ ] Parse localized numbers and currencies only when ICU consumes the entire input and returns a finite value; return `None` for empty, invalid, nonfinite, or partial parses. Currency parsing also returns its ISO code.
- [ ] Preserve explicit input byte lengths through UTF-8/UTF-16 conversion, including embedded NUL. Never truncate number text with `strlen` or lossy UTF-8 conversion.
- [ ] Check native status, length/capacity arithmetic, allocation, and conversion failures. Close every formatter and release all C allocations on success and failure; expose safe Rust ownership.
- [ ] Report the linked runtime ICU version.

### Modeled policy and version scope

- ICU's ordinary locale/style defaults govern grouping, symbols, fraction digits, and parsing grammar. There is no custom locale fallback or custom grammar in this wrapper; ICU's own locale-data inheritance remains in effect.
- Integer formatting uses zero fraction digits and half-even rounding; integer parsing accepts only an entirely consumed integer. These are simulator choices, not confirmed native WoW behavior.
- Input precision is `f64`. No arbitrary-precision/trailing-zero preservation is promised.
- Operational/configuration failures return `Err`; unsuccessful parses return `Ok(None)`. Locale tags and currency codes reject embedded NUL; text is length-delimited.
- The build enforces ICU >=72 in discovery (Unix) and installed headers (all targets). Output and parse behavior depend on linked ICU/CLDR data; no byte-for-byte equivalence with WoW is claimed.
- Host discovery found ICU **78.3**. Focused Linux native behavioral proof is pending. Windows static linking is specified but **untested** here; parent packaging/CI owns that platform proof and vcpkg provisioning.

## How it works

- [Lua API boundary](../lua-api.md)
- [Client profile contract](client-profiles.md)

## Implementation inventory

- `native/intl/bridge.h`: fixed-width C ABI, internal conversion declarations, minimum header version.
- `native/intl/text.c`: checked allocation, UTF conversion, locale conversion, diagnostics, version query.
- `native/intl/number_format.c`: ICU formatter lifetime, formatting, complete-input parsing.
- `src/c_api/intl_native.rs`: safe public wrapper and input policy.
- `src/c_api/intl_native/ffi.rs`: private unsafe calls and output-buffer ownership.
- `src/c_api/mod.rs`: feature-gated native module, no Lua registration.
- `build/intl_native.rs`: feature-scoped Unix/MSVC discovery and C compilation.
- `build.rs`: invokes the native build helper only for the PTR epoch.
- `Cargo.toml` / `Cargo.lock`: optional build dependencies at existing locked versions: `cc =1.2.52` for C compilation, `pkg-config =0.3.32` for Unix discovery, `vcpkg =0.2.15` for deterministic MSVC static library discovery. No Rust bindings generator or C++ ABI dependency.

## Tests asserting this spec

- `src/c_api/intl_native/tests.rs`: grouped library tests for locale grouping, integer rounding, percent/currency formatting, parsing/full consumption, multibyte input, BCP-47 extensions, validation, and linked version.

## Known gaps (current cycle)

- [ ] Run focused Linux wrapper tests and record results.
- [ ] Validate Windows static linking against a provisioned vcpkg tree in parent CI; do not infer success from Linux tests.
- [ ] Test minimum ICU72 separately; this host provides 78.3.

## Out of scope

Lua API registration, audit-manifest credit, Docker/CI/xtask packaging changes, deployment, system package installation, native WoW compatibility claims, and ICU locale-data pinning are parent-owned or separate work.
