# Patch 10.2.7 API page audit

Page 584467, revision 6268738 (March 20, 2025, 19:32:28 UTC), retrieved October 7, 2026 UTC. Branch p1027-page starts from p1100-page revision 8181eb6a7. Default retail carries 12.1.0, not reconstructed 10.2.7.

## Source and accounting

104 inventory occurrences: 69 added, 16 removed, 19 changed. All eight added/removed header counts match. Fifteen later registers, 11.0.0 through 12.1.0, supersede chronologically. Existing parser/extractor handles the page unchanged; all sixteen registers regenerate byte-identically, including the fifteen existing registers. All 98 preserved later source/register/coverage/fixture inputs remain unchanged.

187 unique IDs: 104 inventory + 83 extract. Ledger: 49 partial-development-green, 19 bounded-coverage, 107 audit-pending, 12 metadata-only. Every extract ID assigned once: 17 prose, 31 enumeration, 23 structure and 12 editorial rows. All 71 substantive extract occurrences remain pending. Publication-only changed inventory annotations receive no signature/type/security/native behavior credit. Candidate source matches are not implementation proof.

## Bounded closures and retained gaps

Initial discovery: 61 OK / 43 failures. Four failures were incorrect FontInstance construction: it is an interface, not a frame kind. A FontString implements that interface and permits publication probing. This correction does not establish strict axis validation. Three runtime closures leave 68 OK / 36 exact publication gaps.

C_StableInfo.ClosePetStables now uses existing stable-open state, clears it and queues PET_STABLE_CLOSED with no arguments. Classic ClosePetStables delegates to that same model; retail/PTR no longer register the legacy global. Retail/PTR also stop registering GetWorldPVPAreaInfo; C_PvP.GetWorldPVPAreaInfo remains. Current cached searches find no legacy global consumer. ClosePetStables bare-token hit is the namespace documentation declaration, not a global use. Current successors have consumers: Blizzard_StableUI/Blizzard_StableUI.lua:306 and Blizzard_PVPUI/Mists/Blizzard_PVPUI.lua:582,647. Classic registrations preserved; no Blizzard wrapper or cache edit.

[Per-ID review](../../../data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-gap-review.json) records all 43 discovery failures, three closures, four probe corrections, exact observations and 36 precise retained contract boundaries. Retail addon-prefix registration/throttling, stable roster/favorites/cursor state, item socket/appearance metadata, scenario searches, quest grouping, pet ownership mutation, community selection, unit-token loot interactions and attack state remain gaps rather than placeholders. issecurevalue is not recast as issecretvalue. Model dressing/preview animation availability is an explicit unsupported 3D boundary. GamePadPlunderstormDefaults lacks a measured preset/command contract; no hardware or host settings enabled.

[Extract scout](../../../data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-extract-scout.json) retains each statement and decision. Comma-separated TOC parsing has a candidate implementation/test but cross-flavor load behavior is not credited. Duplicate-file warnings lack a source producer. DisableAddOn excludes only __BuiltIn, not all securely owned Blizzard addons. FontString and font-table setters lack the published strict horizontal/vertical rejection contract. Deprecation CVar default publication does not establish reset across client restart or disabled-wrapper load policy. Addon message allowance (10), decrement, refill (1/second), out-of-instance whisper exemption, global disconnect throttle and wrapper enum-return shifting require distinct state/process/transport proof. One stable-close migration cannot credit the full stable-overhaul prose. Enum values and populated DTO fields remain pending.

## Verification

Runtime/test revision 3b2ae8efc. Two behavioral tests fail before the runtime fix and pass afterward. Sixteen isolated publication sweeps pass exact fixtures; [table](../../specs/patch-10-2-7-publication-sweep.md#local-proof). Negative control changes only C_StableInfo.ClosePetStables added → removed: exactly one new failure, no resolved failures, 36 → 37 gaps and expected exit 101. New full cached Game prefork migration passes; existing seeded PvP successor test passes. Eighteen parser/extractor fixtures pass, tooling unchanged.

Formatting, Mists tests check, separate retail binary build and bounded startup gates remain pending at this accounting commit. Changed Rust reviewed manually: short existing-state producer, explicit profile gates and interface factory; no readability violations. [Proof ledger](../../../data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-proof.json) retains commands/revisions/results. Failed development attempts are preserved: EventQueue has drain, not clear; cached macro module needs its own WowLuaEnv import. Compiler failures provide no behavior credit. No native or independent acceptance claim.

Every command uses explicit cwd p1027-page and its own target. Worktree creation used prescribed canonical Git metadata operation with cwd in empty destination. No canonical working files, siblings, cache/vendor Lua or Wowless edited; no full suite, agents/models, push or merge.

## Sources

- [Provenance](../../../data/patch-api/sources/10.2.7-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.2.7-page-coverage.json)
- [Publication contract](../../specs/patch-10-2-7-publication-sweep.md)
- [Retirement searches](../../../data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-retirement-consumers.json)
- [Register reproduction](../../../data/patch-api/evidence/10.2.7-session-2026-10-07/p1027-register-reproduction.json)

## See Also

- [[patch-11-0-0-api-audit]] — read-only branch base and first later register.
- [[patch-11-0-2-api-audit]] — accounting/proof conventions.
- [[client-profiles]] — supported retail/classic boundaries.
