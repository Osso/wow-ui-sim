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

Shared snapshot, selection, and native execution helpers live in `/syncthing/Sync/Projects/world-of-osso/game-engine/scripts/`. Set `BUILD_HOST_SCRIPTS` to override that dependency. Desktop requires native Rust 1.98.1 and same-ABI native runtime dependencies; there is no Docker runtime or bundled ICU substitution. Existing CI/release Docker pipelines remain separate and unchanged, including the [standalone PTR ICU workflow](ptr-icu-build.md).

## Source, cache, and lifetime

Desktop source and Cargo targets persist under `/home/osso-test/.cache/native-builds/<key>/`. Snapshots include current working edits and selected owned compile/runtime inputs, not only committed source. Previously supplied files are pruned without deleting app-owned runtime directories; content changes receive fresh mtimes, including A/B/A changes. Builds do not hold a runtime lock for the app's lifetime.

Runtime Blizzard caches remain host-local and profile-scoped under `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`; desktop resolves that home as `osso-test`. Host selection does not populate or migrate those caches. SSH leases control remote cleanup. Source SCP has a 60-second attempt limit and at most three transient attempts; permanent failures fail immediately.

## Evidence and pending runtime proof

Reported main-session evidence on October 3, 2026: default native GUI binary build completed in 4m9s; default CLI `--no-addons --no-saved-vars lua-errors` exited 0 with `CLEAN` and zero errors (`target/native-desktop-lua-errors.log`). These are scoped build/CLI observations, not full parity or warning-free acceptance.

Normal visible Windows GUI launch remains unproven. WSLg launched through SSH encountered a Session 0 problem; investigation remains pending. Historical focused-test/check reports are not a fresh current-tree gate.

## Related

- [Build-host contract](specs/build-host.md)
- [Agent commands](../AGENTS.md#development-buildrun)
