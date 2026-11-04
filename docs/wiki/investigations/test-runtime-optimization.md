# Test Runtime Optimization Boundaries

Commit `e1f9b210b1790258a6f865cc3df0e85f6364e8e0` verified the `Blizzard_SharedMapDataProviders` exact dependency-closure fixture. The measured bottleneck is repeated `WowLuaEnv` and addon setup, not timeout polling, process launch, prefork scheduling, or libtest target topology. Exact closure fixtures are safe for ordinary addon surface/load tests when full-game ordering is not the behavior under test; mutable environments must remain isolated per test.

## Content

### Baseline and scope

The 18-test `blizzard_shared_map_data_providers_loads` module previously ran a full game-screen Blizzard load. Controlled measurements compare that baseline with the verified closure fixture at `e1f9b210b1790258a6f865cc3df0e85f6364e8e0`:

| Run | Baseline | Fixed closure | Change |
|---|---|---|---|
| Parallel | 18/18; 6.01s libtest, 6.083s wall, 34.03s user, 4.60s system, 6,412,848 KiB RSS | 18/18; 0.83s libtest, 0.85s wall, 3.25s user, 0.36s system, 1,067,108 KiB RSS | wall −86.03%; RSS −83.36% |
| Sequential | 18/18; 20.10s libtest, 20.140s wall, 19.06s user, 0.90s system, 1,246,460 KiB RSS | 18/18; 2.96s libtest, 2.97s wall, 2.78s user, 0.17s system, 535,624 KiB RSS | wall −85.25%; RSS −57.03% |

The fixed runs retain the same fresh-environment and direct-root-load test boundary; only the dependency closure is narrowed.

### Root cause and verified fix

The bare root closure was invalid: only 16/18 tests passed. Top-level Lua relied on normal eager ambient order, although the target TOC declares no dependencies. The closure therefore lacked SharedXML color helpers and `CVarMapCanvasDataProviderMixin::Init`; the downstream failure surfaced as a nil `DelveEntrancePinMixin`.

`e1f9b210b1790258a6f865cc3df0e85f6364e8e0` supplies ordered `BlizzardAddonOverride` implicit dependencies: `Blizzard_SharedXML`, then `Blizzard_MapCanvas`. The module still uses `build_blizzard_addon_closure_env` with a fresh game-screen environment and direct `Blizzard_SharedMapDataProviders` root load, followed by its existing post-load workaround step. It does not load panel/full UI state and does not share mutable environments.

The shared helpers in `tests/common/blizzard_addon_harness.rs` remain the fixture boundary:

- `new_blizzard_addon_env` creates a fresh game-screen environment;
- `build_blizzard_addon_closure_env` discovers and loads the root's transitive TOC dependencies plus explicit implicit roots in order;
- `load_blizzard_addon_closure_into_env` applies the same closure to an existing fresh environment.

This is the correct fixture for addon API, TOC, registration, and load-state assertions that do not require the complete game startup sequence. It preserves dependency order without paying for unrelated Blizzard addons. It is not a replacement for full-startup tests: `Blizzard_SharedMapDataProviders` remains ineligible for the finalized prefork parent because its fixture loads before post-load workarounds and omits startup events.

### Rejected alternatives

| Candidate | Finding | Decision |
|---|---|---|
| Timeout re-exec polling | `tests/common/timeout_reexec.rs` polls `Child::try_wait()` every 10 ms. Focused timeout-reexec scenarios cover panic aggregation, exact selection, nested guards, bounded child concurrency, permit release after timeout, and process-tree cleanup; polling contributes at most one interval per normal child. | Keep the 10 ms poll. It is negligible beside addon setup and is required for timeout and process-tree cleanup semantics. |
| Timeout child launch, pipes, and handshake | Exact named launch, two drain threads, and the handshake add sub-second overhead, while startup traces show about 0.25 s Lua initialization, 2.8–3.2 s addon loading, and about 0.27 s post-load layout per child. | Do not change the wrapper. It preserves panic aggregation, sibling continuation, output forwarding, and timeout isolation. |
| Prefork worker tuning | Prefork runs are already roughly 6–10 seconds for representative slices. More workers overlap child setup but do not remove it and increase copy-on-write process-tree memory. Persistent workers would also violate mutable-environment isolation. | Keep bounded workers and one isolated child per case; optimize the fixture graph instead. |
| Libtest wrappers | `test_timeout!`/`with_timeout` isolate ordinary tests by exact re-exec, but each ordinary test still constructs its own environment and loads its selected addon graph. | Retain wrappers for failure and timeout semantics; do not treat them as a startup optimization. |
| Grouped integration targets or target splitting | `build.rs` generates one integration harness from roughly 1,077 top-level test modules. Splitting it would duplicate simulator/test dependency compilation without removing per-test full-UI setup. | Do not split targets for this problem. Any future grouped target needs measured closure and compile evidence first. |

### Isolation boundary

