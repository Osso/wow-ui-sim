# Test Suite Performance

Measured default-retail test cost on 2026-10-07, then reuse the identical full-game preload for publication sweeps without deleting cases or weakening assertions. CI integration coverage remains an unapproved branch-only proposal.

## Baseline

At `44e91b8fccb5a6ad1f206a9fccf1b8d9c4ccea34`, four-worker nextest integration ran 10,808 cases in 2717.323s: 10,784 passed, 24 failed, 19 ignored/skipped. Lib ran in 51.959s (1973 passed, six failed); prefork ran in 92.575s (2031 passed). Own default-feature target build took 12m59s; compile time is excluded from test comparisons.

[Timing summary](../../../data/test-perf/2026-10-07-summary.json) holds module totals, top 50 and duration buckets; full per-test raw data (15 MB) was kept out of the repository at `~/.cache/wow-ui-sim-audit/test-perf-2026-10-07/` on the measuring host. [Ranked report](../../../data/test-perf/2026-10-07-report.md) separates source-shape fluff from distinct lifecycle/load-order/partial-env coverage. Source signals are candidates, not execution tracing: helper-only tests are unknown, not cheap.

## Root cause and safe change

Each of 22 patch publication sweeps rebuilt `common/prefork_full_ui_preload.rs` under an exclusive workload gate. Full-run process times totaled 677.391s, including queueing. A matched serial nextest sweep batch passed 22/22 in 128.843s. Ordinary cheap source-boundary tests are numerous but cost little: 61 bootstrap-boundary cases sum to 1.680s.

Sweeps now use `prefork_full_ui_case!` with the existing identical parent preload. Each retains its own child, register, reviewed exact gap set, row count, later-patch supersession, deprecation classifier, and optional output artifact. No shared mutable Lua env across tests; fork isolation preserves case-local changes. The generic runner supplies its existing 120-second child timeout and process-tree cleanup. Publication probes no longer acquire an exclusive gate because their measured contract is publication, not timing. Startup timing tests remain unchanged and exclusive.

The negative-control register changes one existing row to an absent global without changing row count or IDs. Before migration, that sweep correctly exits nonzero with one new gap; retain the same control after migration. Migration acceptance requires all 22 cases, equivalent per-row JSON artifacts, registry name/count preservation, and the negative control. All migration acceptance checks passed at `8b428bba8`; retained artifacts and registry proof are linked below.

## Coverage boundaries

- Publication/absence only: sweeps do not claim signatures, outputs, security, or native behavior parity.
- Existing 24 integration and six lib failures are baseline failures, not hidden or marked ignored.
- Isolated nextest passes all sweep and LoD shard cases, including intentional same-process sequencing inside a case. No measured same-process full-suite comparison establishes that nextest alone fixes cumulative-cache failures.
- Spellbook screenshot fixture, LoD-before-startup dependencies, post-drop template registry checks, and performance timing fixtures remain ordinary tests; completed live snapshot is not their initial state.
- Deletion candidates require user approval. No tests deleted; migrated stable names must appear exactly once across integration and prefork.

## Sources

- [Timing summary](../../../data/test-perf/2026-10-07-summary.json) — module totals, top 50, buckets
- [Ranked report](../../../data/test-perf/2026-10-07-report.md) — names, costs and exclusions
- [Sweep helper](../../../tests/common/publication_sweep.rs) — exact publication assertions
- [Preload helper](../../../tests/common/prefork_full_ui_preload.rs) — identical startup boundary

## See Also

- [[prefork-test-harness]] — fork isolation, bytecode mode and workload coordination
- [[test-runtime-optimization]] — prior fixture eligibility investigations

## Scheduling profiles

`cargo nextest run --test integration --profile local --target-dir target` uses eight workers on sufficiently provisioned hosts. The default, measurement, and CI profiles use four. Profiles never exclude tests or retry failed assertions. `cargo test --test prefork_full_ui --target-dir target` remains required: its harness=false runner is not included by nextest. Count partitions must run on distinct hosts; shared file gates serialize selected expensive workloads on a common host.

## Matched migration proof

At `0034960f7`, 22/22 sweeps pass with one prefork worker in 18.129s including conformance/preload, versus 128.843s for 22 isolated serial nextest processes: 85.9% reduction (7.11x). All 22 per-row JSON artifacts are exactly equal; the modified-row negative control fails and its output artifact is exactly equal before/after. See comparison (raw, off-repo). Whole-suite measurement below does not establish a large local improvement or an eight-worker benefit.

