# Patch 7.2.5 API page audit

Warcraft Wiki page 448509, refetched 2026-10-08 and pinned to revision 4311256 (2021-04-19). Sixteen API-template occurrences; canonical names survive shortened display labels. Source is a short top-level New/Changes summary, not a consolidated inventory. All prose is retained; unnamed functions/events are not invented.

## Coverage matrix

| Statement / surface | Scope | Proof |
|---|---|---|
| All sixteen API identities | Raw publication/absence plus lookup after cached retail startup | Discovery: six exact gaps; final thirteen published / three exact gaps |
| Three C_Garrison tree queries | Real server-input catalog, selected tree context, optional friendship faction, garrison type + class filtering | RED: old synthesized function returns one nil rather than no values for missing pair; four cached and two bare state/argument cases pass |
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

## Occurrence accounting

All **28 identities** are accounted: sixteen inventory and twelve extract rows; seventeen bounded, seven pending and four metadata-only. The three publication gaps are C_UI.Reload, C_Unit and ReloadUI historical absence. Four substantive prose rows remain pending: unnamed commentator functions, unnamed transmog functions, ambiguous C_Unit namespace and full reload/rename semantics. Headings/title and event-documentation editorial context receive no runtime credit. Namespace presence never credits unnamed member behavior.

## Original pre-integration proof

Runtime scope `43dd7ed8e` contains the full model and 43-register/sweep set. [Command receipts](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/p725-context.json) and complete logs retain commands, code revisions, exits and hashes. RED's added uncommitted harness is separately retained and hashed; its receipt revision pins the pre-model runtime, not that harness. No full integration suite, WoW texture tests, vendor edits, agents, push or merge.

- All **43 publication sweeps plus factory pass (44/44)**, across **8,866 observations**. Negative control changes only ChatBubbles.GetAllChatBubbles added → removed: exact **3 → 4** gaps, expected exit 1. Earlier sweep gap sets are unchanged.
- Own four cached behavior cases plus publication pass; two bare tree tests pass. Existing garrison integration selection **28/28** and AnimaDiversion integration selection **42/42** pass. Cached garrison selection **37/38**; the one failure is retained below. The initial prefork AnimaDiversion filter selected zero cases and earns no coverage; the real integration selection supplies proof.
- Parser/extractor/shared-validator fixtures **26/34/8** pass. All **43 registers** and **40/43 extracts** reproduce with recorded/inherited flags. The same 12.0.5/12.0.7 mismatches and 12.1.0 unsupported-template failure are retained, not reported green. All **219 original inputs and 92 old extraction-mode outcomes** remain unchanged.
- Format and requested Mists tests check pass with **zero non-vendor warnings**. Six inherited iced manifest deprecations remain unsuppressed. Retail build and bounded startup pass; startup reports zero Lua errors and prints `[]`.

The read-only own validator scopes registers/sweeps through historical_registers/historical_sweep_tests at the recorded revision and derives counts from files. All **twenty validators pass** (nineteen prior plus own). Own validator also passes in an independent shared clone, still passes with an additional audit register/sweep, rejects protected historical-input whitespace tampering and passes after restoration; [portability proof](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/p725-validator-portability.json). Wiki index/log grow without losing existing content. No __pycache__ is retained.

## Inherited scoped failure

`blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` fails in both this branch and an exact Git archive of base `85c2acb2d`, with the same `ipairs` nil error at Blizzard_AdventuresCombatLog.lua:90. OnLoad calls **GetAutoCombatDamageClassValues**, not one of the three tree queries. [Baseline receipt/context](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/p725-garrison-baseline-context.json) records base source revision, matching unchanged caller/implementation hashes and complete log. Both the existing assertion and vendor code remain intact; no unrelated damage-class shim/model was added. The overall verification runner exits 1 because it faithfully retains that failure; other targeted commands pass. This is not an all-green garrison suite claim.

## Integrated proof after 7.3.0

Rebased onto master `1ade15b52`; commit `2b8d2d245` replaces the 7.3.0 placeholder with the real register. Integrated runtime/register scope is `80bb6f64edb56e257bc6e22cdb163defd86197e1`. Subsequent evidence/validator/wiki commits do not change that runtime scope. [Fresh command ledger](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/integrated/p725-final-command-ledger.md) records commands, actual driver revisions, exits and complete hashed logs separately from immutable original receipts.

- **44/44 registers and 41/44 extracts reproduce** with recorded/inherited flags. The same 12.0.5/12.0.7 mismatches and 12.1.0 unsupported-template error remain attributable inherited failures. Shared receipt extension adds 7.3.0 with `--legacy-summary-tables`, three inventory rows and zero gaps; saved-extract rows are independently refreshed.
- **44 publication sweeps plus factory pass (45/45)**, across **8,869 observations**. Own three gaps remain three; every earlier/later gap set is unchanged. No supersession edits or preservation exceptions added. Negative control adds only the ChatBubbles row: **3 → 4**, expected exit 1, no resolved/stale IDs.
- Own integration **2/2** and prefork **5/5** pass. Garrison integration **28/28**, order hall integration/prefork **6/6 each**, and cached garrison **37/38** match prior scope. AnimaDiversion contributes **42 passing integration cases** within the broader `anima` selection (163/163). The broader prefork `anima` selection passes 9/9 animation cases, **not diversion coverage**. Exact `anima_diversion` prefork selection runs zero cases: diversion tests use `#[test]`, not the prefork macro.
- Exact archived master `1ade15b52` reproduces the sole cached explicit-load failure, with the same `ipairs` nil diagnostic at Blizzard_AdventuresCombatLog.lua:90 and unchanged assertion/caller/backing implementation. No new failures or unrelated shim added. Separately built master and branch **addons-enabled startup both print `[]`**.
- Python fixtures **27/34/8**, format and Mists tests-check pass; **zero non-vendor warnings**, six inherited iced manifest warnings unsuppressed. Original historical 43-register scope remains fixed; integrated validator uses the recorded 44-register revision rather than current globs. [Validator matrix](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/integrated/validator-matrix.json) records **all 21/21 validators passing**. The new validator compares complete branch/master failure payloads byte-for-byte; ignored 3D stderr is not mistaken for new assertion failures.

No native historical catalog, reload lifecycle or unspecified member/event reconstruction is claimed. No vendor/Wowless edits, agents, push or merge; no `__pycache__` retained.

## Sources

- [Pinned wikitext and provenance](../../../data/patch-api/sources/7.2.5-api-changes.provenance.json).
- [Register](../../../data/patch-api/sources/7.2.5-wikitext-register.json).
- [Complete evidence](../../../data/patch-api/evidence/7.2.5-session-2026-10-08/).
- [Spec](../../specs/patch-7-2-5-publication-sweep.md).

## See Also

- [[patch-7-3-2-api-audit]] — template and later source.
- [[patch-audit-validator-portability]] — historical register scope and exact input protection.
