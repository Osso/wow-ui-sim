# Taint-log feasibility handoff

## Active goal
Determine whether pinned rilua exposes an actual tainted-table-read producer usable by a simulator-owned record sink for prose-2026-04-10-200/201. Author staged code/tests only if feasible. Completion: source-backed feasibility decision and exact handoff; exclude repository/external dependency writes, cargo/tests, git mutations, agents and model CLIs. Verification is read-only source inspection; no execution proof claimed.

## Evidence established
- Cargo.toml and Cargo.lock pin rilua 0.1.21 to 6044544b960cd68b4b0c58bb3373412757c2caee.
- Source line 200: “Fixed a bug that caused taint logging to stop working after a reload UI.”
- Source line 201: “Fixed a bug that caused taint logging to not correctly write messages for tainted reads from tables.”
- Scout labels both UNMODELED-BEHAVIOR; logging is separate from taint propagation.
- src/lua_api/globals/state_backed_queries.rs::reload_ui dispatches PLAYER_ENTERING_WORLD(false, true); it does not recreate the Lua environment in this implementation.
- src/cvars.rs:401 adds taintLogObjectSecrets=0; src/cvars.rs:198–199 loads src/cvars.yaml, whose line 1425 defines taintLog='0'. Configuration exists; an event producer does not.

## Decision: BLOCKED — no staged implementation
Both rows remain blocked under the external-rilua write prohibition. A simulator record list can store manually supplied records, but pinned rilua provides no tainted-table-read notification to populate it from actual Lua reads. A sink alone earns no credit. No repo edits, dependency edits, staged Rust files, tests, cargo invocations, git mutations, agents or model CLIs were performed.

### Inspected dependency
Read-only source: /home/osso-test/.cargo/git/checkouts/rilua-fd5a0715e46b5888/6044544/. This is the checkout corresponding to the Cargo.toml/Cargo.lock pin above, not an assumed ~/Repos/rilua tree.

### State vs producer
**State (simulator-owned):** record storage, taintLog gating and lifetime management could live in simulator state. Existing ReloadUI only dispatches an event; claiming environment recreation/rebinding from that function would be incorrect. The actual registered ReloadUI must be exercised for any eventual proof.

**Producer (missing in rilua):** successful Lua table-slot read notification carrying the resolved table, key and slot taint owner, independent of whether the caller is already tainted. Reading existing slot metadata or stack taint does not reveal that an access occurred.

### Exact missing capability and attachment points
There is no existing function/hook name for this capability. **INFERRED proposed interface name:** `LuaState::tainted_table_read_hook` (or a setter named `set_tainted_table_read_hook`); these names are proposals, not available rilua APIs.

The relevant existing function is `src/vm/execute/runtime_ops.rs:419`:

```rust
pub(crate) fn propagate_slot_read_taint(state: &mut LuaState, table_ref: GcRef<Table>, key: Val) {
```

It resolves string/integer slot taint and assigns `state.call_stack[state.ci].taint`. It invokes no host callback and returns immediately if taint mode is off **or the caller already has taint**. A notification placed only after that early return would miss reads by already-tainted callers. Read observation must precede the already-tainted-caller propagation guard, without changing the guard's original propagation semantics. Logging gate policy relative to disabled VM taint mode is not specified by these prose rows.

Existing successful-read paths reaching this helper:
- `src/vm/execute.rs:184–210`, `try_plain_table_get_ref`: plain-table fast path, helper called at line 206.
- `src/vm/execute/runtime_ops.rs:563–599`, `handle_table_gettable`: successful metamethod-aware table hit, helper called at line 580.
- `src/vm/execute.rs:1224`: live-table global-slot hit also reaches the helper; this is not evidence for snapshot fallback diagnostics.

`src/vm/state/api_ops.rs:19–57`, `LuaState::gettable` / `resolve_gettable_chain`, returns successful values directly without this helper or any read callback. Covering API-mediated reads would require routing that successful-read site into an observer too; that broader read coverage is **INFERRED**, not independently specified by the two prose rows.

The full signature above and `fn try_plain_table_get_ref(` / `fn handle_table_gettable(` each occur exactly once in their respective files. The simulator anchor `fn reload_ui(state: &mut LuaState) -> LuaResult<u32> {` occurs exactly once at line 107. These are verified source-location anchors, **not authorized edit instructions**.

### Why existing APIs do not suffice
- `src/stdlib/taint.rs:76–84` registers `debug.isglobalindex`, `setobjecttaint`, `getstacktaint`, `setstacktaint`, and `settaintmode`; no read-event subscription.
- `src/stdlib/taint.rs:328–373` implements `issecurevariable`; `src/vm/table.rs:308–319` exposes slot-taint getters. They query metadata, not access events. Polling cannot distinguish a clean read from a repeated tainted read once caller taint is unchanged.
- `src/vm/state.rs:76–99` defines hook events only for call, return, tail return, count and line. `debug.sethook` (`src/stdlib/debug.rs:1540`) installs those hooks; instruction/line hooks run before execution (`src/vm/execute.rs:1031–1089`), not as successful table-read callbacks. Decoding bytecode/operands through instruction polling would be a new diagnostic interpreter workaround, not an existing tainted-read producer; none is authored.
- `EnvironmentTransferHook` (`src/vm/state/environment_transfer.rs:9–15`) handles arguments/results crossing Lua environments, not ordinary same-environment table reads.
- Read-only Python searches across pinned rilua Rust sources found no taint-log, taint-event, taint-callback or taint-observer surface. Public hook declarations were also inspected. No rg/fd dependency was used.

### Coverage and expected RED evidence
| Row / required behavior | Status | Proof |
|---|---|---|
| 201: tainted table-field read emits a record | BLOCKED: no producer | Source inspection only |
| 201: clean reads emit none; value/caller taint unchanged | Unproved without actual observer | No runtime tests |
| 200: logging continues after registered ReloadUI | BLOCKED: no working producer before or after reload | Existing reload source inspected |
| Disabled taintLog writes none | Unproved sink behavior | CVar default inspected |

**Expected RED failures, not executed:** with a real observer/sink contract established, positive record assertions for tainted reads before ReloadUI, from a PLAYER_ENTERING_WORLD reload listener, and after ReloadUI would fail on the current implementation because it has no logging producer/sink. Already-tainted repeated-read assertions would additionally catch an observer incorrectly attached after the propagation early return. Negative clean/disabled assertions alone could pass vacuously and earn no credit. No compile-ready tests are claimed: inventing a nonexistent record API would produce compile failures rather than behavioral RED evidence.

**INFERRED policies withheld:** native message spelling, record schema, per-read versus deduplicated records, record retention across reload, taintLog numeric levels, and interaction with taint mode. The prose only requires working logging after reload and correctly written messages for tainted table reads. The task's chosen in-memory sink is not proof of native file-format parity.

## Exact handoff edits
None. External VM changes needed to expose the producer are out of bounds. Do not mark either row implemented, excluded, or narrative; record both as BLOCKED pending an authorized rilua tainted-table-read observer. No fallbacks or placeholder sink authored.
