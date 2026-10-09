# Transmog epoch independent bounded verification — PARTIAL / historical BLOCKED

## Scope and actual revisions

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Requested committed test-only correction: `f2eebd359a18c127ac228a4750157414086b13d2`. Read verify and rust-readability skills, current historical module, client-profile spec/architecture and integrated proof SSOT. No delegation.

Formatting began at the requested correction and ended at `a9d2433b6b113aca7ab3e2505c5548a59b4f0445`; concurrent commit contains docs/evidence only (`resume-source-diff.stdout`). Both completed compiles and exact artifact operations ran at a9d2433b6. All recorded compile-source hashes unchanged across each proof command. Not an immutable f2eebd359 execution claim. `proof-ledger.json` records exact argv/cwd, UTC times, exits, actual HEAD, scope hashes and full streams; command-local metadata preserves before/after snapshots.

Only comment/module cfg changed in the historical Rust test: `retail-12-0-0 && !retail-12-0-5`. Complete body byte-identical to 41f1abbeb: all 22 member expectations and MinValue=0, MaxValue=21, NumValues=22 unchanged. This is read-only diff evidence, not a new source-shape test. Production values and current integration assertions untouched.

## Proof matrix

| Gate | Exit | Exact evidence |
|---|---:|---|
| Whole-workspace `cargo fmt --check`, one invocation | 0 | Empty stdout/stderr; PASS |
| Historical offline/locked no-default-feature lib compile, dedicated target | 101 | FAIL: three E0433 errors; no lib executable |
| Historical exact test execution | — | BLOCKED, not executed; no 1/1 claim |
| Default Retail offline/locked lib compile, canonical default target | 0 | PASS; successful build-finished |
| Exact historical filter `--exact --list` on fresh default artifact | 0 | `0 tests, 0 benchmarks`; test absent, not a zero-result execution |
| Current weather/time-category integration exact execution | 0 | PASS: 1 passed, 0 failed, 10,697 filtered out |

Historical compile argv: `cargo test --offline --locked --no-default-features --features profile-retail,retail-12-0-0 --target-dir /home/osso/Projects/wow/wow-ui-sim/target/transmog-epoch-proof --lib --no-run --message-format=json`.
Default compile argv: `cargo test --offline --locked --lib --no-run --message-format=json`.
Exact historical identity: `loader::tests::wow_api_globals::transmog_situation::test_patch_12_0_0_transmog_situation_enum_values`.
Current identity: `patch_12_0_5_enum_additions::transmog_situations_publish_weather_and_time_categories`. These are different tests and contracts; current PASS does not repair/prove historical execution.

Builds had no timeout. Artifact listing/execution used 120-second CommandBuilder timeout. Terminal `.cwd(...).capture().run()` only; no spawn/persisted handles. An earlier evaluation was interrupted after completed formatting and before any retained historical compile result; inspection found no historical process, binary, or streams. Missing compile proof was recovered once, with no rerun of valid formatting/build proof. Prior interrupted launch/completion cannot be established from available artifacts (`interruption.json`); no silent success credit.

## Historical blocker — preserved, not fixed

Unconditionally compiled lib test `src/iced_app/frame_collect.rs::hittable_order_follows_render_buckets_not_raw_child_strata` references GUI-only modules/dependency with `gui` disabled:

- `src/iced_app/frame_collect.rs:378`: `crate::iced_app::strata_emit` unavailable; `src/iced_app/mod.rs:55–56` gates it on gui.
- `src/iced_app/frame_collect.rs:379`: `crate::iced_app::hit_grid` unavailable; `src/iced_app/mod.rs:27–28` gates it on gui.
- `src/iced_app/frame_collect.rs:380`: unresolved `iced::Point`, optional dependency disabled.

These error-source files match f2eebd359 byte-for-byte (`baseline-error-source.json`). Unrelated to the changed transmog cfg. No GUI-enabling alternative, other-profile build, source fix, or extra historical compile was performed. Four compiler unused-import warnings and six manifest deprecation warnings remain; complete rendered diagnostics in `historical-compile-diagnostics.json`, full stderr in `historical-compile.stderr`.

