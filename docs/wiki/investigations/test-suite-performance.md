# Test Suite Performance

The default suite runs necessary tests; `optional-tests` adds historical snapshots and performance tests. `full-suite run --optional` runs the certified targets with that feature. `scripts/release.py` runs this Retail suite before creating and pushing a release tag; `--dry-run` skips tag and push.

Measured default-retail test cost on 2026-10-07, then reuse the identical full-game preload for publication sweeps without deleting cases or weakening assertions. CI integration coverage remains an unapproved branch-only proposal.

## Per-addon budget diagnostics

`heaviest_blizzard_addons_stay_within_per_addon_load_budgets` reports the existing top-eight selection's loader phase durations and cache counters before checking its unchanged budgets. Durations are elapsed nanoseconds, not exclusive CPU time; Lua compilation includes cache lookup/replay, and nested components must not be added to the total. `loaded_addon_count` is the harness's discovered addon count, not a transitive registry count. Tests-only `655f0698b` adds this reporting without changing selection or budgets. Independently accepted saved source `655f0698b` execution passes one logical case, exit0; duplicate parent/timeout-child printouts are not independent cases or measurements. It reports 220 loaded addons: ProjectConstants total 515.210301ms, compile/cache lookup-replay 514.930960ms, Lua call 0.184681ms, one cache hit and zero misses. This is not uncached compilation or handler-body time. See [shared receipt](#saved-fullsuite39-bounded-runtime). Fullsuite39 remains historical FAIL.

## Saved fullsuite39 bounded runtime

Independent saved `runtime-report.md` and `build-seal-report.md` audits accept source `655f0698baad8183d68c88f4be9a2b26d32c2371` only: three unique logical cases PASS, each exit0 — [corrected library scroll fixture](../../specs/scrollframe-script-bindings.md#library-fixture-reconciliation), [corrected 9.1.0 prefork sweep](../../specs/patch-9-1-0-publication-sweep.md#current-widget-query-reconciliation--2026-10-15), and [unchanged-budget phase diagnostic](#per-addon-budget-diagnostics). Parent/child duplicate prints count once. Reports remain private.

Saved no-run build finished in 18m54s, zero compiler-message records, five `client-retail` seals: library tests, integration, prefork_full_ui, wow-cli and wow-sim. Build success is separate from runtime success and does not imply warning-free runtime. Saved seal/stat consistency is not newly rehashed binaries or cryptographic source/external-cache provenance.

Fullsuite39 remains historical FAIL. No original-cause, production-fix, improvement, matched original worker/cache/host conditions, all-profile, native or current-HEAD acceptance follows. Invocations set `WOW_SIM_CASC=0` and `WOW_SIM_NO_SOUND=1`; flags alone do not prove total asset non-use. External UI/cache and SavedVariables inputs are not cryptographically bound. Owning specs retain assertion and known-gap limits.

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

[Migration plan](../../../data/test-perf/prefork-migration-plan.json) preserves original source traces and historical durations (not measured savings). At code revision `331515ca9`, all seven targeted prefork commands pass (38/38 cases); retail integration lists 10,680 tests with all 38 migrated names absent. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with zero non-vendor warnings (six existing iced manifest deprecations remain). `cargo fmt` and `cargo fmt --check` pass. Command exit codes, logs and exact case/exclusion proof are retained in `/tmp/prefork-migrate-2-proof/`. No failed cases needed an original-master comparison; no full suite, vendor changes, push, merge or agents.

## Prefork migration batch 3 (2026-10-08)

Final batch moves 25 cases across fifteen modules/submodules into prefork. Rust assertion tokens and raw Lua probe/assertion literals match `102fb5088` exactly for every moved case; proof retained in `/tmp/prefork-batch3-assertion-proof.json`. ADAPT commits compare addon discovery, notifications, startup/workarounds, 1024×768 Game screen and ticks. Spell/channel/click fixtures retain post-fork mutations; headless layout/quad fixtures retain the six-tick Game settlement tail without replaying startup. Non-retail wrappers and shared configured KEEP fixtures remain integration.

The spellbook keybind case remains deferred: its profile-aware runtime fixture optionally loads profile FrameXML and explicitly synchronizes addon names before S-key lazy loading, with ten ticks per keypress. This differs from the parent preload contract; replacing it would lack a lifecycle-equivalence proof. [Migration plan](../../../data/test-perf/prefork-migration-plan.json) preserves historical timings/classification, records the precise deferral and recomputes remaining priorities.

Three additional cases remain integration after failed equivalence probes: cast-bar lock observes `BottomManagedFrameContainer` under its partial event fixture, versus `UIParent` in completed startup; both chat editbox cases expect SAY/white input, versus CHANNEL/(1, 0.75, 0.5) after completed startup. All three pass unchanged on detached master `102fb5088` using a separate target. Their original tests/loaders are restored rather than normalizing state or weakening assertions. At final test-source revision `26e2c5a67`, all 25 migrated cases pass across targeted prefork filters. Integration lists 10,655 cases: all 25 moved names absent, all four deferrals present. Touched KEEP filters pass 65/65; four deferred cases pass separately. Mists tests-check passes with zero non-vendor warnings (six existing iced manifest deprecations remain); `cargo fmt` and `cargo fmt --all -- --check` pass. Changed Rust readability review finds no introduced violations. Command/revision/result ledger and logs are retained under `/tmp/prefork-batch3-*`; `prefork-batch3-proof-ledger.json` records superseded probes and unchanged proof scopes. No full integration suite, vendor changes, warning suppressions, agents, push or merge.

## Separate current phase observation — 2026-10-13

[Owning ProjectConstants packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-canonical-paired-2026-10-12/projectconstants-621-2026-10-13/README.md) separates normal verbose bucket timing from the still-failing exact fixture; no causal repair claim.

## Separate immediate-EOF symbolization control — 2026-10-14

Independent saved-artifact verification PASS is separate from control FAIL: one immediate-EOF sibling used the prior exact 21-byte comma/address/comma input and GNU addr2line argv. Main exit1, child exit-9, absolute8-second worker deadline; captured27 bytes contain only the zero-address unknown function/location response, no requested-address header. Capture incomplete; no complete-output claim.

Frozen ELF identity reused stored/stat metadata, not new byte authentication. Worker write/flush/close precedes output drain, but no independent milestone proves delivery or child EOF timing. EOF was insufficient within this tested deadline; buffering, OOM, protocol defect, indefinite symbolization failure and perf causality remain unproved. The old first-record warning remains unresolved. Prior persistent timeout and unexecuted-plan checkpoints remain historical; their earlier “no EOF control” statements are not current inventory claims.


## Changed 60-second GNU EOF control — 2026-10-14

Independent saved-control verification passes one direct GNU lookup with the same21-byte input, frozen ELF/address and1GiB hard/768MiB soft limits, but a60-second worker deadline. Complete247-byte output contains both zero sentinels and the requested address mapping to rilua `execute.rs:992`; child/worker exit0, elapsed43.60055s. Write/flush and stdin-close milestones are worker-side observations, not child receipt timestamps.

Recorded CPU42.452358s, peak891957248 bytes (above768MiB soft), swap peak1631158272 bytes and zero recorded OOMKills remain context. Supplied, independently unverified context says the invocation followed the shared live-run hold and overlapped a bounded PTR compile. Neither timing nor resource observations establish causation. Earlier8-second persistent/EOF failures remain unchanged. This control does not establish perf's selected helper/configuration, resolve the first-record warning, or prove native/parent acceptance.

## Upstream tagged symbolizer source — 2026-10-14

Upstream Linux `v7.2.10` source closes the previously unavailable protocol-source gap, not installed-build equivalence or execution cause. Installed Arch package metadata identifies `7.2.10-1`; its patches/build configuration remain unproved.

[addr2line.c](https://github.com/gregkh/linux/blob/v7.2.10/tools/perf/util/addr2line.c) launches `-e <binary> -a -i -f`, sends a comma/newline handshake, then 16 hex digits/newline/comma/newline requests. Responses contain address/function/location records and a GNU zero-address or LLVM comma sentinel; remaining inline records are drained through that sentinel. [symbol.c](https://github.com/gregkh/linux/blob/v7.2.10/tools/perf/util/symbol.c) sets the default read timeout to 5000 ms; [io.h](https://github.com/gregkh/linux/blob/v7.2.10/tools/lib/api/io.h) applies it to poll before read. Historical effective configuration and whole-request duration are not established.

The independently executed immediate-EOF and persistent controls remain failures with sentinel-only output. Source availability establishes neither child request receipt nor response timing, buffering/OOM/performance causality, a repair, or parent acceptance. No replay follows from this checkpoint.
