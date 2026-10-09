# Independent bounded verification — PASS

Revision: aa9004aca92a4ef09592bf4b98753b32ebea0f78.
Canonical cwd: /home/osso/Projects/wow/wow-ui-sim.
All three reviewed tracked files match this commit; recorded SHA-256 hashes remained stable through verification. git status reported only untracked .code-index.db; it was not changed by this verifier.

## Proof ledger

- Once: `python3 -B -m unittest tools.test_full_suite -v`, canonical cwd. Exit 0; 3/3 PASS in 0.490s. UTC 2026-10-09T20:25:48.860805+00:00 through 20:25:49.383232+00:00.
- Once: `cargo nextest run --help`, canonical cwd, resolved Cargo /usr/bin/cargo. Exit 0, stderr empty. UTC 2026-10-09T20:25:49.383589+00:00 through 20:25:49.402389+00:00.
- Separate full stdout/stderr and exit/UTC metadata retained as focused-tests.{stdout,stderr,json} and nextest-help.{stdout,stderr,json}. Revision/diff/status outputs retained separately. hashes.json contains source, installed wrapper, and old installed launcher hashes. scope.json records exact commit-content matches and source-equivalence comparisons.

Exact test output (stdout empty):
```
test_missing_shared_wrapper_fails_without_starting_cargo (tools.test_full_suite.FullSuiteWorkerTests.test_missing_shared_wrapper_fails_without_starting_cargo) ... ok
test_shared_lock_covers_successful_children (tools.test_full_suite.FullSuiteWorkerTests.test_shared_lock_covers_successful_children) ... ok
test_shared_lock_releases_after_failure_and_retains_result (tools.test_full_suite.FullSuiteWorkerTests.test_shared_lock_releases_after_failure_and_retains_result) ... ok

----------------------------------------------------------------------
Ran 3 tests in 0.490s

OK
```

## Behavioral coverage

Actual worker run/run_step functions execute in a new session/subprocess with fake git/Cargo and a temporary real-flock wrapper. External lock blocks the first Cargo child. Integration, prefork, and lib children each remain blocked awaiting test release while the builder lock is demonstrably held. Each receives --offline and --locked, compilation jobs=4 overriding inherited 29, and nextest retains FULL_SUITE_JOBS=3 as test concurrency.

Prefork exit 7 propagates into stored steps/log; parsed sandbox::failure is retained; lib continues successfully. Worker itself still exits 0 for recorded suite failures, matching existing semantics. Lock is reacquirable after completion. Missing wrapper produces an explicit nonzero worker error before Cargo starts; no fallback execution.

## Installed wrapper and detached wiring

Tracked tools/full_suite.py run calls run_step for every Cargo step. run_step requires /home/osso/.worktrees/build-lock.sh, prepends that wrapper, appends offline/locked, and waits synchronously via subprocess.run.

Installed wrapper inspected in full: opens /home/osso/.worktrees/.builder.lock on fd9, acquires flock, executes foreground "$@" retaining fd9, and returns that final command's status. Source proves lock spans foreground child lifetime, including unsuccessful completion; tracked runner has no unlocked alternate path.

submit schedules sys.executable plus os.path.abspath(__file__) and run via systemd-run. Therefore submitting the canonical tracked script schedules that same script, not /home/osso/bin/full-suite. submit/status/new_failures/summarize/resolve/prepare_checkout ASTs match parent commit exactly. Suite lock, checkout/results, failure regex contents, step ordering, and nextest test-worker args remain unchanged; compile jobs independently become 4.

Installed nextest help lists --locked (Require Cargo.lock is up to date) and --offline (Run without accessing the network) under Manifest options. This confirms installed run CLI exposes both flags; no real run or additional exact-argv parser invocation was performed.

## Limitations

/home/osso/bin/full-suite remains old and unchanged: direct unwrapped Cargo, no offline/locked, compile jobs tied to JOBS. Canonical tracked script is the intended fixed entry point; actual operational invocation/submission was not observed. Deployment is separate and was not performed.

Tests substitute a temporary equivalent flock wrapper rather than executing the installed shell wrapper or contending on the production builder lock. Installed wrapper lifetime/status behavior has source-audit proof, while temporary wrapper/process contention has behavioral proof. Lock assertions sample each active child; no continuous tracing, signal, or daemonized-grandchild coverage. Nextest failure-regex/status/master-baseline preservation has source-equivalence proof, not additional runtime tests. Missing wrapper raises before result JSON completion, as explicitly tested; it does not generate a normal completed suite result.

Dependency cache completeness, lockfile freshness, actual builds/full suites/systemd submission are untested. No source edits, delegation, network, Bash, change_working_directory, actual Cargo builds, broad gate, systemd actions, or deployment. Only verification artifacts were written under /tmp/fullsuite-lock-independent/.
