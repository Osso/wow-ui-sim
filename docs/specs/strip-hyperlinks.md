# StripHyperlinks

`C_StringUtil.StripHyperlinks` processes balanced well-formed markup from retail 12.0.0. Source: `src/c_api/c_string_util.rs` and `src/c_api/c_string_util/hyperlinks.rs`. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Remove hyperlink wrappers while retaining visible labels and ordinary UTF-8 text.
- [x] Honor `maintainColor`, `maintainBrackets`, `maintainAtlases` and `maintainTextures` independently; omitted flags default false.
- [x] Remove literal `|n` when `stripNewlines` is true.
- [x] Support the native talent consumer combination `(false, true, false, true, true)`.
- [x] Return exactly one string for supported inputs.

Pinned `StringUtilDocumentation.lua` describes each flag. Literal `|n` preservation when not stripped, escaped-pipe preservation, malformed recognized suffix preservation, and optional-flag Lua truthiness are simulator policies, not native evidence. Non-string text errors explicitly. This slice covers ordinary UTF-8 text; arbitrary byte strings remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_string_util.rs`: registration and argument/result conversion.
- `src/c_api/c_string_util/hyperlinks.rs`: pure bounded markup parser.

## Tests asserting this spec

- `tests/string_util_hyperlinks.rs`: API flag combinations, defaults, Unicode and return arity; `2094af7e2` RED 0/5, then 5/5 behavioral PASS on retail 12.0.0/12.0.5/12.0.7.
- `src/c_api/c_string_util/hyperlinks.rs`: parser fixtures; standalone RED 1/6 and GREEN 6/6 at `1463fce69`.
- Actual unmodified Talent UI consumer fixture: nine output, five strict-text rejection and one color assertions.

Independent proof: `/tmp/verify-strip-hyperlinks-ledger.json`; metadata proof: `/tmp/verify-strip-hyperlinks-metadata-ledger.json`. Fmt/check/default binary builds and startup `[]` passed. The 12.0.0 test-runner stdout directly proves 5/5, but its enclosing exit code, compiler stderr and warning count are **unknown** because recorder output was lost; `/tmp/verify-strip-hyperlinks-proof-reconciliation.json` accepts only that bounded behavioral evidence. Metadata `1f78ffddc` proves one credit, nine renewals, six additions, 15,104 fresh / zero stale hashes, six bindings and validator exit 0 with 3,410 matching rows. Totals **2349 / 1059 / 2**; snapshot **1,085 / 282**.

## Known gaps (current cycle)

No implementation gap remains in this bounded ordinary-markup slice.

## Out of scope

Native malformed/nested markup, coercion, byte-string and string-view lifetime semantics, security and full-LoD behavior remain unverified. No rendering-parser or vendor behavior changes.