Historical full JSON stdout parsed: {'compiler-artifact': 339, 'build-script-executed': 36, 'compiler-message': 8, 'build-finished': 1}, zero unparsed lines. Default stdout parsed: {'compiler-artifact': 656, 'build-script-executed': 74, 'build-finished': 1}, zero unparsed lines. Default has zero compiler-message records; six manifest deprecation warnings remain in stderr. This does not claim warning-free compilation.

## Binary provenance and reuse

Fresh default library binary: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`.
SHA256 before/after listing: `ccee4f0a61d21944140392be890e624bfe406b1de847b37bdda73294eca25452` / `ccee4f0a61d21944140392be890e624bfe406b1de847b37bdda73294eca25452`. Full Cargo artifact and features retained in `default-artifacts.json` and `default-binary-hash.json`.

Current integration binary: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`.
SHA256: `3395a9d9307b65f28b7cb51f423e7d4a315c3793ecbd3e0ac214497780f8356b`, matches caller-specified/retail-toc retained hash and remains unchanged after execution (`integration-binary-hashes.json`). Retained compiled revision: `41f1abbeb83d508b323f51d1f371a3bc506b63b5`. No integration rebuild.

Scope equivalence to retained revision: `integration-scope-diff.stdout` shows only the cfg(test)-only historical module gate/comment and standalone test target manifest metadata. All non-test Cargo tables/features/dependencies/profiles and the integration target are identical (`reuse-scope.json`). Manifest adds the Era cvar standalone target and moves the Era required-feature line off the preceding NPC-health target; neither changes explicit default integration compilation. No claim about those standalone targets. Existing integration test, helpers, production source, Cargo.lock and build configuration unchanged. Parent loader tests are gated `#[cfg(test)]` (`source-boundaries.json`), so changed historical module is not compiled into the integration library dependency. Data/docs differences versus base are source-audit evidence/receipts, not runtime or generated enum input changes. Reuse is scoped, not arbitrary HEAD/binary equivalence.

Full current execution stdout:
```text

running 1 test
test patch_12_0_5_enum_additions::transmog_situations_publish_weather_and_time_categories ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10697 filtered out; finished in 0.30s
```
Full current stderr retained; startup initialization messages, no runtime failure.

## Readability, limitations and preservation

Manual changed-line/surrounding-function readability audit: no introduced violations. Gate explicitly expresses cumulative epoch lifetime; explanatory comment names why. No added nesting, state accumulation, suppressions, parameter overload or duplicated implementation. All test expectations remain literal and unchanged. `rust-code-analysis-cli` and `readability-audit` unavailable; no measured cognitive-complexity claim (`manual-readability.json`).

Historical presence/execution remains unproven because compilation failed upstream. Default omission proves only that artifact's exact test registration. Current integration covers ten weather/time members plus numeric-member and coherent metadata checks before/after post-load; existing fixture does not assert a historical 22-member register or hardcode current count 32. Current later metadata rationale comes from source/SSOT, not a new native capture.

Snapshot hashes cover tracked src/tests/crates/iced-* and .cargo/root manifest/lock/build.rs, not every external registry/cache file; no universal cache-integrity claim. No broad/full suite, cargo check, alternate profiles, native parity, runtime startup acceptance or parent-goal closure. No code/cache/vendor edits, commits, push/deploy/network, new tests or expectation changes. Dedicated target contains build artifacts only. Existing untracked `.code-index.db` remained; no cleanup/ownership claim.

## Host incident notice

Parent reported kernel OOM around 17:54–17:55 UTC killing Chromium and a sibling game-server (agents limit 42 GiB; peak approximately 40 GB). Earlier interruption overlaps that window, but no retained evidence establishes its cause. All compilation here had already ended when notice arrived; nothing canceled or rerun. No further compile needed. Parent's serialization/`--jobs 4` instruction applies to any future compile, not retroactively to recorded completed argv.

**Bounded conclusion:** formatting, fresh default compilation/omission, manual readability and current exact integration PASS. Historical compile FAIL on preserved unrelated GUI test references; requested historical 1/1 blocked. Parent remains open.
