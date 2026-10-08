# Patch 5.4.2 publication audit

Audit the 2013 retail Warcraft Wiki [pinned page](../../data/patch-api/sources/5.4.2-api-changes.wikitext), pageid 262849, revision 2543251. Parent revision 6441253 identifies TOC 50400, December 2013 and build 17688. This is not Mists Classic 5.5.x. See [audit](../wiki/investigations/patch-5-4-2-api-audit.md).

## What it must do

- [ ] Retain every explicit inventory occurrence and all prose/enum rows. Caption counts must match parsed counts; new tool behavior is opt-in and old sources still reproduce.
- [ ] Probe raw/ordinary API publication, concrete widget construction and event registration; apply later retail registers in chronological order. Keep 5.4.7 then 5.4.8 placeholders before 6.0.1, 6.0.2 and later registers. No Classic registers.
- [ ] Assert five published autocomplete priority values against the pinned numbers.
- [ ] Assert existing guild roster backing preserves explicitly qualified names, tracks state changes and returns nil after removal. Record that unqualified input remains unqualified; this is not automatic realm qualification.
- [ ] Preserve absence of the already-missing StartUnratedArena and securerandom globals. No new retirement without complete whole-word cached/source/test scans and pinned later-register checks.
- [ ] Complete targeted sweeps, affected-area tests, Python fixtures, saved-source reproduction, warning-clean non-vendor Mists tests check, format and historical-validator portability gates.

## How it works

- [Publication sweep implementation](../../tests/common/publication_sweep.rs).
- [Historical validator rules](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in caption inventories and bare removal names.
- `tools/extract_patch_non_inventory.py`: opt-in prose/enum preservation.
- `data/patch-api/sources/5.4.2-page-coverage.json`: every source identity and bounded/pending reason.
- `data/patch-api/evidence/5.4.2-session-2026-10-08/validate.py`: sealed evidence and pinned shared-input proof.

## Tests asserting this spec

- `tools/test_patch_mists_register.py`: concrete inventory and extract fixtures.
- `tests/patch_5_4_2_publication_sweep.rs`: prefork publication/absence discovery and accepted gaps.
- `tests/patch_5_4_2_behavior.rs`: cached existing roster backing, numeric constants and existing absence.

## Known gaps (current cycle)

- [ ] Thirty-seven historical glue/auth/patcher events lack accepted catalogue entries and modeled lifecycle producers.
- [ ] `fastrandom` lacks a modeled fast RNG and secure-environment publication boundary. Do not alias generic randomness as a shortcut.
- [ ] Later removal of IsOnGlueScreen conflicts with a consumed current Blizzard boolean of that name. Preserve the consumer.
- [ ] Secure RNG redirection/performance and guild automatic realm qualification lack the required backing data/policy.

## Out of scope

Native 2013 parity, cryptographic/performance claims, Classic 5.5.x, vendor modification, full integration suites, push and merge. Existing publication is not proof of meaningful domain behavior.
