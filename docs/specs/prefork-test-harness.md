# Prefork test harness

The Linux prefork test harness provides a reusable custom test-runner contract for expensive parent-owned test state. Its source lives in `tests/common/prefork.rs`; implementation details are documented at [the prefork harness wiki page](../wiki/systems/prefork-test-harness.md).

## What it must do

### Platform and lifecycle

- [x] Run only on Linux.
- [x] Keep the runner parent single-threaded and reject every fork attempt unless `/proc/self/task` reports exactly one task immediately before the fork.
- [x] Borrow immutable state created by the parent in each selected child without leaking child mutations back to the parent or sibling cases.
- [x] Support one optional `fn()` child setup hook, defaulting to a no-op, after child process-group establishment and before the registered case body under the same panic capture.
- [x] Run one child process for every selected case.
- [x] Bound concurrent children with two default workers and an explicit hard-maximum worker count.
- [x] Honor both `--test-threads` forms and `RUST_TEST_THREADS`, with the command-line value taking precedence.
- [x] Run all runner conformance cases in a fresh subprocess before the normal full-UI preload during ordinary target execution; hide successful conformance output and print it on failure.
- [x] Parse selection before setup so `--list` and zero-match execution run neither conformance nor preload; zero matches report a successful zero-test result, while selected-case setup failure exits explicitly.

### Selection and output

- [x] Support one positional substring filter and exact matching with `--exact`.
- [x] Support repeatable `--skip VALUE` and `--skip=VALUE` exclusions, rejecting empty values.
- [x] Support `--list` with libtest-compatible case and summary data.
- [x] Reject unsupported arguments, duplicate singleton flags/options, and conflicting arguments instead of ignoring them.
- [x] Capture child stdout and stderr by default, suppress successful captured output, and print captured output for failures.
- [x] Inherit child stdout and stderr when `--nocapture` is selected.

### Linux ordinary libtest timeout isolation

- [x] Keep Linux `test_timeout!`/`with_timeout` separate from the prefork runner while re-executing exactly one named libtest test in an isolated child with `--exact --include-ignored --test-threads=1`.
- [x] Coordinate Linux test workloads with a process-local poison-recovering `RwLock` plus a cross-process `flock`: ordinary timeout, prefork, and full-UI workloads use shared access, while performance measurements use exclusive access; ordinary timeout re-executions additionally use a separate two-slot cross-process `flock` permit limiting concurrent full `integration-*` children globally. Outer timeout conformance fixture subprocesses reserve the workload gate exclusively; an inherited marker bypasses that gate in their descendants so nested timeout children still contend only for the shared two-slot permit.
- [x] Acquire the workload permit and timeout-child slot before spawning; start the 120-second child budget only after spawn, and hold the slot through process-tree cleanup, output draining, handshake validation, and failure aggregation.
- [x] Convert ordinary assertion panics in the child into a normal libtest failure in the parent, so later sibling tests continue instead of the integration process aborting.
- [x] On a real timeout, terminate the child process group/tree, reap the child, and return a test failure; the normal `test_timeout!` limit remains 120 seconds, while timeout-reexec conformance fixtures use a 1-second limit only to prove cleanup.
- [x] Forward visible-output flags (`--nocapture`, `--no-capture`, and `--show-output`) to the exact child test while preserving captured output for failure reporting.
- [x] Share process-group and process-tree cleanup helpers with the prefork runner without sharing its parent preload, fork scheduling, or immutable-state contract.
- [x] Treat nested `with_timeout` calls inside an exact-test child as guarded closures in that same child, recording one handshake and avoiding a second re-exec boundary.
- [x] Wrap multi-shard same-process tests in one outer timeout and run their inner shard bodies without nested timeout wrappers, so the outer boundary governs the complete sequence.

### Normal retail full-UI preload

