# Verdict: ACCEPT WITH QUALIFICATIONS

Reviewed clean worktree HEAD `1746620e08d8a7f27bef1de6fca857a934baa16c`, direct parent `6044544b960cd68b4b0c58bb3373412757c2caee`. No blocking propagation, returned-value, or aliasing defect found. Merge risk is low for a passive event sink; this is not an exhaustive table-read tracer or a precise instruction-location API.

## Findings / qualifications

1. **Observer can see stale instruction location on fast reads.** `src/vm/execute.rs:206`, `:1224`, and `src/vm/execute/runtime_ops.rs:479`: callback runs before destination register assignment and before propagation. Dispatch keeps PC in a local (`execute.rs:1010–1024`); plain GETTABLE/GETGLOBAL/SELF fast reads do not first synchronize `CallInfo.saved_pc`. Slow fallback does (`execute.rs:1249`, `:1300`). A sink inspecting saved_pc/source lines can therefore attribute a fast read to an earlier instruction, whereas slow reads have current location. Existing debug line lookup uses saved_pc (`src/stdlib/debug.rs:492`). This is a newly exposed observation limitation, not corrupt VM state or changed Lua behavior. No exact-location promise exists, so not merge-blocking; do not treat callback state as an instruction-complete debug snapshot.
2. **Read coverage documentation and tests are incomplete.** `docs/environment-transfer.md:24` describes “metamethod-aware reads” and excludes “Raw host/stdlib reads,” but metamethod-aware host `LuaState::gettable` also bypasses propagation on successful direct/table-chain results (`src/vm/state/api_ops.rs:18–35`). That exclusion is not stated explicitly. SELF, next/pairs, host gettable, custom environments, snapshot bypass, per-state isolation, and coroutine behavior lack dedicated observer tests. Existing implementation consistently follows existing VM propagation sites; this is a documentation/coverage gap rather than a propagation regression.

## 1. Semantics preservation

The original body beginning with `if !state.taint_mode || state.call_stack[state.ci].taint.is_some()` is byte-for-byte identical to base, confirmed by direct source comparison. All subsequent early returns, string-key copying, numeric-key resolution, metadata lookup, and first-owner assignment remain unchanged (`src/vm/execute/runtime_ops.rs:423–446`). The function as a whole is not byte-identical: the Option check precedes that body.

With `None`, same caller taint and returned values. With a passive, non-panicking hook, same result: callback returns `()` and receives shared state, then unchanged propagation executes. Already-tainted callers retain their prior owner; observation still fires before their early return. Observation deliberately fires with propagation disabled when existing metadata remains. Panic/conflicting RefCell borrows can interrupt execution; docs explicitly prohibit both. “Unchanged” cannot mean arbitrary callback code has no external effects.

## 2. Disabled hot-path cost

Exactly one added source-level Option check per invocation of `propagate_slot_read_taint` (`runtime_ops.rs:420`). **Not confined to a slow path:** ordinary successful non-nil plain reads call it at `execute.rs:206`, even for clean slots, already-tainted callers, or disabled taint mode. Resolved metamethod-chain hits call it at `runtime_ops.rs:615`; live slot-shadow hits at `execute.rs:1224`. Nil/missing reads and bypass paths do not check the Option.

No additional lookup/allocation occurs before learning a hook exists. Enabled observation adds table-arena lookup and string-arena/slot-owner lookup or integer-slot lookup (`runtime_ops.rs:457–473`), without allocation in observer machinery. Secure propagation afterward still repeats its original lookup/string allocation. Callback allocation is host-controlled. No assembly or performance benchmark was run; this is exact source-level cost, not a measured cycle count.

## 3. Read-path matrix

