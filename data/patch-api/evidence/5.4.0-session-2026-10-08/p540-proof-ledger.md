# 5.4.0 proof ledger

Own cwd: p540-page worktree. Own target: `/home/osso/.cache/wow-ui-sim-targets/p540-page`. Long Cargo commands stream to session logs through committed run_proof.py and run asynchronously. No poll-wait. Whole-word scans use `/usr/bin/grep`; retained outputs are untruncated.

| Command / scope | Recorded revision | Result | Validity |
|---|---|---|---|
| Python Mists parser RED | pre-67a3a86cf development | Missing parser methods | Development only |
| Python Mists extractor RED | pre-67a3a86cf development | Missing markup flag | Development only |
| cargo fmt | before each Rust commit | PASS | Later fmt --check is final |
| cargo test --test prefork_full_ui -- patch_5_4_0 | launch 28f10977f; behavior file added/committed as d0b490a60 during dependency compilation | Expected discovery mismatch: 22 gaps; 2 behavior cases pass | RED discovery only; final receipts must cover completed code revision |
| python3 -B -m unittest discover -s tools -p test_*.py | d0b490a60 | 88 PASS | Relevant Python source unchanged afterwards |
| reproduce_sources.py development invocation | d0b490a60 | 56 registers / 53 extracts; three inherited failures | Driver was uncommitted; not final proof |
| reproduce_sources.py committed invocation | 5d09d3ffc | 56 registers / 53 extracts; three inherited failures | Prior source bytes, all extraction-mode outcomes preserved; later ledger/fixture/docs do not change recipe scope |
| Diff extract reproduction | 5d09d3ffc | Byte-identical | Exact copied extractor flags; no later extractor change |
| 31 cached/src/tests bare removal scans | source code d0b490a60 | Untruncated grep evidence, pinned later-register scan | No subsequent src edits; caller tests added earlier |
| cargo test --test prefork_full_ui -- publication_sweep | Pending final driver receipt | Pending | Complete historical source-defined sweep set, no full integration suite |
| cargo test --test prefork_full_ui -- patch_5_4_0_behavior | Pending final driver receipt | Pending | Two current backed behaviors |
| cargo test --test integration patch_5_4_0_behavior | Pending final driver receipt | Pending | Same behavior contract in standalone fixture |
| cargo test --test integration instance_info:: | Pending final driver receipt | Pending | Existing instance-query callers |
| cargo test --test integration protected_frame_enforcement:: | Pending final driver receipt | Pending | Existing forbidden/protection callers |
| cargo fmt --check | Pending final driver receipt | Pending | No source changes after final format required |
| cargo check --no-default-features --features sound,gui,casc,client-mists --tests | Pending final driver receipt | Pending | Zero non-vendor warnings required |
| Synthetic missing-publication negative control | ae0b0b8ff | Expected failure; 22 → 23 gaps | Replace one passing occurrence to preserve fixed row count |
| cargo test --lib frame_state:: | Pending receipt | Pending | Existing source-unit frame security/state callers; no source mutation |
| tools/check_patch_validators.py | Pending sealed revision | Pending | Clean and later-audit phases; own-log tamper must fail |

## Conditional gates

No `src/` runtime code changed and no new retirement occurred. Runtime-module `cargo test --lib` versus master and addons-enabled lua-errors comparison are therefore not required by the conditional runtime-change rule. No full integration suite, source monkey-patching, deployment, agent/model CLI, push or merge.

## Invalidation policy

Command receipts record exact Git revision, command, environment, log hash, exit and invalidation state. Python source/tool changes invalidate reproduction/fixtures; Rust or known-gap fixture changes invalidate affected runtime proofs. Evidence/docs-only commits do not invalidate unrelated runtime/code scopes. Do not rerun broad proof merely for another commit milestone.
