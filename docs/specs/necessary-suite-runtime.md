# NECESSARY Retail runtime

The NECESSARY Retail suite must complete in at most 600 seconds of wall time on a warm target directory, including its incremental no-op build. Report cold compilation, warm no-op build, and test execution separately. Tests remain classified by importance, not duration. Scheduling is implemented in `tools/full_suite.py`; see [runner operation](../remote-builds.md#asynchronous-full-suite).

## What it must do

- [ ] Complete two passing NECESSARY Retail runs within 600 seconds each, with CPU and memory measurements.
- [x] Partition stable libtest discovery into disjoint exact-name selections, including the existing ignored-test entries.
- [x] Balance tests using saved wall-time costs without unstable runtime flags. Give newly discovered tests a unit scheduling weight.
- [x] Preserve failed-test names and a nonzero aggregate result when a shard fails.
- [x] Finish timeout-capacity conformance before ordinary shards; keep known timeout waiters from occupying every ordinary worker.
- [x] Keep product fixture assertions, importance classifications, and test identities unchanged. Harness conformance checks its explicitly configured policy bound.

### Resource limits

- [ ] Run single-threaded integration shards with measured process-tree memory envelopes plus headroom; use the largest observed envelope for unmeasured cases.
- [ ] Request and enforce a private test-slice budget, default 16 CPUs/24 GiB, with lower `--test-cpus`/`--test-mem-gib` overrides and required capacity ADD admission.
- [ ] Record budget, configured worker/permit counts and private slice in result JSON.
- [x] Preserve the global timeout-slot semaphore and exclusive-workload gate; support a configured harness-policy bound from 2 through 6, defaulting to 2 when unset.
- [ ] The runner scales permit/worker counts proportionally to its explicit budget, with at least two permits.

## How it works

- [Runner operation and recorded results](../remote-builds.md#asynchronous-full-suite)

## Implementation inventory

- `tools/full_suite.py`: stable discovery, measured-cost partitioning, service launch and result aggregation.
- `tools/integration-costs.json`: saved Retail integration wall-time weights.
- `tools/integration-timeout-tests.json`: named timeout consumers assigned to dedicated lanes.
- `tools/integration-memory-peaks.json`: measured context-inclusive envelopes, OOM lower bounds and headroom policy.
- `tools/test_full_suite.py`: runner behavior tests using bounded child processes.

## Tests asserting this spec

- `tools/test_full_suite.py::IntegrationShardingTests`: exact-name selection, deterministic cost balancing, ignored entries and aggregate failure reporting.

## Known gaps (current cycle)

- [x] Finish the per-test timing profile and compare two versus six timeout permits.
- [ ] Complete the production budget/memory proof, then report startup CPU-reduction candidates before any fixture change.
- [ ] Record the two complete passing runs and empty before/after test-list diff.

## Out of scope

Changing product fixture assertions or semantics, moving slow tests to OPTIONAL, or modifying Blizzard code. Harness conformance may assert its explicitly configured policy bound; the existing post-spawn deadlines and cleanup remain unchanged.
