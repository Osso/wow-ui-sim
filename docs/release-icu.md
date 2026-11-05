# Release ICU packaging

Current Retail builds require native ICU even without GUI/CASC. Mists does not.
The native API contract lives in [native Intl linking](specs/intl-native-linking.md);
[PTR provisioning](ptr-icu-build.md) owns the shared pinned Windows manifest.

## Native releases

[Release workflow](../.github/workflows/release.yml) builds Retail and Forever
for Linux, Windows and macOS, and Mists for Linux and Windows only. These builds
use the `cargo release-*` aliases:
`--release --no-default-features --features sound,gui,client-<profile>`, explicit
platform target, both `wow-sim` and `wow-cli`, CI Cargo configuration.

- Linux explicitly installs `pkg-config`, `libicu-dev`, ALSA and udev headers.
  Retail reports both ICU package versions and requires >=72. Tarballs retain
  shared linkage: recipient hosts need compatible glibc, GUI/audio dependencies
  and the exact ICU SONAMEs recorded by `ldd`; this is not universal portability.
- Windows Retail checks out the vcpkg revision from `vcpkg.json`, bootstraps it,
  installs `x64-windows-static-md` into that checkout's `installed/`, and exports
  `VCPKG_ROOT` plus `VCPKGRS_TRIPLET`. ICU/data are static; MSVC CRT remains dynamic.
- macOS Retail provisions Homebrew ICU. Before signing,
  [bundle-macos-icu.py](../scripts/bundle-macos-icu.py) follows actual ICU imports,
  copies the transitive ICU dylib closure into `Contents/Frameworks`, rewrites
  imports/IDs to bundle-relative paths, and rejects unresolved non-system imports.
  Sign dylibs before the outer app, verify the signature, then launch the packaged
  executable with `--version`. Mists has no macOS release package.

## Docker

[Dockerfile](../Dockerfile) builds headless Retail with Debian Bookworm ICU
headers. It copies the builder's dereferenced `libicui18n.so.72`, `libicuuc.so.72`
and `libicudata.so.72` into the amd64 distroless Debian12 runtime. Changing the
architecture or distro requires re-proving this explicit runtime closure.

The UI source stage checks out Gethe revision
`b9be9379719047977fca4dba1bb1d9988e8840b3` (selected Retail `12.1.5.70448`),
compares its complete AddOns path set against the committed
[Retail manifest](../data/blizzard-ui-files/retail.txt), and checks every file
before stamping completion/provenance. Provenance records the revision and
manifest SHA256. It does not claim CASC origin. Future profile updates must
move the source revision and manifest together; mismatches fail the image build.

[Docker workflow](../.github/workflows/docker.yml) loads one local image,
then [verify_docker_image.py](../.github/scripts/verify_docker_image.py) checks
`--version`, exact `[]` startup JSON, and two executed tiny-addon tests (Retail
interface epoch and frame-size round-trip). Containers have no network and a
90-second command limit. Zero tests cannot pass. Publication tags/pushes that
same tested image, never a second build. This is not Wowless or general addon
compatibility proof.

## Non-publishing rehearsal

On a remotely available implementation branch, with these workflow definitions
registered on the repository's default branch:

```text
gh workflow run release.yml --repo Osso/wow-ui-sim --ref release-icu-fixes
gh workflow run docker.yml --repo Osso/wow-ui-sim --ref release-icu-fixes
```

Neither workflow needs inputs. Every `workflow_dispatch`, including one on a
tag ref, is non-publishing. Release metadata uses `v0.0.0-ci`; the entire release
publisher requires a `push` event on `refs/tags/v*`. Docker login/push require a
`push` event on a tag. Build/artifact/cache uploads remain allowed. Do not create
a tag to rehearse; if registration blocks dispatch, resolve registration in a
separately authorized repository/fork rather than bypassing these gates.

## Local acceptance — 2026-10-18

Native build/runtime inputs at `6dcc58e5d`, based on `29fac7076`; later
proof-documentation edits do not change those inputs. Builds retain the source
version `0.1.0`; CI rehearsal separately exercises `0.0.0-ci` version/package metadata.

| Target | Executed proof | Result |
|---|---|---|
| Linux | CI Cargo config, `cargo release-linux -j1`, system ICU78.3, 9GiB build limit; copied executable in a 4GiB smoke unit with 90-second child timeout; `ldd` | Build exit0 (48m57s), all three ICU78 imports resolved, startup exit0 / `[]`; persisted CVars stat/hash unchanged |
| Docker amd64 | Actual Dockerfile with committed Cargo config; Rust1.92/Bookworm ICU72; offline version/startup/two-addon-test verifier; exported filesystem and in-image loader inspection | Build/verifier/inspection exit0; two tests passed; all4026 manifest paths, pinned revision/digest and three ICU72 libraries confirmed; test image removed |
| Native Windows x64 | Rust1.98.1/MSVC2022; pinned vcpkg ICU74.2#5 static-md; `cargo release-windows -j2`; `dumpbin`; 90-second headless startup limit | Build exit0 (23m49s), version launch succeeds, no ICU DLL imports, startup exit0 / `[]` after native CASC cache refresh; persisted CVars stat/hash unchanged |
| macOS | No local execution | CI dispatch required; not proven |

Windows first rejected an incomplete old profile cache; an error dialog then
exceeded the smoke timeout. Backing up that profile and using the built
`wow-cli casc sync-blizzard-ui` against the installed game extracted all4026
files with matching Retail12.1.5.70448 provenance. The refreshed cache and sibling
`retail.bak-20261018` were retained; large scratch build outputs were removed.
A PowerShell5 proof harness initially lost the child exit code; retaining the
native process handle established the final explicit exit0, not merely `[]` output.

Windows cache isolation is a separate finding: cache selection uses
`dirs::cache_dir()`, whose Windows implementation calls `SHGetKnownFolderPath`;
no simulator cache-root override is exposed. No environment workaround or
new override was implemented. Existing warnings remain: seven in headless Docker,
three in native Windows; none were suppressed or repaired in unrelated Rust code.

## Proof limits

Local acceptance must separately record actual Linux release compilation,
resolved `ldd` imports and isolated headless startup; final Docker image startup,
addon count, filesystem ICU libraries and manifest/provenance; and native Windows
vcpkg build, release build, `dumpbin /DEPENDENTS` and bounded headless startup.
[Helper tests](../scripts/tests/test_release_helpers.py) exercise rejection and
synthetic closure behavior, not native loaders.

macOS is verified only by CI dispatch, not locally. Builder-host launch is not
clean-recipient portability proof; clean macOS execution without Homebrew ICU
remains separate. Existing ad-hoc signing is not notarization. Distributed ICU
requires its matching full license/third-party notices; provisioning libraries
alone is not a redistribution-notice audit.
