# Integration binary selection

`Cargo.toml` separates the simulator used by tests from opt-in utility binaries.
See the [source investigation](../wiki/investigations/compile-bin-lockwatch.md).

## What it must do

- [ ] `cargo test --test integration --no-run` without `tools` must retain `wow-sim` and omit the other five production binaries.
- [ ] Existing integration and prefork cases, library tests, client profiles, and their runtime behavior must remain unchanged.
- [ ] `--features tools` must enable explicit utility builds; existing GUI requirements remain in force.
- [ ] Release aliases, native `--bin wow-cli` builds, and checked-in utility build scripts must enable `tools`, including release feature overrides.
- [ ] Docker must still build its headless `wow-sim`; full-suite must keep all existing execution stages without enabling unused tools.

## How it works

- [Source investigation and proof boundary](../wiki/investigations/compile-bin-lockwatch.md)
- [Native and full-suite workflow](../remote-builds.md)

## Retail screen-clamp q3

`autotests = false` disables Cargo's implicit test targets, not the
`integration` target's module discovery. The test-harness build script discovers
`tests/screen_clamp_q3.rs` once as an integration module. The optional-module
list enables it only with `optional-tests`; the source also requires
`client-retail`. Its eight ordinary tests run in the `integration` binary.
The `prefork_full_ui_case!` marker separately registers the single original
GameTooltip case for `prefork_full_ui`; it is not a libtest case. Do not add
a second standalone target for the same source file.

On Linux, enable `client-retail` and `optional-tests` and check the lists
before interpreting GREEN or RED. Expect eight matches under `integration`
and one under `prefork_full_ui`:

```sh
cargo test --no-default-features --features sound,gui,casc,client-retail,optional-tests --test integration -- screen_clamp_q3:: --list
cargo test --no-default-features --features sound,gui,casc,client-retail,optional-tests --test prefork_full_ui -- --list screen_clamp_q3::
```

## Implementation inventory

- `Cargo.toml`: five utility target gates; dependency-free `tools` feature.
- `xtask/src/main.rs`: utility admission for both release binaries.
- `scripts/build-host.py`: utility admission for native `wow-cli` selection.
- `scripts/setup-blizzard-ui.{sh,ps1}`, `scripts/test-retail-casc-isolation.py`, and Mists utility build scripts: explicit utility admission.

## Tests asserting this spec

- `scripts/tests/test_build_host.py`: plain CLI build and explicit build/check feature propagation; not executed in this static-only change.
- Existing `tests/` CLI consumers retain `wow-sim`; no cases are removed or ignored.
- Deferred slot acceptance: Cargo JSON production artifact selection and explicit release/Docker/full-suite feature-set builds. No fresh build proof yet.

## Known gaps (current cycle)

- [ ] Execute the handoff slot recipe; measure `CPUUsageNSec`, not wall time.
- [ ] Run build-host regression tests and build/check acceptance after static delivery.

## Out of scope

Moving or disabling CLI-dependent tests, changing dependency/compiler profiles, and claiming CPU savings without a controlled measurement.
