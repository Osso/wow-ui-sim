# Patch 11.0.0 publication sweep

## Contract

Account for all 495 retained inventory occurrences and every non-inventory source ID. Default retail carries 12.1.0; fourteen later registers (11.0.2 through 12.1.0) supersede chronologically. Require exact observed failure IDs, not a tolerated failure count. Publication/absence, CVar defaults and event registerability do not establish signature, populated output, security or native behavior parity.

## Bounded closures

Nine removed members must remain absent through repeated ordinary/raw lookup: three C_MajorFactions members, C_Map.IsMapValidForNavBarDropDown, C_PvP.GetSoloRBGMinItemLevel, two C_Scenario criteria queries, C_Traits.GetStagedPurchases and C_TransmogSets.GetBaseSetsCounts. Qualified cached Lua searches found no consumers; bare-name hits resolve to different namespaces or successor spellings. Do not delete or alter Blizzard deprecation wrappers. Preserve current SpellBook transition aliases. Classic profiles retain prior registration.

## Tests

- `tests/patch_11_0_0_publication_sweep.rs`: full cached Game publication sweep with exact gap fixture.
- `tests/patch_11_0_0_publication_fixes.rs`: repeated lookup absence and callable neighbors.
- `tests/patch_11_0_0_cached_surfaces.rs`: unchanged full cached Game load preserves retirements without new Lua errors.

## Sources

- `data/patch-api/sources/11.0.0-api-changes.provenance.json`
- `data/patch-api/sources/11.0.0-wikitext-register.json`
- `data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-retirement-consumers.json`

## Local proof

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 11.0.0 | 495 | 329 | 166 | PASS |
| 11.0.2 | 34 | 22 | 12 | PASS |
| 11.0.5 | 48 | 38 | 10 | PASS |
| 11.0.7 | 98 | 70 | 28 | PASS |
| 11.1.0 | 116 | 97 | 19 | PASS |
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Fifteen sweeps run alone at 7e51a4525. Later exact fixtures remain unchanged. Repeated lookup RED/GREEN and new full cached Game prefork pass. One-row control introduces exactly one failure (166 → 167). Final format, Mists test check (zero non-vendor warnings), existing major-faction prefork, eighteen parser/extractor fixtures and separate retail build/startup `[]` pass at 2636acc12. Exact revisions/outcomes retained in `p1100-proof.json`; no native acceptance claim.
