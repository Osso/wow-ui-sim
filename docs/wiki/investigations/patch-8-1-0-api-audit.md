# Patch 8.1.0 API audit

Verified source on 2026-10-08: Warcraft Wiki page **464337**, revision **4462737** (2021-12-04T17:01:42Z), refetched rather than inherited from the remaining-pages list. Audit targets current retail, not a reconstructed 8.1 client. [Spec](../../specs/patch-8-1-0-publication-sweep.md) defines the boundary.

## Source accounting

[Raw source, provenance, extract, register and ledger](../../../data/patch-api/sources/8.1.0-page-coverage.json) retain **145 API occurrences and 28 extract occurrences: 173 ledger identities**. Both sides of two calendar renames count separately. The CVar table has 18 additions and 14 removals; parsed counts match the source headers. The build/citation caption lives in the extract, not a duplicated ledger context row. Two literal `?` statements remain source uncertainty; 26 extract rows are metadata, including namespace notes, headings and unexpanded references/links.

8.1.5's exact `--legacy-api-bullets` implementation and fixtures are reused in a separate prerequisite commit, not independently redesigned. New tooling is opt-in: generator `--legacy-api-renames`; extractor `--legacy-cvar-tables` retains captions while excluding table markup. [Provenance](../../../data/patch-api/sources/8.1.0-api-changes.provenance.json) records all flags. Main-thread integration must keep both parser behaviors and replace the sweep's **8.1.5 then 8.2.0** leading placeholders before 8.2.5/later registers. Read-only snapshots of both unmerged registers support the retirement re-addition check; those snapshots are not active supersession inputs yet.

## Coverage matrix

| Capability | Covered | Remaining / boundary |
|---|---|---|
| Current retail publication/absence | 85 / 145 identities | 60 exact reviewed gaps; presence alone never proves behavior |
| Calendar comparison | Supplied year/month/day/hour/minute ordering; rhs-relative sign, equality, weekday independence, unchanged inputs | Native invalid civil-date, secret-value and historical signature parity unverified; other date operations remain temporary providers |
| Unused-member absence | Three retail members, bare/repeated lookup and cached startup | Configuration-warning members retained for live consumers/provider callers |
| Source accounting | All 173 identities, both rename sides, CVar counts and source unknowns | Linked references are not expanded; two `?` rows cannot define an implementation contract |
| Reproduction | 38 registers, 35 saved extracts; 194 original inputs and 74 old extractor-mode outcomes preserved | Three inherited 12.x extract failures remain unchanged |

## Modeled closure

`src/c_api/c_date_and_time.rs` adds a real `CalendarTime` value model and `C_DateAndTime.CompareCalendarTime` over supplied integer fields. Derived civil timestamp ordering excludes weekday; result sign follows retained primary Blizzard documentation (**rhs relative to lhs**). The implementation neither reads the clock nor invents dates. Bare and unmodified cached full-UI tests assert leap-day/month/year/hour/minute boundaries, equality, reversed comparison and no input mutation. Missing tables/fields fail explicitly. This does not upgrade other date/time placeholder operations or claim native security/date-validation parity.

Discovery reports **81 OK / 64 gaps**. Calendar comparison and three safe absences produce **85 OK / 60 gaps**. [Per-ID review](../../../data/patch-api/evidence/8.1.0-session-2026-10-08/p810-gap-review.json) gives each gap's literal, effective expectation, observation and missing model/policy/lifecycle.

## Retirement decisions

[Complete before scans](../../../data/patch-api/evidence/8.1.0-session-2026-10-08/p810-removal-consumers.json) and [after scans](../../../data/patch-api/evidence/8.1.0-session-2026-10-08/p810-whole-callers-after.json) use `/usr/bin/grep -rnE` with whole-word `\b` boundaries (rg unavailable), excluding `*Documentation*` files/directories. Both qualified and bare-name cached scans and complete `src/`/`tests/` caller outputs are retained; bare scans include indirect `pcall(Name, ...)` and `and Name then`. No outputs are truncated in the evidence.

Retired: **C_Map.GetBountySetIDForMap**, **C_Calendar.EventGetClubID**, **C_Calendar.EventSetClubID**. All three have zero qualified/bare cached consumers, zero pre-change simulator callers and zero re-additions in 8.1.5, 8.2.0 or master's later registers. Retail-only marking prevents lookup autostubs; classic paths remain untouched.

Retained: **GetConfigurationWarningString**, **GetConfigurationWarnings**, **SetConfigurationWarningSeen** have current cached consumers. **GetConfigurationWarningSeen** has no cached consumers but a simulator temporary provider and unit-test callers. Initial marking did not prevent that provider's later raw republication: the failing retirement test falsified the unused-provider assumption. Its retirement was withdrawn, not hidden by a shim or weakened caller test. Profile-safe provider/test migration needs a modeled contract. All fourteen removed CVars were already absent; no new retirement/default was added.

## Recorded problematic boundaries

Friend/ignore/who mutations and selection require coherent identity/index/request state; read-only fixture lists are insufficient. Calendar club/current/next-event IDs require editor ownership and staging. Channel reset/swap and club stream joins need ordered channel state and transitions. Achievement supersession, bounty maps, mount/pet item relations, currency bonuses, item scrap/power eligibility, mission environment counters and PvP reward/area rules require actual relation/policy metadata. Summons, party referrals, questline requests and reports need pending identities and completion lifecycles. Debug dashboard is a native-client diagnostics domain. `gxMTOpaque` has no pinned default/control contract. Three later-page absence expectations remain lookup-fabrication gaps; this audit does not silently implement those later pages' retirements.

## Proof and integration

[Evidence directory](../../../data/patch-api/evidence/8.1.0-session-2026-10-08/) contains streamed command logs, receipts, discovery/negative controls, reproduction and accounting scripts. Long Cargo work runs asynchronously with the dedicated p810 target; no polling waits, full integration suite, agents/models, session-cwd changes, push or merge.

Targeted final gates are recorded in `p810-proof.json`; acceptance status is recorded by `validate.py` in `p810-artifact-acceptance.json`. Validator compares original mutable shared inputs at **recorded base/audit git revisions**, never frozen digests against the post-merge checkout; receipt cwd is repo-relative and no absolute worktree/target assertion exists. Counts derive from files. Later audits legitimately changing shared files do not invalidate historical input preservation.

## Sources

- [Pinned page/provenance](../../../data/patch-api/sources/8.1.0-api-changes.provenance.json).
- [Retained Blizzard DateAndTime documentation](../../../data/patch-api/evidence/8.1.0-session-2026-10-08/DateAndTimeDocumentation.lua) and its fetch receipt.
- [Occurrence ledger](../../../data/patch-api/sources/8.1.0-page-coverage.json).
- [Handoff procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-8-2-5-api-audit]] — reproduction/accounting template.
- [[patch-8-1-5-api-audit]] — reused bullet parser and next integration register.
