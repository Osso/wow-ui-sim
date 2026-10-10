# Independent post-event-repair stock verification — 2026-10-10

**OVERALL: PASS for the recorded normal Forever stock startup and separate Lua-error-state process. Not native, GUI, cold-state, or whole-profile acceptance.**

Read/followed `verify` skill as independent verifier. Artifact-only inspection: no builds, tests, reruns, operations, or delegation. Read repository source, receipts, compiler JSON, and full recorded streams; recomputed hashes and compared inventories. Only this requested report written. Raw error messages, private payloads, and WTF/EditMode details omitted.

## Evidence scope

- Current epoch: `20261010T192210Z` under this report's parent directory.
- Prior integration: `../forever-aura-block-event-green-current/20261010T191909Z`.
- Historical stock baseline: `../forever-stock-startup-current/20261010T184613Z`.
- Current submitted revision and observed repository HEAD: `ef55bf873ce7c9bcca3d88a3d690d447edf7a897`.
- Repair commit: `d5dcc8b5953ef1e62a49f58845919077167a4a0a`. Its implementation change is one finite-list event literal in `src/event/valid_events.rs`; accompanying spec change is not a runtime redesign.

## Proof matrix

| Recorded capability | Evidence | Result / boundary |
|---|---|---|
| Normal stock startup completes | `runtime-result.json`: exit 0; completion marker true; marker at `startup.stderr:171` | PASS for this dump-tree process |
| Actual Lua-error collection is empty | `lua-errors.stdout` is exactly `[]\n` (3 bytes), parsed as an empty array; `lua-errors-result.json`: exit 0 | PASS: 0 records, not empty/missing stdout |
| Occurrence counts agree | `lua-errors.stderr:134`: `Lua errors: 0 unique, 0 occurrence(s)`; line 136: `Unattributed Lua errors: 0 occurrence(s)` | PASS for separate Lua-errors process |
| Historical categories absent | Full current startup and Lua-errors stdout/stderr contain none of the 19 historical exact messages and no `UNIT_AURA_BLOCK_LIST_CLEARED` literal | PASS for observed streams only |
| Cached normal artifact and source scope agree | Compiler-artifact records, SHA-256, inventories, repair-commit blob comparisons below | PASS with provenance limits |

Dump-tree success is not itself proof of an empty Lua-error pool. The separate `lua-errors` process provides that proof for its own state. Worker source confirms two distinct executable invocations; no claim that both share one in-memory pool.

## Compiler/artifact provenance

Recorded build command: `/usr/bin/cargo build --bin wow-sim --no-default-features --features sound,gui,casc,client-wowforever --offline --locked --message-format=json --timings -j 12`.

`compile-result.json`: exit 0, source equality true, stream errors empty. Compiler JSON ends with `build-finished.success: true`. `compile.stderr` reports **0.34s**, plus six deprecated Clippy manifest-key warnings in `iced-wgpu-patched/Cargo.toml`; successful build is not warning-free.

**`fresh: true` means Cargo reused an up-to-date cached normal artifact, NOT that this epoch newly emitted a normal binary.** Prior GREEN integration compilation at `191909Z` contains a normal `wow-sim` compiler-artifact with `fresh: false`, the same executable path, feature set, and non-test profile. Current stock compiler-artifact has `fresh: true` at that path. Both profiles: opt level 1, line-tables-only debug info, debug assertions and overflow checks enabled, `test: false`. Thus the recorded compiler lineage supports reuse of the normal artifact previously emitted during integration compilation. Prior GREEN's sealed/hash receipt is for the integration test executable, not the normal binary; it does not independently establish the earlier normal binary's hash or uninterrupted identity between epochs.

Current normal sealed artifact:

- Original: `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`.
- Executed seal: `20261010T192210Z/wow-sim-sealed`.
- Independently recomputed SHA-256: `f90b051d13dae3980e3ab8c38a400a98c45e841486e33751c12693b5a05b79d8`, matching `artifact.json`.
- Both execution receipts report artifact unchanged.
- Exact recorded features: `aura-containers`, `aura-instance-enumeration`, `aura-xml-widgets`, `base-spell-relationships`, `casc`, `client-wowforever`, `forbidden-animation-aspects`, `forbidden-aspects`, `gui`, `native-duration-formatting`, `numeric-rule-formatters`, `on-update-modes`, `player-cast-durations`, `rodio`, `sound`, `timed-signal-maps`.

