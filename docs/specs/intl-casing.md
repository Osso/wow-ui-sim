# PTR Unicode casing

`src/c_api/c_intl/casing.rs` supplies Unicode casing for the global C_Intl namespace and existing locale-context userdata. See [locale context storage](../wiki/systems/locale-context-storage.md).

## What it must do

- [x] Publish `ToLower(text)`, `ToUpper(text)`, and `FoldCase(text)` on PTR C_Intl and locale contexts; preserve earlier-retail namespace absence.
- [x] Lower/upper use the current `GetLocale()` result globally and the receiver's current stored identifier for context calls. `SetLocale` affects subsequent casing without changing other contexts.
- [x] Handle Turkish dotted/dotless I, context-sensitive Greek final sigma, and expanding uppercase mappings such as sharp S.
- [x] Fold with Unicode default full folding, independent of global/context locale: sharp S expands, sigma forms converge, dotted I retains its combining dot, and ligatures expand.
- [x] Preserve empty text. Reject nonstring/missing text, invalid UTF-8, and invalid context receivers without lossy replacement.
- [x] Simulator locale policy: translate four ASCII-letter WoW-style identifiers into a two-letter language and two-letter region for casing, otherwise parse an ICU locale identifier. Preserve stored bytes verbatim. Reject malformed identifiers during lower/upper instead of silently choosing a root locale.
- [x] Folding neither parses the stored locale nor calls `GetLocale()`, because its mapping is locale-independent.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_intl/casing.rs` — locale parsing and the six casing functions.
- `src/c_api/c_intl.rs` — namespace/userdata registration and shared locale-byte accessors.
- `src/c_api/c_intl/text.rs` — existing strict UTF-8 input validation.
- `Cargo.toml` / `Cargo.lock` — direct `icu_casemap = "=2.1.1"` with only `compiled_data`, and `icu_locale_core = "=2.1.1"` with only `alloc`; defaults disabled. ICU supplies full context-sensitive Unicode mappings and locale parsing instead of handwritten approximations. Existing dependency versions remain unchanged; casing introduces `icu_casemap` and its matching data package. Version updates require deliberate review.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_casing.rs`
- Existing `intl_` library tests retain normalization, scalar length, and enum/profile coverage.

## Known gaps (current cycle)

- [ ] Pinned six APIs declare `MayReturnNothing=true` but do not specify its conditions. Simulator returns one string on success and explicit errors on invalid input; native failure behavior remains unverified.
- [ ] WoW-tag translation, accepted ICU identifier syntax, locale selection, and error policy are simulator choices, not established native canonicalization rules.
- [ ] Native Unicode-data version equivalence, embedded-NUL semantics, secret arguments, taint, and protected access remain unverified.

## Out of scope

Titlecasing, collation, formatting, locale negotiation, and changes to opaque locale storage are separate APIs/tasks. Default Unicode folding deliberately does not select Turkic-specific folding.
