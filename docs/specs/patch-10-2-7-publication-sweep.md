# Patch 10.2.7 publication sweep

## Contract

Account for every inventory occurrence and non-inventory statement in Warcraft Wiki page 584467, revision 6268738 (2025-03-20T19:32:28Z). Default retail is the 12.1.0 surface, not historical 10.2.7. All fifteen later registers (11.0.0 through 12.1.0) supersede chronologically.

The isolated cached Game UI sweep probes 104 inventory occurrences. Its failed ID set must exactly equal the reviewed fixture. Publication, absence, CVar defaults and event registration do not establish signatures, populated outputs, security or native parity. `FontInstance` is probed using a FontString implementing that interface, not an invented frame kind.

The page ledger must contain every inventory/extract ID once, with explicit proof scope or precise retained gap. Linked external pages are not expanded. Preserve all later source/register/fixture inputs and deprecation wrappers.

## Bounded behavior

`C_StableInfo.ClosePetStables` clears existing stable-open state and queues `PET_STABLE_CLOSED` with no arguments. Retail/PTR no longer register legacy `ClosePetStables` or `GetWorldPVPAreaInfo`; namespace successors remain. Current qualified cached searches find no legacy consumers. Classic profiles retain both registrations and use the same stable model. No new placeholder, vendor edit or historical epoch feature.

## Verification

- [x] Sixteen isolated publication sweeps with exact fixtures.
- [x] New stable state/event and repeated-lookup behavior tests; cached prefork migration.
- [x] One-row negative control adds exactly one failure without resolving existing failures.
- [x] All fifteen old registers regenerate byte-identically; exhaustive source-ID accounting.
- [ ] Formatting, Mists tests check with zero non-vendor warnings, separate retail binary build and bounded startup `[]`.

## Local proof

See `data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-proof.json`. Development proof is not native or independent acceptance.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |
