# Existing default check + full format audit

**PASS with six manifest deprecation warnings retained. No workload rerun.**

Source recorded: `a5397a2d60ebdd2dc813309671bfb26534e3621c`; production delta recorded: `05665ddad` (user shorthand 056). Artifact provenance only; no source-tree authentication or claim about later HEAD. Full admission, submission, revision, toolchain, stdout, stderr and terminal files read. Entire check stdout parsed, not sampled. Retained units independently observed with read-only `systemctl --user show`; both loaded, active/exited, Result=success, ExecMainCode=1, ExecMainStatus=0. Launcher exit 0 is recorded separately from unit exit 0.

## Whole format — PASS

`/usr/bin/env RUSTC=/usr/bin/rustc /usr/bin/cargo fmt --all -- --check`

Artifact: `<epochs>/crafted-quality-full-format-20261014T022047581793Z`

Unit elapsed: 61 seconds, October 13, 2026 21:20:48–21:21:49 CDT (October 14, 2026 02:20:48–02:21:49 UTC). Stdout 0 bytes; stderr 0 bytes; unit exit 0.

Actual MemoryPeak **552820736 bytes**, above soft MemoryHigh **536870912** by **15949824** bytes, below hard MemoryMax **1073741824** (1 GiB). CPUQuota 200%. Soft-high crossing is preserved, not reported as staying below both limits. No OOM/event-counter claim.

## Default Retail check — PASS with warnings

`/usr/bin/env RUSTC=/usr/bin/rustc /usr/bin/cargo check --offline --locked --lib --bins -j4 --message-format=json-render-diagnostics`

Artifact: `<epochs>/crafted-quality-default-check-20261014T022056044194Z`

Stdout **522804 bytes**, **733 JSON records**: 658 compiler-artifact, 74 build-script-executed, 1 build-finished. Final record exactly `{"reason":"build-finished","success":true}`. **0 compiler-message diagnostics**. Stderr **1274 bytes**: **six manifest deprecations**, one six-warning summary, Checking line, Finished line. Cargo elapsed **47.03s**; terminal timestamps span 47 seconds, October 13, 2026 21:20:56–21:21:43 CDT (October 14, 2026 02:20:56–02:21:43 UTC). Unit exit 0.

Actual MemoryPeak **5863276544 bytes**, below MemoryHigh **8589934592** (8 GiB) and MemoryMax **9663676416** (9 GiB). CPUQuota 400%; Cargo jobs 4.

Selected root targets: **1 library + 6 binaries**, all profile.test=false and fresh=false. A separate fresh root custom-build artifact is not counted as a seventh binary. Target test-capability metadata does not mean tests ran.

- `wow_ui_sim` (lib)
- `panel-visual-metrics` (bin)
- `bench_talents` (bin)
- `bench_steady_state` (bin)
- `bench_spellbook` (bin)
- `wow-sim` (bin)
- `wow-cli` (bin)

Default feature configuration exactly:

`aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, prefork-full-ui, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound`

## Check stderr — complete, verbatim

```text
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
warning: `iced_wgpu` (manifest) generated 6 warnings
    Checking wow-ui-sim v0.1.0 (<repo>)
    Finished `dev` profile [optimized + debuginfo] target(s) in 47.03s
```

## Toolchain and boundaries

```text
cargo 1.99.0 (5f94df478 2026-08-27) (Arch Linux rust 1:1.99.0-1)
rustc 1.99.0 (b940084d7 2026-09-28) (Arch Linux rust 1:1.99.0-1)
binary: rustc
commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4
commit-date: 2026-09-28
host: x86_64-unknown-linux-gnu
release: 1.99.0
LLVM version: 23.1.1
```

Both submissions used <repo>, agents.slice, RemainAfterExit=yes, KillMode=control-group, append stdout/stderr. Full admission snapshots, exact submitted argv, revision/status text, toolchain, actual observed unit properties, all seven input-file sizes/SHA-256 hashes per artifact, and full root-target records are preserved in proof.json. Admission recorded allowed=true for both; admission is not a claim of peak staying below soft high.

Excluded: tests, PTR/other feature configurations, native behavioral/render acceptance, parent-goal completion, linked binary/library/archive inspection, source-tree authentication. No build/check/fmt/test reruns, service mutations, delegation, Bash, cwd changes or repository edits. Only the two requested audit outputs are written.
