# Independent duration fixture compiled verification — FAIL

Scope: canonical `/home/osso/Projects/wow/wow-ui-sim`, HEAD `4142f3d416ff0fa9d5505926e85e5d48ac662f7c`. Authorized compile/list/module batch only. No source edits, formatting rerun, CVar rerun, broad checks, profile/startup tests, network, delegation, or operations. No parent/native closure.

## Results

Compile PASS (exit 0). Listing PASS (exit 0): 7 tests, changed `numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text` present exactly once. Module batch FAIL (exit 101): **6 passed, 1 failed, 0 ignored, 0 measured, 10691 filtered out**, reported test duration 0.12s. No reruns or auto-backgrounding.

Failure: `tests/numeric_rule_formatter.rs:190:6`, Lua `(string):27: NumericFormatter expected`; the custom table `binding:SetFormatter({Format = ...})` fails before final custom:8.2 assertion. Earlier 2/2/9/8 assertions are reached without failure. Real duration input passes the retained scalar-input failure boundary; custom callback compatibility is NOT proven and this fixture is NOT green. Runtime unchanged; no remediation authorized/performed.

## Exact command ledger

### compile

```json
{
  "argv": [
    "cargo",
    "test",
    "--offline",
    "--locked",
    "--jobs",
    "4",
    "--test",
    "integration",
    "--no-run",
    "--message-format=json"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "start": "2026-10-09T19:53:20.327630+00:00",
  "end": "2026-10-09T19:53:53.841916+00:00",
  "elapsed_seconds": 33.514599023008486,
  "exit_code": 0,
  "stdout_sha256": "5f16633a8c95d1e6a7245cff3f68d36251e2b9280216645b2ccc9cd9cd77e1da",
  "stderr_sha256": "c3829c526428af813573a7beab633563e92fff9e5d4ee8d33d51a1b1498a2ed2"
}
```

Full independent streams: `/tmp/duration-fixture-compiled-independent/compile.stdout`, `/tmp/duration-fixture-compiled-independent/compile.stderr`.

### list

```json
{
  "argv": [
    "/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a",
    "numeric_rule_formatter::",
    "--list"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "start": "2026-10-09T19:54:10.071920+00:00",
  "end": "2026-10-09T19:54:10.081433+00:00",
  "elapsed_seconds": 0.009506601985776797,
  "exit_code": 0,
  "stdout_sha256": "f736b3960a650dd19ea57ae04a1465e025eb49a0079085cc21fdac1d3540ca98",
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

Full independent streams: `/tmp/duration-fixture-compiled-independent/list.stdout`, `/tmp/duration-fixture-compiled-independent/list.stderr`.

### batch

```json
{
  "argv": [
    "timeout",
    "90",
    "/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a",
    "numeric_rule_formatter::",
    "--nocapture"
  ],
  "cwd": "/home/osso/Projects/wow/wow-ui-sim",
  "start": "2026-10-09T19:54:10.081576+00:00",
  "end": "2026-10-09T19:54:10.216096+00:00",
  "elapsed_seconds": 0.1345302929985337,
  "exit_code": 101,
  "stdout_sha256": "130cd5241bad0e531edcd670391246a967f7a6b8248f3272f412bf5cc7b25071",
  "stderr_sha256": "8d7b3baeda500695d6c98e7d1459cf0222cb34abbb07de8edbad6a13f32c4a69"
}
```

Full independent streams: `/tmp/duration-fixture-compiled-independent/batch.stdout`, `/tmp/duration-fixture-compiled-independent/batch.stderr`.

## Artifact

```json
{
  "cargo_message": {
    "reason": "compiler-artifact",
    "package_id": "path+file:///home/osso/Projects/wow/wow-ui-sim#0.1.0",
    "manifest_path": "/home/osso/Projects/wow/wow-ui-sim/Cargo.toml",
    "target": {
      "kind": [
        "test"
      ],
      "crate_types": [
        "bin"
      ],
      "name": "integration",
      "src_path": "/home/osso/Projects/wow/wow-ui-sim/tests/integration.rs",
      "edition": "2024",
      "doc": false,
      "doctest": false,
      "test": true
    },
    "profile": {
      "opt_level": "1",
      "debuginfo": "line-tables-only",
      "debug_assertions": true,
      "overflow_checks": true,
      "test": true
    },
    "features": [
      "aura-containers",
      "aura-instance-enumeration",
      "aura-xml-widgets",
      "base-spell-relationships",
      "casc",
      "client-retail",
      "default",
      "forbidden-aspects",
      "gui",
      "native-duration-formatting",
      "numeric-rule-formatters",
      "on-update-modes",
      "player-cast-durations",
      "profile-retail",
      "retail-12-0-0",
      "retail-12-0-5",
      "retail-12-0-7",
      "retail-12-1-0",
      "rodio",
      "sound"
    ],
    "filenames": [
      "/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a"
    ],
    "executable": "/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a",
    "fresh": false
  },
  "sha256": "3f1de3a47d357ff4f247e8f21e655d4faf08192b41d6b0f1956f9d8afd1136e4",
  "build_finished": [
    {
      "reason": "build-finished",
      "success": true
    }
  ],
  "compiler_messages": []
}
```

Fresh integration artifact (`fresh: false` means compiled, not reused), build-finished success true. Features are exactly those in Cargo JSON above; no other profile claim. Full Cargo JSON parsed (738 records), all compiler-message diagnostics reviewed: 0.

## Full warnings and module outputs

Compile stderr (six manifest deprecation warnings plus summary; no suppression):

```text
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
warning: `iced_wgpu` (manifest) generated 6 warnings
   Compiling wow-ui-sim v0.1.0 (/home/osso/Projects/wow/wow-ui-sim)
    Finished `test` profile [optimized + debuginfo] target(s) in 33.46s
