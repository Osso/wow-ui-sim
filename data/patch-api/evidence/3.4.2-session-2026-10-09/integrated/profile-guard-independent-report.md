# Bounded independent profile-guard verification — 2026-10-09

**PASS — cross-profile aggregate guard and supplied terminal receipts only.** Not parent integration, native parity, or full-handoff completion.

## Revisions and provenance

Read-only worktree `/home/osso/.worktrees/wow-ui-sim-p342-source`: observed clean HEAD `f1adc06a517c427dfedae4fd06d5a5a84ea8b62f`. Canonical observed `ec600ac7da0a9d44c2dcc43e489c83af0763d74c`; its factory file is byte-identical to fixed worktree file. Commit diff adds only `#![cfg(feature = "client-wrath")]` to factory file.

**Receipt limitation:** v2 JSONs contain argv, exit, stdout and stderr, but no sealed revision/cwd/environment. Their compiler streams identify the p342 worktree. Caller associates these jobs with f1adc; clean observed HEAD and exact source match support that association, but do not independently attest the revision at execution time. No stronger revision claim made.

## Coverage matrix

| Scope | Concrete evidence | Proof level |
|---|---|---|
| Original cross-profile failure | Both original integrated RED receipts exit 101, E0432 unresolved `publication_sweep::run_factory_publication_sweep`; factory import line 6, helper line 520 cfg-excluded | Recorded reproduction at aggregate compile boundary |
| Default retail aggregate | `cargo test --test prefork_full_ui -- publication_sweep --nocapture`: v2 exit 0; `test result: ok. 72 passed; 0 failed; 72 total` | Recorded targeted aggregate compilation and 72 filtered tests; not full suite |
| Mists test compilation | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`: v2 exit 0; completed dev profile | Recorded compilation of selected test targets; no Mists test execution |
| 3.3.5 current output | 124 rows: 80 matches / 44 gaps, same original counts documented in retained source proof | Independently counted supplied observations; not a new run |
| 3.3.3 current output | 36 rows: 25 matches / 11 gaps, same original counts documented in retained source proof | Independently counted supplied observations; not a new run |
| Wrath retained controls | Original GREEN 6/6; five ledger inputs match, factory matches after removing sole new guard | Static applicability of retained execution; no new Wrath run |
| Formatting | Fresh `cargo fmt --check` in p342 worktree: exit 0, empty stdout/stderr | Fresh format proof only |

## Literal feature behavior and wiring

`Cargo.toml` standalone `patch_3_4_2_factory` target requires `client-wrath`. That restriction selects a Cargo target; it does not gate a module imported by another target. `build.rs:83-87` emits path-based modules for discovered files without reading Cargo target required-features. Generated `integration_tests.rs` includes `patch_3_4_2_factory` (observed at lines 4648-4649 or 4636-4637 in generated variants). `tests/integration.rs:1` and `tests/prefork_full_ui.rs:13` include this aggregate.

The new file-level inner cfg applies when this file is either standalone crate or imported module. With default retail features the predicate is false; with the supplied no-default Mists features it is false; with client-wrath it is true. It excludes the complete factory module and its Wrath-only import under retail/Mists. It is a positive feature condition, not profile detection: any feature set enabling client-wrath makes the predicate true. Existing aggregate `allow` attributes are generator-owned and unchanged; no suppression added by the fix.

## Retained Wrath proof and hashes

Original factory body SHA256 and current file stripped of the new guard both equal `6d8054a68b3e275b4c9c79fc0cfdf82a858f46f1049055be3019ca68c8872147`. Current guarded file SHA256: `34a435acc63dd93925665b85cf0fc8d1edeca3b9e8459bbeeaf33109f8b48897`.

All five original GREEN ledger hashes match: Cargo.toml, normalized factory body (including retail-rejection and foreign-successor controls), shared publication helper, exact known-gap fixture and inventory. Original→current inspected paths show only added prior verification report/receipt, not changes to original historical evidence or these inputs. Cargo.lock diff is empty. Retained 6/6 evidence concerns bare `WowLuaEnv::new`, client-wrath interface 38001—not native Wrath Classic 30402, loaded Blizzard UI, native event validity, arguments/security, or fresh runtime execution.

## Warnings and readability

Full JSON stdout/stderr loaded and retained without truncation. Every warning in both v2 streams is one of the same six inherited `iced-wgpu-patched/Cargo.toml` deprecated lint-name warnings (plus their aggregate count). Both warning lists exactly match original RED receipts. No new nonvendor warning; not warning-free. Retail CVar default-difference diagnostics remain separate from publication matching and are not compiler warnings.

Manual Rust readability review of the sole changed line: clear positive cfg, no new branching/nesting/state/side effects or TODO/FIXME/HACK/XXX; no warning suppression. No unrelated cleanup credited or requested.

## Boundaries and exact receipt

No Cargo tests/check/build/suites run by verifier; only authorized fresh format check. No edits to repo, Bash, cwd switch, deployment/operations, delegation or model CLI. Only requested `/tmp` report and receipt written. Initial mistaken RED path was resolved to canonical `data/patch-api/evidence/3.4.2-session-2026-10-09/integrated/`; failed persistent-runtime variable probes produced no repo changes or repeated commands.

Exact fresh git/format argv, cwd, exits and streams, full original/v2 receipt contents, stream/artifact SHA256 values, generated include excerpts/hashes, complete original GREEN receipt, input comparisons and current gap IDs: `/tmp/profile-guard-independent-receipt.json`. Earlier git outputs recovered from saved tool logs without rerunning commands. Historical GREEN combined log remains combined, not invented split streams.

**Parent portable gate pending. Full-suite Garrison failure unresolved.** These terminal targeted jobs do not close either gate.
