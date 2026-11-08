# Independent bounded artifact audit

**Goal OPEN.** Saved controlled rebuild succeeds; two corrected fixture cases pass; original same-chunk debug probe still fails. No runtime fix or root-cause completion claim.

## Scope and attribution

Audited epoch `20261009T225558Z`, parent `worker.py`, `request.jsonl`, submission/controller evidence, and sibling `linker-package-invalidation` selection evidence. Followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as independent verifier. No delegation, builds, tests, Cargo check, service operations, cache cleaning, toolchain changes, or edits outside this audit directory. Read saved logs, parsed all saved Cargo JSON records, read immutable Git objects, hashed existing binaries, and checked rustfmt on isolated Git-object fixture snapshots only.

Epoch revision: `1fac15bd0e477cd0fdf1f03dd0219dbbedfd4111`. Fixture revision: `611e10d2d76b368d01f0c61b006fe2d787746f37`. Same-chunk trace revision: `6a4d43fc16008a9346eaad927b871354f184cc55`. The two fixture files at 611 and debug probe file at 6a are byte-identical to their epoch Git objects and match epoch source SHA256 records. Later working-tree changes are not attributed to this run. Isolated sources retained under `source/<revision>/`.

## Proof matrix

| Claim | Result | Evidence / boundary |
|---|---|---|
| Controlled compilation | PASS | `cargo-result.json`: exit 0; all 739 stdout records parse; final record `{"reason":"build-finished","success":true}` |
| Currency fixture | PASS | case1: 1 passed, 0 failed, 10697 filtered out; service `Result=success`, `ExecMainStatus=0` |
| Search fixture | PASS | case2: 1 passed, 0 failed, 10697 filtered out; service `Result=success`, `ExecMainStatus=0` |
| Debug same-chunk probe | FAIL | case3: 0 passed, 1 failed, 1977 filtered out; service `Result=exit-code`, `ExecMainStatus=101` |
| Fixture static scope / formatting | PASS | 611 changes exactly two fixture files; test names/counts preserved (currency 4/4; search 2/2); scoped snapshot rustfmt exit 0, empty stdout/stderr |

These are three exact selected tests, not full-suite proof. Service observation command exit 0 is not the failing test's exit: case3's actual service payload status is 101. Saved case stdout/stderr and receipts were read, not regenerated.

## Artifact identity and build contract

Captured existing binary hashes at **2026-10-09T23:06:04.652946Z**, before later main rebuilds. All three match epoch `artifact-identities.json` and their compiler-artifact records occur verbatim in saved Cargo stdout:

| Artifact | SHA256 |
|---|---|
| `target/debug/wow-sim` | `a5e7c397129ec917ea32eb081bf5631b3cdccb8ae69c5e8c997cac981885988d` |
| `target/debug/deps/wow_ui_sim-bcad23723a7b3b0d` | `b9ad2547283fb1390df2c52c39271d3972a3b105f17a5dde686bcfde67ff0a2d` |
| `target/debug/deps/integration-1db16289b0998b2a` | `168fea2c7aaf7d0d9993f1283b2ddd3e7238eaa1356f1103016e8c575f1d303e` |

`current-artifact-capture.json` retains paths, sizes, mtimes and capture time. Case1/2 submission hash equals the integration hash; case3 equals the lib-test hash. Each saved selector listing reports exactly `1 test, 0 benchmarks`; test payload uses `--exact --nocapture --test-threads=1` behind `/usr/bin/timeout 90`. Only case3 overrides `WOW_SIM_TRACE_DEBUG_GETTERS=1`.

Compilation argv, identical in worker/process/result records:

```
/usr/bin/cargo test --offline --locked --lib --test integration --no-run --message-format=json
```

Cwd is the canonical repo; explicit override `CARGO_BUILD_JOBS=4`. Profile: opt-level `1`, debuginfo `line-tables-only`, debug assertions and overflow checks true; lib/integration are test artifacts, wow-sim is not. Default retail profile with `client-retail`, `profile-retail`, `sound`, `gui`, `casc` and patch features through `retail-12-1-0`; no `fast-build`, PTR, or alternate client feature. Full exact 20-feature lists retained in compiler receipts and `flags-before.json`.

