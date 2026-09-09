# PTR Unicode titlecasing

`src/c_api/c_intl/casing.rs` provides per-word `ToTitle` for C_Intl and locale-context userdata. Shared locale selection and UTF-8 validation follow [Unicode casing](intl-casing.md).

## What it must do

- [x] Publish `C_Intl.ToTitle(text)` and `context:ToTitle(text)` only on PTR. Both take required `cstring` text and return one string on modeled success.
- [x] Use the current global locale or receiver's stored locale; context mutation affects later calls without changing other contexts.
- [x] Titlecase each ICU word separately, lowercasing its remaining cased letters with ICU default titlecase options; copy non-word segments verbatim, preserving punctuation and whitespace.
- [x] Handle multiword mixed case, Turkish dotted I, Dutch IJ, and expanding Unicode mappings rather than uppercasing only the first character.
- [x] Preserve empty input and reject malformed UTF-8, nonstrings, invalid receivers, and malformed locale identifiers using existing casing policy.
- [x] Preserve earlier-retail absence and existing normalization, scalar length, context storage, and lower/upper/fold behavior.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_intl/casing.rs` — shared registration/locale handling plus per-word titlecasing.
- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_titlecase.rs` — focused behavior and profile coverage.
- `Cargo.toml` / `Cargo.lock` — direct `icu_segmenter = "=2.1.1"`, defaults disabled, `compiled_data` and `auto` enabled. Word segmentation is required because ICU's titlecase mapper handles one segment, not all words. `auto` supplies complex-script segmentation; compiled data avoids runtime data downloads. Existing `icu_casemap = "=2.1.1"` supplies titlecase mappings. Upgrade versions deliberately; no unrelated dependency upgrades.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_titlecase.rs`
- Existing `intl_` grouped library tests cover regressions.
- At `0081d03f0`, `cargo test --lib --offline --no-default-features --features sound,gui,client-ptr intl_ -- --nocapture` passed 10 tests; the corresponding `client-retail` run passed 5. RED first failed both PTR titlecase tests with `ToTitle missing`.

## Known gaps (current cycle)

- [ ] Per-word invariant segmentation and default ICU titlecase options are simulator choices, not confirmed WoW titlecasing policy or locale-tailored word boundaries.
- [ ] Both pinned APIs declare `MayReturnNothing=true`; native conditions are unknown. Modeled success returns one string; invalid input errors instead of silently replacing bytes.
- [ ] Native Unicode-data version, embedded-NUL semantics, locale negotiation, secret arguments, taint, and protected access remain unverified.

## Out of scope

Other C_Intl algorithms, new locale parsing rules, and changes to lower/upper/fold behavior. No native or security conformance is inferred from ICU-backed tests.
