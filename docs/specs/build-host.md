# Native build host

`scripts/build-host.py` selects native desktop/local compilation and execution. Operational details and evidence live in the [workflow guide](../remote-builds.md).

## What it must do

- [ ] Explicit host overrides the shared saved default; failures never fall back to another host. Raw Cargo stays local.
- [ ] Build, check, test, and run preserve binary/profile/features/release selection, arguments, exit status, and Cargo test filters/separators; failed builds do not run.
- [ ] `--no-build --run` uses an existing selected-host binary without compiling; missing binaries fail explicitly.
- [ ] Desktop snapshots preserve working inputs, prune only supplied files, and rebuild on content changes; running apps do not block independent builds.
- [ ] Profile launches preserve live/PTR/Mists/all, foreground/release/no-build options, and host-local profile caches; SSH lifetime controls remote cleanup.

## How it works

- [Native development guide](../remote-builds.md)

## Implementation inventory

- `scripts/build-host.py` — source snapshot and native adapter.
- `scripts/start-profile.sh` — Python profile launcher.
- Canonical game-engine `scripts/native_build_hosts.py`, `depot-build.py`, `build_hosts.py` — native execution, snapshots, and shared host selection; location overridable with `BUILD_HOST_SCRIPTS`.

## Tests asserting this spec

- `scripts/tests/test_build_host.py` — selection, source inputs, mode/argument/status behavior, no-build, and running-app boundary.
- `scripts/tests/test_start_profile.py` — foreground/background profile routing and launch options.
- Shared game-engine native-runner tests cover transport/lifetime and source synchronization separately.

Tests are inventoried, not rerun by this documentation audit; requirements remain unchecked rather than borrowing historical reports as a fresh gate.

## Known gaps (current cycle)

- [ ] Normal visible Windows GUI proof: SSH/WSLg Session 0 investigation pending; see guide evidence limits.

## Out of scope

CI/release Docker changes, distro migration, ICU bundles, host fallback, and full client parity certification.
