# Independent duration fixture verification — PENDING serialized compile grant

Canonical checkout: `/home/osso/Projects/wow/wow-ui-sim`. Observed initial HEAD: `4142f3d416ff0fa9d5505926e85e5d48ac662f7c`. No tracked edits, delegation, network, operations, compilation, or tests performed. No production/native parity or parent-goal credit.

## Static audit

Read verify and rust-readability skills, `/tmp/duration-callback-contract-decision.md`, retained `data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/current-cvar-duration-red/report.md`, and numeric-rule-formatter/duration-text-binding specs. Retained RED streams read in full: exit 101, 0 passed/1 failed, `expected LuaDurationObject at argument 1` at the scalar SetDuration boundary. CVar retained PASS remains valid; not rerun.

Commit changes one Rust file, `tests/numeric_rule_formatter.rs`; also contains docs and retained evidence, so it is not literally a one-file commit. Rust diff adds an explicit `C_DurationUtil.CreateManualClock(0)`, a local `create_duration(seconds)` helper using `CreateDuration`, `SetClock(clock)`, and `SetTimeFromStart(0, seconds)`, replaces the two scalar inputs with durations, and changes custom `.Format` to read `duration:GetRemainingDuration()`. No runtime source changed by this commit. All five original observable assertions remain: binding `2`, label `2`, label `9`, copied Down formatter `8`, custom callback `custom:8.2`.

Manual changed-Rust readability audit: no violations found. The added helper has a single construction responsibility, explicit deterministic clock, meaningful names, no branching/nesting/state accumulation, no warning suppressions. The changed Rust test remains a straight-line env.exec behavior test; added Lua code is readable and directly samples the public object method. No broad automated readability command run.

Spec audit: custom table callback receives original duration object (source `src/c_api/duration_text_binding.rs`), not sampled scalar text; corrected callback reads that object's remaining value. Explicit zero clock/start and default rate 1 preserve 1.2/8.2. Numeric formatter/font-string/copy assertions unchanged. Tests cover modeled contract only; native callback representation, native timing parity, other profiles, client startup, and production changes are excluded. Spec's historical 'seven focused cases' wording is not used as an observed runtime count.

## Command ledger and scope

`cargo fmt --check` run exactly once in canonical cwd: exit 0, full stdout empty, full stderr empty. Separate streams saved as `format.stdout`/`format.stderr`; receipt `format.json`. Exact command start/end times were not retained because Pyrun evaluation locals did not persist across calls; explicitly disclosed, not reconstructed or rerun. `scope-before.json` provides actual earlier capture epoch and selected hashes. `code-before.json` records actual HEAD, epoch, and tracked src/tests/crates/Cargo/build/config hashes for later drift comparison.

Initial exploratory git commands lacked explicit cwd and returned exit 128 (not a repository); subsequent commands used canonical cwd. Initial report relative path was absent; parent supplied retained absolute path. No failed command was retried to recover logs. No auto-backgrounded command.

## Pending

Main mailbox compile grant still required. After grant: exactly one offline locked jobs4 integration no-run JSON compile with separate full streams/exit/times, fresh artifact selection/hash/features, one listing proving changed case exactly once, one bounded numeric_rule_formatter module batch <=90s, full output review, post-proof hashes. No completion claim before runtime proof.
