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