- [x] Enter process-local parent-bypass bytecode-cache mode before building one parent-owned 1024x768 `ScreenKind::Game` `WowLuaEnv` from the synced default-retail Blizzard UI cache.
- [x] Stop GC, compile Blizzard Lua from source without reading or writing the bytecode pack, discover eligible game-screen `Blizzard_*` non-LoD roots plus their transitive hard TOC dependencies and explicit LoD startup roots from the filtered candidate pool, load that dependency order without SavedVariables or third-party addons, and fire `ADDON_LOADED` after every successful load. `Blizzard_Game` depends on `Blizzard_TimeManager`, `Blizzard_CooldownBroadcaster`, `Blizzard_BoostTutorial`, and `Blizzard_CombatLog`; CombatLog's declared base and processor dependencies precede it. Standalone Game-only LoD roots `Blizzard_MacroUI` and `Blizzard_TrainerUI` publish their loaders before prefork startup events while remaining excluded from glue screens. Standalone Game-only LoD root `Blizzard_AchievementUI` publishes `AchievementFrame_LoadUI` before `AlertFrame` receives `ACHIEVEMENT_EARNED`, preserving real achievement-toast queue behavior while remaining excluded from glue screens. LoD root `Blizzard_RaidUI` depends on `Blizzard_RaidFrame`, so RaidFrame loads first. Unrelated non-Blizzard cache directories and LoD roots remain excluded, and disabled bytecode caching remains disabled.
- [x] Restore post-cleanup globals after `Blizzard_EnvironmentCleanup`, sync the string metatable, apply post-load workarounds, restart bootstrap GC, and run the normal game startup event sequence.
- [x] After successful startup, when bytecode caching is enabled, seal the bypassed cache as empty and initialized so read-only children cannot reload the disk pack; preserve whether `pack.bin` exists, return a successful zero-byte release outcome, and fail if cache state was initialized or populated before sealing. Disabled caching returns a successful no-op.
- [x] Fail setup explicitly with addon context on addon-load, `ADDON_LOADED`, EnvironmentCleanup restoration, startup Lua, bootstrap-GC, or bytecode-cache sealing errors instead of continuing with a partial parent snapshot.
- [x] Run registered full-UI cases as immutable `fn(&WowLuaEnv)` children with a 120-second child timeout and read-only bytecode-cache child setup.
- [x] Define migrated cases with the explicit `prefork_full_ui_case!` marker; `build.rs` parses marker items with `syn`, recursively follows literal `#[path = "..."] mod` children, renders the registry with `quote`, and assigns stable `<module>::<function>` names. Patch-manifest test references accept these generated marker cases alongside ordinary `#[test]` functions.
- [x] Include the generated integration module tree in the prefork target so mixed modules compile once while unmarked tests remain under libtest.
- [x] Prove nested-module discovery and inherited startup state with `prefork_full_ui_nested::fixture::preloaded_parent_has_normal_game_startup`.
- [x] Register the generated default-retail full-UI registry with stable `<module>::<function>` names and retain 9 manual/nested prefork cases.

### Final coverage and eligibility

- [x] List exactly 1,965 cases in the dedicated default-retail prefork target: 1,956 marker-generated full-environment cases and 9 manual/nested prefork cases.
- [x] Retain exactly 8,412 ordinary integration cases after 14 one-for-one migrations.
- [x] Audit all 290 remaining ordinary startup-like tests and confirm zero eligible cases remain for the finalized shared parent preload.
- [x] Classify exclusions as dependency/load-order/absence, pre-start, lifecycle, partial-fixture, alternate-screen, render-sensitive, thread-sensitive, profile-specific, post-drop, or version-specific; none is equivalent to normal complete retail startup.
- [x] Preserve exclusion rationale: the finalized parent is incompatible with pre-start and lifecycle cases, while partial/custom, alternate-screen, render-sensitive, and thread-sensitive cases are not normal-retail startup; migrated cases retain the same 120-second child timeout and process-tree cleanup.

### Bytecode-cache child contract

- [x] Keep the Lua bytecode cache writable by default in production; use process-local parent-bypass mode only for the dedicated prefork preload.
- [x] In parent-bypass mode, return cache misses without loading `pack.bin`, compile source normally, and treat suppressed stores as non-failures; disabled caching remains a successful no-op.
- [x] Allow a forked child setup hook to transition the inherited bypassed state into one-way process-local read-only mode without changing parent state.
- [x] Preserve current-pack and legacy cache hits while suppressing legacy promotion and on-disk migration.
- [x] Compile cache misses normally without counting suppressed cache stores as failures.
- [x] In read-only mode, never create, append, replace, compact, truncate, remove, rename, or stage temporary bytecode-cache files.
- [x] Preserve warm-cache conformance in a fresh subprocess with an isolated `XDG_CACHE_HOME`: parent prewarm creates `pack.bin`, releases non-empty in-memory pack state, a unique child Lua chunk compiles, the prewarmed cache tree bytes and metadata remain identical, and the parent remains writable.
- [x] Prove parent bypass in a separate fresh process against an existing pack: parent source and child source compile, sealing reports zero loaded bytes, and neither phase changes cache bytes, metadata, or directory contents.

### Performance proof

