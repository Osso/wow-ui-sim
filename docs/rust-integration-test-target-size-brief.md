# Rust Integration Test Target Size Brief

Status: historical target fan-out fixed in `02f1d5a6 Consolidate integration test harness`; dev-only helper extracted at `256e54ef1`, bounded saved compile/list/selected-case and timestamp-cache proof admitted; parent OPEN.

## Root Cause

The historical large fresh `target/` directories were caused by Rust integration-test target
fan-out, not by incremental compilation or stale build accumulation.

Cargo treats every top-level `tests/*.rs` file as a separate integration-test
crate. Before consolidation, this repository exposed hundreds of those top-level test
targets, so a full `cargo test` or `cargo test --no-run` linked hundreds of
standalone executables. Each executable pulls in the simulator library plus the
large GUI, CASC, rendering, and Lua/runtime dependency graph.

Observed in `../wow-ui-sim/.claude/worktrees/patient-swooping-pebble`:

- `target/debug`: about 201 GiB
- `target/debug/deps`: about 203 GiB
- `target/debug/incremental`: 0 bytes
- extensionless ELF executables in `target/debug/deps`: 925
- total size of those executables: about 200.6 GiB
- average executable size: about 222 MiB
- Cargo metadata test targets for `wow-ui-sim`: 936
- built executable names matching Cargo test targets: 916

So the dominant cost is roughly:

```text
hundreds of integration-test binaries * ~220 MiB each = ~200 GiB target
```

## Why This Happens

This is normal Cargo behavior, but it is the wrong test layout at this scale.

The pre-consolidation shape was effectively:

```text
tests/a.rs
tests/b.rs
tests/c.rs
...
```

Cargo links `a`, `b`, `c`, and every other top-level file as separate binaries.
That is fine for a small number of integration tests. With hundreds of files it
turns linking and disk usage into the main cost.

Debug information is a secondary multiplier. One representative test binary was
about 225 MiB and became about 113 MiB after stripping debug sections, so debug
data accounts for a large fraction of each binary. Even stripped, however,
hundreds of standalone binaries would still consume excessive space.

## Preferred Test Layout

Historical consolidation implementation (not source proof of the approved generation move):

- `Cargo.toml` sets `autotests = false`.
- `Cargo.toml` declares the custom `frame_positions` target and one
  consolidated `integration` test harness.
- `tests/integration.rs` includes a build-script-generated module list.
- `build.rs` discovers top-level `tests/*.rs` files and emits module
  declarations into `OUT_DIR/integration_tests.rs`.
- Existing test files remain in place, but Cargo no longer treats each one as a
  separate linked test executable.

Verified after the change:

- `cargo metadata --no-deps --format-version 1` reports 2 test targets:
  `frame_positions` and `integration`.
- `cargo test --test integration --no-run` links one integration test binary.
- Removing obsolete linked test executables reduced the local `target/` from
  37 GiB to about 4 GiB.

Consolidate top-level integration tests into a small number of harness crates,
with the existing tests moved under module directories.

Example:

```text
tests/
  integration.rs
  integration/
    lua_api/
    layout/
    rendering/
    casc/
    blizzard_ui/
```

`tests/integration.rs` can declare:

```rust
mod integration;
```

Rust's test harness will still discover `#[test]` functions inside those
modules, but Cargo links one integration-test binary instead of one binary per
file.

A practical split for this repository is probably a small set of harnesses, not
one file per test and not necessarily one giant harness:

- `tests/lua_api.rs`
- `tests/rendering.rs`
- `tests/layout.rs`
- `tests/casc.rs`
- `tests/blizzard_ui.rs`
- `tests/perf.rs` for expensive or ignored performance tests
- one harness per client-profile lane when feature gating requires it

The first priority is reducing hundreds of top-level test crates to single-digit
or low-double-digit harness crates.

### Bounded Target Policy

Do not restore one Cargo target per test file. The previous layout generated
about 916 standalone test executables averaging roughly 222 MiB each and grew a
fresh target directory to about 201 GiB. Every new `[[test]]` entry therefore
requires target-count review.

Use a small, bounded set of grouped harnesses by subsystem or incompatible
client-profile lane. Standalone targets are justified only when they isolate a
real feature/profile boundary or a frequently edited workflow from the giant
integration harness. They must accumulate related tests rather than fan out one
target per symbol or source file.

