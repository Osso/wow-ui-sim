# Wrath 3.4.1 current factory handoff

Branch `p341-factory`, isolated `/home/osso/.worktrees/wow-ui-sim-p341-factory`, canonical master base `f95eed96e`. Discovery target commit `467acfa6e`; pinned measurement/test/doc commit `da31b9ed1`.

## Current measurement

| Literal entries | Matches | Classifier gaps |
|---|---:|---:|
| 211 globals | 133 | 78 |
| 44 events | 35 | 9 |
| 77 CVars | 71 | 6 |
| One LogFps Command | 0 | 1 |
| 333 total | 239 | 94 |

Exact identities/details are in `measurement-ledger.json`; exact 94 mismatch IDs in `known-gaps.json`. Global raw/lookup distinctions and all CVar current value/default strings retained. Matching functions supply no behavior/model credit. Models added: **NONE**.

Of 61 effectively expected-present CVars, four unpublished, 42 exact default strings, 15 published string differences. Eight numeric-format-only differences, seven numeric differences; `published-default-differences.json` retains all exact values. Four unavailable defaults are publication gaps, not published-default gaps. Same-line 3.4.2 inventory changes only AssignPFCDistribution and DynamicVRSSensitivityThreshold to expected absent. Actual 3.4.3 source has no explicit identities; no foreign successors/TBC history used.

## Proof and limits

`proof-ledger.json` retains exact argv/environment/revision/exit, separate full streams and source/input hashes. Discovery RED exits 101 at `467acfa6e`, empty gap fixture, all 333 observations persisted. Targeted GREEN **6/6**, exit 0 at `da31b9ed1`; every observation equals discovery. Formatting before both commits. Six inherited headless-library warning groups and one binary unused import remain; discovery-only shared helper warnings eliminated through a profile-rejection control, without suppression. No further test-input changes; receipt/doc-only final commit does not invalidate proof.

All 19 original historical seals independently hash-match, original seal file hashed, historical source/evidence/cache Git diff exit 0. Source ledger/status/counts/validators/logs untouched. Historical validators not rewritten or rerun. Fresh evidence owns only current simulator measurement, never native/historical observations.

Boundary: bare `WowLuaEnv::new`, `client-wrath` only, configured **38001**, not native Classic **30401**, no Game/publisher/loaded UI/cache probes. Wrath accepts invented event names: all native availability/removal/payload/dispatch claims UNPROVEN. Console has zero Command-kind records on this profile; a LogFps CVar test record does not publish a Command. Native command catalog parity, grammar/output/execution UNPROVEN. All 211 unspecified signatures, CVar persistence/effects and both source prose summaries remain UNPROVEN.

No production API/model/shim/vendor/parser/cache edits. No broad suites/checks/other-profile/readability/final gates, delegation/model CLI, push/merge/deploy or operations. Main owns integration and native final acceptance.