Never share a mutable `WowLuaEnv` across tests. Tests mutate Lua globals, frame trees, addon-loaded state, registries, SavedVariables, and seeded domain state. The safe optimization narrows each test's dependency closure while still creating a fresh environment per test. Reusing a live environment or persistent worker would make sibling order and state leakage part of the test contract.

## NECESSARY Retail scheduling profile — 2026-11-04

The [600-second warm-run contract](../../specs/necessary-suite-runtime.md) concerns the complete NECESSARY Retail set, not only the historical closure fixture above. Source `73d0de5b5` yielded 11,968 timed passing integration cases and 18 ignored entries. One-off libtest JSON timing collection used `RUSTC_BOOTSTRAP=1`; the adopted runner uses only stable discovery and saved metadata.

The three-thread profile was continued in disjoint single-thread partitions after preserving its completed cases. The 8,725 remaining cases took 634.64 seconds under a 16-CPU cap, consumed 4,921.64 CPU-seconds and reached a 13,794,742,272-byte aggregate cgroup peak. No cgroup OOM occurred. This is profiling evidence, not a complete passing sub-600-second full-suite run.

Three timeout-harness conformance cases measured 180.70, 237.99 and 295.78 seconds while competing for the exclusive workload gate. Their fixture launcher explicitly reserves exclusive capacity before testing the global two-child semaphore. The scheduling change runs the conformance module before ordinary shared workloads. It does not change the semaphore, timeout implementation, fixture or assertions.

A separate exact LoD-shard probe passed in 5.63 seconds, consumed 5.57 CPU-seconds and reached 898,904,064 bytes, versus 72.40 seconds of queued wall time in the profile. Queued wall weights are not exclusive callback or startup CPU costs.

The initial source-outline inventory listed 482 named `test_timeout!`/`with_timeout` consumers. It omitted macro-wrapped and macro-generated functions, including the GenericTraitUI prior-state case and generated LoD shards, so it is not a complete inventory. Two dedicated scheduling lanes keep these semaphore waiters from occupying every ordinary worker; the other lanes can run ordinary tests while both child permits are occupied. Classification changes scheduling only: newly added or unclassified cases still encounter the unchanged global semaphore. The fixture-equivalence restrictions above remain binding; this work does not migrate tests to a broader prefork parent.

