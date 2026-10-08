# Patch 5.4.8 publication and combat restrictions

Audit the pinned [2014 retail source](../../data/patch-api/sources/5.4.8-api-changes.wikitext), not Mists Classic. [Audit](../wiki/investigations/patch-5-4-8-api-audit.md) owns occurrence accounting and historical limits.

## What it must do

- [x] Probe every registered identity in the prefork cached retail runtime, applying later retail registers only; distinguish publication from behavior.
- [x] Retain precise absent/retired-CVar and native-policy limits rather than adding compatibility shims.
- [x] Block insecure combat writes to the page's currently readable CVars before value, scale, persistence or event mutation; allow secure writes and out-of-combat addon writes.
- [x] Block insecure `SetUIVisibility(false)` in combat, allow `true`, and preserve secure/out-of-combat transitions.
- [ ] Keep extraction and register generation opt-in/reproducible; validate historical scope at pinned Git revisions in clean/later-audit checkouts.

## How it works

- [Audit and evidence](../wiki/investigations/patch-5-4-8-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `src/c_api/cvar_combat_policy.rs`: retail protected-name policy, no default publication.
- `src/lua_api/globals/set_cvar_verb.rs`: shared legacy/C_CVar write access before side effects.
- `src/lua_api/globals/ui_visibility.rs`: existing UIParent visibility model with combat hide protection.
- `src/lua_api/taint.rs`: active caller taint query.
- `tools/gen_patch_wikitext_register.py`: opt-in Breaking changes identity capture.
- `tests/patch_5_4_8_publication_sweep.rs`: current-retail publication sweep.
- `tests/patch_5_4_8_combat_restrictions.rs`: observable combat state/side-effect contract.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py`: changed CVar/API bullet fixture.
- `tests/patch_5_4_8_publication_sweep.rs`.
- `tests/patch_5_4_8_combat_restrictions.rs`.
- `data/patch-api/evidence/5.4.8-session-2026-10-08/validate.py`.

## Known gaps (current cycle)

- [ ] Seven publication gaps: `bloatTest`, `bloatnameplates`, `bloatthreat`, `consolidateBuffs`, `maxAlgoplates`, `repositionfrequency`, `targetOfTargetMode`; no current default/model supplied by this page.
- [ ] Historical combat transitions for 14 currently unreadable settings; exact native error/notification/default parity remains unproved.
- [ ] Replace ordered 6.0.1/6.0.2 placeholders after their retail registers integrate; no intersecting own identity at recorded queued revisions.

## Out of scope

Historical client emulation, exact native failure text/notifications, linked forum expansion, Classic 5.5.x register supersession, vendor edits and full integration suite.
