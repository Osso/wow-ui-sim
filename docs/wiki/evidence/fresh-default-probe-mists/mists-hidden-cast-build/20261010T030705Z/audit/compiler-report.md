# Compiler epoch audit: mists-hidden-cast-build/20261010T030705Z

**Compiler: PASS. Captured-scope equality: PASS. Historical artifact byte binding: UNPROVEN.**

## Exact acceptance boundary
- Recorded revision: `903711da8b9d6472f718e407af350c576fe4a7e0`. Revision comes from submission, not HEAD; this audit does not independently match the saved hashes to a Git commit tree.
- Purpose: Compile committed Mists hidden-cast fixture, ordinary chat/cast controls and eight Mists profile targets; not original prefork proof.
- Saved command: `["/usr/bin/cargo", "test", "--offline", "--locked", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--test", "pandaria_installed_addons", "--test", "mists_compat_bootstrap", "--test", "mists_forbidden_frames", "--test", "mists_world_map_opacity", "--test", "mists_nameplate_scale", "--test", "mists_currency_list", "--test", "mists_dialog_helpers", "--test", "mists_class_colors", "--test", "integration", "--no-run", "--message-format=json"]`.
- Captured cwd: `/home/osso/Projects/wow/wow-ui-sim`. Start `2026-10-10T03:07:05.146660+00:00`; compiler end `2026-10-10T03:10:05.798125+00:00`; outcome end `2026-10-10T03:10:05.842850+00:00`.
- Client feature: **client-mists**. Cargo profile: `test`. Resolved features: `casc, client-mists, gui, rodio, sound`.
- Selected artifact profile: `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": true}`.
- Acceptance is compilation only (`--no-run`), not passed tests, runtime compatibility, original prefork proof, or the queued final Mists check. No docs HEAD or future/current source is promoted to this epoch.

## Full saved evidence audit
- Read all eight saved evidence files, every compiler JSON record and diagnostic, full stderr, submission/resources, and both complete source hash snapshots. Exact evidence SHA-256/byte counts are sealed in `scoped-manifest.json` and `hash-report.json`.
- Cargo result exit `0`; outcome exit `0`; one terminal `build-finished` with `success=true`. Submission/result argv and cwd agree. Selected test target coverage agrees with submitted scope: `True`.
- Compiler JSON: 747 records: `{"build-finished": 1, "build-script-executed": 74, "compiler-artifact": 671, "compiler-message": 1}`; all nonblank lines parse as JSON. Source snapshots: 3839 entries each; parsed equality `True` and byte equality `True`; saved `source_equal=true` independently confirmed.
- Captured scope counts: `{".cargo": 1, "Cargo.lock": 1, "Cargo.toml": 1, "build.rs": 1, "src": 1653, "tests": 2182}`. Complete paths/digests preserved in scoped manifest; no source payload reproduced.
- Resource record: `{"available_gib": 36.579429626464844, "load_one": 9.36, "lock_already_held": true, "observed": "2026-10-10T03:07:05.089054+00:00"}`. Environment override: `CARGO_BUILD_JOBS=4`. Recorded boot identity unchanged: `True`. These are submission observations, not hermetic resource/environment guarantees.

## Selected compiled artifacts
- `mists_currency_list`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_currency_list-43b74ef9e0f23ac4`; current SHA-256 `a8b997761eb0e71641c7f3c10872ca33066cffb7dbe8d7097c85c8242ee233d8`; bytes `340901960`; stable during read `True`; fresh `False`.
- `pandaria_installed_addons`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/pandaria_installed_addons-5c4ab2a050a21582`; current SHA-256 `2a525b412d251cc553ebb52de8ee3b504f05baa815a26fae91e34256868faecc`; bytes `238052856`; stable during read `True`; fresh `False`.
- `mists_nameplate_scale`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_nameplate_scale-7b616f0b74b8ec9e`; current SHA-256 `34e5414fa75ad6eba113acbb01252b696f9f1b194764eafa50f53c24c88bd6b5`; bytes `340925064`; stable during read `True`; fresh `False`.
- `mists_class_colors`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_class_colors-7e269e94f7d0907e`; current SHA-256 `f1510ae361527e2badd4ff4b3bb9dc08f806630675cbc044f672ccf75e5e30af`; bytes `340922048`; stable during read `True`; fresh `False`.
- `mists_dialog_helpers`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_dialog_helpers-ec9786fa01af7347`; current SHA-256 `b8e3827c1b4460c9e6b8c99eb695b89d75925c1f02c36c70695b129114c324ca`; bytes `340911112`; stable during read `True`; fresh `False`.
- `mists_world_map_opacity`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_world_map_opacity-c4e80a81b344dfca`; current SHA-256 `08f8efd79e1eb2379221eb9d6790fa77a396bae2afc59a7f75110f0280788cfc`; bytes `340893200`; stable during read `True`; fresh `False`.
- `mists_forbidden_frames`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_forbidden_frames-92ccbb08b3976d61`; current SHA-256 `d5d802a6e4ff6571fa5984a6419b17ba389d34ed1eb503deb9bb04aba96c0023`; bytes `340877952`; stable during read `True`; fresh `False`.
- `mists_compat_bootstrap`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_compat_bootstrap-c15dadd74a574f6c`; current SHA-256 `2531d3e45062eee5904704345906e9788ea1b2a8254f6480e23cc476e0d03175`; bytes `341314640`; stable during read `True`; fresh `False`.
- `integration`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-f15e4c1da9a8b8f4`; current SHA-256 `3209569e20c752c771a845422b9a598cb9736c50465b87fe03bb3bacdc7c8c20`; bytes `437996056`; stable during read `True`; fresh `False`.

All 17 project compiler-artifact records and 18 emitted file observations are sealed in the scoped manifest. Incidental executable targets: `bench_steady_state, bench_spellbook, bench_talents, wow-sim, panel-visual-metrics, wow-cli`. They do not expand the submitted test acceptance scope.
**Hash limitation:** no artifact hashes were saved at build end. Audit-time hashes identify current files only, even when stable during read; concurrent replacement before this read cannot be excluded. Main owns artifact selection/execution. This audit neither runs nor freezes binaries.

## Warnings retained
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
- Compiler JSON warning `dead_code`: function `is_enabled` is never used (src/lua_api/handler_timing.rs:12); not suppressed or reclassified as vendor warning.
- Saved stderr reports six `iced_wgpu` manifest warnings; actual vendor warning records: 6. Compiler JSON errors: 0. No warning-free claim.

## Preserved exclusions
- external path dependencies.
- data inputs outside captured tracked src/tests/build/config scope.
- inherited environment.
- runtime cache/addon files outside source scope.
- untracked index.

External dependency/data/cache contents and inherited environment remain outside captured source equality. Cargo-emitted dependency metadata is parsed but does not attest excluded dependency bytes. Scope equality is not a hermetic build claim.

## Actions
Only read/hash inspection and these audit metadata writes. No builds, tests, reruns, backend calls, delegation, runtime operations, or waiting/polling on queued work.
