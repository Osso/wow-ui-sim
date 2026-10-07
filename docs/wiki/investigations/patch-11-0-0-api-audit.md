# Patch 11.0.0 API page audit

Page 585562, revision 6726780 (May 25, 2026, 20:32:45 UTC), retrieved October 7, 2026 UTC. Branch p1100-page starts from p1102-page revision df3c37b2e. Default retail carries 12.1.0, not reconstructed expansion-launch behavior.

## Source and accounting

495 inventory occurrences: 346 added, 103 removed, 46 changed. Eight added/removed header counts match exactly. Fourteen later registers, 11.0.2 through 12.1.0, supersede chronologically. Existing parser/extractor handles this page without modification; all fifteen registers regenerate byte-identically. All fourteen existing registers and later source/fixture/coverage inputs remain unchanged.

842 unique IDs: 495 inventory + 347 extract. Ledger: 233 partial-development-green, 96 bounded-coverage, 494 audit-pending, 19 metadata-only. Every extract ID is assigned once: ten summary/spell-transition, 66 menu, eight mouse-input, 141 enumeration, 91 structure, twelve deprecation and nineteen metadata rows. Source candidates confer no runtime credit. All 328 substantive extract occurrences remain behavior-pending; 166 publication gaps remain. Changed inventory annotations receive publication-only proof.

## Bounded closures and retained gaps

Initial discovery: 320 OK / 175 gaps. Nine cheap namespace retirements leave 329 OK / 166 exact gaps. C_MajorFactions.GetFeatureAbilities, IsPlayerInRenownCatchUpMode and RequestCatchUpState; C_Map.IsMapValidForNavBarDropDown; C_PvP.GetSoloRBGMinItemLevel; C_Scenario.GetCriteriaInfo and GetCriteriaInfoByStep; C_Traits.GetStagedPurchases; C_TransmogSets.GetBaseSetsCounts now remain absent through repeated ordinary/raw lookup. Qualified cached searches found no consumers; bare-name hits use other namespaces or successor names. Classic registrations and Blizzard wrappers remain untouched.

[Per-ID review](../../../data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-gap-review.json) records all 175 initial failures, nine closures, exact observations, implementation/cache candidates and precise missing contract boundaries. Account-bank/transfer services, spell-bank producers, quest rewards/account filters, PvP queues, crafting operations, transmog filters and smaller access/data queries remain gaps rather than new placeholders. Four missing renderer/platform CVar defaults remain explicit configuration gaps; no hardware settings enabled. ModelSceneActor glue-model registration is an intentional 3D scope gap.

GetMouseFocus remains because current cached Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:398 calls it. Five SpellBook transition aliases remain: cached Blizzard_Deprecated/11_0_0_SpellBookAPITransitionGuide.lua:130,132,138,140,142 assigns them. No deprecation wrapper deleted. Other published legacy globals remain coordinated global/successor migration gaps, not fabricated consumer claims. The page itself lists SPELL_TEXT_UPDATE as added at line 617 and removed at line 626: both IDs retained, removed occurrence stays an exact source-conflict gap. No guessed historical winner.

[Extract scout](../../../data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-extract-scout.json) preserves each statement and proof boundary. Menu examples need complete cached template/input/callback tests; mouse propagation needs overlapping-region dispatch/focus-order proof. Spell identifiers and player/pet banks need concrete conversion tests. Enum values and populated DTOs are not established by declaration existence. Historical 11.0.0 deprecation is distinct from planned 11.0.2 removal. Source typo repeating OnMouseDown remains literal.

## Verification

Runtime/test revision 7e51a4525. Repeated-lookup test RED before implementation, GREEN afterward. Fifteen isolated publication sweeps pass unchanged exact later fixtures; [table](../../specs/patch-11-0-0-publication-sweep.md#local-proof). Negative control changes only C_AdventureMap.GetAdventureMapTextureKit added → removed: exactly one new failure, no resolved failures, 166 → 167 gaps, expected exit 101. New full cached Game prefork retirement test passes.

At accounting revision 2636acc12, formatting and Mists test check pass with zero non-vendor warnings. Six existing iced vendor manifest deprecations plus summary remain unsuppressed. Existing one-filter major-faction prefork passes (one case); eighteen extractor/register fixtures and deterministic extract pass. Separate retail binary build and bounded startup exit 0 with JSON `[]`. Changed Rust lines reviewed manually; no readability findings. Final results are recorded in the [proof ledger](../../../data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-proof.json). Failed development commands remain evidence: initial standalone test target was invalid (autotests=false); two cached-test import corrections followed compiler diagnostics. A guessed existing map filter selected zero tests and receives no proof credit. No native or independent acceptance claim.

Every command uses explicit cwd p1100-page and its own target. Worktree creation uses the prescribed canonical Git metadata operation with cwd in the empty destination. No canonical working files, siblings, cache/vendor Lua or Wowless edited; no full suite, agents/models, push or merge.

## Sources

- [Provenance](../../../data/patch-api/sources/11.0.0-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.0.0-page-coverage.json)
- [Publication contract](../../specs/patch-11-0-0-publication-sweep.md)
- [Retirement searches](../../../data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-retirement-consumers.json)
- [Artifact validator](../../../data/patch-api/evidence/11.0.0-session-2026-10-06/p1100-validate.py) — source hashes, chronological expectations, every source ID, exact gaps and proof scopes.

## See Also

- [[patch-11-0-2-api-audit]] — branch base and first later register.
- [[patch-11-0-5-api-audit]] — accounting conventions.
- [[client-profiles]] — supported retail/classic boundaries.
