# Contiguous ASCII spaces

`C_StringUtil.RemoveContiguousSpaces(text, maxAllowedSpaces)` caps consecutive ASCII spaces. Implementation: `src/c_api/c_string_util.rs`; [audit context](../wiki/investigations/patch-12-0-0-api-audit.md).

## What it must do

- [x] Retail 12.0.0+ returns one string with each ASCII-space run capped at the requested count; tests cover limits 0, 1, and 2, including boundary runs.
- [x] Preserve other bytes, including tabs, newlines, NUL, UTF-8 and invalid UTF-8; preserve empty and space-free strings.
- [x] Reject missing, nonnumeric, negative, fractional, and nonfinite limits as explicit simulator policy, not native validation proof.

## How it works

- [Patch API audit model](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_string_util.rs`: namespace registration and byte-preserving transformation.

## Tests asserting this spec

- `tests/c_api_surface.rs`: four `c_string_util_remove_contiguous_spaces_*` cases.

## Known gaps (current cycle)

- [ ] Native coercion, validation/error details, taint/secrets, historical availability, and consumer behavior remain unproven. Independent bounded verification passed at `e1f7fd2a5`: retail 6/6 each on 12.0.0, 12.0.5, and 12.0.7; Mists 1/1 covers only existing helper absence, not this API's availability. The readability-only `36dce310d` names an equivalent positive validation predicate; postchange fmt/check are fresh, while profile/build/startup proof is explicitly reused on that equivalence.

## Out of scope

Native coercion, validation/error details, taint/secrets, and consumer behavior remain unproven. ASCII spaces are not generalized to Unicode whitespace.
