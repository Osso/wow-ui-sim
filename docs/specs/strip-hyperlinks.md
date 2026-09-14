# StripHyperlinks

`C_StringUtil.StripHyperlinks` processes balanced well-formed markup from retail 12.0.0. Source: `src/c_api/c_string_util.rs` and `src/c_api/c_string_util/hyperlinks.rs`. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Remove hyperlink wrappers while retaining visible labels and ordinary UTF-8 text.
- [ ] Honor `maintainColor`, `maintainBrackets`, `maintainAtlases` and `maintainTextures` independently; omitted flags default false.
- [ ] Remove literal `|n` when `stripNewlines` is true.
- [ ] Support the native talent consumer combination `(false, true, false, true, true)`.
- [ ] Return exactly one string for supported inputs.

Pinned `StringUtilDocumentation.lua` describes each flag. Literal `|n` preservation when not stripped, escaped-pipe preservation, malformed recognized suffix preservation, and optional-flag Lua truthiness are simulator policies, not native evidence. Non-string text errors explicitly. This slice covers ordinary UTF-8 text; arbitrary byte strings remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_string_util.rs`: registration and argument/result conversion.
- `src/c_api/c_string_util/hyperlinks.rs`: pure bounded markup parser.

## Tests asserting this spec

- `tests/string_util_hyperlinks.rs`: API flag combinations, defaults, Unicode and return arity; `2094af7e2` RED 0/5.
- `src/c_api/c_string_util/hyperlinks.rs`: parser fixtures; standalone RED 1/6 and GREEN 6/6 at `1463fce69`.

Ledgers: `/tmp/strip-hyperlinks-api-red-ledger.json`, `/tmp/strip-hyperlinks-parser-development.json`. Integrated verification pending.

## Known gaps (current cycle)

- [ ] Independent profile, startup and bounded consumer verification.
- [ ] Audit evidence and provenance update.

## Out of scope

Native malformed/nested markup, coercion, byte-string and string-view lifetime semantics, security and full-LoD behavior remain unverified. No rendering-parser or vendor behavior changes.
