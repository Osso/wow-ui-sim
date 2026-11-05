# NECESSARY Retail runtime

The NECESSARY Retail suite must complete in at most 600 seconds of wall time on a warm target directory, including its incremental no-op build. Report cold compilation, warm no-op build, and test execution separately. Tests remain classified by importance, not duration. Scheduling is implemented in `tools/full_suite.py`; see [runner operation](../remote-builds.md#asynchronous-full-suite).

## What it must do

- [ ] Complete two passing NECESSARY Retail runs within 600 seconds each, with CPU and memory measurements.
- [x] Partition stable libtest discovery into disjoint exact-name selections, including the existing ignored-test entries.
- [x] Order tests using saved wall-time costs without unstable runtime flags. Give newly discovered tests a unit scheduling weight.
- [x] Pull small cost-bounded batches from one shared queue until it is empty, so a lane that finishes early takes more work.
- [x] Record per-test wall time, wait4 CPU, maximum RSS and kernel memory peak per name on stable libtest, and refresh the saved cost data from a finished run.
- [x] Run library, prefork and other certified targets concurrently with integration inside the same requested CPU/memory budget.
- [x] Preserve failed-test names and a nonzero aggregate result when a batch fails, and treat a missing selected-case outcome as a failure.
- [x] Finish timeout-capacity conformance before ordinary shards; keep known timeout waiters from occupying every ordinary worker.
- [x] Keep product fixture assertions, importance classifications, and test identities unchanged. Harness conformance checks its explicitly configured policy bound.

### Resource limits

- [x] Run single-threaded integration shards with measured process-tree memory envelopes plus headroom; use the largest observed envelope for unmeasured cases.
- [x] Request and enforce a private test-slice budget, default 16 CPUs/24 GiB, with lower `--test-cpus`/`--test-mem-gib` overrides and required capacity ADD admission.
- [x] Record budget, configured worker/permit counts and private slice in result JSON.
- [x] Preserve the global timeout-slot semaphore and exclusive-workload gate; support a configured harness-policy bound from 2 through 6, defaulting to 2 when unset.
- [x] The runner scales permit/worker counts proportionally to its explicit budget, with at least two permits.

### Timeout-entry inventory

- [x] With `WOW_SIM_TIMEOUT_INVENTORY` unset, preserve default behavior and output and create no inventory file.
- [x] When set to a writable file path, append the current test name and a newline at each parent `with_timeout` entry. Do not record arguments or payloads.
- [x] Do not duplicate entries for the guarded re-exec child or nested guards in that child. Report file I/O failures explicitly.
- [x] Regenerate the saved lane list from the source-identified production proof inventory, in a separate data commit. This observes macro-generated and indirect consumers without parsing Rust source.

## How it works

- [Runner operation and recorded results](../remote-builds.md#asynchronous-full-suite)

## Implementation inventory

- `tools/full_suite.py`: stable discovery, measured-cost partitioning, service launch and result aggregation.
- `tools/integration-costs.json`: saved Retail integration wall-time weights.
- `tools/full_suite_worker.py`: per-name native invocation and resource recording.
- `tools/refresh_integration_costs.py`: merges measured durations into the saved weights.
- `tools/integration-timeout-tests.json`: named timeout consumers assigned to dedicated lanes.
- `tools/integration-memory-peaks.json`: measured context-inclusive envelopes, OOM lower bounds and headroom policy.
- `tools/test_full_suite.py`: runner behavior tests using bounded child processes.

## Tests asserting this spec

- `tools/test_full_suite.py::IntegrationShardingTests`: exact-name selection, deterministic cost balancing, ignored entries and aggregate failure reporting.

## Known gaps (current cycle)

- [x] Finish the per-test timing profile and compare two versus six timeout permits.
- [x] Complete the production budget/memory proof, then report startup CPU-reduction candidates before any fixture change. The [source-identified proof](../wiki/investigations/test-runtime-optimization.md#native-timeout-inventory-and-complete-budget-proof--2026-11-04) passed in 1146.645s; this does not satisfy the 600s runtime target.
- [ ] Record the two complete passing runs and empty before/after test-list diff.

## Out of scope

Changing product fixture assertions or semantics, moving slow tests to OPTIONAL, or modifying Blizzard code. Harness conformance may assert its explicitly configured policy bound; the existing post-spawn deadlines and cleanup remain unchanged.
