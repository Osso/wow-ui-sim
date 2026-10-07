# Patch 11.1.5 API page audit

Page 621744, revision 6726775 (May 25, 2026), retrieved October 7, 2026 UTC. Audit accounts for 125 inventory occurrences and 99 non-inventory rows. Requested evidence path keeps October 6; git host timestamp stamped October 6 (-0500). Default retail carries 12.1.0, not reconstructed historical 11.1.5.

## Source and supersession

All eight inventory header counts match. All nine later registers, 11.1.7 through 12.1.0, apply chronologically; latest add/remove wins. One reversal receives metadata-only credit. Ten registers regenerate byte-identically. Thirty-six existing later raw sources, registers, coverage ledgers and known-gap fixtures remain unchanged against starting master `f3c07b9bd`.

The TOC code example retains its symbolic `[API LatestInterface]` rather than expanding today's value. Wikitext is authoritative; generated plaintext excludes collapsed inventories and preserves every remaining nonblank row. Headings, resource links and explicit build transition receive no runtime credit. Example comments remain contractual candidates, not silently discarded editorial context.

## Root causes and bounded fixes

Initial cached sweep: 81 OK / 44 gaps. Six old GameEnvironmentManager/GameModeManager members and SpellBook.GetTrackedNameplateCooldownSpells were fabricated by namespace autostub lookup. Current cached retail Lua search finds no references; existing epoch-scoped removal policy blocks their repeated ordinary/raw lookup without deleting deprecated wrappers or changing classic registration.

Missing IsInGlobalEnvironment now compares actual Lua caller environment with the global table. Initial Lua helper fails after a custom-environment tail call discards its caller frame; a native primitive observes the caller before replacement. Secure/global/custom chunks and secure-to-global swap pass without weakening assertions. Both behavioral tests fail before fixes and pass afterward.

**Retained historical-removal conflict:** UpdateUIParentPosition stays published. Current cached `Blizzard_UIParentUtil/UIParentUtil.lua:13` defines it; `Blizzard_Game/Shared/EventImplementation.lua:48,304` calls it; `Blizzard_EditMode/Shared/EditModeUtil.lua:88` hooks it. No later register reverses that historical page row. It remains an exact gap, not an accepted deprecated alias.

Remaining 36 publication gaps need explicit producers/policies: TOC interface-version selection; color override lifecycle; cooldown availability; game-mode catalogue/login; guild rename; upgrade DTOs; priority logging; world-map action; perks checkout; owned pets/search; spell-range subscriptions; telemetry; atlas elements; source location; unit ownership; hook prohibition; host attention; text-color invalidation. Two tabard-border getters remain intentional 3D/model scope gaps. [Per-ID review](../../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-gap-review.json) records every initial defect and exact decision. Generic callable lookup is not modeled publication. No placeholders added.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Unsuperseded add/change publication | 73 | Partial-development-green; signatures/output/security/behavior unproven |
| Unsuperseded removals | 15 | Bounded shared policy: strict absence/registration rejection; zero alias acceptances |
| Later-superseded inventory OK | 1 | Metadata-only; no historical credit |
| Inventory gaps | 36 | Missing explicit producer/policy or retained current-consumer/scope boundary |
| Non-inventory contracts / editorial context | 85 / 14 | Exhaustive ranked scout / metadata-only, no runtime credit |

224 unique IDs: 73 partial / 15 bounded / 121 pending / 15 metadata. Page-accounting and publication audit complete; ledger remains in-progress because behavioral and producer gaps remain. Extract batches: summary 11, color overrides 4, TOC/inheritance 18, enums 34, structures 18; every extract ID assigned once.

## Development proof

[Contract and sweep table](../../specs/patch-11-1-5-publication-sweep.md#local-proof) record ten isolated passing sweeps at runtime/test revision `1fb6571e1`, exact 36-gap fixture, and one-row negative control (36 → 37, expected exit 101). Two bounded behavior tests, existing environment regression, one isolated EditMode prefork consumer case, and thirteen extractor/register fixtures pass. Formatting, Mists test check without non-vendor warnings, separate default retail build and bounded exit-0 startup `[]` pass. Six pre-existing iced manifest warnings plus summary remain unsuppressed. Checks/build/startup run at `bc72538b5`; later changes are evidence/docs only. Changed Rust lines manually reviewed for readability.

Seven later observation maps equal previous 11.1.7 evidence. Differences in 11.2.0 (Browser:NavigateTo retention) and 12.0.0 (two rilua secret-helper closures) already exist on starting master; this audit changes no later fixture. [Validator](../../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-validate.py) checks hashes, exact expectations/gaps/credit, negative control, complete allocation, preserved inputs and revision-scoped proof summaries. Cargo logs are ignored local artifacts; result and verification-summary JSON are retained.

## Process exception

[Pyrun cwd incident](../../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-cwd-incident.json): default command/filesystem cwd did not follow worktree switch. First source capture/commit accidentally modified canonical master. Exact commit moved into required worktree; canonical restored to clean original master before any tests. All subsequent commands/files use explicit worktree cwd/paths. The never-touch-canonical constraint was violated temporarily; restored final state does not erase it. No full suite, agents/models, push, merge, vendor edits or Blizzard monkey-patching.

## Sources

- [Publication contract](../../specs/patch-11-1-5-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.1.5-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.1.5-page-coverage.json)
- [Extract scout](../../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-extract-scout.md)
- [Proof ledger](../../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-proof.json)

## See Also

- [[patch-11-1-7-api-audit]] — template and next supersession boundary.
- [[patch-11-2-0-api-audit]] — current-consumer retirement lesson.
- [[client-profiles]] — supported profiles and retail epochs.
