# Patch 10.1.0 publication sweep

Account for page 230704, revision 2236681 (2023-06-15T22:35:50Z) against current retail 12.1.0.

## Requirements

- Preserve all 129 inventory occurrences and every non-inventory occurrence.
- Apply all twenty-one chronological later registers, 10.1.5 through 12.1.0.
- Require an exact reviewed gap set; publication/absence/event registration grants no signature, output, security, historical or native parity.
- Preserve later inputs, classic profiles and cached/vendor Lua.

## Verification

Every isolated publication sweep, exact one-row negative control, extraction reproduction, formatting, Mists test check and bounded startup. Affected behavioral/prefork tests required only for changed runtime surfaces.

## Evidence

[Page audit](../wiki/investigations/patch-10-1-0-api-audit.md) owns per-ID proof boundaries.