- [x] Preserve the initial nine migrated behaviors with parent bypass enabled.
- [x] Reduce serial wall time from 23.49 seconds to 10.12 seconds (56.9%).
- [x] Move nine settled `party_frame_tree` behaviors one-for-one from integration to the existing full-retail prefork parent, reducing the comparable exact batch from 36.4509 seconds to 11.9914 seconds (67.1%).
- [x] Move five settled `objective_tracker_tree` behaviors one-for-one from integration to the existing full-retail prefork parent, reducing the comparable exact batch from 19.0040 seconds to 9.0833 seconds (52.2%).
- [x] Reduce `/usr/bin/time` process maximum RSS from 1,190,600 KiB to 788,236 KiB (33.8%) and sampled process-tree PSS from 1,189,511 KiB to 1,040,276 KiB (12.5%).
- [x] Record sampled process-tree RSS separately: it rises from 1,196,596 KiB to 1,523,432 KiB because RSS counts shared copy-on-write pages in both parent and child, while PSS apportions them.
- [x] Prove workload-gate shared-reader overlap, exclusive exclusion, same-process exclusive serialization, release after error/panic/process exit, and acquisition before timeout measurement.

### Subsequent behavior migrations

- [x] Move 17 additional normal complete-retail-startup behaviors: six `Blizzard_AddOnList` surface cases, five micro-menu/game-menu cases, and six CharacterFrame `ShowUIPanel`/`HideUIPanel` cases. Independent verification records 8,393 integration cases, 1,984 prefork cases, zero overlap, and 17/17 passing.
- [x] Verify commit `2995da11d` after moving eight equivalent `chat_frame::test_chat_*` cases: 8/8 pass, integration lists 8,385 cases, prefork lists 1,992, and overlap is zero. The 13.1985s-to-6.326s runtime change is directional only because power mode changed. Keep `test_chat_editbox_click_type_and_submit` and `test_chat_editbox_text_color_after_activation` in ordinary integration because complete startup changes their asserted initial channel and edit-box color.
- [ ] Verify commit `465acd9da` after moving the two equivalent GroupFinder post-start behaviors. Implementer evidence records 8,383 integration and 1,994 prefork cases; do not claim runtime improvement from the 4.224s integration baseline versus the 6.26s standalone prefork run because shared preload is included.

### Failure handling

- [x] Catch panics and report panic text as structured failure data.
- [x] Distinguish panic, signal, unexpected exit, and timeout failures.
- [x] Establish and parent-verify each child process group before releasing the child test body or scheduling another case.
- [x] On timeout, send `SIGTERM`, wait a bounded grace period, send `SIGKILL`, reap the child, and remove its process tree.
- [x] On parent-side runner failure, terminate every active process group, reap every direct child, and close every owned descriptor before returning.
- [x] Exit unsuccessfully when argument validation, runner operation, or any selected case fails.

## How it works

- [Prefork test harness system](../wiki/systems/prefork-test-harness.md)
- `prefork_full_ui_case!` marker bodies are the single implementation of migrated cases; they are not also emitted as ordinary `#[test]` functions.

## Implementation inventory

- `Cargo.toml` — declares the dedicated Linux prefork conformance target contract.
- `build.rs` — keeps the custom target root out of the generated integration harness and builds the stable marker registry with `syn`/`quote`, including literal `#[path]` child modules.
- `tests/common/prefork.rs` — reusable eager and lazy-state test-only runner APIs and execution behavior.
- `tests/common/prefork_full_ui_preload.rs` — target-only normal retail game-screen preload and migrated-case helpers.
- `tests/prefork_full_ui.rs` — custom target entry point, real-case registry, fixture cases, and behavioral conformance checks.
- `tests/common/workload_gate.rs` — Linux shared/exclusive process-local and cross-process workload coordination.
- `tests/workload_gate.rs` — workload-gate overlap, exclusion, cleanup, and timeout-boundary proofs.
- `tests/test_keybindings_panels_detail.rs` — nine manually registered immutable-environment case bodies.
- `tests/blizzard_behavioral_messaging_loads.rs` — two marker-defined immutable-environment case bodies; its lightweight tests remain under libtest.
- `tests/prefork_full_ui_nested/fixture.rs` — nested-module registry and preloaded-startup behavior fixture.
- `src/loader/bytecode_cache.rs` — writable, parent-bypass, and read-only process modes; mutation-boundary enforcement; and empty-state sealing before child forks.
- `src/loader/mod.rs` — narrow doc-hidden prefork cache entry points.

## Tests asserting this spec

- `tests/prefork_full_ui.rs`

## Known gaps (current cycle)

- [x] Migrate the initial nine `test_keybindings_panels_detail` `WowLuaEnv` cases onto the reusable runner.
- [x] Add the two behavioral-messaging cases and one nested registry/preloaded-startup fixture.
- [x] Benchmark the parent-bypass prefork execution against the retained current in-target baseline.
- [x] Migrate every eligible normal-retail full-environment test and complete the final 304-test ordinary startup-like eligibility audit.

## Out of scope

- Non-Linux prefork support, because the contract depends on Linux process and `/proc` semantics.
- General runtime read-only behavior beyond Lua bytecode-cache mutation suppression in explicitly configured forked test children.
- Compatibility fallbacks for unsupported custom-runner arguments.
