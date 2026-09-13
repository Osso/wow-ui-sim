# Truncate when zero

`C_StringUtil.TruncateWhenZero(number)` returns an integer string after rounding down, or an empty string when the integer is zero.

## What it must do

- [x] Retail 12.0.0+ exposes the helper and returns exactly one string.
- [x] Ordinary finite nonnegative inputs round down; 0, 0.1 and 0.99 return empty strings, while 1.9, 2.99 and 10.75 return `"1"`, `"2"` and `"10"`.
- [x] Calls do not retain earlier results.
- [x] Reject missing, nonnumber and nonfinite inputs as explicit simulator policy, not native validation proof.

## How it works

- [Patch audit model](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_string_util.rs`: registration and number-to-string callback.

## Tests asserting this spec

- `tests/c_api_surface.rs`: three `c_string_util_truncate_when_zero_*` tests; RED 0/3 at `b8032c876`; GREEN 3/3 at `cfdd647c4`. Independent verification pending.

## Known gaps (current cycle)

- [ ] Native coercion, validation/error details, negative and extreme finite formatting, locale, historical availability and consumer semantics remain unproven.

## Out of scope

Secrets/security and native numeric-policy conformance beyond the documented ordinary cases. This slice does not credit neighboring rounding APIs.