Both retained pre-invalidation fingerprints agree on rustc fingerprint `13444744534853132663`, config fingerprint `12296471850835478129`, and flags:

```
-C target-cpu=native -C link-arg=-fuse-ld=mold -C link-arg=-Wl,--thread-count=4
```

Their feature arrays exactly equal rebuilt artifact feature arrays. Limitation: successful compiler-artifact JSON does not record complete rustc argv; inherited environment/external toolchain binary identity is not attested by this epoch. These flags are proven for retained pre-invalidation fingerprints, not an independently captured post-build rustc command. Worker adds no linker/toolchain override; do not upgrade that into complete environment equality.

Cargo stdout contains 664 compiler-artifact records, 74 build-script-executed records, one successful build-finished, and zero compiler-message records. Of 664 artifacts, 10 belong to this package (one fresh build-script artifact; nine non-fresh library/bin/test artifacts), and **654 dependency artifacts are all fresh**. Compilation also emits package binaries beyond the three selected identities; it is not literally a three-artifact-only build. Saved stderr reports six deprecated iced-wgpu manifest lint-name warnings, summarized as six warnings; no warning-free claim.

## Parent, lock and exact invalidation

Parent submission exit 0 queued `/home/osso/.worktrees/build-lock.sh /home/osso/.local/bin/pyrun-jsonl`, with request-file stdin and append-only controller streams. Worker and request hashes independently match submission: worker `130197c9f7274e7d2f94972ec1488feb36a3291a500f9b3eed7c2806b7856d55`; request `e655e9187c1a0db3c5668eab32cdbc00bb2a91f5bbbe8ee479a001405e97caf9`. Entry repeats worker hash. Controller stdout completes with this exact epoch; stderr records waiting then acquisition. Read lock wrapper: fd 9 holds `.builder.lock` while invoking child command. No parent systemd final-status snapshot exists here; completed controller output plus persisted Cargo result/outcome are the retained completion proof.

After lock acquisition, at `22:55:58.548806Z`, worker records MemAvailable **39.75541305541992 GiB**, load **7.57**, requiring at least 6 GiB and load below 24. Invalidation begins at `22:55:59.040106Z`; result recorded `22:55:59.619000Z`; compilation starts `22:55:59.620906Z`. Worker checks resources before deletion, validates source hashes, checks exactly six existing generated paths and protected hashes, deletes only that explicit list, checks absence/protected hashes, then compiles. Boot ID before/after equals `931d4a8a-2178-4666-bf53-776e5eac97be`.

Exact selected paths (all relative to repo `target/debug/`):

```
incremental/wow_ui_sim-04lvz7pej7y93
incremental/wow_ui_sim-35s05hzg2gmxy
.fingerprint/wow-ui-sim-3282a118e07cdd18
.fingerprint/wow-ui-sim-bcad23723a7b3b0d
deps/libwow_ui_sim-3282a118e07cdd18.rlib
deps/libwow_ui_sim-3282a118e07cdd18.rmeta
```

Sibling selection paths equal epoch list exactly, with no duplicates. Saved mapping records uniquely associate production tag `3282a118e07cdd18` (73/73 CGU stems) and lib-test tag `bcad23723a7b3b0d` (48/48) to those incremental roots. Independently re-parsing both prior saved linker diagnostics reproduces **73** and **48** unique target-tagged CGU stems. The retained directory inventory lists five package incremental roots; mapping selects two. Limitation: saved mapping supplies counts/unique-root decisions, not a full per-CGU-to-root content proof; historical deleted directory contents cannot be rechecked. `incremental-object-suffix-map.json` is empty and is not affirmative mapping evidence.

Rejected broad package clean was **dry-run only**: `13570 files, 29.8GiB total`, `no files deleted due to --dry-run`. Earlier nonblocking invalidation attempt returned EAGAIN and performed no deletion. Successful worker never runs cargo clean; generated directories may now exist because successful compilation recreates them. `selectedstate.json` distinguishes current presence from historical removal.

