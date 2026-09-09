# PTR text length

## Contract

Pinned 12.1.5 declares `C_Intl.Length(text: cstring)` and `LuaLocaleContext.Length(text: cstring)`, returning `result: number`, with `MayReturnNothing=true` and `SecretArguments=AllowedWhenUntainted`.

## Simulator behavior

- Both APIs return exactly one number: the Unicode scalar count of valid UTF-8 text, computed independently of locale.
- Empty text counts as zero; `é` counts as one, decomposed `e` plus combining accent as two, `😀` as one, and `👩‍💻` as three.
- Reject malformed UTF-8 and nonstring inputs through the same validation used by [normalization](intl-normalization.md). No replacement characters or new dependencies.
- Context calls validate the existing userdata receiver without changing its stored locale. Independent contexts produce the same count.
- PTR only; earlier retail retains namespace absence. Existing [context storage](intl-locale-context.md) and normalization APIs remain unchanged.

Scalar counting and explicit errors are simulator assumptions, not confirmed native counting units or failure behavior. No claim of byte, grapheme, UTF-16-unit, or native cstring/NUL semantics. Native no-return conditions and secret/taint enforcement remain unverified.

## Behavioral tests

`src/loader/tests/wow_api_globals/patch_12_1_5_intl_length.rs` covers both APIs, empty/ASCII/composed/decomposed/emoji/ZWJ input, malformed bytes, receiver validation, independent contexts, return arity, and earlier-retail absence.