`patch_12_1_audit` is the PTR-only grouped target for 12.1 audit work. Its full,
unfiltered command must remain at or below 60 seconds on the development host:

```bash
cargo test --test patch_12_1_audit \
  --no-default-features --features sound,gui,casc,client-ptr
```

Keep that budget by batching related assertions into lifecycle/family tests and
reusing each loaded UI environment within the family. Do not create one
full-UI startup per audited symbol. If the full target approaches 60 seconds,
optimize fixture sharing or grouping before adding another target.

## Compile Selection Findings

Official [Cargo test target-selection documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
says selecting integration tests or benchmarks
automatically builds binary targets. The [Cargo target reference](https://doc.rust-lang.org/cargo/reference/cargo-targets.html)
distinguishes `test = false` disabling default testing from disabling normal
binary builds. Sources verified by main on 2026-10-11 UTC (October 10 local);
not fetched again for this update.

The [existing jobs12 evidence](remote-builds.md#required-unit2-builds--measured-cgroup-cpu-window)
and [seven-profile receipts](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/unit2-jobs12-2026-10-10/final-report.md)
retain unchanged compile selection: each saved unit2 compile built six
production binaries, the library, and integration target. Neither worker nor
helper supplied `--bin` selectors. A direct integration-test consumer of
`wow-sim` is established; automatic compilation of the other binaries does not
establish test invocation of them. Tests do invoke `wow-sim`.

Main preserved integration, prefork, and library acceptance scopes, plus distinct
normal-startup proof. Existing per-command CPU/wall measurements cannot allocate
per-binary link cost. No supported CLI exclusion mechanism was established from
these docs; no flags, manifest, or product changes were applied, and no speedup
is claimed. The seven-profile receipts remain whole-command measurements, not
per-target attribution.

### Approved generation boundary — 2026-10-11

The [harness contract](specs/prefork-test-harness.md#dev-only-automatic-harness-generation-approved-2026-10-11-verification-pending) assigns automatic integration/prefork generation to an internal dev-only crate and removes test generation/watches from the root production build script. Discovery, cfg, original constructors/assertions and CLI case identities must remain unchanged. The change targets test-input cache invalidation, not Cargo target selection: the underlying six production binaries remain selected for integration builds; no unsupported exclusion flag is proposed.

Source ownership is confirmed at `256e54ef1`: unpublished workspace crate `crates/test-harness` is a root dev-dependency; its build script owns discovery/watches, while root `build.rs` retains production responsibilities. Macros expand at both integration/prefork item sites and the Retail registry expression.

[Retained evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/dev-only-harness-2026-10-11/README.md) separates compile/list parity, 13 selected cases, two original prefork bodies and generated-inner-byte preservation from the unchanged-content timestamp trial. That trial keeps the root library and six binaries 7/7 fresh; only helper library and integration are nonfresh, versus eight root artifacts before extraction. Helper RunCustomBuild is dirty while its compiled build-script artifact stays fresh.

Changed revisions and artifact graphs preclude normalized speedup. Cgroup CPU/wall ratios do not establish saturation, per-binary cost or native-host acceptance. One temporary new top-level marker is discovered, executed and removed with baseline registries restored; production artifacts stay fresh. Historical Mists cast-bar RED, full-suite/native gaps and parent handoff remain open.

## Worktree Target Strategy

Per-worktree `target/` directories avoid Cargo lock contention but multiply this
problem by every active agent worktree. A single shared `CARGO_TARGET_DIR` saves
disk, but parallel agents can block on Cargo's target lock.

The workable compromise is a small pool of shared target directories, assigned
per agent or per task lane:

```text
/syncthing/Sync/Projects/wow/.cargo-targets/wow-ui-sim-a
/syncthing/Sync/Projects/wow/.cargo-targets/wow-ui-sim-b
/syncthing/Sync/Projects/wow/.cargo-targets/wow-ui-sim-c
/syncthing/Sync/Projects/wow/.cargo-targets/wow-ui-sim-d
```

That caps disk growth while reducing lock contention compared with one global
target directory. It is still a mitigation. The root fix is consolidating the
integration-test crates.

## Command Guidance

Avoid broad full-target test builds in disposable worktrees unless they are
really needed:

```bash
cargo test specific_test_name
cargo test --test lua_api specific_test_name
cargo build --bin wow-sim
```

Avoid treating `target/debug/incremental` as the default suspect for large fresh
targets in this repository. Check `target/debug/deps` and count linked test
executables first.