```

list stdout:

```text
numeric_rule_formatter::numeric_rule_formatter_components_apply_division_modulo_then_rounding: test
numeric_rule_formatter::numeric_rule_formatter_copies_its_configuration_and_replaces_or_clears_rules: test
numeric_rule_formatter::numeric_rule_formatter_formats_ellesmere_aura_duration_breakpoints: test
numeric_rule_formatter::numeric_rule_formatter_formats_resource_bar_countdown_and_rounding_modes: test
numeric_rule_formatter::numeric_rule_formatter_rejects_invalid_rules_without_replacing_valid_state: test
numeric_rule_formatter::numeric_rule_formatter_selects_threshold_before_rounding_and_clamps_afterward: test
numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text: test

7 tests, 0 benchmarks
```

list stderr:

```text
```

batch stdout:

```text

running 7 tests
test numeric_rule_formatter::numeric_rule_formatter_copies_its_configuration_and_replaces_or_clears_rules ... ok
test numeric_rule_formatter::numeric_rule_formatter_components_apply_division_modulo_then_rounding ... ok
test numeric_rule_formatter::numeric_rule_formatter_rejects_invalid_rules_without_replacing_valid_state ... ok
test numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text ... FAILED
test numeric_rule_formatter::numeric_rule_formatter_formats_resource_bar_countdown_and_rounding_modes ... ok
test numeric_rule_formatter::numeric_rule_formatter_formats_ellesmere_aura_duration_breakpoints ... ok
test numeric_rule_formatter::numeric_rule_formatter_selects_threshold_before_rounding_and_clamps_afterward ... ok

failures:

failures:
    numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 10691 filtered out; finished in 0.12s

```

batch stderr:

```text
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.000s] [Startup] WowLuaEnv::new begin
[  0.002s] [Startup] SimState::default complete in 1.65ms
[  0.002s] [Startup] SimState::default complete in 1.64ms
[  0.002s] [Startup] SimState::default complete in 1.65ms
[  0.002s] [Startup] SimState::default complete in 1.65ms
[  0.002s] [Startup] SimState::default complete in 1.64ms
[  0.002s] [Startup] SimState::default complete in 1.77ms
[  0.002s] [Startup] rilua VM created in 105.20µs
[  0.002s] [Startup] rilua VM created in 106.03µs
[  0.002s] [Startup] rilua VM created in 109.56µs
[  0.002s] [Startup] rilua VM created in 105.06µs
[  0.002s] [Startup] SimState::default complete in 1.77ms
[  0.002s] [Startup] rilua VM created in 112.62µs
[  0.002s] [Startup] builtin frames initialized in 69.17µs
[  0.002s] [Startup] builtin frames initialized in 71.02µs
[  0.002s] [Startup] template registry cleared in 7.42µs
[  0.002s] [Startup] builtin frames initialized in 81.90µs
[  0.002s] [Startup] rilua VM created in 80.70µs
[  0.002s] [Startup] builtin frames initialized in 84.12µs
[  0.002s] [Startup] template registry cleared in 1.97µs
[  0.002s] [Startup] template registry cleared in 3.74µs
[  0.002s] [Startup] template registry cleared in 1.53µs
[  0.002s] [Startup] intrinsic templates registered in 2.88µs
[  0.002s] [Startup] initial app_data lua handle installed in 480.00ns
[  0.002s] [Startup] intrinsic templates registered in 4.01µs
[  0.002s] [Startup] rilua VM created in 83.67µs
[  0.002s] [Startup] initial app_data lua handle installed in 431.00ns
[  0.002s] [Startup] builtin frames initialized in 83.43µs
[  0.002s] [Startup] intrinsic templates registered in 6.21µs
[  0.002s] [Startup] template registry cleared in 1.48µs
[  0.002s] [Startup] intrinsic templates registered in 2.82µs
[  0.002s] [Startup] initial app_data lua handle installed in 140.00ns
[  0.002s] [Startup] initial app_data lua handle installed in 150.00ns
[  0.002s] [Startup] intrinsic templates registered in 3.26µs
[  0.002s] [Startup] initial app_data lua handle installed in 140.00ns
[  0.002s] [Startup] builtin frames initialized in 52.10µs
[  0.002s] [Startup] template registry cleared in 1.49µs
[  0.002s] [Startup] intrinsic templates registered in 2.72µs
[  0.002s] [Startup] initial app_data lua handle installed in 151.00ns
[  0.002s] [Startup] builtin frames initialized in 60.94µs
[  0.002s] [Startup] template registry cleared in 1.62µs
[  0.002s] [Startup] intrinsic templates registered in 2.83µs
[  0.002s] [Startup] initial app_data lua handle installed in 161.00ns
[  0.111s] [Startup] init_lua_state complete in 108.70ms
[  0.111s] [Startup] final app_data lua handle installed in 340.00ns
[  0.111s] [Startup] initial screen globals installed in 127.15µs
[  0.111s] [Startup] WowLuaEnv::new complete
[  0.112s] [Startup] init_lua_state complete in 109.88ms
[  0.112s] [Startup] final app_data lua handle installed in 371.00ns
[  0.112s] [Startup] initial screen globals installed in 113.28µs
[  0.112s] [Startup] WowLuaEnv::new complete
[  0.112s] [Startup] init_lua_state complete in 110.39ms
[  0.112s] [Startup] final app_data lua handle installed in 351.00ns
[  0.112s] [Startup] init_lua_state complete in 110.41ms
[  0.112s] [Startup] final app_data lua handle installed in 301.00ns
[  0.112s] [Startup] initial screen globals installed in 98.94µs
[  0.112s] [Startup] initial screen globals installed in 83.25µs
[  0.112s] [Startup] WowLuaEnv::new complete
[  0.112s] [Startup] WowLuaEnv::new complete

