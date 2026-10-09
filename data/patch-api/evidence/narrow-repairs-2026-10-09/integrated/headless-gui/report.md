# Headless / GUI independent verification — PARTIAL (GUI awaiting serialization)

## Scope and source epochs
Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Requested test-only correction: `4163299fac218bcd84057897cc4ffdc81a80e42a`.
Read/followed verify and rust-readability skills as assigned verifier; no delegation. Actual HEAD before/after formatting, historical compile and both historical executions is this revision; final metadata operations may observe later docs/SOURCE commits. Compile scope aggregate SHA256: `eedb93291c0f362c8e942008ef6c385d891a916571e390a8752193534be78cd3`; per-file hashes and command-local times/revisions in `*.json` and `proof-ledger.json`. Scope covers tracked src/tests/crates/iced-* and .cargo/root Cargo.toml/Cargo.lock/build.rs, not every external cache/registry or untracked file.

Concurrent docs/SOURCE/evidence epochs are separate from compile-source scope: `revision-changes.stdout` and command snapshots preserve observed revisions; no universal immutable-checkout claim. Initial status has only existing untracked `.code-index.db`.

## Proof matrix
| Gate | Result |
|---|---|
| `cargo fmt --check`, once after changed block | PASS exit 0; empty full streams |
| Historical headless lib compile, once warm dedicated target | PASS exit 0; build-finished success=true |
| Historical Transmog exact execution | PASS 1 passed / 0 failed; 1653 filtered out |
| Historical headless hit-order exact execution | PASS 1 passed / 0 failed; 1653 filtered out |
| Default Retail GUI lib compile | NOT STARTED — active external compilers at preflight |
| Default GUI hit-order exact execution | NOT RUN — fresh artifact pending |
| Fresh default historical absence listing | NOT RUN — fresh artifact pending; never execute zero matches |
| Current weather/time integration | Prior retained 1/1 proof remains applicable; not rerun |

Exact commands/cwd/times/exit codes and all streams are retained. Builds use offline/locked/jobs4 and no timeout; test binaries use 120-second execution bounds. No broad suite/checks/default integrations/other profiles.

Historical binary: `/home/osso/Projects/wow/wow-ui-sim/target/transmog-epoch-proof/debug/deps/wow_ui_sim-5da96163b92cdc95`.
SHA256: `b51dc5c060b5971140325378a4ac2bc2cee7165315806e3ac22f6110aec85360`; identical before/after both executions (`*-artifact-hash.json`). Features: aura-instance-enumeration, profile-retail, retail-12-0-0. Full Cargo JSON artifact metadata in `historical-compile-inspection.json`.

## Correction and readability
`repair-diff.stdout`: only existing GUI hit-grid block gained `#[cfg(feature = "gui")]` and lexical braces. Common setup and ID-order assertion remain outside it. Normalizing only gate/braces/indentation reproduces the previous test body byte-for-byte (`readability-and-boundaries.json`); GUI point and both topmost assertions unchanged. This is read-only comparison, not a source-shape regression test. Production render/layout untouched by correction.

Historical Transmog module byte-identical to f2eebd359; exact 12.0.0 gate unchanged; all 22 members and metadata MaxValue=21/MinValue=0/NumValues=22 unchanged. Manual changed-line/surrounding-test readability audit: no introduced violations. Metrics tools unavailable; no measured complexity claim. No readability edits/suppressions.

## Full warnings preserved
Historical compile produced 12 warning compiler-message records, zero errors, no unparsed JSON lines. Four unused imports; dead-code warnings for remove_missing_marker, read_custom_set_items, ensure_known_asset_cached, aura_matches_filter_string, find_uncached_talent_icons, check_icon_cached, Frame cooldown helpers; unread encoding_key_hex field. Six iced-wgpu manifest lint-name deprecation warnings, plus aggregate summary line. Full rendered warnings with exact file:line in `historical-compile-diagnostics.txt/json`; full stderr and complete Cargo stdout retained. No warning-free claim and no warning changes.

## Preservation / incidents / limits
Original partial report and integrated transmog-epoch evidence preserved; read-only hashes in `prior-original-report-hash.json` and `preserved-prior-evidence-hashes.json` (49 files). Current weather/time proof not invalidated by this test-only edit; no new integration execution.

Initial metadata capture incident: `combined_to_file` executed git rev-parse and returned CommandResult; appended `.run()` raised AttributeError. Saved full initial HEAD stream, missing exit explicitly recorded in `capture-incident.json`; no build launched, no command rerun to recover logs. Subsequent commands use self-contained terminal `.capture().run()` and immediately persist exit/full streams. No command auto-backgrounded or build aborted.

External game-engine cargo/rustc processes initially blocked builds. Historical preflight was clear, historical compile completed 2026-10-09T19:12:02.247275+00:00–2026-10-09T19:13:52.811223+00:00; default preflight found a new external cargo build, so default compile was not launched. Preflight snapshots do not prove continuous host exclusivity during historical compilation; resumed external builds are a host coordination limitation. Hold requests sent to world-of-osso session; no cancellations, service starts, agent CLI or delegation. Full process observations retained in `build-wait-*.stdout` and preflight files.

User-reported host OOM around 17:54 killed browser/sibling game server. Distinct from prior concrete three E0433 GUI-reference compile errors; historical fresh compile now succeeds, without attributing those errors to OOM.

Environment key names/patterns inspected in `environment-keys.json`; no environment values retained. No code/cache/vendor edits, commits, push/deploy/network or new GUI/host services. Cargo-produced target artifacts only. No native parity, full parent goal, full suite or runtime acceptance claim.

## Final bounded status

Default GUI verification remains BLOCKED by continuing external cargo/rustc processes after multiple clear-window requests. Latest snapshot: `build-wait-19.stdout` at 2026-10-09T19:19:15Z, four compiler processes. Neither default compile nor default artifact execution/listing launched; no build rerun or zero-match execution. This is a coordination blocker, not a compile failure. Historical compile/test evidence remains valid at final source scope.

Final actual HEAD: `679a80f58be973a174a56de29107c32d59245a12`; final compile-source SHA256: `eedb93291c0f362c8e942008ef6c385d891a916571e390a8752193534be78cd3`. `final-epoch-diff.stdout` separately records concurrent committed docs/SOURCE changes (empty if none). `preservation-final.json`: {"integrated_unchanged": true, "original_partial_unchanged": true}.

Planned, unexecuted default command: `cargo test --offline --locked --jobs 4 --lib --no-run --message-format=json`. After one successful fresh compile, run only exact GUI hit-order filter (expected 1/1) and exact historical `--list` if required to establish absence for the new artifact. No readiness claim for these pending gates.
