# Saved compiler epoch audit

PASS — compiler evidence only; no reruns.

- Epoch: `20261010T013017Z`; source `fc824cb921342c06f506eb92e8c81d52bdf28fcd`.
- Cargo exit `0`, `--no-run`; started 2026-10-10T01:30:17.737623+00:00, ended 2026-10-10T01:32:43.678780+00:00. No test execution claimed.
- Full stdout read/parsed: 740 JSON records, 665 compiler-artifact, 74 build-script-executed, one terminal build-finished success=true; zero compiler-message records.
- Full stderr read: six vendor manifest warnings, one six-warning summary; no own-code warnings/errors observed.
- All 3839 source snapshot entries equal; complete snapshot file bytes equal. Seven scoped Git files match both epoch snapshots.
- Current HEAD `a07406603aff8548c383a13e2a63c84c42d82bd2` is separate from compiler revision. Changed paths: `docs/addon-loading-pipeline.md`, `docs/event-system.md`, `docs/on-update-dirty-handlers.md`, `docs/wiki/index.md`, `docs/wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md`, `docs/wiki/log.md`.

## Provenance and resource ordering

Queue 2026-10-10T01:10:47.887859+00:00 invokes build-lock.sh before pyrun-jsonl; controller stderr records waiting (20:10) then acquired (20:30). Worker resource measurement 2026-10-10T01:30:17.670751+00:00 precedes submission 2026-10-10T01:30:17.737623+00:00: available 39.456112 GiB, load 2.67, lock_already_held=true. Current wrapper holds fd 9 flock while invoking worker; worker reads resources before cargo spawn. This proves recorded invocation/order, not independent historical lock-state attestation.

Worker SHA-256 `9db13d63edd8d5d45b2eff0d970f98415b070f4af0ee0e03620db923cc5a836a` matches queue/submission. Request SHA-256 `fec082a6d15cd6366de36c3cb6691d8581e29a7c2a1247edf9fe1bf70f182054` matches queue; request executes inspected worker. Controller completion names this epoch. Boot IDs match; no stream-errors file.

## Check ledger

- PASS: cargo_exit_zero
- PASS: source_full_mapping_equal
- PASS: source_full_raw_bytes_equal
- PASS: source_equal_record_true
- PASS: revision_expected
- PASS: worker_hash_matches_submission_and_queue
- PASS: request_hash_matches_queue
- PASS: resource_thresholds_pass
- PASS: resources_before_cargo_after_queued_lock_wrapper
- PASS: complete_json_terminal_build_finished_success
- PASS: no_compiler_messages
- PASS: six_vendor_warnings
- PASS: scoped_git_bytes_equal
- PASS: selected_artifacts_records_and_hashes_match
- PASS: no_stream_errors_file
- PASS: same_boot_id
- PASS: argv_cwd_consistent

## Scoped Git byte ledger

| Path | Bytes | SHA-256 | Result |
|---|---:|---|---|
| `src/bin/wow_sim/addon_loading/warning_report.rs` | 5700 | `a30eaa6bfbd06b4527ee4e25da2530bab31740732818824594333cb1b4121d8a` | PASS |
| `src/lua_api/execution_budget.rs` | 8579 | `861975f99c721d58d956ab0340da79f27cfab6a49375e42752de8ac82a24bfd5` | PASS |
| `src/loader/lua_file.rs` | 22380 | `7df176ef4608718ecf8224af32c4c4cd93d95ccd9c22a9a75a9d5dd9649a17d4` | PASS |
| `tests/tooltip_text_layout.rs` | 7271 | `90b9b71c8b087eea1d3b181166ad4289bbd5f855d4afc3df4d3828041bb5c287` | PASS |
| `tests/utility_api.rs` | 24908 | `65a27193e474479fd489e0fe69a3e9962a00d19720f8292d06c4d7ce0b9e36ad` | PASS |
| `tests/spell_api.rs` | 15738 | `48f695e9839f06579e0341dd7ea5dd8f6363b15733e0fbb907dcb529a923dac9` | PASS |
| `tests/wowforever_table.rs` | 2432 | `4c7ce90399dbb2e9cc6e3080b451b99fcf707b3a764e1348935596fa25bd3ea7` | PASS |

