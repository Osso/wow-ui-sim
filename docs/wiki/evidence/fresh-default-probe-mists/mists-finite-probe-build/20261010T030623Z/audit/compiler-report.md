# Compiler epoch audit: mists-finite-probe-build/20261010T030623Z

**Compiler: PASS. Captured-scope equality: PASS. Historical artifact byte binding: UNPROVEN.**

## Exact acceptance boundary
- Recorded revision: `9cf08121af8f0167d5ea4646ba3b4e21e6cabdcf`. Revision comes from submission, not HEAD; this audit does not independently match the saved hashes to a Git commit tree.
- Purpose: Compile ignored finite owner-budget startup binary unit probe at committed source; production budget policy unchanged.
- Saved command: `["/usr/bin/cargo", "test", "--offline", "--locked", "--bin", "wow-sim", "--no-run", "--message-format=json"]`.
- Captured cwd: `/home/osso/Projects/wow/wow-ui-sim`. Start `2026-10-10T03:06:23.218267+00:00`; compiler end `2026-10-10T03:06:45.803974+00:00`; outcome end `2026-10-10T03:06:45.848249+00:00`.
- Client feature: **client-retail**. Cargo profile: `test`. Resolved features: `aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound`.
- Selected artifact profile: `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": true}`.
- Acceptance is compilation only (`--no-run`), not passed tests, runtime compatibility, original prefork proof, or the queued final Mists check. No docs HEAD or future/current source is promoted to this epoch.

## Full saved evidence audit
- Read all eight saved evidence files, every compiler JSON record and diagnostic, full stderr, submission/resources, and both complete source hash snapshots. Exact evidence SHA-256/byte counts are sealed in `scoped-manifest.json` and `hash-report.json`.
- Cargo result exit `0`; outcome exit `0`; one terminal `build-finished` with `success=true`. Submission/result argv and cwd agree. Selected test target coverage agrees with submitted scope: `True`.
- Compiler JSON: 732 records: `{"build-finished": 1, "build-script-executed": 74, "compiler-artifact": 657}`; all nonblank lines parse as JSON. Source snapshots: 3839 entries each; parsed equality `True` and byte equality `True`; saved `source_equal=true` independently confirmed.
- Captured scope counts: `{".cargo": 1, "Cargo.lock": 1, "Cargo.toml": 1, "build.rs": 1, "src": 1653, "tests": 2182}`. Complete paths/digests preserved in scoped manifest; no source payload reproduced.
- Resource record: `{"available_gib": 35.591556549072266, "load_one": 13.83, "lock_already_held": true, "observed": "2026-10-10T03:06:23.159818+00:00"}`. Environment override: `CARGO_BUILD_JOBS=4`. Recorded boot identity unchanged: `True`. These are submission observations, not hermetic resource/environment guarantees.

## Selected compiled artifacts
- `wow-sim`: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_sim-e99132585a38db9e`; current SHA-256 `6e18e9bf5c9a357ce103c220f99351d5b0c8f80c70a0d5c64a3462c6d0542654`; bytes `366440208`; stable during read `True`; fresh `False`.

All 3 project compiler-artifact records and 4 emitted file observations are sealed in the scoped manifest. Incidental executable targets: `none`. They do not expand the submitted test acceptance scope.
**Hash limitation:** no artifact hashes were saved at build end. Audit-time hashes identify current files only, even when stable during read; concurrent replacement before this read cannot be excluded. Main owns artifact selection/execution. This audit neither runs nor freezes binaries.

## Warnings retained
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
- warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
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
