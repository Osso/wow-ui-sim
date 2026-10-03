# Diagnostics / throttle authoring handoff

Scope: `prose-2026-03-25-093`, `prose-2026-03-25-095`, `prose-2026-03-12-050`. Repository read-only; outputs staged under `staging/diag-throttle/`. No builds, tests, git mutations, agents, or model CLIs.

## Source contract

- `data/patch-api/sources/12.0.5-api-changes.txt:93`: “Added improved error messaging for various security-related errors involving tables.” No named diagnostics, message text, error type, argument numbering, or operation inventory is specified. Native wording remains unestablished. No invented exact-message assertions or compatibility credit.
- Same source `:95`: “Addon execution throttles will no longer be applied while processing PLAYER_LOGOUT and ADDONS_UNLOADING events.”
- Same source `:50`: “Addon execution throttles will no longer apply while the PLAYER_LOGOUT and ADDONS_UNLOADING events are being processed.” These two occurrences describe one behavioral requirement, not two separate models.

## 093 decision and exact current diagnostics

**State-backed diagnostic production is possible; native improvement remains unspecified.** Existing VM table `security_flags`, read-only/frozen state, secret-wrapper identity and caller taint drive actual access/mutation rejection. The producer is rilua's table-access/mutation boundary, not a new C_* registration.

Source-derived literal messages (not executed observations or established Blizzard wording):

| Condition | Current message |
|---|---|
| Tainted access to DisallowTaintedAccess table | `tainted access to secured table` |
| Secret key on DisallowSecretKeys table | `secret key access to secured table` |
| Tainted security-policy/wrap/secret-unwrap operation | `table security operation requires an untainted caller` |
| Unsupported option2 | `SecretWrapContents is not supported` |
| Invalid security option | `invalid TableSecurityOption` |
| Native settablesecurity non-table argument | `settablesecurity requires a table` |
| Collected table at guard boundary | `table has been collected` — internal failure, not ordinary violation |
| VM read-only mutation / setmetatable / native table mutator | `attempt to modify a read-only table` |
| Arena-frozen mutation / simulator frozen-table mutator | `attempt to modify a frozen table` |
| Protected __metatable replacement | `cannot change a protected metatable` |
| Simulator freeze/isfrozen non-table input | `bad argument #1 (table expected, got <type>)` |

Primary producer: pinned rilua `6044544/src/table_security.rs:30–66,248–270`; mutation producers: `vm/table.rs:1143`, `vm/execute/runtime_ops.rs:738,746`, `stdlib/base.rs:585,609,706`, `stdlib/table.rs:106`; simulator `src/lua_api/globals/real/table_freeze.rs:49` and `table_extensions.rs:38–45`. Full report records dependency path and revision.

These Rust runtime errors normally become `pcall` results `false, <string>` (`rilua/stdlib/base.rs:394–457`), without a structured security-error table. Runtime Display prints the message; simulator event reporting separately adds handler context. Prose establishes neither this shape nor replacement wording.

**Epoch caveat:** native table-security global registration is Forever-only (`env_init/mod.rs:75–81`). Later Retail's `settablesecurity` bootstrap is a no-op. `table.freeze` registration is gated at 12.1.0 (`globals/register.rs:145–146`), not historical 12.0.5. Host-installed VM guards in housing tests do not prove retail API parity. Do not change these adjacent registrations under row093.

No new diagnostic tests: source specifies no text/shape predicate. Category assertions would be INFERRED; exact wording assertions would invent native requirements. Existing guard/freeze tests already cover rejection. No blanket rewording or placeholder credit.

## 095/050 minimal design and state/producer split

**Observable and designable, not implemented or accepted.** Real frame `RegisterEvent` / `SetScript('OnEvent', ...)` consumers can expose completed side effects and error-handler reports. Both event names are recognized. No src producer actually fires them during logout/unload; `ReloadUI` currently emits PLAYER_ENTERING_WORLD only.

No simulator execution-budget enforcement found. Timing telemetry, GC/resource budgets and test/network timeouts are unrelated. PTR `GetScriptBucketThrottleLimits` is explicitly a zero mock, not enforcement or a source of 12.0.5 limits.

1. **State:** per-addon supplied limit/consumption records, current executing-addon identity, accounting window if required by supplied policy, and scoped `teardown_exempt` flag. No invented native defaults or fake public setter/getter.
2. **Execution producer:** trusted VM accounting consumes actual execution work. A deterministic instruction budget is **INFERRED simulator policy**, not native millisecond throttling. Source does not establish limits, reset periods, attribution or reject-versus-pause behavior.
3. **Lifecycle producer:** enter exemption before all callbacks/frame handlers for either teardown event; restore previous state after success/error. Wire every event entrance and actual logout/unload production, not merely event-name recognition.
4. **INFERRED scope/accounting:** nested synchronous dispatch inherits exemption; delayed callbacks do not. Exempt work neither consumes nor refills ordinary budget. Native confirmation required before parity credit.
5. **Observable proof:** supplied-budget normal workload cannot complete; identical workload completes for each teardown event; next normal event remains budgeted; handler error restores scope; another addon retains independent accounting.

Rilua count hooks exist (`vm/state.rs:463–509`, `vm/execute.rs:1031–1063`), but they are mutable/per-thread debug state, not trusted independent budget enforcement. A Lua hook can be replaced/cleared; hook errors can be caught by pcall. Post-handler timing cannot stop a nonreturning script. Do not substitute debug-hook scaffolding or callback-count throttling. A trusted VM meter and lifecycle producers are integration gaps; dependency edits are outside this authoring task.

## Exact handoff edits and RED status

- New staged report: `staging/diag-throttle/docs/diag-throttle-audit.md`, mirroring `docs/diag-throttle-audit.md`. Optional documentation application only.
- **Existing-file edits: none. Full replacement code: none.** No registration, dependency, manifest or runtime edits are proposed.
- Reference anchors verified unique (one occurrence each): `env_events.rs` signature `    pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {`; `globals/state_backed_queries.rs` signature `pub(crate) fn dispatch_event_now(`; `script_helpers/event_dispatch.rs` signature `pub fn fire_named_event_state(state: &mut LuaState, event_name: &str, args: &[Val]) {`. These locate future integration, not patches to apply.
- **Expected executable RED failures: none authored.** There is no source-grounded diagnostic assertion or implemented supplied-budget contract to test. Future ordinary-budget failure is a design expectation, not measured RED. Success-only teardown tests against today's unthrottled runtime would be vacuous.
- **Existing expectation changes: none.** Message-sensitive tests include Forever secured maps, housing pending/decor destroy/category/variant guards, private-aura guards and freeze argument tests. Preserve them; full report records paths/lines.

No Rust test file was staged, so no extra binary, cfg, raw-string or numeric-conversion issue is introduced. Future tests belong to one top-level auto-included integration module, begin `#![cfg(feature = "retail-12-0-5")]`, and require the actual producer contract first.

## Proof ledger / row accounting

Read scout sections, exact source lines, registration gates, pinned VM guards/hooks, relevant tests and event-dispatch paths. Python source searches and unique-anchor counts only. No cargo/tests/git mutations, agents, model CLIs or repository writes. No compiled, RED/GREEN, deployed or native-wording proof claimed.

Keep all three rows pending: 093 lacks a specified/native diagnostic delta; 095/050 have one feasible exemption design but no trusted budget enforcement or actual teardown lifecycle proof. Documentation does not earn compatibility credit.
