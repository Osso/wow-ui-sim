# Prefork test harness

The Linux prefork test harness provides a reusable custom test-runner contract for expensive parent-owned test state. Its source lives in `tests/common/prefork.rs`; implementation details are documented at [the prefork harness wiki page](../wiki/systems/prefork-test-harness.md).

## What it must do

### Dev-only automatic harness generation (approved 2026-10-11; verification pending)

- [x] Generate the integration module tree and prefork marker registry automatically in an internal dev-only crate; production builds must not depend on that crate.
- [x] Remove test generation and test-source watches from the root production build script; retain unrelated production build responsibilities.
- [x] Preserve automatic test discovery, recursive literal `#[path]` module discovery, cfg/profile eligibility, and stable public CLI `<module>::<function>` case identities without manual registration.
- [x] Preserve original fixture constructors, case bodies/assertions, and integration/prefork partitioning; moving generation must not normalize fixture state or change runner behavior.
- [x] Invalidate dev-only generated harness outputs when discovery inputs change, without test-only edits invalidating the production build script/library through generation or watches. Preserve Cargo's underlying six-binary integration-test selection; this is a cache-invalidation change, not a CLI exclusion flag.

[Retained bounded proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/dev-only-harness-2026-10-11/README.md): source `256e54ef1`, Retail/Mists compilation/listings and 13 selected cases pass; format and default-profile check exit 0 with existing warnings. Generated inner bodies match the original snapshots apart from macro envelopes. Two additional original Retail bodies pass through the ordinary prefork runner, including the generated nested startup case; constructors/assertions are unchanged.