The [runner operation](../../remote-builds.md#asynchronous-full-suite) describes the current scheduling implementation. Two complete warm NECESSARY runs, including incremental no-op builds, remain the acceptance requirement; the profile and bounded Python tests do not establish that target.

## Explicit budgets and memory envelopes — 2026-11-04

Main approved the timeout count as configurable harness policy, not native behavior. `WOW_SIM_TIMEOUT_CHILD_SLOTS` defaults to 2 and accepts 2 through 6. A source-bound targeted control first failed with observed 2 versus requested 6; the configured producer then passed explicit 4, explicit 6 and unset/default 2, and rejected 0. Product fixture assertions and test identities remain unchanged.

The six-slot measurement completed all 469 explicitly assigned timeout consumers in 360.5–429.0 seconds without a genuine timeout failure. Its overall result was not full-coverage acceptance: two ordinary 3 GiB shards were OOM-killed, leaving 2,222 selected cases uncompleted. The two unchanged literal-two conformance assertions also failed in that experiment. Overall timing and CPU totals from that incomplete run must not be presented as complete-suite savings.

The two same-process failure prefixes completed under a 9 GiB probe ceiling at process-tree peaks of 2,431,856,640 and 2,489,200,640 bytes. They did not reproduce the prior 3 GiB OOM in the quieter/warm probe context. The [memory table](../../../tools/integration-memory-peaks.json) retains the actual 3 GiB OOM observations as lower bounds, together with sampled retained-context envelopes. It applies 50% headroom and rounds up to GiB. Unmeasured cases use the largest observed envelope, giving a conservative 5 GiB shard ceiling rather than the failed fixed 3 GiB cap.

The [runner](../../../tools/full_suite.py) requests a private 16-CPU/24-GiB test slice by default, requires capacity ADD before tests, and permits lower explicit CPU/RAM overrides. It derives workers and timeout permits from that requested budget and records the budget and slice resources. This is an enforced ceiling, not an exclusive host-RAM reservation. A complete warm production proof remains required after the policy and memory-budget changes.

## Native timeout inventory and complete budget proof — 2026-11-04

The saved lane list was regenerated from parent timeout entries in source `cb55bc12d4a8a024f43f54dbff0156d41fb82e84`, Retail NECESSARY run `production-budget16`, attempt `1793823945197603514` (2026-11-04 14:25:45–14:44:51 local). The raw inventory SHA256 is `462fa5e1f55eec484d9ac3249b1e457f90047302c1c8b96cc8e7f4f608ad8b91`. Sorting and deduplicating its lines gives exactly the 672 names in [the scheduling data](../../../tools/integration-timeout-tests.json). All names are registered integration cases. Relative to the old 482-name source inventory, 197 names are newly observed and seven old names were not entered in this run. This is observed execution coverage, not proof about every unexecuted branch or ignored body.

The complete run passed every target with zero failed cases; its 26 failure-map values are empty arrays, as in master baseline `c2621a5aa5a771f26362ded5567b6958275d90be`. All 16,699 target-qualified identities are unchanged. End-to-end wall was 1146.645s: 45.329s warm build (695/695 artifacts fresh), 1039.6s sequential test steps and 61.716s other runner overhead. Cold compilation was not measured. Test CPU was 8872.819s and kernel aggregate peak was 17,600,987,136 bytes under the enforced 16-CPU/24-GiB budget; no OOM occurred.

Integration took 847.4s. After 8.5s conformance, ordinary shard 8 was last at 838.7s, selecting 1,146 names including 16 previously unclassified timeout consumers. Ordinary lanes took 788.9–838.7s; the six known-timeout lanes finished in 285.3–384.2s. All 197 newly observed ordinary-lane consumers still obeyed the global six-slot semaphore. Static lane completion leaves unused workers; exact startup-exclusive CPU attribution is not available from these wall timings. Prefork, lib and other serialized steps added 192.2s. This is a complete passing resource-budget proof, not the required sub-600-second acceptance.

## Shared-queue and overlap runner — 2026-11-04

The runner now keeps one cost-ordered queue and lets every lane pull small batches (about 20 measured seconds, at most 25 names). Each batch is one service that runs one native process with `--test-threads=1 --format=pretty --color=never --exact NAME...` and derives each case's wall time from its completed `test NAME ... ok` line. A measured attempt that instead gave every case its own process cost 2,892 extra CPU-seconds (+33%) in harness startup, so isolation is now reserved for cases costing at least 5 seconds or carrying a reserved memory envelope; only those record their own wait4 CPU and kernel `memory.peak`. Conformance still runs alone first; guarded batches are capped at the configured permit bound minus one lane so semaphore waiters cannot fill every worker.

Library, prefork and the remaining certified targets run in one overlapping thread. While tail work is pending, integration lanes shrink to keep a 9 GiB tail unit inside the requested memory budget; the tail takes the remaining CPUs. Admission counts unmeasured work at the 1.5 GiB worker share and measured work at its recorded envelope, while each batch keeps the previous per-unit `MemoryMax` ceiling. Measured integration CPU averaged about 9.6 cores with 16 lanes, and the tail cost 192 serialized seconds, so overlap removes tail wall time rather than integration throughput.

Per-name records are written beside the attempt result and merged into `tools/integration-costs.json` by `tools/refresh_integration_costs.py` in a separate data commit. The old fixed partitions balanced saved weights almost exactly but left six lanes idle for hundreds of seconds; the new queue reuses that capacity.

## Sources

- [Scheduling contract](../../specs/necessary-suite-runtime.md) — warm-run target and unchanged coverage requirements
- [Full-suite runner](../../../tools/full_suite.py) — stable discovery and exact-name scheduling
- [Saved wall-time weights](../../../tools/integration-costs.json) — all 11,968 measured passing cases
- [Timeout scheduling names](../../../tools/integration-timeout-tests.json) — native parent-entry observations from the source-identified proof above

- [SharedMapDataProviders test](../../../tests/blizzard_shared_map_data_providers_loads.rs) — commit `e1f9b210b1790258a6f865cc3df0e85f6364e8e0`, implicit dependency override, and retained assertions
- [Blizzard addon closure harness](../../../tests/common/blizzard_addon_harness.rs) — fresh environments and dependency-ordered closure loading
- [Blizzard addon override type](../../../src/loader/mod.rs) — ordered implicit dependency contract
- [timeout re-exec implementation](../../../tests/common/timeout_reexec.rs) — exact child launch, polling, output drains, handshake, and cleanup
- [timeout re-exec tests](../../../tests/timeout_reexec.rs) — panic, timeout, sibling, descendant, and nested-guard proofs
- [prefork runner](../../../tests/common/prefork.rs) — bounded workers and isolated child execution
- [generated integration harness](../../../build.rs) — top-level module discovery and target generation
- [Blizzard UI test lanes](../reference/blizzard-ui-test-lanes.md) — unit versus addon-bootstrap test intent
- [Prefork test harness](../systems/prefork-test-harness.md) — full-UI preload contract and eligibility boundary
- `/tmp/pi-shared-map-baseline-sequential.result.json` — ephemeral controlled sequential baseline
- `/tmp/pi-shared-map-fixed-*` — ephemeral fixed parallel/sequential measurements; repository claims remain understandable without these artifacts

## See Also

- [[prefork-test-harness]] — immutable full-UI parent, child isolation, and finalized eligibility
- [[blizzard-ui-test-lanes]] — test intent and startup-shape boundaries
- [[development-phases]] — broader performance-regression roadmap
