# Independent source / saved-receipt audit

OVERALL: PASS for the requested diagnostic contract at `0e4bf4161d58cbbf6572a5be08cac7f5019e316f`.

Read/followed verify and rust-readability skills. Audit only: no tests, checks, builds, operations, delegation, or reviewer backend executed. Read-only Git inspection, receipt parsing/hash validation and this requested report write. Bounded receipt wait: 150 seconds total; all final receipts became available. Receipt epoch: `20261010T183607Z/` beneath this directory.

## Source evidence

- [EXIST] PASS: both changed Rust files and both documentation files exist in the commit. Current changed Rust files exactly match committed source.
- [SUBSTANTIVE] PASS: `src/loader/lua_file.rs:220-244` captures the existing opt-in, executes the existing protected call once, formats success/error by borrowing its result, then returns the same result through unchanged error mappings. `src/lua_api/execution_budget.rs:98-109` adds a concrete success formatter using existing counter encoding.
- [WIRED] PASS: `execute_lua_file` → `execute_compiled_lua_file` → tainted `execute_budgeted_addon_file` → `format_file_budget_success`. The path remains gated by `retail-12-0-5`; `handler_timing.rs:13-15,62-76` establishes the existing environment-presence opt-in, independent of duration filtering.
- [ANTI-PATTERN] PASS: no new TODO/FIXME/HACK/XXX markers, placeholder bodies, suppressions, or commented-out implementation in changed lines.
- Source comparison `12124ded6..0e4bf4161` shows only diagnostic selection changes in `lua_file.rs`; both loader fixture bodies are unchanged. Successful fixture at lines 385-440 checks returns 19/42, final private total42, nested Lua and sequential meter continuity. Original error fixture preserves error strings, exhaustion100, failed next-file effects and subsequent unrelated evaluation.
- Error formatter/counter encoding, result conversions, owner selection, quota initialization/reset/exemption rules and handler-error logging are unchanged. Untainted/compile/setup boundaries remain outside this logging path.
- Rust-readability: no violations in changed lines. Short explicit formatting helper, effect-visible logging, bounded branching, no new mutable accumulation or warning suppression. Metrics commands deliberately not executed under no-check-rerun constraint.

## Exact saved runtime evidence

Read complete stdout/stderr for all four invocations, not merely result summaries. Every invocation receipt reports exit0 and unchanged sealed artifact.

| Case | Observed result | Diagnostic evidence |
|---|---|---|
| Existing success fixture, opt-in1000 | 1 passed; 0 failed | Exactly2 success records: First.lua limit1000 used_before0 used_after4; Second.lua limit1000 used_before4 used_after16 |
| Same exact fixture, environment flag removed | 1 passed; 0 failed | Exactly0 file-budget records |
| Original error fixture, opt-in1000 | 1 passed; 0 failed | Prior success0→2, then exactly2 error records2→100 and100→100, limit100 |
| Formatter/error tests | 9 passed; 0 failed | All9 named tests present, including escaped metadata, missing snapshots, unlimited meters and error controls |

Total: 12 passing test executions, 11 distinct tests; four subprocess invocations. Success identities/chunk ordering and limit/continuity/increasing counters independently inspected against raw lines, agreeing with `success-counter-validation.json`.

`checks/fmt.json`: exit0, source_equal=true; both output streams empty.
`checks/default-check.json`: exit0, source_equal=true; full stderr ends with completed dev profile.
`compile-result.json`: exit0, source_equal=true, no stream errors. Entire Cargo JSON stream parsed; final build-finished success=true. Corresponding library test artifact has fresh=false and matching default/retail feature list including retail-12-0-5.

Warnings: compile and default-check retain six iced_wgpu manifest deprecations (hyphenated Clippy keys). Exit0 is not warning-free. Those files are outside this diagnostic commit; no suppression added.

## Source/artifact binding

- `submission.json`, fmt/default-check receipts bind revision `0e4bf4161d58cbbf6572a5be08cac7f5019e316f`; observed repository HEAD matched that revision.
- Independently compared all3853 entries in `source-before.json` with `source-after.json` and current filesystem SHA256: all equal. Both changed Rust files also match `git show` for the requested commit.
- Parsed Cargo compiler-artifact executable matches `artifact.json` original path and features. Compile was not a fresh-cache-only receipt: fresh=false.
- Independently hashed the sealed binary: `c202569db345fc13b7e65c9b23ae3fd18b1e21136f3a9542a6192ce11d230146`, matching artifact receipt. Every invocation argv targets that sealed binary and each receipt records artifact_unchanged=true.
- `final-gate-outcome.json` reports PASS, agreeing with raw logs and independently checked binding.

Limits: this is bounded local source/receipt provenance, not hermetic attestation. Worker explicitly excludes external dependency provenance, untracked files/index, inherited environment and runtime assets outside fixture inputs. No saved actual cargo/rustc version receipt inspected. This audit establishes the requested diagnostic behavior, not full-addon startup repair, native WoW quota parity, exclusive/self instruction attribution, or absence of unrelated failures. No runtime gate remains PENDING in this requested scope.