thread 'numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text' (4074594) panicked at /home/osso/Projects/wow/wow-ui-sim/tests/numeric_rule_formatter.rs:190:6:
native formatter feeds the duration binding consumer: Lua(Runtime(RuntimeError { message: "(string):27: NumericFormatter expected", level: 2, traceback: [] }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[  0.113s] [Startup] init_lua_state complete in 111.11ms
[  0.113s] [Startup] final app_data lua handle installed in 471.00ns
[  0.113s] [Startup] init_lua_state complete in 111.16ms
[  0.113s] [Startup] final app_data lua handle installed in 70.00ns
[  0.113s] [Startup] initial screen globals installed in 92.04µs
[  0.113s] [Startup] WowLuaEnv::new complete
[  0.113s] [Startup] initial screen globals installed in 90.93µs
[  0.113s] [Startup] WowLuaEnv::new complete
[  0.113s] [Startup] init_lua_state complete in 111.39ms
[  0.113s] [Startup] final app_data lua handle installed in 161.00ns
[  0.113s] [Startup] initial screen globals installed in 83.54µs
[  0.113s] [Startup] WowLuaEnv::new complete
ELLESMERE_NUMERIC_RULE_FORMATTER_GREEN
```

## Reused scope-valid evidence and drift

Prior report remains unchanged and still records its earlier PENDING epoch. Reuse its fmtcheck PASS once, changed Rust readability/spec audit PASS, not a runtime success claim. Exact old fmt times unknown/lost, not reconstructed. Prior source hash set equals pre-compile and post-batch hash set; HEAD unchanged. Full path/hash ledgers: `pre.json`, `post.json`; prior ledger `/tmp/duration-fixture-green-independent/code-before.json`. No prior or pre/post source drift.

Prior evidence SHA-256:

```json
{
  "/tmp/duration-fixture-green-independent/report.md": "7ef389fae04d2dd11904aa4f6dc91050fb1ddb947d1145612e8ca401969d1554",
  "/tmp/duration-fixture-green-independent/format.json": "cbb20b5c75b77f9ecebe83e3a2642102ff7b32d8fddbc11dc673678aa2369945",
  "/tmp/duration-fixture-green-independent/code-before.json": "9da8984bddefaa0d887dc6c712a99390cade0bcef0ea2213cbbe2e42ac363a7c",
  "/tmp/duration-fixture-green-independent/format.stdout": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "/tmp/duration-fixture-green-independent/format.stderr": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

Changed Rust SHA-256: `12c126215c4827e1621edc5b0a1da63caf895cb91dfe5b492cdc6d2da6aa6c8e`.

Main notified immediately after compile/batch completion to release builder window. This report verifies only the requested bounded proof, not parent completion/native parity.