## Final proof and unapproved CI

All 12839 active names remain, with 22 moved and no lost/added/overlap names (registry proof (raw, off-repo)). Final eight-worker integration ran 10786 cases in 2669.664s: 10762 passed and the exact same 24 baseline failures. Final prefork passed 2053/2053 in 59.504s. Lib proof remains the unchanged six-failure baseline. Format/check pass; six pre-existing vendor manifest warnings remain untouched. See proof ledger (raw, off-repo).

Local whole-suite reduction is only 1.8% for integration, with changed worker count and uncontrolled host load. Do not advertise the 85.9% sweep-batch improvement as a whole-suite result. Eight-worker scheduling has no demonstrated material benefit; prefer four-worker defaults.

The unapproved workflow (kept unmerged on branch `test-perf`, `.github/workflows/integration.yml`) is manual, requires explicit approval, and runs 16 count partitions on provisioned separate self-hosted hosts plus lib/prefork separately. It never downloads or mutates Blizzard source. No push/PR trigger, push, merge, or remote run occurred. Trusted runner policy, available host capacity, cache provenance and baseline failure resolution remain activation decisions.

Exact partition estimates (raw, off-repo) model 142.1–173.3s per four-worker shard using measured durations; these are not CI executions and exclude compilation/queueing. Full per-shard table and constraints live in the [report](../../../data/test-perf/2026-10-07-report.md). Local calibration: shard 1 ran 675/675 in 198.745s; shard 5 ran 674 cases in 162.916s with three exact baseline failures. Applying the maximum observed/model ratio (1.344) gives planning estimates of about 191–233s per shard, excluding build/queueing. Other shards remain modeled, not measured. Calibration proof (raw, off-repo).

## Final validation

Workflow actionlint passes; changed Rust files have no introduced readability violations. Final artifact gate (raw, off-repo) checks stable coverage, row-level equivalence, exact one-gap negative control, complete partition union, protected tracked paths, and source links. No external verifier/agent/model invocation was used.

Nextest explicitly rejects the custom prefork listing (`--format terse` is unsupported); retained failure (raw, off-repo) proves the separate cargo job is necessary. Running integration alone omits migrated sweeps. Commit sequence and merge risks are in the [report](../../../data/test-perf/2026-10-07-report.md).

## Slow integration fixture classification (2026-10-08)

[Migration plan](../../../data/test-perf/prefork-migration-plan.json) classifies every measured integration case taking >=1s, with original source/function/line references, reasons, per-class and per-module totals, and disjoint source-file priorities. Snapshot eligibility is source-reviewed, pending migrated-case proof; historical process durations include workload queueing and are not matched serial timings. Twenty-two publication cases in the timing input are already prefork and must not be counted as new migration savings.

- MIGRATE: 93 tests, 1101.078s.
- ADAPT: 93 tests, 629.149s.
- KEEP: 1496 tests, 6728.398s.

Most slow fixtures are bare environments, selected addon closures, Glue, explicit clean-LoD coverage, custom preload, screenshot/font startup or timing/process tests. Those stay integration; widening their addon/state boundary merely to hit a numeric migration target would change coverage. The classified source revision lists 10,814 integration tests (10,816 output lines) and 2,077 prefork cases, rather than the older 2,053 baseline.

## Prefork migration batch 2 (2026-10-08)

Migrated 38 ADAPT cases across NewPlayerExperience, ObliterumUI, OrderHallUI, PartyPoseUI, PerksProgram, PlunderstormBasics and PlunderstormPrematchUI. Each module has its own commit with its fixture-equivalence argument. Same 1024×768 Game screen, complete eager addon dependency closure and parent-provided startup/workarounds; retain the exact explicit LoD load in each child, without replaying startup or ADDON_LOADED. Assertions observe publication, loaded bookkeeping and addon-specific errors, not lifecycle ordering. All 38 probe/assertion bodies are byte-identical after removing fixture calls. No batch-2 cases deferred; non-retail Plunderstorm wrappers retain their original integration fixtures.

[Migration plan](../../../data/test-perf/prefork-migration-plan.json) preserves original source traces and historical durations (not measured savings). Targeted prefork, integration listing and Mists tests-check verification pending in `/tmp/prefork-migrate-2-proof/`; no full suite, vendor changes, push, merge or agents.
