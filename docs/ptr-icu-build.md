# Optional PTR ICU4C build

The `retail-12-1-5` feature enables the native ICU4C boundary; `client-ptr`
selects that epoch. Default `client-retail`, `client-mists`, release aliases,
and the retail Docker image do **not** require ICU4C. Existing ICU4X Rust
algorithms remain separate from this native dependency.

The native contract belongs in [ICU native linking](specs/intl-native-linking.md).
This page covers provisioning and the independent
[PTR ICU4C workflow](../.github/workflows/ptr-icu.yml), not native WoW equivalence.

## Platforms

| Platform | Build provisioning | Link/runtime contract |
|---|---|---|
| Linux | `pkg-config` and ICU4C development package, e.g. Debian/Ubuntu `libicu-dev` | `icu-i18n` and `icu-uc` >=72; shared ICU libraries and matching data must be available at runtime |
| macOS | Homebrew `icu4c` and `pkg-config`; `PKG_CONFIG_PATH=$(brew --prefix icu4c)/lib/pkgconfig` | Shared Homebrew ICU libraries must remain available at their recorded install names; no portable app bundling is added here |
| Windows MSVC x64 | Pinned vcpkg manifest below; explicit `VCPKG_ROOT` and `VCPKGRS_TRIPLET=x64-windows-static-md` | Static ICU/data, dynamic MSVC CRT; no ICU DLL copying or bundling |

The CI library-test build uses `sound,gui,client-ptr` to cover the existing
GUI-enabled library configuration. Linux therefore also provisions
`libasound2-dev` and `libudev-dev`. These are not ICU dependencies.

Do not set `VCPKGRS_DYNAMIC`, use a different vcpkg triplet, or add
`-C target-feature=+crt-static` for this Windows configuration. Missing native
prerequisites must fail the PTR build, not select an alternate implementation.

## Reproducible Windows provisioning

[`vcpkg.json`](../vcpkg.json) pins the builtin registry to
`ef7dbf94b9198bc58f45951adcf1f041fcbc5ea0` (official tag `2025.06.13`).
The workflow reads that same field for the vcpkg checkout, avoiding two
independent version choices. Its official ICU port is **74.2, port revision 5**.

From the repository root in PowerShell, after checking out vcpkg at that commit:

```powershell
$env:VCPKG_ROOT = (Resolve-Path .ci-vcpkg).Path
$env:VCPKGRS_TRIPLET = 'x64-windows-static-md'
& "$env:VCPKG_ROOT/bootstrap-vcpkg.bat" -disableMetrics
& "$env:VCPKG_ROOT/vcpkg.exe" install --triplet x64-windows-static-md --host-triplet x64-windows-static-md "--x-manifest-root=$PWD" "--x-install-root=$env:VCPKG_ROOT/installed"
```

The explicit install root is important: the native Rust build searches
`VCPKG_ROOT/installed`, not an unrelated repository `vcpkg_installed` directory.
The host triplet is also static because the ICU port builds host tools.
Unix package-manager versions are intentionally not pinned by this manifest;
the workflow records their `pkg-config` versions. Formatting data may differ
across ICU releases and platforms.

Official pin sources:

- [vcpkg tag](https://github.com/microsoft/vcpkg/tree/ef7dbf94b9198bc58f45951adcf1f041fcbc5ea0)
- [ICU port metadata](https://github.com/microsoft/vcpkg/blob/ef7dbf94b9198bc58f45951adcf1f041fcbc5ea0/ports/icu/vcpkg.json)
- [Static-library/dynamic-CRT triplet](https://github.com/microsoft/vcpkg/blob/ef7dbf94b9198bc58f45951adcf1f041fcbc5ea0/triplets/x64-windows-static-md.cmake)
- [Manifest versioning](https://learn.microsoft.com/en-us/vcpkg/users/versioning)

## CI proof path

The separate workflow leaves the release, retail-test, Mists-test, and Docker
workflow matrices unchanged. It builds the existing grouped Rust library test
binary with:

```text
cargo test --lib --locked --no-default-features --features sound,gui,client-ptr --no-run --message-format=json
```

[`run_ptr_icu_tests.py`](../.github/scripts/run_ptr_icu_tests.py) selects that
binary from Cargo's artifact records, rejects an empty `intl_native::tests::`
filter, and executes only those wrapper tests with a 90-second runtime limit.
Compilation is separate from that limit. No WoW installation is required by
these native-wrapper tests.

The same script records `ldd` on Linux, `otool -L` on macOS, or MSVC
`dumpbin /DEPENDENTS` on Windows. Unix checks require resolvable ICU imports;
Windows rejects any ICU DLL import. Executed wrapper tests provide the native
link/call smoke rather than merely accepting a successful build with zero tests.
Cargo artifact records, linkage output, and test output are uploaded per platform.

Docker only gains `COPY build/` and `COPY native/`, so the feature-gated build
helper and C inputs are present in its context. The build still selects
`client-retail`; no ICU package, image runtime dependency, or PTR image is added.

## Runtime notices and evidence limits

A future distributed PTR binary that incorporates ICU must carry the matching
ICU copyright/license and third-party notices. For the pinned Windows port,
vcpkg installs these in
`$VCPKG_ROOT/installed/x64-windows-static-md/share/icu/copyright` from the
[ICU 74.2 LICENSE](https://github.com/unicode-org/icu/blob/release-74-2/LICENSE).
That file includes Unicode License V3 and additional notices; retain the full
installed file, not only a short license label. No release package is produced
by this workflow. Existing non-PTR packages are unchanged.

Local packaging preflight, 2026-09-10:

| Check | Evidence in this slice |
|---|---|
| Workflow/script/manifest syntax | `actionlint .github/workflows/ptr-icu.yml`, Python `compile()`, JSON and YAML parsing passed |
| Smoke-runner preflight | Synthetic Cargo artifact selection and process-output fixture passed; missing artifact and zero matching tests rejected. This is not native ICU proof. |
| vcpkg pin | Official tag resolves to the manifest SHA; that revision's `versions/baseline.json` records ICU `74.2#5` |
| Existing packaging | Release/test/Docker workflows, xtask, features and runtime image configuration unchanged; only Docker source inputs added |
| Platform execution | Linux, macOS and Windows native builds/linkage/runtime smoke **not run by this slice**; no local package installation |

Windows static linkage is a CI requirement, not a verified local result.
Parent integration owns platform jobs and final acceptance; inspect successful
job output before claiming platform support is proven. PowerShell was not
available locally, so provisioning commands have only workflow syntax proof.