Individual captured `git show <revision>:<path>` calls only; no git cat-file --batch.

## Executable artifact ledger

### wow_ui_sim-test
- `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`
- Target kind ['lib']; test=True; fresh=False; profile `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": true}`.
- SHA-256 `cbde2b9ad3ae7a0d832de6f4fea679f05d1b2f31384c4257f76bac8f024c5fdf` matches saved selection; exact record present in complete stdout.
- Observed size 390515376 bytes, mode 0o755, mtime_ns 1791595942808503382; executable bit present.
### wow-sim-normal
- `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`
- Target kind ['bin']; test=False; fresh=False; profile `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": false}`.
- SHA-256 `4c4730adbd8bc1c4b92988b138d6f941e325ffe7207c64ba73db264548fc8ecc` matches saved selection; exact record present in complete stdout.
- Observed size 407116480 bytes, mode 0o755, mtime_ns 1791595943717518771; executable bit present.
### wow-sim-test
- `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_sim-e99132585a38db9e`
- Target kind ['bin']; test=True; fresh=False; profile `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": true}`.
- SHA-256 `aa180a5b174638c27f23abff15db2eadde0f7d7ef44ff096ad4a9aa8fab37759` matches saved selection; exact record present in complete stdout.
- Observed size 351865536 bytes, mode 0o755, mtime_ns 1791595947322579876; executable bit present.
### integration-test
- `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`
- Target kind ['test']; test=True; fresh=False; profile `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": true}`.
- SHA-256 `d592e1f1fd077b623e01b6491d260a249a1c32d6c6b4a9bf89ed5cd1a863258b` matches saved selection; exact record present in complete stdout.
- Observed size 466978472 bytes, mode 0o755, mtime_ns 1791595963477855089; executable bit present.

Selected lib unit-test, integration test, wow-sim unit-test and normal wow-sim artifacts are covered. Complete target metadata/features/filenames and input hashes are in `scoped-manifest.json`; selected features include client-retail/default, not other client profiles.

## Complete compiler stderr

```text
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
warning: `iced_wgpu` (manifest) generated 6 warnings
   Compiling wow-ui-sim v0.1.0 (/home/osso/Projects/wow/wow-ui-sim)
    Finished `test` profile [optimized + debuginfo] target(s) in 2m 25s
```

## Boundary

Compilation proof only for saved epoch and seven scoped source files at fc824cb92. Source-before/after equality covers the entire captured tracked src/tests/build/config mapping, not all repository inputs or all Git byte identity. Artifacts are hashes verified at audit time, not evidence tests passed. Exact external path dependencies, native provenance, inherited environment, excluded data/cache/addon inputs and current/all-profile builds are not established. Current executable metadata was not recorded historically; current lock-wrapper bytes were not sealed in submission. Main-owned 14 exact cases and fresh startup remain separate; neither was run here. No backend, delegation, operations or raw publication.

## Normal library artifact

Normal `wow_ui_sim` library record: test=false, fresh=false; profile `{"debug_assertions": true, "debuginfo": "line-tables-only", "opt_level": "1", "overflow_checks": true, "test": false}`. JSON identifies both rlib and rmeta. Current metadata/hashes below are observations, not historical hash matches (selection did not seal these files).

- `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/libwow_ui_sim-3282a118e07cdd18.rlib`: 429984524 bytes, mode 0o644, mtime_ns 1791595905204873292, SHA-256 `975086ce5be2e13010322efc2c62e0b4ff0933d856e409281d6625b451570c06`.
- `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/libwow_ui_sim-3282a118e07cdd18.rmeta`: 135447254 bytes, mode 0o644, mtime_ns 1791595833674818701, SHA-256 `b3dc2a960f407f8e968fe653ddb997c688f4a9a2302c0ac2db30d7c4bfaab50c`.
