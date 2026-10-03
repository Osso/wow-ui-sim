# Tainted table read hook

Goal: observer plus behavioral tests and docs; preserve propagation and disabled hot path. Only designated worktree; no push, agents, consumer edits, or deployment.

Baseline: 6044544b960cd68b4b0c58bb3373412757c2caee on tainted-read-hook; clean. Repo CLAUDE.md/AGENTS.md absent.

RED: `cargo test -j 4 --test integration tainted_table_read` on baseline plus new tests; exit 101; log tainted-read-red.log.

Behavioral RED (API installed, no dispatch): same command; exit 101. Log tainted-read-behavior-red.log.

Implementation committed: 1746620e08d8a7f27bef1de6fca857a934baa16c. Immutable state hook; one disabled option branch; original propagation unchanged. New tests also cover prepared root/shadow globals and rawget exclusion.

`cargo test -j 4 --test integration tainted_table_read` at 1746620: exit 0. See tainted-read-green.log.

`cargo test -j 4 taint -- --skip tainted_table_read` at 1746620: exit 0. Log existing-taint.log.

Existing taint gate: 12 unit + 18 integration tests passed. Manual changed-Rust readability audit: no new warning suppressions, small observer/setter, shallow branching, existing propagation untouched. No findings.

`cargo fmt -- --check` at 1746620: exit 0.

Full suite ONCE: `cargo test -j 4 --no-fail-fast -- --test-threads=4` at 1746620; exit 101. Log full-suite.log.

Full run: 774 unit passed; integration 487 passed / error_msg_call_global failed (generic nil call instead of named global). Oracle 277 passed. Pyrun UTF-8 decoder failed on fuzz output (byte 0xa2), closing stdout and causing fuzz/doc BrokenPipe; these results are invalid harness evidence, not established VM failures. Re-run only affected targets with file-backed stdout; no second full suite.

`cargo test -j 4 --test proptest_fuzz -- --test-threads=4` at 1746620: exit 0; fuzz-recovery.log.

`cargo test -j 4 --doc -- --test-threads=4` at 1746620: exit 0; doc-recovery.log.

Baseline 6044544: `cargo test -j 4 --test integration error_msg_call_global -- --exact --test-threads=4` exit 101. Log baseline-existing-failure.log.

`cargo check -j 4` at 1746620: exit 0. Log cargo-check.log. Baseline error_msg_call_global reproduces identical message; unrelated failure left unchanged. Fuzz recovery 5/5 and doctests 3/3 pass. Oracle reference binary absent: 277 reported successes include graceful no-reference skips, not oracle equivalence proof. Final branch tainted-read-hook clean, no push.

## Final report

Commit: `1746620e08d8a7f27bef1de6fca857a934baa16c` on `tainted-read-hook`. Clean worktree; not pushed. Required co-author trailer included.

### API

Exported from `rilua::vm::state`:

```rust
pub type TaintedTableReadHook =
    fn(&LuaState, GcRef<Table>, Val, &str, Option<&str>);

pub fn set_tainted_table_read_hook(&mut self, hook: Option<TaintedTableReadHook>);
```

Callback arguments: immutable state, supplying table, original key, slot owner, caller taint before propagation. Shared state permits logging through interior-mutable app data but prevents safe VM mutation/reentry. Storage mirrors EnvironmentTransferHook's optional function pointer; defaults to None. Observation precedes the existing propagation guard, even for already-tainted callers or disabled propagation.

### Files changed

```text
src/vm/state.rs
src/vm/state/tainted_table_read.rs
src/vm/execute/runtime_ops.rs
tests/integration.rs
tests/helpers/tainted_table_read.rs
docs/environment-transfer.md
CHANGELOG.md
```

### Coverage

Nine behavioral tests pass: correct table/key/owner and secure caller; repeated already-tainted integer reads; clean/missing exclusion; removal; unchanged values and caller taint with/without hook; GETGLOBAL; prepared root and live-shadow globals; table/function __index behavior; full GC between install/read; disabled-propagation observation; rawget exclusion.

Table-valued __index chains report only the final supplying slot. Function-valued __index reports ordinary tainted slot reads performed inside the function, not a synthetic proxy read. Raw host/stdlib reads and prepared-global snapshots that already bypass slot-read propagation remain unobserved.

### Commands and results

All post-implementation evidence covers commit 1746620; baseline evidence covers 6044544. No subsequent source changes invalidate it.

| Command | Result |
| --- | --- |
| `cargo test -j 4 --test integration tainted_table_read` | Initial missing-API compile failure; scaffold behavioral RED: 7 failed as expected; final GREEN: 9 passed |
| `cargo test -j 4 taint -- --skip tainted_table_read` | 12 unit + 18 integration passed |
| `cargo fmt -- --check` | Passed |
| `cargo check -j 4` | Passed, no warnings |
| `cargo test -j 4 --no-fail-fast -- --test-threads=4` | Run once: 774 unit passed; integration 487 passed / 1 failed; oracle 277 reported successful (reference binary absent, graceful skips); fuzz/doctests lost valid evidence to Pyrun UTF-8 decoding/BrokenPipe |
| `cargo test -j 4 --test proptest_fuzz -- --test-threads=4` | File-backed output recovery: 5 passed |
| `cargo test -j 4 --doc -- --test-threads=4` | File-backed output recovery: 3 passed |
| `cargo test -j 4 --test integration error_msg_call_global -- --exact --test-threads=4` | Baseline 6044544: identical existing failure |

`cargo fmt` formatted changes before commit. Full suite was not repeated. Logs are beside this report.

### Existing failure

`error_msg_call_global` fails both at baseline 6044544 and with this change. It expects `attempt to call global 'foo' (a nil value)` but receives `attempt to call a nil value`. Left unchanged as unrelated. Fuzz/doctest BrokenPipe failures were capture-harness artifacts; both targets pass with file-backed output. No other VM test failures observed; oracle equivalence remains unverified without the reference binary.

### Hot-path cost

Reasoned from source and repo performance notes; no timing claim. With no hook, exactly one added Option/function-pointer check before the unchanged propagation guard; no extra key resolution, metadata lookup, allocation, or callback. Existing propagation body is untouched. Enabled observation performs an additional metadata lookup and callback; secure propagation still performs its existing lookup. Owner strings are borrowed, not cloned by the observer. The function pointer survives GC and adds no GC roots.
