# Patch 7.2.5 API page audit

Warcraft Wiki page 448509, refetched 2026-10-08 and pinned to revision 4311256 (2021-04-19). Sixteen API-template occurrences; canonical names survive shortened display labels. Source is a short top-level New/Changes summary, not a consolidated inventory. All prose is retained; unnamed functions/events are not invented.

## Coverage matrix

| Statement / surface | Scope | Proof |
|---|---|---|
| All sixteen API identities | Raw publication/absence plus lookup after cached retail startup | Discovery: six exact gaps |
| Three C_Garrison tree queries | Real server-input catalog, selected tree context, optional friendship faction, garrison type + class filtering | RED: old synthesized function returns one nil rather than no values for missing pair; bounded model added, GREEN pending |
| Chat bubbles, action membership, LFG search, owner/controller global | Existing publication retained | No new native parity claim |
| C_Commentator / C_TransmogSets additions | Namespace publication only | Page does not enumerate functions |
| Event documentation additions | Editorial statement retained | No event identities or payloads given |
| ReloadUI → C_UI.Reload | Historical rename recorded; no retirement | Current retail consumers require ReloadUI; full reload lifecycle missing |
| C_Unit namespace | Recorded problematic | Page links unqualified UnitIsOwnerOrControllerOfUnit; no explicit namespace member given |

## Root cause and modeled behavior

Discovery finds raw-absent garrison members despite synthesized lookup functions. Existing `GarrisonTalentState` models talents/unlock quests, not tree catalogs or active tree selection. No Blizzard Lua defect is implicated.

`src/c_api/c_garrison_trees.rs` adds a retail-only `GarrisonTrees` backing model: optional `current_tree_id` plus a `BTreeMap` catalog keyed by ID. Each catalog row contains garrison type, class ID and optional friendship faction ID. Current-tree queries read mutable simulator context, and enumeration filters both dimensions, returns detached arrays and returns no values for an unknown pair. IDs use deterministic ascending order, not a claimed native ordering. Inputs are explicit server/test data, not a guessed Legion catalog. Existing talent/progression fields are untouched; Mists and other profiles retain their existing surfaces.

Current cached `GarrisonInfoDocumentation.lua` documents nullable current-tree/friendship IDs and `GetTalentTreeIDsByClassID(garrType, classID)` with `MayReturnNothing`. Secret-argument policy, exact numeric coercion and native catalog/order captures remain unverified. No tree progression/research implementation is claimed.

## Retirement decision

**No retirements.** The sole candidate is the historical ReloadUI rename. Complete `/usr/bin/grep -rnE` whole-word `\bReloadUI\b` scans retain thirteen current retail consumer lines (excluding `*Documentation*` files/directories), five src lines and twelve test lines. Whole-name matching includes indirect `pcall(Name, ...)` and guarded uses, not only direct calls. Qualified and bare forms coincide for this global. The [scan receipt](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/p725-retirement-scans.json) pins the full master and p730-page register sets/revisions; neither re-adds ReloadUI. Live consumers alone prohibit retirement.

`C_UI.Reload` has a cached consumer in InterfaceUtil.lua, but existing ReloadUI/GUI reload only dispatch notifications; neither reconstructs the Lua VM or reloads SavedVariables. Adding an event-only alias would disguise the missing lifecycle, so no alias/shim was added. The page's historical absence is retained as a precise current-retail gap rather than breaking callers.

## Verification

Implementation committed before acceptance. Required targeted proof: all publication sweeps, own cached/bare tree tests, existing garrison/anima callers, the three parser/validator fixture scripts, saved artifact reproduction, Mists tests check and format. No full integration suite, WoW texture tests, vendor edits, agents, push or merge. Long Cargo commands retain complete asynchronous logs with command/revision/hash receipts.

Final counts and proof receipts are added after verification. The 7.3.0 placeholder remains first in `later_registers`; integration must replace it with the merged register and refresh own proof.

## Sources

- [Pinned wikitext and provenance](../../../data/patch-api/sources/7.2.5-api-changes.provenance.json).
- [Register](../../../data/patch-api/sources/7.2.5-wikitext-register.json).
- [Complete evidence](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/).
- [Spec](../../specs/patch-7-2-5-publication-sweep.md).

## See Also

- [[patch-7-3-2-api-audit]] — template and later source.
- [[patch-audit-validator-portability]] — historical register scope and exact input protection.
