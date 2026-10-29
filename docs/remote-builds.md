# Native desktop/local development

Normal builds **and runs** use `scripts/build-host.py`. `desktop` means native Rust and runtime in desktop WSL as `osso-test`; `local` means both on this machine. Choose local while gaming on desktop. Raw `cargo` commands remain local.

## Commands

```sh
python3 scripts/build-host.py --run
python3 scripts/build-host.py --build-host local --run
python3 scripts/build-host.py --bin wow-cli --features tools --run -- --no-addons --no-saved-vars lua-errors
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

## Local slot builds

Use `scripts/slot-build.py` for shared-slot Cargo builds. Valid slots are 0, 1 and 3;
slot 2 was retired on 2026-10-27 and both `slot-build.py --slot` and
`full-suite --target-dir` reject it. The Cargo command must pass
`--message-format=json` before any `--`. Without it, artifact certification has no
compiler-artifact records, so `slot-build.py` rejects the request before launch. It bind-mounts the lane
worktree at `~/.cache/wow-ui-sim-srcmount/slot-N` inside an unprivileged
user service, paired with target `~/.cache/wow-ui-sim-targets/slot-N` and
`CARGO_INCREMENTAL=1`. Incremental reuse depends on that fixed BindPaths source
path per persistent target slot; do not build through changing worktree paths.
The stable-path experiment
(`~/.local/state/wow-ui-sim/handoff/compile-throughput-20261025/stablepath-experiment.md`)
measured lower touch/edit CPU with incremental enabled. Its single-slot cache
size is not a long-run growth bound.

Coordinate slot ownership before launch. For new claims, the claimant writes
`slot-N/.owner` with mode `0600`, containing exactly the `--owner` string, without
a newline; remove it on release. Existing JSON markers remain unchanged for
running lanes. This helper checks ownership; it does not claim or release slots.
It checks `capacity.sh` once with `CAPACITY_ID` from `--capacity-id` and refuses
anything except a literal `ADD` verdict. It does not poll for admission.

```sh
scripts/slot-build.py --slot 3 --worktree "$HOME/.worktrees/wow-ui-sim-lane" --unit lane-build --owner lane-owner --capacity-id lane-capacity -- /usr/bin/cargo test --test integration --no-run
scripts/slot-build.py --status lane-build
```

Launch prints the unit name without waiting for the build. The retained service
runs in `agents.slice` with `MemoryMax=9G` and `MemorySwapMax=0`.
`RUSTC_WRAPPER` is set to an empty value so Cargo configuration cannot select
sccache instead of the incremental compiler. Status prints
`SubState`, `Result`, `CPUUsageNSec`, and `WallTimeSec` (elapsed while running).
Use the helper from the checkout that contains it; it builds `--worktree`.

### Slot size limit

Before Cargo starts, the worker checks `.owner` and acquires each existing
Cargo build profile's `.cargo-lock` non-blockingly. A held lock refuses pruning
with `slot build lock busy` and the lock path. Locks remain held from inventory
through deletion, so Cargo cannot build in those profiles during pruning.
The worker prunes when allocated disk usage exceeds `--slot-cap-gib` (default 130 GiB; user decision 2026-10-29, one
integration-target slot uses 100-120 GiB), stopping below 80% of the cap
(default 104 GiB). Units with `deps` or `incremental` members go first, then
fingerprint/build-only units; each group is ordered by newest member mtime. Fingerprints,
matching deps files, build directories, and incremental sessions are removed
together per name/hash within each build-profile directory. Fingerprint
package/target aliases are joined only at the same hash. Independently hashed
incremental directories are separate cache units; different revisions are not
coalesced. Same-hash library sidecars, including `.so.dwp`, belong to the
library's unit; a recent sidecar protects every member.

Units whose outputs Cargo reported to the build in `.verified-source-identity.json`
or to any build in `receipts/*.json` that finished (`finished_ns`) within 7 days (by path or hardlink identity, including receipt paths
outside the slot) or modified within
`--prune-recent-hours` (default 2) are retained. Protected units and unclassified
files can prevent reaching the cap; the summary reports the remaining bytes. Allocated
bytes count hard links once; predicted reclaimed bytes exclude links retained
outside the unit. Each removal prints a JSON manifest with paths and bytes to
the service log. Worker pruning diagnostics use stderr, leaving stdout byte-exact
for the launched command. Standalone `--prune-dry-run` reports use stdout.
Receipts and source-identity files are never removed. Receipts list every
Cargo-reported output (fresh or rebuilt) and `build_script_out_dirs`; Cargo does
not touch fresh units, so mtime alone would rank a build's live dependencies and
build-script outputs as the oldest units. The verified `artifacts` hash map is
cumulative across builds and does not protect anything.

Read-only inspection does not require ownership or launch a build:

```sh
scripts/slot-build.py --slot 3 --prune-dry-run
```

It prints the same removal plan and before/remaining byte totals without deleting
anything. Inspecting an active slot gives a moving snapshot. Actual pruning
requires the existing exclusive ownership claim and all profile locks; do not
share a claim between concurrent builds. No `cargo clean` is used.

### Source freshness and receipts

The fixed source mount hides worktree changes from Cargo's path identity. Older
file mtimes can therefore reuse another tree's artifacts, even with incremental
compilation disabled. The helper records HEAD, a binary dirty-diff hash (including
untracked file contents), dirty paths, and per-target artifact hashes in
`slot-N/.verified-source-identity.json` only after the child exits zero and the
guard accepts. The separate `slot-N/.source-identity.json` records attempts,
including building, failed, and rejected runs; it never supplies the content or
artifact baseline. Existing attempt-only slots have no verified baseline and
receive the conservative unknown-source checks.
Before Cargo starts, a changed verified identity refreshes mtimes for the
HEAD-to-HEAD diff and both trees' dirty/untracked paths without changing bytes.
An unknown baseline or unsuccessful previous attempt derives input roots from
offline `cargo metadata --no-deps`:
package manifest directories and path dependency directories, plus root
`build.rs`, `Cargo.toml`, and `Cargo.lock`. All tracked files under those roots
are refreshed. This repository's root package also covers build-script inputs
in `build/`, `installer/`, `native/intl/`, and `tests/`. If metadata fails, all
tracked files are refreshed. A root package therefore also refreshes its docs;
this conservative full refresh is needed when prior source is unknown or the
previous build did not succeed.

Each unit writes `slot-N/receipts/UNIT.json` with its source identity, changed
paths (`changed_paths`, the mtime refresh set), content changes
(`content_changed_paths`), command, result and artifact SHA256 values. The command arguments and
stdout/stderr bytes are passed through unchanged, including wrappers that capture
Cargo JSON internally. The worker reads visible stdout for this build's Cargo
`compiler-artifact` records, without changing the submitted arguments. Use Cargo's
JSON message format to expose those records. Receipts retain the records and each
package's metadata-derived manifest-parent and path-dependency roots.

Every repo-owned compiler-artifact record is judged on its own inputs, including
libraries, binaries and tests. Ownership comes from `manifest_path` or target
`src_path` under the worktree, fixed source mount or utility mount. Registry and
Git package sources (`registry+`/`git+`) are external and excluded. Records without
ownership paths cannot prove they are external and are checked.

The worker snapshots Cargo `.d` dep-info before the child starts and reads it
again after the child exits.
It uses each record's `filenames` and `executable` paths to find rustc's
`deps/<name>-<metadata-hash>.d`, including hard-linked hashed executables when Cargo
reports only the uplifted binary. Each snapshot retains its own hard-link mapping;
other metadata hashes are not guessed by name. The uplifted binary's adjacent `.d`
is used only when no matching rustc dep-info exists. Each record's inputs include
only its own previous and current dep-info. No executable-to-library linkage is
guessed: Cargo fingerprints rebuild dependents when a library rebuilds. The guard
checks each library's own compiler inputs for the stale-mtime hazard. Inputs include
`include_str!`/`include_bytes!` files, not only Rust sources. Each record's
package adds its `build.rs`, `Cargo.toml`, and `Cargo.lock`, plus the workspace
`Cargo.lock`. Receipts retain these inputs as
repo-relative paths. The worker strips the slot's fixed source mount, its
`ct-bins-slot-N` utility mount (both under the launcher's `SOURCE_ROOT`), or the
original worktree root after parsing Make escaping and line continuations.
Inputs outside these roots are ignored. Missing or unreadable dep-info for a
repo-owned record fails closed, even with no content diff. Missing previous
dep-info is allowed when Cargo reports `fresh=false` and valid current dep-info
exists; unreadable previous dep-info still fails. Zero current repo-relative
inputs fail before package inputs are added. A content diff outside all valid
record inputs does not make any artifact stale; this includes documentation,
tools and non-embedded runtime files. Receipts record unmatched content changes
under metadata-derived package roots as informational `unread_package_paths`.
This includes unread Rust source paths and does not reject the build.

Content changes are computed from the last successfully verified source identity,
not the latest attempt or the mtime refresh set. Failed and rejected builds never
advance that baseline, so retrying cannot erase a pending content diff. When the content diff intersects a
record's own inputs, Cargo must report `fresh=false`. If that record is an
executable, its SHA256 must also differ from the previous hash at the same path.
Otherwise the run fails with `stale artifacts`. Libraries do not require changed
hashes. A changed library, `.rmeta`, or another executable cannot certify an
executable whose own inputs changed. Unreported artifacts and records with
disjoint inputs are not judged for freshness. A test-only edit therefore does not
reject an unaffected fresh bin, even within the same package. Records affected
only by mtime refresh require `fresh=false`; executables also require a current
hash, but identical rebuilt bytes are accepted.
For a fresh artifact, a root `Cargo.lock` change does not reject reuse when its
package and transitive dependency closure is unchanged. The closure digest covers
each reachable package's name, version, source, checksum, and sorted dependency list. Other input checks are unchanged.
The existing conservative root derivation and source mtime refresh are unchanged.

The slot-wide identity is only a source-refresh baseline, not proof that every
cached profile or metadata hash was built from that tree. Each reported repo-owned
output also needs an `artifact_certificates` entry keyed by its exact slot-relative
output path. The certificate contains the output SHA256 and content SHA256 values
for its current normalized dep-info and package inputs. Previous-only dep-info
inputs remain in stale-decision evidence, not the rebuilt output's certificate.
Certificates for unreported
outputs retain their original input hashes across unrelated builds. A `fresh=true`
record must match its own certificate, including its complete input set and output
bytes, even when the slot-wide content diff is empty. A missing certificate fails
closed. Certificates also record the lock closure digest. For a legacy certificate,
the worker reads `Cargo.lock` from the recorded revision and uses it only when its
SHA256 matches the certificate's lock input. A missing revision, failed Git read,
or digest mismatch rejects the fresh artifact. A changed closure also rejects it.
For lock-closure rejections and uncertified fresh artifacts, the worker evicts the rejected record's outputs
and matching fingerprint entries and records them in `evicted_artifacts` in the
receipt. For an uplifted executable, the worker identifies its matching hashed `deps` artifact and evicts it with its fingerprint. If the fingerprint cannot be identified, the worker deletes only the reported rejected outputs and fails with an explicit error; it does not wipe unrelated fingerprints. Other rejection behavior is unchanged. A `fresh=false` record with valid
dep-info and output hashes can establish a certificate. Changing receipt identities
does not provide proof.

Directory dep-info inputs are certified as recursive content trees. Entries are
sorted by relative name and hashed with length-delimited names and content digests;
changes, additions, removals and renames below the directory change its digest.
Symlinks follow filesystem resolution, including symlink-sensitive `..` traversal;
missing targets and directory cycles fail closed. Directory digest values use a
`tree-sha256:` prefix to distinguish trees from files, including inside nested
trees. Existing file digest values are unchanged. A changed descendant also
matches the directory input in mtime-refresh checks, content-change checks and
unread-input reporting. Thus nested edits retain the changed-output-SHA
requirement for rebuilt executables, not only certificate checks for fresh reuse.

The worker hashes tracked/untracked source files, their parent directories, known
source dep-info inputs and package manifests before launching the child. Parent
snapshots cover directory inputs first named by newly generated dep-info. Shared
subtrees and files are hashed once per snapshot. It hashes each record's source inputs
again afterward and rejects changed or previously unsnapshotted source inputs.
Dep-info input spellings retain `..` components and are resolved with
`os.path.realpath(strict=False)` before classification. Source and current-slot
inputs must also exist at their original, unnormalized spelling, so a removed
ancestor cannot be hidden by resolution.
Inputs inside the resolved source mounts keep repo-relative certificate keys.
All other inputs anywhere inside the current slot's resolved target directory
are required to exist and are content-hashed, using resolved absolute paths as
certificate keys. This includes target-triple layouts, any `build/<package>/out/`
depth, and external symlink aliases to slot-local files. These generated inputs
can be created or replaced by the child; their final hashes must match the
certificate on fresh reuse. Resolved paths outside the source mounts and current
slot remain external, including other slots, toolchain and registry inputs. Missing current compiler inputs
fail closed. Optional absent package files are represented by null hashes. Certificates advance only with an accepted
build, alongside the verified identity. Failed or rejected attempts cannot certify
an output. All reported filenames are hashed, including untouched `.rmeta` files,
not just executables or newly modified files.

When no compiler-artifact record is visible (for example, a wrapper captures Cargo
stdout), a successful child fails closed only if the diff intersects package inputs.
A visible library-only record is checked on its own dep-info and freshness.
A hidden stream with no package-input diff does not fail. Wrappers remain supported,
but captured output or filesystem hashes cannot substitute for the missing records.
For receipt evidence, the worker still scans newly touched executables/libraries
and rehashes previous paths. Receipt, hidden, and incremental subdirectories are
excluded from discovery. Those discovered paths do not select freshness candidates.
Checks can record library artifacts without an executable. Inspect the retained
unit's terminal result as well as its receipt; launch success is not build success.

## Host selection and dependencies

`python3 scripts/build-host.py --save-build-host desktop` saves the shared default in `~/.config/game-engine/build-host`; use `local` to change it. The configured default is desktop. Explicit `--build-host` overrides it for one invocation. Host failure is an error, never a host fallback.

Shared snapshot, selection, and native execution helpers live in `/syncthing/Sync/Projects/world-of-osso/game-engine/scripts/`. Set `BUILD_HOST_SCRIPTS` to override that dependency. Desktop uses pinned Rust 1.98.1 through rustup and same-ABI native runtime dependencies; local uses installed Arch Cargo/rustc without rustup, not fallback; observed 2026-10-10: `/usr/bin/cargo` and `/usr/bin/rustc` 1.99.0 (installed state, not a local pin). Record actual `cargo -V` and `rustc -Vv` for build evidence; there is no Docker runtime or bundled ICU substitution. Existing CI/release Docker pipelines remain separate and unchanged, including the [standalone PTR ICU workflow](ptr-icu-build.md).

## Source, cache, and lifetime

Desktop source and Cargo targets persist under `~/.cache/native-builds/<key>/` on desktop. Snapshots include current working edits and selected owned compile/runtime inputs, not only committed source. `crates/` includes internal workspace packages such as the dev-only test harness: Cargo resolves their manifests even for binary-only builds. The helper is not thereby a production compilation dependency. [Staged input proof](../data/test-perf/evidence/dev-harness-build-inputs-2026-10-11/README.md) records missing-manifest failures and corrected local workspace resolution; it is not a desktop build/transport pass. Previously supplied files are pruned without deleting app-owned runtime directories; content changes receive fresh mtimes, including A/B/A changes. Builds do not hold a runtime lock for the app's lifetime.

Runtime Blizzard caches remain host-local and profile-scoped under `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`; desktop resolves that home as `osso-test`. Host selection does not populate or migrate those caches. SSH leases control remote cleanup. Source SCP has a 60-second attempt limit and at most three transient attempts; permanent failures fail immediately.

### Local admission CPU scope

Measure CPU usage and capacity over the same cgroup scope. On October16,2026, local `agents.slice/cpu.max` was `max 100000` and main `nproc` reported32 CPUs; a measurement service with `CPUQuota=20%` reported1 from `nproc`, while `nproc --all` reported32. The then-current local capacity helper paired slice-wide CPU counters with a `nproc` denominator when the slice quota was unlimited, so a smaller polling-service quota produced a misleading `agents 9.9/1 CPU` verdict. This is not evidence that the slice has a one-core limit.

Main temporarily removed only its polling unit's quota, preserving its PID, low priority, bounded memory and normal queue/memory/pressure admission; actual build limits and guards stayed separate. The shared helper was subsequently changed by the other main session to use `getconf _NPROCESSORS_ONLN` for an unlimited host-wide slice, retiring the need for that wrapper workaround. Main inspected the changed quota expression and observed `getconf` returning32 in the same20%-quota service boundary. This is bounded source/probe evidence, not a complete shared-helper gate. Receipts: `~/.local/state/wow-ui-sim/handoff/iconbackground-capacity-scope-main-disposition.md` and `~/.local/state/wow-ui-sim/handoff/capacity-getconf-shared-fix-20261016T112030751633Z.json`. No FIFO bypass or host-slice policy change; no Cargo bottleneck or jobs12 speedup claim.

## Evidence and pending runtime proof

Main-observed evidence on October 3, 2026: actual native desktop GUI binary build passed (4m9s); headless CLI startup `--no-addons --no-saved-vars lua-errors` returned `[]`, exit 0 (`target/native-desktop-lua-errors.log`). These are bounded build/CLI observations, not full parity or warning-free acceptance.

Normal desktop GUI failed before its first frame: `target/native-desktop-gui-long.log` ends with connection reset (101) while creating the Wayland event loop. Engine `target/native-wslg-boundary.log` records the correct `/run/user/1000/wayland-0` symlink to `/mnt/wslg/runtime-dir/wayland-0`, repeated Weston SIG11 in `stderr.log`, and repeated approximately 102-second RDP peer disconnect/restart cycles. Windows msrdc ran in Session 0 while the active console was Session 1. Host compositor crash is observed; crash root cause and session causality are not established. This is not a simulator Lua/CASC/font stall. No WSL, service, or process restart was performed.

The actual local `wow-cli` helper built and printed `--help` in 0.35 seconds: bounded local CLI proof, not GUI acceptance or a local server pass. Local capability remains supported; actual local runtime acceptance is deferred, not required now for the current gate. User rejected local server build attempts; actual local server proof is deferred and its no-local-compile guard remains. Corrected source gate passed: two repaired fixtures, zero Ruff diagnostics across 14 files, and pycompile for nine changed files. The prior 94 passing fixtures retain revision-scoped credit; this is not a fresh 96-test run. Evidence: engine `target/native-migration-corrected-verify.md`. Historical focused-test/check reports are not a fresh current-tree gate. Normal desktop GUI remains unresolved; source-gate success would not complete the whole workflow.

### Measured jobs12 compile — October 10, 2026

Reported Cargo timing: **117.5 s**, optimized test-profile `--lib` + `--test integration`, revision `d80ee1b7f5fc0aed8e868c593dc84c71ca3ef817`. Library: **60.3 s** (frontend **17.09 s**, codegen **43.21 s**); distinct library-test harness: **104.3 s**; integration: **52.92 s**. Concurrent unit durations are not additive wall phases. Prior **125.2 s integration-only** measured different work/source; no normalized speedup or measured optimization is claimed.

Cargo job limit **12**, observed peak **8 concurrent Cargo units**; cgroup ceiling **12 CPUs**, consumption **985.701681 CPU-s** (~**8.39 CPUs coarse average** over 117.5 s), memory peak ~**13.45 GiB**. Process samples are partial: their elapsed clock has its own monotonic origin, with no established Cargo-start offset or compiler-phase attribution. These observations do not establish saturation or a bottleneck.

Toolchain label: **reported timing-HTML rustc 1.99.0**, not independent Rustc/ABI proof or hermetic dependency provenance. See the [existing compile brief](rust-integration-test-target-size-brief.md) and [sanitized timing summary and hash receipts](../data/test-perf/evidence/jobs12-map-display-2026-10-10/summary.json). Native goal remains open; this retention adds no acceptance gate.

### Mists trace compile — separate jobs12 observation

[Timestamped Cargo HTML summary](../data/test-perf/evidence/jobs12-mists-trace-2026-10-10/summary.json), start October10 at22:22:48UTC, independently admits **148.2s wall**, **737 units /728 fresh /9 dirty**, and a **130.25s library** (69.05s frontend +61.20s codegen). Seven final targets overlap after library completion; peak compiler-unit concurrency is **7** with jobs12. Benchmark-named bins record compilation, not benchmark execution.

The 299 CPU samples average53.1513 and peak78.5714 on Cargo's report scale; CPU-core denominator and core utilization remain unknown. Compiler version1.99.0 is HTML-recorded metadata. This HTML-only admission establishes neither invocation/source endpoints/exit status nor a controlled jobs12 speedup; earlier build evidence remains separate. [Retention hash](../data/test-perf/evidence/jobs12-mists-trace-2026-10-10/hash-manifest.json).

### Five-profile saved compile receipts — jobs12, different workloads

[Independent compiler-message report](../data/test-perf/evidence/jobs12-five-profile-2026-10-10/report.md) admits each profile's **663 artifact records:655 fresh/8 nonfresh**. Eight nonfresh targets are one library, **six binaries**, and one integration test; benchmark-named binaries were compiled, not benchmarked. Submission-to-compile-end wall seconds: Wrath190.922430, Mists106.577779, Era128.349182, Anniversary142.407461, Forever73.417071. Compile exits0; six dependency manifest deprecations each.

Features differ (including Mists prefork and Forever capabilities); recorded revisions also differ. Source/artifact admission and ordinary runtime proof remain separate. These intervals establish no CPU attribution, matched cache state, controlled speedup or whole-profile acceptance. [Report hash](../data/test-perf/evidence/jobs12-five-profile-2026-10-10/hash-manifest.json).

### Required unit2 builds — measured cgroup CPU window

[Seven-profile cost/resource evidence](../data/test-perf/evidence/unit2-jobs12-2026-10-10/final-report.md) retains already-required jobs12 builds: each663artifact records/655fresh/8nonfresh, one library+sixbins+integration. Compile intervals sum552.436541s; features/revisions/cache states differ, no controlled speedup comparison.

Actual cgroup counters yield5.70–6.45 coarse logical CPU equivalents per compile window under a12-CPU quota, including compiler/worker stream handling, not frontend/codegen or exclusive-rustc attribution. Recorded memory limit16GiB; cumulative unit peak reaches about16GiB after PTR, not per-profile peaks or proof of OOM. Throttling counters have no assigned cause. No resource-limit change or extra build experiment.

### Remaining currency profiles — bounded jobs12 observation

[Saved retry2 packet](../data/patch-api/evidence/currency-seven-profile-2026-10-11/README.md#compilation-observation) records already-required Era/Anniversary/Forever builds:131.890/135.727/143.911s submission-to-compile-end, each665 artifacts/656fresh/9nonfresh (build script, library, six bins, integration). Six dependency manifest deprecations per lane remain. These are different profile/cache workloads, not a controlled speedup comparison.

A completed19-snapshot sampler observes90.178s across late Era/early Anniversary:241.883cgroupCPU-s,2.682CPU equivalents and maximum4rustc processes sampled. Interval includes worker/linker/test work; peak is a lower bound, not actual concurrency maximum. No whole-build utilization, twelve-busy-core exclusion or causal bottleneck claim. Earlier aligned unit2/timing-HTML evidence remains separate; no extra rebuild was submitted for sampling.

### Approved dev-only generation boundary — bounded saved proof

The [harness contract](specs/prefork-test-harness.md#dev-only-automatic-harness-generation-approved-2026-10-11-verification-pending) requires automatic integration/prefork generation in an internal dev-only crate, with no test generation/watches in the root production build script. This changes cache invalidation, not native host selection or Cargo's underlying six-binary integration selection; no unsupported CLI exclusion flag is part of the change.

Source `256e54ef1` extracts generation/watches into unpublished dev-dependency `crates/test-harness`, leaving production responsibilities in root `build.rs`. Two item macros populate integration/prefork module trees; an expression macro supplies the Retail registry. Explicit workspace builds may select the helper; Cargo.lock retains its root watch.

[Retained local evidence](../data/test-perf/evidence/dev-only-harness-2026-10-11/README.md) admits Retail/Mists compile/list parity, 13 selected cases, two original Retail prefork bodies, generated-inner-byte preservation, format exit 0 and default-profile check exit 0 with existing warnings. The existing-test timestamp trial keeps the root library and six binaries fresh, rebuilding only helper library and integration; native output snapshots are unchanged.

A separate local temporary-marker discovery/execution/cleanup control passes, and cached Retail stock startup reports zero Lua errors with audio diagnostics. These results do not establish desktop cache invalidation or native-client acceptance. Changed revisions/cache graphs prohibit normalized speedup or CPU-saturation claims. Historical Mists cast-bar RED, full-suite failures, desktop GUI failure and broader handoff gaps remain open.

## Asynchronous full suite

`python3 tools/full_suite.py submit [REF] --profile PROFILE --target-dir SLOT --build-jobs 1` starts an asynchronous worker. Profiles are `retail` (default), `ptr`, `wrath`, `mists`, `wowforever`, `era`, and `anniversary`. `run` executes the worker synchronously. Both default to `origin/master`; the worker resolves REF after fetching, so pass a full SHA for revision-bound runs.

The suite flock serializes runs. Each SHA uses a detached worktree at `~/.worktrees/wow-ui-sim-fullsuite-<sha12>`. The worker rejects a mismatched or dirty worktree. It admits the explicitly selected slot through `capacity.sh`, then claims `.owner` exclusively. After the run, it moves the claim to `~/.local/state/wow-ui-sim/handoff/full-suite/<sha12>-<profile>/`. If the build unit's state remains unknown after three checks, it retains the claim, records a failure, and exits nonzero for reconciliation.

The build uses `scripts/slot-build.py` and `cargo test --no-run --all-targets --locked --offline --message-format=json`, with the selected profile features. The slot guard supplies the stable source mount, admission, artifact certification, and receipt. Only a successful service and receipt admit test execution. The integration and library binaries run for every profile; `prefork_full_ui` runs for Retail and Mists. Other profiles record it as not applicable. Test services bind the same source mount and use isolated CVar data. Build and test services use `MemoryMax=9G` and `MemorySwapMax=0`.

`--build-jobs` defaults to 1; `FULL_SUITE_JOBS` defaults to 16 test threads. Each run records phase commands, exits, durations, failures, and `WOW_SIM_TIMEOUT_FIXTURE_CAPACITY` in an attempt receipt `~/Projects/wow/full-suite-results/<sha>-<profile>-attempt-<timestamp>.json`, with a matching log. A non-PASSED attempt exits nonzero. Pre-claim failures and an unwritable receipt store can prevent a receipt.

Only a PASSED attempt promotes `~/Projects/wow/full-suite-results/<sha>-<profile>.json`; master runs also promote the matching profile baseline. The worker stages and fsyncs both files, backs up existing files, then renames the baseline before the canonical result. The canonical rename is the commit point. A rename failure triggers rollback; a process or machine crash between the two renames is not covered by rollback. `status [REF]` displays canonical profile results, any legacy Retail result, and the latest attempt for the selected SHA. The attempt includes its status, finish timestamp, and phase inferred from recorded steps. Without REF, status selects the newest attempt, or the newest canonical result when no attempt exists; it also displays the newest PASSED canonical result if it belongs to another SHA.

The Python tests use fake launchers and do not establish real systemd or suite behavior. Submit each profile separately against a coordinated approved slot.

Main deploys with `./deploy.sh` from the repository root (a Python installer,
not Bash). It installs only `tools/full_suite.py` as executable
`~/bin/full-suite`; no simulator build, service restart or suite submission.
Tracked edits alone do not update the installed entrypoint.

## Related

- [Build-host contract](specs/build-host.md)
- [Agent commands](../AGENTS.md#development-buildrun)
