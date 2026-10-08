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
| `python3 -B -m unittest discover -s tools -p test_*.py` | Receipt revision; tools | PASS (count from tools-tests.txt) | Tool changes |
| `cargo fmt --check` | Receipt revision; Rust | PASS | Rust changes |
| `cargo test --test prefork_full_ui -- publication_sweep` | Branch plus pinned master; separate outputs | Pending asynchronous jobs | Relevant runtime/test changes |
| `cargo test --test integration publication_sweep_client_lines -- --nocapture` | Retail; receipt revision | Pending asynchronous job | Classifier/control changes |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_5_5_ -- --nocapture` | Mists; receipt revision | Pending asynchronous job | Mists/test changes |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Mists; receipt revision | Pending asynchronous job | Source/config changes |
| Own Mists test with P552_SWEEP_REGISTER=negative-register.json | One injected row; expected exit 101 | Pending asynchronous job | Sweep/test changes |
| `python3 -B tools/check_patch_validators.py REVISION` | Final committed proof | Pending | Validator/evidence changes |

Complete command vectors, revisions, exit codes, target paths and log SHA-256 values live in each `*.proof.json`. No cargo command is rerun just to recover output; asynchronous workers retain complete logs. Later evidence/docs commits do not invalidate unchanged runtime proof.
