# NECESSARY Retail runtime

The NECESSARY Retail suite must complete in at most 600 seconds of wall time on a warm target directory, including its incremental no-op build. Report cold compilation, warm no-op build, and test execution separately. Tests remain classified by importance, not duration. Scheduling is implemented in `tools/full_suite.py`; see [runner operation](../remote-builds.md#asynchronous-full-suite).

## What it must do

- [ ] Complete two passing NECESSARY Retail runs within 600 seconds each, with CPU and memory measurements.
- [x] Partition stable libtest discovery into disjoint exact-name selections, including the existing ignored-test entries.
- [x] Balance tests using saved wall-time costs without unstable runtime flags. Give newly discovered tests a unit scheduling weight.
- [x] Preserve failed-test names and a nonzero aggregate result when a shard fails.
- [x] Finish timeout-capacity conformance before ordinary shards; keep known timeout waiters from occupying every ordinary worker.
- [x] Keep test assertions, importance classifications, and the complete test list unchanged.

### Resource limits

- [ ] Run single-threaded integration shards with individual memory limits and account for shared host capacity.
- [ ] Preserve the existing global two-child timeout semaphore and exclusive-workload gate.

## How it works

- [Runner operation and recorded results](../remote-builds.md#asynchronous-full-suite)

## Implementation inventory

- `tools/full_suite.py`: stable discovery, measured-cost partitioning, service launch and result aggregation.
- `tools/integration-costs.json`: saved Retail integration wall-time weights.
- `tools/integration-timeout-tests.json`: named timeout consumers assigned to dedicated lanes.
- `tools/test_full_suite.py`: runner behavior tests using bounded child processes.

## Tests asserting this spec

- `tools/test_full_suite.py::IntegrationShardingTests`: exact-name selection, deterministic cost balancing, ignored entries and aggregate failure reporting.

## Known gaps (current cycle)

- [ ] Finish the per-test profile and select concurrency from measured CPU and memory use.
- [ ] Record the two complete passing runs and empty before/after test-list diff.

## Out of scope

Changing assertions, moving slow tests to OPTIONAL, modifying Blizzard code, or changing timeout semaphore semantics.