## State and cache limits

All **3845** source-scope hash entries are equal before/after and equal prior failed epoch: 1653 src, 2182 tests, seven data files, Cargo.toml, Cargo.lock, build.rs. This proves equality of the inventoried scope, not all files, external path dependencies, inherited environment, or untracked `.code-index.db`.

Worker verifies two protected binary hashes immediately before/after deletion; audit current protected hashes also match records (see `selectedstate.json`). Prefork artifact and separate `target/mists-wrapper-durable` integration artifact were not selected. Dependency cache not selected and 654/654 dependency artifacts remain Cargo-fresh. No epoch whole-cache after snapshot exists; prior `cache-before.json` alone cannot establish complete Blizzard cache before/after equality or whole dependency-cache byte equality. Controlled successful rebuild is evidence compatible with artifact-state inconsistency, not proof of the original linker failure's root cause or durable repair.

## Fixture semantics and readability at 611

Currency snapshot lines 193–199 keeps amount 1234, currency ID 7, icon ID 9_999_777 and explicitly probes/asserts `GetLocale() == "enUS"`. Expected value changes only modeled locale amount to `1,234`; icon markup remains `|T9999777:12:12:0:0|t` (width/height/offset fields unchanged). This is local modeled behavior, not native-client capture. Original forwarding/nil-icon tests remain; four test functions unchanged.

Search snapshot lines 93–96 saves original OnTextChanged script and disables the automatic binding while explicitly driving handler for `ab` then `abc`; line 140 restores original script. Expected exact signatures remain:

```
set_called=0 hide_called=1 update_called=0 show_called=0
set_called=1 query=abc finished=true hide_called=0 update_called=0 show_called=1
```

Counts, forwarding, finished state, assertions and preview-routing stubs remain unchanged; two test functions unchanged. This corrects double-driving fixture input, not simulator/vendor event behavior. Teardown restoration is on normal completion; this audit does not assert error-path restoration.

Read every changed line in Git diff at 611 under rust-readability criteria: no new Rust nesting, suppressions, opaque conditionals, duplicate implementations, or hidden state effects found. Scoped rustfmt on the two isolated 611 snapshots: exit 0, stdout/stderr empty, recorded in `rustfmt-611-snapshots.json`. Implementation receipt's earlier formatting/readability claims are supporting evidence, not substituted for these snapshot checks. No whole-repo readability claim.

## Failure boundary correction

Case3 retains **28** Rust `[DebugGetterTrace]` records and **eight** in-chunk `[DebugGetterProbe]` records: both getters at probe_start, after_secure_delegate, after_button_metatable, after_editbox_metatable. Both remain functions with stable identities. No after_secretwrap or before_callstack_height stage is logged.

Saved panic:

```
src/lua_api/workarounds/temporary/debug_environment_defaults.rs:260:14
(string):19: attempt to call a userdata value
```

Immutable 6a probe shows next expression after last successful stage is **`secretwrap(marker)()`**, before getter calls. Thus failing boundary is that chained call, not GetCallstackHeight/GetErrorCallstackHeight. It does not distinguish first `secretwrap(marker)` from invoking its result. `probe-6a-line-map.txt` places chained call on textual line 18 and next trace on 19; saved runtime reports 19. This observed discrepancy is precisely why source-line maps alone/PC offsets cannot establish the failing call. Stage ordering provides the bounded conclusion; no VM root cause or fix claimed.

## Outputs and privacy

`evidence.json`, `current-artifact-capture.json`, `selectedstate.json`, `flags-before.json`, isolated Git sources/diff, scoped rustfmt receipt, probe map and retained prior linker diagnostics provide local proof. `hashmanifest.sha256` covers audit outputs and selected original evidence inputs; it excludes itself. `privacy.json` records bounded credential-pattern scan (75 selected files, zero candidates) and private local-path metadata. This is local evidence, not a public-redacted bundle. Goal stays **OPEN** because case3 fails and broader acceptance is unproven.
