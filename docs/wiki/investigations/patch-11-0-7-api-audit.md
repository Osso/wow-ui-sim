# Patch 11.0.7 API page audit

Page 609319, revision 6726777 (May 25, 2026, 20:32:18 UTC), retrieved October 7, 2026 UTC. Evidence directory retains the requested October 6 session label. Audit accounts for 98 inventory occurrences and 83 non-inventory rows. Default retail carries 12.1.0, not a reconstructed 11.0.7 client. Branch p1107-page starts at p1110-page revision 6aba97a94; no later audit inputs changed.

## Source and supersession

All eight added/removed header counts match. Eleven later registers, 11.1.0 through 12.1.0, apply chronologically; latest add/remove wins. Three reversals receive metadata-only credit: C_GameModeManager.GetCurrentGameModeRecordID, C_GameModeManager.GetGameModeDisplayInfo and C_SuperTrack.GetNextWaypointForMap. Wikitext is authoritative; retained text does not expand external linked pages. The Enum.AddOnProfilerMetric transclusion is an explicit pending reference, not an invented historical enum table or silently dropped row.

Direct HTTP retrieval returned 403. Read-only browser MediaWiki query retained exact revision content and provenance. No cached/vendor Lua was edited or replaced. All command cwd values and build artifacts belong to this worktree; no sibling target reuse or artifact copying.

## Root causes and bounded fixes

Initial cached sweep: 65 OK / 33 gaps. Four removed members were fabricated by namespace autostub lookup: the three WorldLootObject callouts on C_ArrowCalloutManager and C_WorldLootObject.GetCurrentWorldLootObjectSwapInventoryType. Full current cached retail Lua search found no consumers. Existing retirement policy now blocks repeated ordinary/raw lookup on supported retail epochs. AcknowledgeCallout stays callable; classic registration is untouched.

RemoveRaidTargets was absent despite existing GUID-keyed marker state used by SetRaidTarget/GetRaidTargetIndex. The modeled global clears that map, returns no values, queues RAID_TARGET_UPDATE and dispatches notification after mutation. Concrete player/two-party-member/hostile-target fixtures verify clearing, callback-visible state, repeated empty clearing and reassignment. Notification timing follows simulator SetRaidTarget policy; native restricted-action authorization remains unmodeled. Cached RaidMarkersDocumentation.lua:89–92 documents removal of all markers with no arguments/results. Full cached SecureActionButton_OnClick dispatch now reaches the real clear-all producer without substituting Blizzard code.

Two unit behavior tests fail before implementation and pass afterward. Five bounded publication closures leave 28 exact gaps. [Per-ID review](../../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-gap-review.json) retains each initial defect, observation and closure/boundary.

**Preserved deprecation wrappers:** cached Blizzard_DeprecatedLFG/Deprecated_LFG.lua:9 aliases C_LFGInfo.IsPremadeGroupEnabled; lines 10–15 define C_LFGList.GetSearchResultMemberInfo. Current search matches are these definitions, not active call sites. Shared sweep accepts loaded deprecation source/alias identity; this is not strict absence or successor DTO proof. Both wrappers remain untouched, and their two isolated prefork tests pass. GetSearchResultPlayerInfo remains a publication gap; its cached wrapper does not close it.

Remaining gaps cover role policy, match-result DTOs, game-rule frame-strata conversion policy, garrison visibility, cross-faction/premade eligibility, LFG member/authentication producers, lobby lifecycle, quest geometry, difficulty redirection, spectating, tracked-item names, vignette metadata/health, world-loot state/interaction and CPU/GPU bottleneck telemetry. No placeholder publication added.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Unsuperseded add/change publication | 56 | Partial-development-green; signatures/output/security/behavior unproven |
| Unsuperseded removals | 11 | Nine strict bounded absence/rejection probes; two loaded deprecated wrapper/alias acceptances |
| Later-superseded inventory OK | 3 | Metadata-only; no historical credit |
| Inventory gaps | 28 | Exact missing producer/model/policy boundaries |
| Non-inventory contracts / editorial context | 69 / 14 | Exhaustive ranked scout / metadata-only; no runtime credit |

181 unique IDs: 56 partial-development-green, 11 bounded-coverage, 97 audit-pending, 17 metadata-only. Page accounting/publication audit is complete; ledger remains in-progress because behavior/producer gaps remain. Extract batches: profiler 9, populated LFG search DTO 1, enum rows 31, structure rows 28, editorial 14. Every extract ID assigned once. Existing profiler and LFG implementations are candidates, not page-specific semantic proof.

## Development proof

[Spec and sweep table](../../specs/patch-11-0-7-publication-sweep.md#local-proof) and [proof ledger](../../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-proof.json) retain command/cwd/revision/result scopes. Runtime revision 9b4119c9e; final cached-consumer test revision e29e8b98f. Later edits are evidence/docs only.

All twelve sweeps run alone and pass, matching unchanged later fixtures. Negative control changes only C_AccountStore.BeginPurchase added → removed: exactly one new gap, no resolutions, 28 → 29, expected exit 101. Two new unit tests GREEN after RED. Three isolated prefork cases pass: cached secure clear-all dispatch and both deprecated-LFG surfaces. Initial cached-consumer fixture incorrectly accessed SECURE_ACTIONS, a local table at SecureTemplates.lua:260; corrected fixture uses exported SecureActionButton_OnClick. Failed proof stays retained and marked superseded, not hidden.

Sixteen extractor/register fixtures, deterministic extract reproduction, all twelve byte-identical register regenerations, artifact validation and formatting pass. Artifact proof runs at 45e5d932a; later evidence/docs edits do not invalidate runtime or accounting scope. Mists test check has zero non-vendor warnings; six pre-existing iced manifest deprecations plus vendor summary remain unsuppressed. Separate retail build and bounded exit-0 startup return []. Changed Rust lines manually reviewed for readability. Local Cargo logs are ignored; retained JSON records exact results. No full suite, agents/models, independent/native acceptance, push or merge.

## Sources

- [Publication contract](../../specs/patch-11-0-7-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.0.7-api-changes.provenance.json)
- [Page ledger](../../../data/patch-api/sources/11.0.7-page-coverage.json)
- [Extract scout](../../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-extract-scout.md)
- [Retirement consumer searches](../../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-retirement-consumers.json)
- [Artifact validator](../../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-validate.py)

## See Also

- [[patch-11-1-0-api-audit]] — branch base and immediate supersession boundary.
- [[patch-11-1-5-api-audit]] — exact-gap/page-ledger conventions.
- [[client-profiles]] — supported profiles and current retail epoch.
