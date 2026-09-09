# PTR Unicode normalization

`src/c_api/c_intl/normalization.rs` provides Unicode normalization through the existing PTR C_Intl namespace. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] `Normalize(text: cstring, form: NormalizationForm)` supports Nfc=0, Nfd=1, Nfkc=2, Nfkd=3 and returns one normalized string for valid input.
- [x] `IsNormalized(text: cstring, form: NormalizationForm)` returns one boolean.
- [x] Normalize canonical combining order, Hangul composition/decomposition and compatibility ligatures; preserve empty input and idempotence.
- [x] Simulator policy: reject invalid UTF-8, nonstring text, and missing/nonintegral/unknown forms with explicit errors. Never replace malformed bytes silently.
- [x] Preserve existing locale contexts and earlier-retail namespace absence.

## How it works

- [Lua API architecture](../lua-api.md)
- [Opaque locale contexts](intl-locale-context.md)

## Implementation inventory

- `src/c_api/c_intl/normalization.rs` — validated UTF-8 normalization and predicate.
- `src/c_api/c_intl.rs` — PTR namespace registration; existing context API unchanged.
- `Cargo.toml` / `Cargo.lock` — direct `icu_normalizer = =2.1.1`, matching the already locked version. Default features disabled; only `compiled_data` requested. ICU supplies normalization algorithms and data instead of a partial handwritten Unicode implementation. No dependency upgrades intended; updates require deliberate review.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_normalization.rs`

## Known gaps (current cycle)

- [ ] Pinned Normalize declares `MayReturnNothing=true`, but specifies no failure condition. This simulator chooses explicit errors consistently with IsNormalized rather than guessing when to return no values. Native failure/invalid-input behavior remains unverified.
- [ ] `SecretArguments=AllowedWhenUntainted` enforcement remains unverified.
- [ ] Native Unicode data-version equivalence and cstring embedded-NUL behavior remain unverified.

## Out of scope

Locale-sensitive casing, collation, formatting, parsing, other context methods, and native security/coercion guarantees are not part of normalization.
