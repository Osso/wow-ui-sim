# Independent startup receipt gate — PASS

Verified 2026-10-10. Completed receipts inspected; no pending execution. PASS is limited to default-profile API-registration/bootstrap and normal headless startup observed in these two runs. Build warnings are present; this is not a warning-free build claim.

## Receipt and execution

Evidence directory: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z`.
Controller completion: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/controller.stdout` records completed execution. Worker outcome records one successful build and exactly two selected executions. No duplicate build/run, full-suite polling, delegation, operations, or commits performed by verifier.

Build: `/usr/bin/cargo build --bin wow-sim --offline --locked --message-format=json --timings -j 12`, cwd `/home/osso/Projects/wow/wow-ui-sim`. Exit 0; stream errors 0; build-finished success true. Full Cargo JSON inspected programmatically: 729 records (1 build-started, 653 compiler-artifact, 74 build-script-executed, 1 build-finished); compiler-message records 0. Complete runtime stdout/stderr inspected, not collector JSON alone.

Both executions used `/usr/bin/timeout 90` and `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/wow-sim-sealed`, cwd `/home/osso/Projects/wow/wow-ui-sim`, `WOW_SIM_NO_SOUND=1`:

1. `--no-addons --no-saved-vars lua-errors`: exit 0. Stdout `[]` plus newline (3 bytes). Stderr lines 215–219 independently report final observed summary, `Status: CLEAN`, `Lua errors: 0 unique, 0 occurrence(s)`, owners 0, unattributed occurrences 0. No completion marker requested for this run; marker count 0 is expected.
2. `--no-addons --no-saved-vars --exec-lua "print('[CountApiStartupComplete]')" dump-tree --filter __CountApiStartup_NoMatchingFrame__`: exit 0. Exactly one completion marker at stderr line 251, following login, world-enter and post-login events. Frame Tree header at stderr line 283. Filter intentionally matches no frames; empty tree output is not the startup proof. Marker and successful exit independently establish completion.

## Counts across complete runtime streams

| Run | Root error diagnostics | Wrapper error diagnostics | Other error/panic/failure diagnostics | Budget diagnostics | Runtime warnings | Completion marker | Exit |
|---|---:|---:|---:|---:|---:|---:|---:|
| lua-errors | 0 | 0 | 0 | 0 | 0 | 0 (not requested) | 0 |
| normal-startup | 0 | 0 | 0 | 0 | 0 | 1 | 0 |

Counts concern emitted diagnostics, not invisible failures. Informational `seterrorhandler restored` and zero-error summary lines are not errors. Cache metric `replay_fail=0`/`store_fail=0` is not a failure. No hidden-startup/root/wrapper/handler-budget warning or traceback observed. Normal dump reports 39,381 anchored and 9,189 unanchored objects; these are informational counts, not proof of layout correctness.

Build stderr has 6 distinct manifest deprecation warnings plus 1 aggregate warning line (`generated 6 warnings`), all at lines 1–7. Deprecated Clippy keys: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. This warning finding remains; no changes authorized or made.

## Source, binary, profile, flags

Submission revision: `9635468d994be53d1f3571a8005115a49864c18a`. Independent current HEAD: `3ab3633cc8e08dbba22b361aaf01250a6a7a875a`; HEAD moved, but all 3,853 worker-hashed files still match. Before/after source maps are identical; both SHA-256 `76b30a8eee3fd76a05b14ce87f92f052ea8530525f93dfb7beb7581f89c5d69c`.

