# Independent standalone Forever verification — bounded PASS

## Revision and execution
Requested canonical revision: f611a6752a14b4d660902417c59e14073de9ac31.
Canonical HEAD initially matched, then advanced before execution to 045e396b0c6f717c2d962b97cdf46b736a3328ce (Era audit data/docs commit). No checkout changes made. `equivalence.json` records exit-0 diff against f611a6752 for compiled source/config/vendor/new-test scope. Pre/post scope hashes identical for both commands; HEAD stable during execution. Existing untracked `.code-index.db` unchanged by verifier.

One initial setup evaluation failed with undefined `head`; another stopped at the exact-HEAD assertion after concurrent advance. Neither reached Cargo or produced command receipts. Requested Cargo test executed exactly ONCE. No log-recovery rerun.

Explicit cwd for every CLI: `/home/osso/Projects/wow/wow-ui-sim`. Installed `/usr/bin/cargo`, inherited environment, CARGO_TARGET_DIR unset. Test stderr confirms canonical `target/debug/deps/forever_runtime_contracts-44000c10e8eb2f89`; artifact hash retained in compiled-artifact.json. No GUI launch, build-host wrapper, delegation, edits/commit/push/deploy, broad check, or full suite.

| Command | UTC start → finish | Evidence |
|---|---|---|
| `/usr/bin/cargo test --offline --locked --no-default-features --features client-wowforever --test forever_runtime_contracts -- --nocapture` | 2026-10-09T16:15:15.414956+00:00 → 2026-10-09T16:16:32.864145+00:00 | exit 0; 3 passed, 0 failed, 0 ignored, 0 measured, 0 filtered; test execution 0.18s |
| `/usr/bin/cargo fmt --check` | 2026-10-09T16:16:32.907924+00:00 → 2026-10-09T16:16:42.821112+00:00 | exit 0; stdout/stderr empty |

Full separated streams, argv, environment, cwd, revisions, pre/post SHA256 scope, UTC times, duration, exit code and stream hashes automatically retained in runtime-test.* and format.*. Environment-bearing receipts mode 0600.

Exact stdout:
```text
running 3 tests
test forever_invalid_chat_prefix_preserves_existing_records ... ok
test forever_removed_public_combat_log_getters_are_absent ... ok
test forever_chat_senders_append_ordered_environment_local_records ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

## Behavioral coverage
| Contract | Actual assertions / proof |
|---|---|
| Two available senders; successful ordered records | patch-tests/forever_runtime_contracts.rs:25–56 checks both functions, all three success codes 0, then exact Vec equality against `(addon, ACE, one, PARTY, empty)`, `(addon_logged, BUG, plain text, PARTY, empty)`, `(addon, ACE, three, WHISPER, Bob-Realm)` in that order. Runtime PASS. |
| Environment-local state | :58–59 creates a second direct WowLuaEnv after accepted sends; its log is empty. Runtime PASS for this one-way isolation observation; not a full bidirectional mutation test. |
| Both invalid-prefix senders preserve existing log | :63–76 first appends one accepted control record, snapshots whole log, calls each sender with empty prefix and checks return 1, then checks whole-log equality. Runtime PASS for combined postcondition. No intermediate per-call snapshot; static src/c_api/addon_messages.rs:156–176 confirms shared send_chat appends only for SUCCESS (0), supporting no append for either rejected call. |
| Specific getter absence | :80–88 checks only raw C_CombatLog.GetCurrentEventInfo, public member lookup, and raw global CombatLogGetCurrentEventInfo are nil. Runtime PASS. No generic namespace absence inference. |
| Cargo wiring | Cargo.toml:199–202 explicitly registers target/path and required-features client-wowforever; file-level cfg also matches. Actual test binary compiled and ran. |
| Retail restriction | Static, unchanged: registration.rs:60–64 and c_combat_log.rs:11–19 remain Retail-12-0-0-only. Not part of removed-getter proof; no fresh Retail/Forever restriction runtime assertion. |

No native parity, security/taint, transport, inbound delivery/no-echo, real login lifecycle, loaded Blizzard UI, BNet, or broad acceptance credit.

## Reused library proof
Prior `/tmp/forever-cfg-independent/retail-check-result.json` is exit 0 for `cargo check --offline --locked --no-default-features --features client-retail --lib` at 811a0491b, 14:23:08.422279–14:23:41.224937 UTC. Inspected actual receipt and source diffs. 811a0491b→1044215d0 changes only ten Retail-true cfg attributes; 1044215d0→f611a6752 has no runtime src/build.rs/lock/config/vendor changes. Manifest changes in that interval only add explicitly gated test targets (Era and Forever); library/features/dependencies unchanged. Reused only as Retail library/type evidence, not runtime behavior. New Forever target itself compiles its affected feature path. No redundant broad checks added.

## Changed Rust readability
Read/followed rust-readability skill; audited every line of the sole Rust file changed by f611a6752, patch-tests/forever_runtime_contracts.rs (88 lines). rust-code-analysis-cli unavailable on PATH; manual complexity/nesting audit used per skill. Helper body 14 lines; test bodies 34, 16 and 10 lines, below 200-line test threshold. No branches/loops; helper/map closure depth bounded; no suppressions, complex conditionals, state-accumulation loops, hidden I/O, parameter overload, TODO/FIXME/HACK/XXX, or unrelated sequential responsibilities. Literal tuples and Lua status expectations are concrete test fixtures. No readability violations identified. No automated metric values claimed.

## Warnings
14 warning diagnostics: 6 vendor manifest deprecations, 7 library warnings, 1 binary warning. Same categories/sites retained in prior successful Forever report; no new warning caused by standalone target established. No suppression applied.

- iced-wgpu-patched/Cargo.toml: deprecated hyphenated Clippy keys large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity.
- src/blizzard_ui_sync.rs:28 PROVENANCE_SCHEMA unused; :180 CacheProvenance::new unused; :663 remove_missing_marker unused.
- src/c_api/c_secrets.rs:110 is_never_secret_aura unused; src/casc_asset_fallback.rs:93 ensure_known_asset_cached unused.
- src/render/font.rs:44 encoding_key_hex unread; src/widget/frame.rs:601/610 cooldown_elapsed_since_start / cooldown_remaining_seconds unused (one diagnostic).
- src/bin/wow_sim/main.rs:20 unused SavedVariablesManager import.

## Conclusion
PASS: standalone 3/3 simulator contracts, affected Forever compilation, formatting and manual changed-Rust readability. Revision caveat: actual execution at 045e396b0 with compiled scope equivalent to requested f611a6752, not an execution at exact f611a6752 HEAD. Coverage limitations above remain explicit, not silently promoted to native/security/delivery or Retail runtime proof.
