# Independent five-profile compile source audit

Date: 2026-10-10. Scope: Wrath, Mists, Era, Anniversary, WowForever; compile-only compatibility of the shared field and constructor. Finalization uses saved receipts under `<private-state>/verification/five-profile-compile-current/20261010T201438Z` and the existing source audit below. No Cargo/check/build/test execution, operations, delegation, repository edits, waits, or polling performed. Default/PTR checks (097) remain retained proof only; not repeated or reclassified.

## Verdict

**SOURCE: PASS. BOUNDED FIVE-PROFILE COMPILE: PASS (5/5 exit 0). RUNTIME/NATIVE/CACHE: NOT VERIFIED.** `outcome.json` records `status: COMPLETED`, `source_equal: true`; its five results agree with `results.json` and the individual result receipts. No compile receipt is pending or missing. This closes only the bounded compiler gate, not the entire Goal.

## Source evidence

- Actual representation: `<repository>/src/lua_api/state/sim_state.rs:575`, `pub map_display_hide_icons: HashMap<i32, bool>`. No feature gate on this field. No separate MapSimState type or invented field identifier used in this audit.
- Actual initializer: `<repository>/src/lua_api/state.rs:15` defines `build_empty_sim_state!`; line 494 contains unconditional `map_display_hide_icons: HashMap::new()`. `Default for SimState` (line 940) calls `new_empty`; `new_empty` (line 970) calls `build_empty_state`; `build_empty_state` (line 974) expands this macro at line 975. Neither the initializer nor this construction chain is profile-gated. All five profiles share this field/template at source level. The saved five successful compiler receipts now establish that the shared field and unconditional initializer compile under all five submitted profile feature sets.
- `<repository>/src/c_api/c_map.rs:75-76` gates registration of `GetMapDisplayInfo` on `retail-12-1-0`. The selector `read_map_display_id` (line 192) and implementation `c_map_get_map_display_info` (line 210) carry the same gate. The implementation reads `map_display_hide_icons` (line 214).
- `Cargo.toml` feature definitions: Wrath/Era/Anniversary have empty bundles; Mists enables only `prefork-full-ui`; Forever enables named shared capabilities but no retail epoch. With worker `--no-default-features --features sound,gui,casc,client-<profile>`, none of these five bundles transitively enables `retail-12-1-0`. Sound/GUI/CASC do not enable a retail epoch. Thus current registration, selector, and getter are excluded for all five, while the shared field and initializer remain included.

| Profile | Check exit | Source equal | Individual warnings | Manifest summary | Shared field/initializer | Current getter/selector |
|---|---:|---|---:|---|---|---|
| Wrath | 0 | true | 6 | generated 6 warnings | Compiled | Excluded |
| Mists | 0 | true | 6 | generated 6 warnings | Compiled | Excluded |
| Era | 0 | true | 6 | generated 6 warnings | Compiled | Excluded |
| Anniversary | 0 | true | 6 | generated 6 warnings | Compiled | Excluded |
| WowForever | 0 | true | 6 | generated 6 warnings | Compiled | Excluded |

Each `<profile>.stderr` contains six individual `iced-wgpu-patched/Cargo.toml` deprecated Clippy lint-name warnings, followed by one `iced_wgpu` manifest summary and a successful `Finished` line. Count: 6 diagnostics per profile, 30 across five logs; 7 `warning:` lines per profile if summary lines are included (35 total). No Rust source warning appears in these logs. Each stdout is empty; all receipts report `command_performed: true` and `stream_errors: []`. The current `GetMapDisplayInfo` implementation is NOT compiled by these older/non-retail feature sets; these checks prove shared state construction, not current getter behavior.

## Hash scope

Saved `source-before.json` and `source-after.json` contain identical mappings of 3,853 tracked source/config entries. Each profile receipt and final outcome also records `source_equal: true`. Submission revision: `63f3a8a7373c8e52ffcc005bb443a9cf64b59fbf`. The explicit C API wiring, map implementation, map field/constructor inputs, and feature-selection hashes below are equal in both saved snapshots. This is saved worker input equality, not a live repository check, whole-environment freeze, or external dependency/runtime provenance claim.

| File (repository-relative) | SHA-256 |
|---|---|
| src/lua_api/state/sim_state.rs | 5da06c389ae6441ffbbe45c9acea4bba0f10bf70c710e21b03d62b6553f4d5cf |
| src/lua_api/state.rs | 798a0c52f09e0968e6f6ef02839d598786dc5c8d06d0dceb14e4c74a408ef473 |
| src/c_api/mod.rs | 2b7241f282613f370e1b726bce309099638dc2d2688b342e7579e8052a8d9798 |
| src/c_api/c_map.rs | 2cd9313d7540696840e5b1c90813ea52d85e54bb13fd07859ba8c127fa0bd95c |
| Cargo.lock | 859f258fa21eb92f7f0c83c77d526ea0ce3e7535b0970df2e39c459023c0b9cf |
| Cargo.toml | f439cb619c2bff21706e7c0f175cfb63701a0383c1dac732b2308ef759285961 |
| src/client_profile.rs | 8f603ce85c8b4d67f2fdbc15194d47f0d3365f0c96d7dd85ce21e3a893554839 |

## Worker/resource policy

Inspected `worker.py` and submitted request. Worker serializes the five profiles; each command is `/usr/bin/cargo check --offline --locked --no-default-features --features sound,gui,casc,client-<profile> -j 12`. Admission requires MemAvailable >= 6 GiB, load1 < 24, and no cargo process whose cwd resolves to the canonical repository. Five bounded attempts use capped exponential delay plus jitter. Worker hashes tracked source/config before checks and compares after each, records stream errors and exit status, and stops on drift/invalid proof. Exclusions explicitly include untracked files, inherited environment, external dependency provenance, and runtime assets.

`queue-submission.json` records launch exit 0 with `agents.slice`, `MemoryMax=16G`, `CPUQuota=1200%`, Type=oneshot, and no-block. This proves requested cgroup limits/submission, not live enforcement: no service/process inspection was authorized or performed.

## Saved receipt disposition

All five result receipts, five stderr logs, both source snapshots, version receipts, `results.json`, and `outcome.json` are present. Final worker completion: `2026-10-10T20:17:16.188044+00:00` (WowForever result). No pending compiler evidence remains.

Saved versions (both exit 0, empty stderr):

- Cargo: `cargo 1.99.0 (5f94df478 2026-08-27) (Arch Linux rust 1:1.99.0-1)`.
- Rustc: `rustc 1.99.0 (b940084d7 2026-09-28) (Arch Linux rust 1:1.99.0-1)`.

The saved rustc receipt records `--version`, not `-Vv`; verbose compiler/host metadata is not supplied and was not newly queried.

Inspection limitation: the initial file-only batch output truncated the large snapshots and aggregate receipts. Saved tool output preserved the individual receipts/logs/versions; a bounded JSON projection reread the aggregate receipts and snapshots to recover their fields and explicit hashes. Thus finalization was not strictly a single read of every artifact. No worker polling, waiting, command rerun, or live source check occurred.

## Combined proof scope

Default/PTR checks (097) are retained prior scope-check proof only, alongside the now-completed five-profile compiler receipts. The specified saved set does not provide new Default/PTR receipts or their exact counts; none are invented or promoted to current acceptance. No rerun occurred.

These five checks compile shared state/construction with the recorded profile flags. The existing source audit establishes unconditional `map_display_hide_icons` initialization for all five profiles and exclusion of the current gated getter. They do not load runtime caches, exercise that getter, validate native selector semantics, prove current runtime/native/cache acceptance, or complete the entire Goal.
