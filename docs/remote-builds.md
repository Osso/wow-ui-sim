# Native desktop/local development

Normal builds **and runs** use `scripts/build-host.py`. `desktop` means native Rust and runtime in desktop WSL as `osso-test`; `local` means both on this machine. Choose local while gaming on desktop. Raw `cargo` commands remain local.

## Commands

```sh
python3 scripts/build-host.py --run
python3 scripts/build-host.py --build-host local --run
python3 scripts/build-host.py --bin wow-cli --run -- --no-addons --no-saved-vars lua-errors
python3 scripts/build-host.py --check
python3 scripts/build-host.py --test --test integration FILTER -- --nocapture
python3 scripts/build-host.py --no-default-features --features sound,gui,casc,client-ptr --check
python3 scripts/build-host.py --no-default-features --features sound,gui,casc,client-ptr --test --test integration FILTER -- --nocapture
```

`--test` is the last helper option: everything after it goes to Cargo, including test filters and the test-executable separator. `--check` checks the selected binary; `--bin wow-cli` selects the CLI. Alternate profiles disable default features. `--release` selects release output; `--no-build --run` runs an existing selected-host binary without Cargo.

```sh
scripts/start-profile.sh live
scripts/start-profile.sh --build-host local ptr
scripts/start-profile.sh all
```

The Python profile launcher preserves `live`/`retail`, `ptr`, `mists`, and `all`, selecting profile-specific Cargo features. `WOW_SIM_START_RELEASE=1`, `WOW_SIM_START_FOREGROUND=1` (single profile only), and `WOW_SIM_START_NO_BUILD=1` retain their meanings. Background logs and helper PIDs appear in `target/profile-runs/`; a printed PID is not GUI readiness proof.

## Host selection and dependencies

`python3 scripts/build-host.py --save-build-host desktop` saves the shared default in `~/.config/game-engine/build-host`; use `local` to change it. The configured default is desktop. Explicit `--build-host` overrides it for one invocation. Host failure is an error, never a host fallback.

Shared snapshot, selection, and native execution helpers live in `/syncthing/Sync/Projects/world-of-osso/game-engine/scripts/`. Set `BUILD_HOST_SCRIPTS` to override that dependency. Desktop uses pinned Rust 1.98.1 through rustup and same-ABI native runtime dependencies; local uses installed Arch Cargo/rustc 1.98.1 without rustup, not fallback; there is no Docker runtime or bundled ICU substitution. Existing CI/release Docker pipelines remain separate and unchanged, including the [standalone PTR ICU workflow](ptr-icu-build.md).

## Source, cache, and lifetime

Desktop source and Cargo targets persist under `/home/osso-test/.cache/native-builds/<key>/`. Snapshots include current working edits and selected owned compile/runtime inputs, not only committed source. Previously supplied files are pruned without deleting app-owned runtime directories; content changes receive fresh mtimes, including A/B/A changes. Builds do not hold a runtime lock for the app's lifetime.

Runtime Blizzard caches remain host-local and profile-scoped under `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`; desktop resolves that home as `osso-test`. Host selection does not populate or migrate those caches. SSH leases control remote cleanup. Source SCP has a 60-second attempt limit and at most three transient attempts; permanent failures fail immediately.

## Evidence and pending runtime proof

Main-observed evidence on October 3, 2026: actual native desktop GUI binary build passed (4m9s); headless CLI startup `--no-addons --no-saved-vars lua-errors` returned `[]`, exit 0 (`target/native-desktop-lua-errors.log`). These are bounded build/CLI observations, not full parity or warning-free acceptance.

Normal desktop GUI failed before its first frame: `target/native-desktop-gui-long.log` ends with connection reset (101) while creating the Wayland event loop. Engine `target/native-wslg-boundary.log` records the correct `/run/user/1000/wayland-0` symlink to `/mnt/wslg/runtime-dir/wayland-0`, repeated Weston SIG11 in `stderr.log`, and repeated approximately 102-second RDP peer disconnect/restart cycles. Windows msrdc ran in Session 0 while the active console was Session 1. Host compositor crash is observed; crash root cause and session causality are not established. This is not a simulator Lua/CASC/font stall. No WSL, service, or process restart was performed.

The actual local `wow-cli` helper built and printed `--help` in 0.35 seconds: bounded local CLI proof, not GUI acceptance or a local server pass. Local capability remains supported; actual local runtime acceptance is deferred, not required now for the current gate. User rejected local server build attempts; actual local server proof is deferred and its no-local-compile guard remains. Corrected source gate passed: two repaired fixtures, zero Ruff diagnostics across 14 files, and pycompile for nine changed files. The prior 94 passing fixtures retain revision-scoped credit; this is not a fresh 96-test run. Evidence: engine `target/native-migration-corrected-verify.md`. Historical focused-test/check reports are not a fresh current-tree gate. Normal desktop GUI remains unresolved; source-gate success would not complete the whole workflow.

## Asynchronous full suite

`python3 tools/full_suite.py submit [REF]` schedules the detached worker;
`python3 tools/full_suite.py status [REF]` reads stored results. The worker keeps
its existing suite lock and dedicated checkout/results under `~/Projects/wow/`.
Each actual Cargo step runs through the required installed
`/home/osso/.worktrees/build-lock.sh`, sharing the builder lock with sibling
builds for the entire child lifetime, including unsuccessful exits. Locking the
submit launcher alone does not coordinate detached workloads. A missing wrapper
fails explicitly; there is no unlocked execution path.

Cargo steps use `--offline --locked` and `CARGO_BUILD_JOBS=4`; dependencies must
already be cached and the lockfile current. `FULL_SUITE_JOBS` controls nextest
test workers only (default 16), independently of compilation jobs. Nonzero Cargo
exits and parsed test failures remain in the stored JSON and log, and later
steps still run. Submission/status behavior is unchanged.

Bounded worker regression tests (temporary executable fixtures and a real flock,
no actual builds or systemd): `python3 -m unittest tools.test_full_suite -v`.
Changes to the tracked runner do not update `/home/osso/bin/full-suite`;
installation is a separate deployment step.

## Related

- [Build-host contract](specs/build-host.md)
- [Agent commands](../AGENTS.md#development-buildrun)
