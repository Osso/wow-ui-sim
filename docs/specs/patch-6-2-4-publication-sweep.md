# Patch 6.2.4 publication sweep

Audit [Warcraft Wiki page 212964, revision 2072181](../../data/patch-api/sources/6.2.4-api-changes.wikitext). Current retail publication/absence does not establish historical native numeric-ID or tuple parity.

## What it must do

- [x] Retain each standalone addition, both rename identities, every bare removal and the explicit realmName CVar removal; account for every plaintext statement and preserve the unexpanded diff reference.
- [x] Probe raw/lookup publication and CVar value/default absence after unmodified cached startup, applying all merged later registers. Keep 7.0.1 then 7.0.3 placeholders before 7.1.0.
- [x] Require the exact known-gap set; prove one-identity negative control and bounded current parent/game identity/count behavior with real backing state, in bare and cached environments.
- [x] Retain complete whole-word cached-retail and src/tests scans for all removed names and exact master/p703 later-register checks. Any consumer prevents new retirement; preserve classic compatibility.
- [x] Preserve previous inputs/extraction outcomes, reproduce registers/extracts using recorded flags, run only targeted requested gates, compile Mists tests with zero non-vendor warnings and preserve wiki index/log.
- [ ] Seal the read-only, checkout-independent historical validator and prove extra later audits do not expand scope; tampered protected inputs must fail.

## How it works

- [Audit](../wiki/investigations/patch-6-2-4-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in `--indented-api-lists`, no default behavior change.
- `tools/extract_patch_non_inventory.py`: opt-in `--retain-patch-diff-reference`, recorded alongside existing `--preserve-examples`.
- `tests/patch_6_2_4_publication_sweep.rs`, `tests/data/patch_6_2_4_sweep_known_gaps.json`: register-driven cached publication and exact gaps.
- `tests/patch_6_2_4_behavior.rs`: existing C_BattleNet parent/game IDs, wrong-kind GUIDs, friend index vs ID, secondary lookup and removal/count mutation.
- `data/patch-api/sources/6.2.4-page-coverage.json` and `data/patch-api/evidence/6.2.4-session-2026-10-08/`: complete occurrence accounting and portable proof.

## Tests asserting this spec

Python generator/extractor/validator fixtures; own prefork sweep and negative control; own bare/cached successor-state tests; existing `c_battle_net_probes` and cached `blizzard_deprecated_battle_net` cases. [Audit and exact command ledger](../wiki/investigations/patch-6-2-4-api-audit.md) retain the proof boundaries.

## Known gaps (current cycle)

- [ ] Exhaustive historical strict Account ID vs Game Account ID input/invalid-ID policy.
- [ ] Seventeenth return of numeric-ID `BNGetGameAccountInfo` and gameID→parent translation.
- [ ] Sixth return of `BNGetFriendInfoByID` and active-game-account selection; current index-based successor chooses first account and ignores optional selection.
- [ ] Unquantified all-functions Toon→GameAccount architectural migration beyond named examples.
- [ ] Authoritative current-realm backing for GetRealmName rather than the existing temporary constant.

These are seven retained source rows across five topics, not publication gaps. No retired numeric-ID API is reintroduced to emulate an obsolete historical surface.

## Out of scope

Native 6.2.4 reconstruction, linked diff-subpage audit, guessed return tuples/active-account policies, shims, vendor behavior edits, full integration suite, texture tests requiring absent WoW data, push/merge and other-worktree changes.
