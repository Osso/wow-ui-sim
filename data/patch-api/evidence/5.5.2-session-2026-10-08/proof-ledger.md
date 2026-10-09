# 5.5.2 proof ledger

Goal: pin page 686956, account for every source row under Mists Classic, preserve retail observations, and pass portable proof validation. Excludes push/merge/delegation, runtime redesign, vendor edits and inferred API contracts. Required gates are the user-requested sweeps, controls, Mists tests/check, Python fixtures, reproduction, formatting and validator portability.

## Scope

- Base: `896086537a2b3c1ead5886d5ae3e430d56e7ef20`.
- Source/test commit: `39bb21f41`; only new Mists-gated test and empty known-gap fixture change the src/tests/tools/build/Cargo scope.
- No shared `src/` change. Touched-module integration/prefork/lib regression gates therefore have no additional modules to exercise.
- Rust readability: changed test manually inspected against rust-readability checklist; one bounded test closure, no warning suppressions, added abstraction or non-test function. The 5.5.3 harness is preserved literally with only source identity and same-line successors changed.

## Commands and proof status

| Command | Revision / scope | Result | Invalidation |
|---|---|---|---|
| MediaWiki query pageid 686956, current main-slot revision | Captured response; revision 6778080 | TOC 50502; resources-only stub | New fetch/revision requires new audit |
| `python3 -B .../reproduce_sources.py 39bb21f41` | All Git-pinned sources and tools | 59 registers identical; 56 extracts identical; three inherited failures exact | Source/tool changes |
| `/usr/bin/grep -RInw ...` | 39bb21f41 src/tests and Mists cache, Documentation excluded | Complete outputs and hashes in scan-receipts.json; no retirement candidates | New retirement candidates or cache changes |
| `python3 -B -m unittest discover -s tools -p test_*.py` | Receipt revision; tools | PASS: 87 tests | Tool changes |
| `cargo fmt --check` | Receipt revision; Rust | PASS | Rust changes |
| `cargo test --test prefork_full_ui -- publication_sweep` | Branch plus pinned master; separate outputs | PASS: branch 57/57 and pinned master 57/57; all 9,749 observations on 56 retail pages identical | Relevant runtime/test changes |
| `cargo test --test integration publication_sweep_client_lines -- --nocapture` | Retail; receipt revision | PASS: 3/3 | Classifier/control changes |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_ -- --nocapture` | Mists; receipt revision | PASS: 4/4; all three page outputs `{}` | Mists/test changes |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Mists; receipt revision | PASS: zero non-vendor warnings; seven vendor-manifest warning lines | Source/config changes |
| Own Mists test with P552_SWEEP_REGISTER=negative-register.json | One injected row; expected exit 101 | PASS: exit 101, exact `register row count changed`, left 1 / right 0 | Sweep/test changes |
| `python3 -B tools/check_patch_validators.py 896086537a2b3c1ead5886d5ae3e430d56e7ef20` | Pinned master only; master-gate-report.json | PASS: clean 43/43, later 44/44; does not cover own new validator | Pinned historical scope remains fixed |
| `python3 -B tools/check_patch_validators.py REVISION` | Committed branch proof; exact revision and both phase results in gate-report.json | gate-report.json / gate-summary.json retain the acceptance result | Sealed validator/evidence changes; later docs/report-only commits do not invalidate |

Complete command vectors, revisions, exit codes, target paths and log SHA-256 values live in each `*.proof.json`. No cargo command is rerun just to recover output; asynchronous workers retain complete logs. Later evidence/docs commits do not invalidate unchanged runtime proof.

## Final acceptance boundary

All workers completed with their expected exits. Complete logs were reused; no Cargo command was rerun to recover output. `context.json` seals the completed receipts, exact retail comparison, source accounting, reproduction, scripts and pinned prior-validator-set receipt before the acceptance commit.

The branch gate records its exact committed revision and clean/later-audit results in `gate-report.json`; `gate-summary.json` exposes the counts. `validator-output.txt` retains own-validator output. `tamper-proof.json` records the own sealed-log rejection and byte-identical restoration. These report files are excluded from the seal to avoid self-reference. Later wiki/spec/report-only commits leave every sealed acceptance input unchanged.

User forbids agents and model CLIs; verification is main-thread command/artifact proof, not independent model review. No push, merge, vendor edit, runtime change or delegation is part of this audit. Wiki index/log byte and line counts are checked against the prior commit before every commit.
