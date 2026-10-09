# Reserved GUI verification

GUI stage PASS with compile-exit capture limitation.

- Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`.
- Before HEAD: `b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f`; after HEAD: `b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f`.
- Before/after frame_collect.rs SHA256: `b50f1dc966712d779a9a458d989b3353627975552155d5976c3d329b7c752160` / `b50f1dc966712d779a9a458d989b3353627975552155d5976c3d329b7c752160`.
- Before/after status unchanged: untracked `.code-index.db`; no verifier source changes.

## Compile

Command: `cargo test --offline --locked --jobs 4 --lib --no-run --message-format=json`. Invoked exactly once, no timeout, default target/features, serialized jobs=4. Cargo reports `build-finished success=true`; Finished in 0.17s. Artifact fresh=true (warm cached artifact), feature list includes gui/default/client-retail. Full output read: 566421 bytes / 739 lines, 656 compiler-artifact records, 74 build-script-executed records, 1 successful build-finished record; no compiler-message records.

Capture limitation: actual compile exit integer was not retained. This runtime's combined_to_file executes terminally, returning CommandResult; requested appended .run() raised after compile completed before assignment. No compile rerun. Pre-invocation ledger and full combined stream survived. Preliminary scope-capture API mismatches happened before Cargo invocation.

Six existing iced-wgpu-patched/Cargo.toml deprecated Clippy manifest-key warnings preserved verbatim in compile.nonjson.log and original combined stream. No warning suppression or edits.

## Exact GUI test

Binary: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`

Binary SHA256: `97f8c673c65a614ad4d8188935a631f770bc58f987e6f02f4ab8bbc1bee8b994`

Command: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d iced_app::frame_collect::tests::hittable_order_follows_render_buckets_not_raw_child_strata --exact --nocapture`. Cwd canonical; timeout 120 seconds. Observed CommandResult.exit_code=0. **1 passed, 0 failed, 0 ignored, 0 measured, 1977 filtered out**. Finished in 0.00s.

Source lines 377–388: common collected-order assertion plus both GUI HitGrid assertions compiled under artifact gui feature and executed in this passing exact test (independent HIGH wins; transparent MEDIUM panel wins when independent excluded). No zero-match test execution.

## Evidence

- `/tmp/gui-window-independent/command-ledger.json`: argv/cwd/times/revisions/source hash/binary hash and exit-capture disclosure.
- `/tmp/gui-window-independent/compile.combined.log`: complete stdout/stderr combined arrival-order stream; Cargo JSON lines mixed with plain stderr, not separately attributable streams.
- `/tmp/gui-window-independent/compile.nonjson.log`: complete non-JSON lines including all manifest warnings.
- `/tmp/gui-window-independent/compiler-artifact.json`: exact selected Cargo artifact.
- `/tmp/gui-window-independent/test.combined.log`: complete combined test stream.

Parent notified immediately after compile/exact-test completion that reserved builder can be released. No historical/fmt/weather/broad/integration/profile/addon/client runs; no edits/commits/push/deploy/network/service changes/delegation. Parent owns retention and full-goal completion.
