# Patch 10.2.6 publication sweep

Account for every occurrence in Warcraft Wiki page 582132, revision 6268742 (2025-03-20T19:36:42Z). [Audit](../wiki/investigations/patch-10-2-6-api-audit.md) describes the implementation and evidence. Default retail carries 12.1.0, not reconstructed 10.2.6.

## What it must do

- [x] Probe all 220 inventory occurrences in isolated cached Game UI; require the exact reviewed failure set. Apply all sixteen later registers, 10.2.7 through 12.1.0, chronologically.
- [x] Account for all 358 unique inventory/extract IDs once, preserving substantive gaps and distinguishing publication from behavior.
- [x] Retail/PTR must not fabricate `C_CameraDefaults.GetCameraFOVDefaults`, `C_TaskQuest.GetUIWidgetSetIDFromQuestID` or legacy `CanSummonFriend` on repeated lookup. Keep existing global camera defaults and namespace RAF successor published. Cached retail searches find no removed-API consumers; classic registrations remain unchanged.
- [x] Preserve `profanityFilter`: current cached Settings consumer still requires it. Preserve Blizzard deprecation wrappers and all existing registers/fixtures.
- [x] A one-row negative control adds exactly one failure without resolving existing failures. Seventeen isolated publication sweeps, retirement RED/GREEN, cached prefork, formatting, Mists tests check and bounded retail startup pass.

## How it works

- [Page audit](../wiki/investigations/patch-10-2-6-api-audit.md) — publication classifier, supersession, retained boundaries and proof.
- [Client profiles](../wiki/systems/client-profiles.md) — retail/classic profile gates.

## Implementation inventory

- `src/c_api/patch_retired_members.rs` — two namespace retirement markers inside the existing retail-only module.
- `src/lua_api/globals/stubs/global_stubs.rs` — legacy RAF global stays classic-only.
- `tests/common/publication_sweep.rs` — unchanged register-driven cached Game publication classifier.
- `data/patch-api/sources/10.2.6-*` — pinned source, register and full occurrence ledger.

## Tests asserting this spec

- `tests/patch_10_2_6_publication_sweep.rs` — exact inventory/publication fixture.
- `tests/patch_10_2_6_publication_fixes.rs` — repeated raw/ordinary lookup, preserved successors.
- `tests/patch_10_2_6_cached_surfaces.rs` — same boundary after cached Game loading, retained CVar and no new Lua errors.
- `data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-validate.py` — exhaustive source-ID, supersession and retained-proof validation.

## Known gaps (current cycle)

- [ ] Twenty exact publication gaps remain, with per-ID reasons in `p1026-gap-review.json`. `profanityFilter` is intentionally retained for a current consumer, not silently retired.
- [ ] `gxMTDecals` publishes but historical default 1 differs from current 0. Classifier logs default drift separately from publication failure; ledger grants no default parity credit.
- [ ] All 121 substantive extract occurrences retain precise pending enum/DTO/migration contracts in `p1026-extract-scout.json`.

## Out of scope

Historical epoch reconstruction, native parity, expanded linked pages, new placeholder producers, vendor edits, 3D rendering, full suites, push, merge and independent model review are excluded by task/profile boundaries.

## Local proof

`data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-proof.json` records commands, revisions and output hashes. Publication-only success does not establish signature, populated output, security or behavioral parity.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.2.6 | 220 | 200 | 20 | 0 |
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
