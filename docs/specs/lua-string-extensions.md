# PTR Lua string extensions

Five PTR-only string methods provide byte-oriented matching and one-sided trimming in `src/lua_api/globals/real/string_extensions.rs`. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) records their signatures; [Lua API architecture](../lua-api.md) describes registration.

## What it must do

The byte-set, case-sensitive, literal-matching semantics below are **simulator assumptions requested for this implementation**, not independently observed native behavior. The register establishes two required `stringView` matching arguments and one boolean return; trims accept `str` plus `characters` with default ` \r\n\t` and return one `stringView`.

- [ ] PTR publishes `string.contains`, `startswith`, `endswith`, `ltrim`, and `rtrim`; earlier retail omits them.
- [ ] Matching operates literally and case-sensitively on arbitrary bytes. Empty needles match, including empty inputs; nonempty needles do not match empty inputs.
- [ ] Trimming removes only consecutive bytes belonging to the supplied byte set from the selected edge; the opposite edge remains unchanged. Empty sets remove nothing.
- [ ] Omitted trim characters use exactly space, carriage return, line feed, and tab, not vertical tab or form feed. Explicit nil uses the same default as a simulator convention.
- [ ] NUL and invalid UTF-8 survive matching and trimming unchanged; no text decoding or Lua pattern expansion occurs.
- [ ] Existing `string.trim` and global `strtrim` behavior remains unchanged.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/string_extensions.rs`: native byte-oriented operations.
- `src/lua_api/globals/real/mod.rs`: PTR module gate.
- `src/lua_api/globals/register.rs`: PTR registration into the existing string library.
- `src/loader/tests/wow_api_globals/mod.rs`: grouped test wiring.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_string_extensions.rs`: matching, trimming, binary preservation, legacy behavior, and profile publication.

## Known gaps (current cycle)

- [ ] Confirm modeled byte-set, case, literal, empty-needle, and nil-default semantics against native PTR observations.
- [ ] Validate the pinned `AllowedWhenUntainted` security contract independently.

## Out of scope

- Unicode character boundaries, normalization, or locale matching: this model deliberately processes bytes.
- Native coercion, invalid-input errors, secret/taint/protected/forbidden behavior, and string-view identity/lifetime: signatures and ordinary tests do not establish these contracts. Nonstrings produce explicit simulator errors rather than guessed coercions.
- Changes to existing trim/global methods or audit-manifest credit: separate work.
