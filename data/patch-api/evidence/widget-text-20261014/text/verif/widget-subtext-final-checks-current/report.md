# Saved format/default source compile: PASS, bounded

Production `e37f5426a1da8d721b0ab3ff0341877f59100c02`; test/source checkpoint `62f115d262613a848180143d6b30e2a0b3eb63b0`. Independent inspection of submission/argv, revision, admission, toolchain, terminal status and complete stdout/stderr. Saved evidence only; no commands rerun.

| Check | Exact Cargo argv after env | Saved result | Output |
|---|---|---|---|
| Raw format | `/usr/bin/cargo fmt --all -- --check` | launcher 0; ExecMainStatus 0; Result success | stdout/stderr empty (0 bytes each) |
| Default source compile | `/usr/bin/cargo check --offline --locked -j 8` | launcher 0; ExecMainStatus 0; Result success | stdout empty; six manifest deprecations plus one summary; Finished dev profile in 16.22s |

Both terminal snapshots report ActiveState=active, SubState=exited (RemainAfterExit=yes), not a running build. Format snapshot: 2026-10-14T18:11:40.109767+00:00; check snapshot: 2026-10-14T18:11:40.388469+00:00.

Toolchain: Cargo 1.99.0 (5f94df478 2026-08-27), rustc 1.99.0 (b940084d7 2026-09-28), x86_64-unknown-linux-gnu, LLVM 23.1.1; `/usr/bin/cargo`, `RUSTC=/usr/bin/rustc`. Recorded metadata, not freshly sampled toolchain.

## Saved admissions and reported terminal limits

| Check | Claimed high/max/CPU | Reported high/max bytes | Reported MemoryPeak bytes | Reported CPUQuotaPerSecUSec |
|---|---|---|---|---|
| Format | 1G / 1G / 200% | 1073741824 / 1073741824 | 1075867648 | 2s |
| Check | 5G / 6G / 800% | 5368709120 / 6442450944 | 5369307136 | 8s |

Format reported peak marginally exceeds both high/max by **2125824 bytes**. Check reported peak exceeds high by **598016 bytes**, below max. Preserve these actual properties; do not claim peaks stayed within all limits. No memory-event samples available at terminal; **no OOM or causality inference**. CPU 2s/8s values are reported quotas only, not consumption or current live-service verification.

Both admissions recorded allowed=true. Format projected=19684633308, host_available=41143111680; check projected=27221457508, host_available=39046361088. Complete admission raw samples retained in proof.json, not parent/slice health or enforcement proof.

## Warnings retained, no cleanup

Six `iced-wgpu-patched/Cargo.toml` deprecated Clippy lint spellings: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. Full exact stderr retained in proof.json. Not warning-free; no suppression or adjacent lint fix.

## Manual Rust readability

Read/followed `<private-agentconfig>/skills/rust-readability/SKILL.md`. Reviewed changed producer, explicit 27-field record, input map/export, type25 feed insertion and query registration, plus changed Rust consumer assertion helper/calls; shared Rust test helper read as context. Archived source root:
`<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-tag-lifecycle-green-20261014T175706489371Z/source`.

No actionable violation found within changed scope. Named projection writers, explicit mutation, early nil return, short changed assertion helper; no new warning suppression. No automated metric/lint invocation, whole-repo audit or cleanup. Full inspected Rust texts and exact scoped locations retained in proof.json.

Reused prior source audits at `widget-subtext-retail-final-runtime-current/proof.json`: recorded empty production diff e37→62 and archived/committed source-text association. No new diff/hash command. Source identity is saved metadata/text association, not cryptographic authentication. Earlier runtime results are not counted as format/check proof.

## Proof boundary

Raw Cargo format success is **not** standalone macro-test formatting validation or all-source correctness proof. Default check is independent source compile evidence, not tests/all-targets/all-features. No PTR/native/parent/full-suite/runtime/behavior proof. No reruns, builds, tests, binary reads/hashes, delegation or canonical edits.

Sources:
- `<private-state>/verification/widget-discrete-progress-current/widget-subtext-final-format-20261014T175934640748Z`
- `<private-state>/verification/widget-discrete-progress-current/widget-subtext-final-default-check-20261014T180421302774Z`

`report.md` and `proof.json`: directory 0700, files 0600; fsynced and exact readback/mode checks performed.