Worker tracked-file hash scope: `/home/osso/Projects/wow/wow-ui-sim/src`, `/home/osso/Projects/wow/wow-ui-sim/tests`, `/home/osso/Projects/wow/wow-ui-sim/Cargo.toml`, `/home/osso/Projects/wow/wow-ui-sim/Cargo.lock`, `/home/osso/Projects/wow/wow-ui-sim/build.rs`, `/home/osso/Projects/wow/wow-ui-sim/.cargo/config.toml`, `/home/osso/Projects/wow/wow-ui-sim/data/blizzard-ui-files`, and existing tracked listfile inputs selected by worker. Paths in saved hash maps are repository-relative; independently resolved against the absolute repository path. Excludes untracked files/index, external dependency contents, inherited environment, and runtime cache contents. Equal source hashes do not independently prove complete source-to-artifact provenance.

Cargo artifact executable: `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`. Sealed binary independently hashes to `c9dd3209fbc3c10e8c40dc930cdf3890e8725b4bbe7a0ff8704fe0168956c552` (407,312,872 bytes), matching worker artifact receipt and both post-run unchanged checks. Cargo reports `fresh=true`: successful guarded build reused a fresh artifact, not a newly compiled binary.

Normal `bin`/`wow-sim` artifact profile has `test=false`, opt_level `1`, debuginfo `line-tables-only`, debug_assertions and overflow_checks true. Target metadata `test=true` means target is test-eligible, not that this artifact is a test harness. Build command has no test or feature override flags. Manifest defaults are sound, gui, casc, client-retail; artifact includes default, client-retail, profile-retail, retail-12-0-0/5/7 and retail-12-1-0 with their derived capability features.

`/home/osso/Projects/wow/wow-ui-sim/.cargo/config.toml` configures Linux clang linker and rustflags `-C target-cpu=native -C link-arg=-fuse-ld=mold -C link-arg=-Wl,--thread-count=4`; these are source configuration evidence, not independently captured effective rustc argv. Inherited environment is excluded by worker. No claim that environment could not override flags.

## Cache/runtime scope

Both runs loaded 291 Blizzard addons and reported bytecode cache 1,569/1,569 hits, lookup_miss/replay_fail/stored/store_fail all 0. Both loaded an EditMode layout cache from WTF despite no SavedVariables; cache source printed `[REDACTED: private WTF layout-selection/realm/character identifiers]`. Both confirm third-party addon loading and SavedVariables disabled. Normal dump used CASC fonts; lua-errors initially used the font fallback with casc=false. Sound disabled in both.

Profile runtime source convention is `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns`; worker does not hash its contents or establish the exact cache provenance used at runtime. Actual runtime cache bytes, WTF layout bytes, CASC assets, external dependencies and effective inherited environment are outside this receipt's identity proof. PASS is a warm-cache observed-startup result, not cold-cache or cache-integrity proof.

## Stream identity

All paths below belong to `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z`:

| Absolute path | Bytes | Lines | SHA-256 |
|---|---:|---:|---|
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/compile.stdout` | 563084 | 729 | 666a89f2a092b325dc75556968c64072279da9494f46c0690d205da3f127ce7b |
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/compile.stderr` | 1345 | 9 | 715f035bca5e35e6a0a979dffc4b6615e86e168aabbae07309fafee562c0236d |
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/lua-errors.stdout` | 3 | 1 | 37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570 |
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/lua-errors.stderr` | 14690 | 219 | ed482f4e57a6f592f40495d3c146e815cd6b5eba129d4d54f0ef137789960914 |
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/normal-startup.stdout` | 1086 | 16 | 3c9fdad351ae6657a4dac106430e0a935e0b7477fe88f2e1815756a7f2b36f9e |
| `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/normal-startup.stderr` | 17921 | 284 | 5b6592c078eb20179ef44d325783a6882e9cd48888f9ad1d44239ec50971f7e3 |

No native WoW parity, GUI execution, rendering correctness, all-profile coverage, or exhaustive individual API behavior claim.

## Publication provenance

Sanitized derivative. Original parent-report SHA-256: `aa4053f0d930fcfc5b3b8c1140daf6ab7e19ddf53be799c5f667ebfb0d952582`. Raw runtime streams remain private; stream hashes identify inspected evidence, not published payloads.