The existing-test timestamp trial keeps the root library and six binaries fresh while rebuilding only the helper library and integration test. A separate temporary top-level module/marker control is discovered exactly once, runs one original startup-assertion body, and disappears after cleanup with baseline names restored; production targets remain fresh. Current cached Retail stock startup reports zero Lua errors with audio diagnostics. These are bounded local checks, not all-profile/native acceptance. Changed revisions/cache graphs prohibit normalized speedup or saturation claims. Historical Mists cast-bar RED, full-suite failures, native-host and broader handoff gaps remain open.

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
- [x] Coordinate Linux test workloads with a process-local poison-recovering `RwLock` plus a cross-process `flock`: ordinary timeout, prefork, and full-UI workloads use shared access, while performance measurements use exclusive access; ordinary timeout re-executions additionally use a separate cross-process `flock` permit limiting concurrent full `integration-*` children globally. `WOW_SIM_TIMEOUT_CHILD_SLOTS` configures this harness policy from 2 through 6; an unset variable means 2. Invalid values fail explicitly. Conformance checks the configured bound, not a native-client requirement. Outer timeout conformance fixture subprocesses reserve the workload gate exclusively; an inherited marker bypasses that gate in their descendants so nested timeout children still contend only for the shared two-slot permit.
- [ ] Support the opt-in [`WOW_SIM_TIMEOUT_INVENTORY`](necessary-suite-runtime.md#timeout-entry-inventory) parent-entry manifest without changing default output or behavior.
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
- [x] Define migrated cases with the explicit `prefork_full_ui_case!` marker; the dev-only harness build script parses marker items with `syn`, recursively follows literal `#[path = "..."] mod` children, renders the registry with `quote`, and assigns stable `<module>::<function>` names. Patch-manifest test references accept these generated marker cases alongside ordinary `#[test]` functions.
- [x] Include the generated integration module tree in the prefork target so mixed modules compile once while unmarked tests remain under libtest.
- [x] Prove nested-module discovery and inherited startup state with `prefork_full_ui_nested::fixture::preloaded_parent_has_normal_game_startup`.
- [x] Register the generated default-retail full-UI registry with stable `<module>::<function>` names and retain 9 manual/nested prefork cases.

### Final coverage and eligibility (historical sealed audit epoch)

- [x] List exactly 1,965 cases in the dedicated default-retail prefork target: 1,956 marker-generated full-environment cases and 9 manual/nested prefork cases.
- [x] Retain exactly 8,412 ordinary integration cases after 14 one-for-one migrations.
- [x] Audit all 290 remaining ordinary startup-like tests and confirm zero eligible cases remain for the finalized shared parent preload.
- [x] Classify exclusions as dependency/load-order/absence, pre-start, lifecycle, partial-fixture, alternate-screen, render-sensitive, thread-sensitive, profile-specific, post-drop, or version-specific; none is equivalent to normal complete retail startup.
- [x] Preserve exclusion rationale: the finalized parent is incompatible with pre-start and lifecycle cases, while partial/custom, alternate-screen, render-sensitive, and thread-sensitive cases are not normal-retail startup; migrated cases retain the same 120-second child timeout and process-tree cleanup.

### Exact startup fixtures (bounded Retail proof at `3de874658`, 2026-10-09)

[Current-epoch migration placement](../../data/test-perf/prefork-migration-status-2026-10-09.md) reconciles all 186 historical MIGRATE/ADAPT names. The original migration plan's four integration statuses and remaining counts are historical, superseded by the exact-fixture placement below; its classifications, timings, source traces and deferrals remain unchanged. Placement coverage does not close acceptance gates.

- [x] Migrate the two chat editbox cases, cast-bar edit-mode case, and Mainline spellbook keybinding case using their original fixture constructors and unchanged assertions, not completed Game preload or post-fork state normalization.
- [x] Preserve manual chat startup without `UPDATE_CHAT_WINDOWS`, initial SAY/white behavior, original cast-bar startup and anchor transitions, and profile-specific spellbook loading and key dispatch.
- [x] List each stable case name exactly once in the existing `prefork_full_ui` target and exclude migrated Retail cases from ordinary integration.
- [x] Inspect original non-Retail feature gates and constructor/body preservation without execution credit.
- [ ] Verify retained non-Retail fixture behavior through execution.
- [ ] Apply existing filtering, timeout, process cleanup, parent-isolation and bytecode-cache contracts to every fixture group; propagate any group failure to the target exit status. Retained 21/21 runner conformance is generic; independent controlled 3/3 at `96e88494a` closes only the next bounded requirement, not original full-fixture/cache/all-profile acceptance. See [controlled proof and limits](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#controlled-controller-independent-epoch).
- [x] Exercise the public controller with bounded deterministic registries for each enabled real group (`chat`, `cast-bar`, `spellbook`): exact selection runs only its named child; exact skip/unmatched selection initializes no fixture or cache; a failing child makes the controller fail while later groups still run in fresh processes; timeout removes both child and descendant, preserves disk cache bytes/metadata, and permits subsequent fresh-process setup/sealing and success. This test-only mode must use existing group subprocess/fixture paths without changing original constructors, bodies, profile gates or cache semantics.

[Bounded proof SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#exact-startup-fixtures-retained-redgreen-epoch) links immutable RED/GREEN reports and byte retention. At `3de874658`, four original bodies pass after fork; prefork lists 2,325 names with each selected case once, Retail integration lists 10,681 with all four absent. These counts are a new epoch, not replacements for historical sealed counts. Fresh single-fixture parent processes preserve one-shot global bytecode sealing; no reset, normalization or callback framework. Original constructors, bodies and gates remain unchanged. Non-Retail execution is UNVERIFIED; no speedup or parent completion credit. Full suite `unitfull-suite-1791563703.service` completed FAIL; proof SSOT records unchanged failure sets and saved-log execution of all four migrated cases, reported separately from the base-group summary. Bounded proof is not full-suite acceptance.

#### Later saved original-group executions — default Retail956

[Accepted saved956 suite](../wiki/investigations/fullsuite-956-2026-10-10.md) remains overall FAIL. Its saved log lines13983–13999 record the original chat pair2/2PASS, cast-bar1/1PASS and Mainline spellbook1/1PASS in the default Retail build. Thus matching-profile spellbook execution is retained, not absent. The enclosing prefork step exits1 because a different Game-group case fails; four auxiliary passes do not make the target pass. This later source epoch supplements historical3de evidence without replacing original constructors/assertions or proving every cache/process/native invariant. Later scalar/export changes and pending714 suite are separate scopes.

[Mists222248 diagnostic](../wiki/investigations/mists-cast-startup-visibility.md#temporary-simulator-trace--original-assertion-still-fail) executes only the original cast case0PASS/1FAIL; older084353 chat2PASS remain a distinct receipt. Later cast assertions stay unreached; no all-profile/current-source acceptance follows.

#### Original Retail four-case checkpoint — 2026-10-13 (bounded)

[Public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/original-retail-prefork-2026-10-13/README.md) retains the exact four-selector matrix and [independent report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/original-retail-prefork-2026-10-13/independent/report.md): original chat2/cast-bar1/spellbook1 PASS, actual wait exits0/0/0 at recorded compiled `a6723200be40145ec352666b538998bf4035a608`, production `50f6f0ea8`. Full captured runtime stderr has no observed failure diagnostics; compiler retains six inherited manifest deprecations. Selected retention is 32 complete TEXT payloads/104,446 public bytes; Current bounded retention PASS after `4308beb6d` historical-path normalization; original verifier FAIL preserved. Prior 32 TEXT transforms/104,446 public bytes, 20 JSON equivalents and 37 links reused without rehash. Local saved followup: `verification/original4-retention/privacy-followup.md`. Saved evidence only: no new body audit, conformance or cache-equality credit. Historical RED/Mists cast-bar FAIL, all-profile, performance, native GUI and full-parent limits remain unchanged; no requirement boxes closed by this checkpoint.

#### Current original Retail four-case checkpoint (bounded)

[Current 14-text evidence packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/current-original-retail-four-2026-10-14/README.md) retains the aggregate/cast reports and complete new chat/spellbook bundles with literal-only privacy transformations. Independent copy/privacy gate passed at `2585517f4`: 14 whole matches/40,061 bytes/11 literal rules, zero privacy findings and 16 public Git/stat receipts. Bounded retention only; historical packets and wider acceptance remain separate.

Accepted private saved-artifact reports `current-original4-current/report.md` and `current-original-cast-bar-current/report.md` record **4 distinct PASS / 0 FAIL / 4 total**: chat2/cast-bar1/spellbook1 at compiled `1b707b28b16eb291e36180b2a12ac2f0411264af`, reusing snapshot `6bdb02b06`. Docs-only runtime/test equivalence is accepted source-scope metadata, not renewed byte authentication. Exact-group dispatch uses the original constructors and assertion bodies, not the default completed-Game preload/full registry. No current `85d8` Store proof is supplied.

Chat proves click/focus, keyboard submission of one SAY message, cleared editbox and white/alpha assertions; cast-bar proves Retail lock/anchor/parent transitions; spellbook proves S-key show/hide and an empty interaction-window error list. These are not pixel, real-network, casting-duration or Mists assertion proofs. Supplied full runtime streams have no observed explicit failure diagnostics, but chat/cast constructors continue or discard load/event results and spellbook clears earlier errors. No comprehensive startup-error freedom follows.

Unchanged observed cache/manifest/marker stat fields and marker text belong only to the accepted cast-bar report, not chat/spellbook or byte/descendant equivalence. Conformance invocation wiring and successful aggregate execution are not case-level conformance diagnostics. Historical original4 evidence remains historical, not additional current cases. Full registry/preload, complete conformance, native/all-profile and general parent acceptance remain open; no requirement boxes close.

### Saved suite983 registry comparison — 2026-10-14 (bounded)

[Repair matrix](../wiki/investigations/fullsuite-983f02-2026-10-14.md#bounded-repair-follow-up) records exact saved copied1b94 list2328 minus four appended original chat2/cast-bar1/spellbook1 fixtures = historical observed primary2324, including set and multiplicity equality. This closes only that comparison, not preload execution, original intended full-source completeness or historical binary authentication; historical suite983 FAIL and19 unknown integration skips remain. The separate unchanged normal-preload marker passes1/1 through the real parent/fork route, establishing UIParent/WorldFrame presence and1024×768 queries. It does not assert complete addon sequence, event order, render/native behavior or all-case acceptance.

### Retail and Mists target eligibility

- [ ] Admit `prefork_full_ui` for the Retail and Mists client bundles without admitting other profiles through their bundles.
- [x] Reject the exact unsupported manual combination `--no-default-features --features profile-retail,retail-12-0-0,prefork-full-ui` without `client-retail` or `client-mists`; capability admission must not imply profile support. Other unsupported combinations remain unverified.
- [ ] Preserve Retail's default full-UI registry/preload and exact fixture groups.
- [x] Under Mists, list and dispatch only the original chat pair when GUI is enabled and the original cast-bar case; do not run Retail full-UI preload or registry checks. Saved GUI-enabled Mists compile/list/original dispatch proof below; this does not imply all original bodies pass.
- [ ] Preserve each original constructor, assertion, process isolation and cache contract. Listing/admission is not original-case execution proof.

Current source uses the `prefork-full-ui` Cargo capability because `required-features` is conjunctive, not a profile OR. Retail-only conformance checks remain enabled for Retail; Mists checks its complete public list against its original fixture names. Cargo admission RED at `verification/mists-prefork-admission-red/20261010T083406Z` exits101 because the old target requires `client-retail`. Epoch `20261010T083809Z` compiles and lists the three GUI-enabled Mists names, but original execution is blocked by conformance's obsolete extra Retail-group count (17/18 conformance cases pass). The count now includes that group only for Retail; original constructors/assertions remain unchanged. Saved post-fix epoch `20261010T084353Z`, source `845bea6d5`, records Cargo compile exit0 and captured-source equality; public listing contains exactly the three original names. Each original chat selector actually runs one case, PASS exit0. After conformance passes, the original cast-bar body actually runs one case and FAILS exit1: `Mists startup cast bar should remain hidden`. Cache before/after is equal across all captured paths (`cache_equal=true`, no changed paths). Evidence: `<PRIVATE_VERIFICATION_ROOT>/mists-prefork-admission-fixed/20261010T084353Z/{submission.json,outcome.json,cargo-result.json,execution/results.json,execution/*.stdout}`. This is not 3/3, clean-startup or native proof. Preserved Retail runtime remains UNPROVEN; broader admission and original-fixture/cache contracts stay open. Main investigates the cast-bar root cause. Historical Retail epochs above remain separate.

#### Manual capability rejection — expected negative compile (2026-10-10)

Independent audit `<PRIVATE_VERIFICATION_ROOT>/prefork-manual-negative-current/independent-report.md` inspected completed receipts at `<PRIVATE_VERIFICATION_ROOT>/prefork-manual-negative-current/20261010T172442Z`, submission revision `9635468d994be53d1f3571a8005115a49864c18a`. Exact command: `/usr/bin/cargo check --test prefork_full_ui --offline --locked --no-default-features --features profile-retail,retail-12-0-0,prefork-full-ui --message-format=json --timings -j 12`. This valid Retail/interface-120000 profile lacks both supported client markers. The fresh library artifact reports exactly `aura-instance-enumeration`, `prefork-full-ui`, `profile-retail`, `retail-12-0-0`; its package/manifest match the `prefork_full_ui` test diagnostic. Exactly one error reaches `<REPO>/tests/prefork_full_ui.rs:4:1`: `prefork_full_ui requires client-retail or client-mists`. No missing-profile error; `build-finished.success=false`, check exit101. Expected negative acceptance, **not a test pass**; no runtime execution.

Retained small evidence: `<REPO>/data/test-perf/evidence/prefork-exact-fixtures-2026-10-09/manual-capability-rejection/{diagnostics-sanitized.json,outcome.json,source-seal-summary.json,independent-audit.sha256}`. Source-before/after maps are identical (23,691 selected tracked entries); summary retains snapshot and audited-source hashes. Independent-report SHA-256: `52be0900351669ce84e7e77c18c25e4e98b3b78b13f4bbe599e499d1e9f02c07`. Nine compiler warnings and six manifest deprecations prevent warning-clean credit. Seal scope excludes GUI-gated source, untracked files, external dependency provenance, inherited environment and runtime assets. This closes only the exact manual rejection: no supported-client success, behavioral GREEN, native parity or all-profile proof; original Mists cast failure and current Retail runtime scopes are unchanged. No build/check repeated for this documentation update.

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
- [x] Verify commit `2995da11d` after moving eight equivalent `chat_frame::test_chat_*` cases: 8/8 pass, integration lists 8,385 cases, prefork lists 1,992, and overlap is zero. The 13.1985s-to-6.326s runtime change is directional only because power mode changed. At this historical epoch, keep `test_chat_editbox_click_type_and_submit` and `test_chat_editbox_text_color_after_activation` in ordinary integration because complete startup changes their asserted initial channel and edit-box color; the later exact-fixture epoch above supersedes their placement, not these counts.
- [x] Verify commit `465acd9da` after moving the two equivalent GroupFinder post-start behaviors: 2/2 pass, integration lists 8,383 cases, prefork lists 1,994, and overlap is zero. The 4.224s integration baseline versus 6.245s standalone prefork run does not establish a speedup because shared preload is included.
- [x] Verify commit `279bbb8d5` after moving four equivalent WorldMap/GameMenu behaviors: 4/4 pass, integration lists 8,379 cases, prefork lists 1,998, and overlap is zero. The 4.325s integration baseline versus 5.703s standalone prefork run includes shared preload and does not establish a speedup.
- [x] Verify commit `2c0441c22` after moving seven equivalent Collection/Escape behaviors: five Wardrobe surface/interaction cases plus Collections Journal and Adventure Guide Escape-close cases. Independent proof records 7/7 pass, integration 8,372, prefork 2,005, and zero overlap. The 14.840s-to-5.799s runtime change is directional only because power mode changed after the baseline.
- [x] Verify commit `a6924a8c9` after moving six completed-startup player, action-bar, dropdown, and chat-config behaviors: 6/6 pass, integration 8,366, prefork 2,011, and zero overlap. Keep pre-start channel-checkbox instrumentation in ordinary integration. Six concurrent integration processes took 12.5426s versus one prefork process at 8.4078s (33.0% directional because methodology differs). `cargo fmt --check` and `cargo check` passed; protected hash matched.

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
- `crates/test-harness/` — internal unpublished workspace crate, used only as a root dev-dependency. Its build script owns automatic module/marker generation and test-source watches; `syn`/`quote` are its build dependencies. Exported macros emit module items in both test entry points and the caller-resolved Retail registry expression in `GENERATED_FULL_UI_CASES`.
- `build.rs` — production-only compiler-revision/Cargo.lock, optional native ICU, and Windows resource responsibilities; no test generation or test-source watches. An explicit workspace-wide build may still select the helper independently.
- `scripts/build-host.py` — includes `crates/` in native snapshots; Cargo requires the helper workspace manifest even when only a production binary is selected.
- `Dockerfile` — copies the helper crate before dependency fetching, without copying the test tree into that layer. [Staged Cargo metadata proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/dev-harness-build-inputs-2026-10-11/README.md) covers workspace resolution, not container/native build acceptance.
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

- [x] Source audit at `256e54ef1`: confirmed internal dev-only crate placement, dependency boundary, and removal of root test generation/watches. Copied discovery/parser/render bodies and original fixture inputs are unchanged apart from root lookup and macro envelopes; this is not compiled expansion or cache acceptance.
- [ ] Native-host and wider fixture/profile acceptance remain open. Retained compile/list, 13 selected cases, two original bodies, one temporary new-input control, stock startup, generated-byte parity and timestamp-cache proof have distinct bounded scopes; no full-suite or parent completion follows.

- [x] Migrate the initial nine `test_keybindings_panels_detail` `WowLuaEnv` cases onto the reusable runner.
- [x] Add the two behavioral-messaging cases and one nested registry/preloaded-startup fixture.
- [x] Benchmark the parent-bypass prefork execution against the retained current in-target baseline.
- [x] Migrate every eligible normal-retail full-environment test and complete the final 304-test ordinary startup-like eligibility audit.
- [x] Fixture-only verification for `d7cec2b25`: the positive wrapped-click fixture uses `SecureHandlerBaseTemplate`, because vendor `GetFrameHandle(self, true)` requires explicit protection. The retained plain-button negative preserves original click behavior while restricted prebody does not run; caller taint and header identity remain asserted. A 286.7s non-incremental target build and `tainted_addon_secure_handler` 2/2 pass, with format/check. This does not establish broader retail completion.

## Out of scope

- Non-Linux prefork support, because the contract depends on Linux process and `/proc` semantics.
- General runtime read-only behavior beyond Lua bytecode-cache mutation suppression in explicitly configured forked test children.
- Compatibility fallbacks for unsupported custom-runner arguments.