Toolchain receipts: cargo 1.99.0, rustc 1.99.0, host `x86_64-unknown-linux-gnu`; both probe exits 0. No toolchain commands rerun.

## Source/cache comparisons

Parsed inventories, rather than JSON formatting, were compared:

- Current 3853-file source-before/source-after maps identical. Prior GREEN 3853-file source maps identical internally and semantically identical to current stock maps. Their serialization differs, not their path/hash values.
- Current live inventoried source: **0 missing/hash mismatches**.
- Current 4730-file vendor cache-before/cache-after maps identical. Baseline cache map identical to current. Live cache: **0 missing/hash mismatches, 0 extra files**.
- Worker records cache-after immediately following dump-tree, before separate Lua-errors execution. It does not provide a separate historical post-Lua-errors cache snapshot. Present live cache agrees with the recorded map.
- Repair-commit blobs match current inventory for validator, finite-constants tests, and `src/c_api/c_secrets.rs`. Validator SHA-256: `8f7d35c6c324bff7a09c741fb8153441d31acc51c6aae49681c76a5929396b35`; tests: `1c97e9a1064969e694f649bf65611e7e1ea7d23a605f563ae5d5ded5e9b9c355`; secrets source: `016cf9e232eae792e475df7c4e7b33660045d7e3d5520d11bfbc2d936437a8ec`.
- Historical baseline-to-current inventoried source differs in exactly three paths: `src/event/valid_events.rs`, `tests/wowforever_finite_constants.rs`, `src/c_api/c_secrets.rs`. Therefore this is not a controlled baseline comparison changing only the event literal.

## Historical delta and raw-stream audit

Historical Lua-errors stdout parses to **19 unique records / 36 occurrences**, exit **1**. Sanitized partition:

| Historical category | Baseline unique / occurrences | Current Lua-errors |
|---|---:|---:|
| Unknown declared aura-block event | 14 / 16 | 0 / 0 |
| Attribute failures | 1 / 14 | 0 / 0 |
| Pool failures | 4 / 6 | 0 / 0 |

Current full stream sizes: startup stdout **919 bytes / 13 lines**; startup stderr **12271 / 204**; Lua-errors stdout **3 / 1**; Lua-errors stderr **8856 / 136**. All four scanned in full against historical exact messages: **0 matches**. No file-budget-error or handler-budget-error records. Startup stderr's error-token line 120 is the diagnostic `seterrorhandler restored`, not a Lua failure. Actual zero-state proof remains the JSON array, summary counts, and exit receipt—not absence of generic `error` substrings.

The 14 attribute occurrences and six pool occurrences disappeared in this recorded delta. **No causal explanation established or asserted.** No private-root/bucket redesign reviewed or requested.

## Execution limits

Both invocations use the sealed normal executable, `--no-addons`, `--no-saved-vars`, a 90-second timeout, `WOW_SIM_NO_SOUND=1`, and unset `WOW_SIM_LOG_HANDLER_TIMINGS`. Compiled sound support exists; audio is disabled during execution. Third-party addons are excluded, not stock Blizzard UI. SavedVariables disabled does **not** establish clean WTF/EditMode state: both processes contain EditMode state logs, and worker explicitly excludes fully sealing that state.

Stock dump-tree logs `casc=true` at startup; separate Lua-errors logs `casc=false` at `lua-errors.stderr:15`. Hence separate mode/process/asset scope must remain explicit. No GUI presentation, sound behavior, CASC-enabled Lua-errors state, third-party addon load, native parity, event generation/payload secrecy, all-profile, or cold-cache claim.

Remaining provenance exclusions: external dependencies/source-to-artifact chain, untracked files/index, inherited environment, runtime assets outside recorded fixture inputs. Current evidence closes the bounded observed stock/separate-error-state comparison, not those exclusions.