| Path | Fires? | Documentation / behavioral proof |
|---|---|---|
| Plain GETTABLE | Non-nil tainted slot: yes | Explicit; secure, integer, repeated, clean/missing tests |
| GETGLOBAL | Yes, including live root/shadow slot reads | Explicit; ordinary global and prepared root/shadow tests; custom env untested |
| SELF | Yes, same plain/fallback helpers (`execute.rs:1293–1314`) | Implicit under plain reads; dedicated test absent |
| Table-valued __index | Final supplying slot only | Explicit; multi-hop chain tested |
| Function-valued __index | No synthetic event; VM reads inside function fire | Explicit; real read and synthetic return tested |
| rawget | No | Explicit; negative test |
| next / pairs iteration | No event for yielded fields (`src/stdlib/base.rs:873–927`) | Implicit raw-stdlib exclusion; negative tests absent |
| Raw host globals / Table::raw_get | No (`src/api.rs:70–77`, `src/handles.rs:56–65`) | Generic raw-host exclusion; dedicated tests absent |
| Host LuaState::gettable | Direct/table-chain hits: no; VM reads inside Lua __index function: yes | Ambiguous documentation; dedicated test absent |
| GETGLOBAL-slot snapshot bypass | No (`execute.rs:1196–1198`, `:1230`) | Explicit; dedicated negative test absent |

String and finite integer keys are the existing tracked slot-metadata domain. No new tracking of boolean/table/function/fractional keys was introduced.

## 4. Safety / state ownership

No new unsafe code. Callback takes a temporary shared reborrow of mutable VM state; owner/table/string borrows are shared, and mutation resumes only after callback returns (`runtime_ops.rs:457–486`). Safe Rust cannot mutate/reenter this VM through `&LuaState`; app-data interior mutation is intentionally permitted. Retained GC handles are not roots; borrowed owner/caller text must be copied. GC survival is tested.

Pointer is per LuaState (`src/vm/state.rs:682`), initialized to None (`:832`), not process-global. Coroutines swap stacks/call frames into the same LuaState, without swapping this field (`state.rs:1032–1100`), so observer is shared across coroutines within one VM and sees the active coroutine's caller taint. Separate states remain independent. Calls are synchronous on the executing thread; the pointer adds no synchronization. Existing feature-gated Send permits moving Lua between threads, not simultaneous mutable execution (`src/lib.rs:123–137`); thread-local logging sinks must account for migration. None of these ownership cases has a dedicated observer test.

## 5. Nine tests: behavioral, useful, not exhaustive

Tests execute Lua and assert actual returned values, propagated caller taint, event count/order, actual supplying table/key, slot owner, and pre-read caller owner. They are not source-shape tests. Moving observation below the early return would fail already-tainted/repeated-read, propagation-disabled, and preservation cases. Moving it after propagation would fail secure-caller records. Wrong supplying-table reporting would fail index-chain and shadow cases. Unconditional notifications would fail clean/missing/raw-access cases. No-hook and hook-enabled values/taint are both checked.

Weakest assertions: disabled-propagation test checks count/caller but not complete record; rawget is the only explicit bypass negative test. Mutation testing was not run, and absent path tests above remain absent.

## 6. Reported error_msg_call_global failure

`tests/integration.rs:4340–4347` is identical at base and HEAD. It calls missing global foo and expects a global-name error. Missing foo is nil, so neither direct nor fallback read calls propagation; this test installs no observer. Compiler/error-name logic is unchanged by this commit. Source evidence supports unrelated pre-existing failure. **Base failure was not independently reproduced:** both authorized filters exclude this test, and no additional test command was run. Do not describe its reported base failure as observed proof from this review.

## 7. Scope and proof ledger

Diff is seven files, 285 insertions, zero deletions: observer declaration/setter, field/default/module export, observer invocation/helper, nine tests and integration wiring, changelog and documentation. No dependency, compiler, error formatter, or coroutine implementation changes. No TODO/FIXME/HACK placeholder appears in added implementation. Worktree remained clean; no source edits, commits, pushes, agents, or model CLIs.

| Command, each run once at reviewed HEAD | Observed result |
|---|---|
| `cargo test -j 4 --test integration tainted_table_read` | Exit 0; **9 passed, 0 failed**, 479 filtered out |
| `cargo test -j 4 taint -- --skip tainted_table_read` | Exit 0; **12 unit + 18 integration passed, 0 failed**; other targets ran 0 tests |

**Total observed: 39 passed, 0 failed.** No later source changes invalidated proof. Full suite, base failure reproduction, instruction-location tests, benchmarks, and mutation testing were not performed.
