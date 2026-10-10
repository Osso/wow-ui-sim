# Saved timing-helper gate audit

**PASS — bounded gate only.** Saved default library compilation and selected Mists compilation succeed without an `is_enabled` warning. Combined with earlier bounded formatting evidence, this closes only the timing-helper cfg/compiler gate. No commands, builds, tests, checks, delegation, service changes or other operations were run. Python was used only to read, parse, hash and write audit artifacts. Followed verify skill as an artifact verifier; no re-delegation.

## Default compiler evidence

Epoch: `/home/osso/.local/state/wow-ui-sim/verification/timing-gate-default-check/20261010T043957Z`. Submission records revision **`2e46082d0dc9c719134aa490c44d97b0da657157`**, not `6751` or `014`. This is the recorded compile revision; no current HEAD lookup or independent Git commit attestation was performed.

Exact argv: `["/usr/bin/cargo", "check", "--offline", "--locked", "--lib", "--message-format=json"]`. Recorded cwd: `/home/osso/Projects/wow/wow-ui-sim`. Environment override: `CARGO_BUILD_JOBS=12`. No explicit feature arguments or `--no-default-features`; Cargo's actual `wow_ui_sim` library artifact reports:

`aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound`

Submission and Cargo receipt argv/cwd agree. Cargo exit **0**, outcome cargo exit **0**, `build_performed=true`, outcome `source_equal=true`. Independently parsed source maps: **3,839 entries each, identical**. Cargo JSON fully consumed: **727 records** = **652 compiler artifacts + 74 build-script-executed + 1 build-finished(success=true)**. **0 compiler-message records, 0 compiler errors, 0 `is_enabled` warnings**. Library artifact is **non-fresh**, target `lib`, dev profile opt-level 1, line-tables-only debug info, not a test execution. Build-end executable seals are an empty array; this check is not executable/run proof.

Recorded submission starts `2026-10-10T04:39:57.438030+00:00`; Cargo ends `2026-10-10T04:41:02.236736+00:00`; outcome ends `2026-10-10T04:41:02.589430+00:00`. Timestamps are retained literal UTC metadata, not revision identity or performance proof. Boot ID unchanged.

## Relevant source and wiring

Read helper and both callers; full file bytes hash-match **both** default and Mists captured maps. `Cargo.toml` and `src/lua_api/mod.rs` also match both maps. Exact hashes are in `aggregate.json` and `hashmanifest.json`. This binds inspected source to saved compile snapshots, not current HEAD acceptance.

- `src/lua_api/handler_timing.rs:12-15`: `#[cfg(feature = "retail-12-0-5")]` immediately precedes `pub(crate) fn is_enabled() -> bool`; body remains `min_duration_ms().is_some()`.
- `src/loader/lua_file.rs:221-223`: identically gated expression calls `crate::lua_api::handler_timing::is_enabled()`.
- `src/lua_api/execution_budget.rs:48`: handler caller remains; module declaration gated by the same feature at `src/lua_api/mod.rs:19-20`.

Default Cargo artifact includes `retail-12-0-5`: helper and callers remain available to that compilation. Mists artifact excludes it: helper and those callers are excluded together. Source and compiler evidence only; no runtime timing/budget behavior tested.

## Exact remaining external warnings

Each inspected compile stderr contains **7 warning-prefixed lines**: **6 unique external manifest deprecation diagnostics + 1 summary**, not seven unique diagnostics. Each Cargo JSON stream has **0 compiler-message warnings**. Across both streams: **14 warning-prefixed lines, 12 diagnostic occurrences, 6 unique diagnostic kinds, 2 summaries**.

All six arise from `iced-wgpu-patched/Cargo.toml` Clippy lint names:

```text
large-enum-variant → large_enum_variant
map-entry → map_entry
match-wildcard-for-single-variants → match_wildcard_for_single_variants
redundant-closure-for-method-calls → redundant_closure_for_method_calls
trivially-copy-pass-by-ref → trivially_copy_pass_by_ref
type-complexity → type_complexity
```

Exact summary in each: `warning: `iced_wgpu` (manifest) generated 6 warnings`. Full diagnostic lines preserved in aggregate. Neither build is globally warning-free. No external warning remediation performed.

## Combined evidence — strictly bounded

| Evidence | Identity | Result and limit |
|---|---|---|
| Earlier saved formatting report | `6751c0f37088de11d0cc864f9313922a5b9cbdab` | Recorded `cargo fmt --check`: exit 0, empty stdout/stderr. Historical formatting scope only; no transfer to all later changes. |
| Saved default library check | `2e46082d0dc9c719134aa490c44d97b0da657157` | exit 0, source maps equal, zero `is_enabled` warning; default library compilation only. |
| Saved Mists selected compilation | `0148437a7868b0a25d0ce2fca504378d4558b5ee` | exit 0, source maps equal, zero `is_enabled` warning; selected `--no-run` compiler proof only. |

Formatting source: `/home/osso/.local/state/wow-ui-sim/verification/timing-gate-current/report.md`. Only its bounded formatting evidence is used, not its historical pre-fix compiler warning or unrelated source/readability claims.

Mists compiler source: `/home/osso/.local/state/wow-ui-sim/verification/mists-currency-observer-current/20261010T043019Z`. Exact argv: `["/usr/bin/cargo", "test", "--offline", "--locked", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--test", "mists_currency_list", "--test", "mists_compat_bootstrap", "--test", "mists_dialog_helpers", "--no-run", "--message-format=json"]`. Actual library features: `casc, client-mists, gui, rodio, sound`; library non-fresh. **740 JSON records = 665 compiler artifacts + 74 build-script-executed + 1 successful build-finished; 0 compiler-message records**. Source maps independently equal, 3,839 entries. Only compiler streams/submission/maps/receipts/cgroup/seals were consumed; no Mists runtime/profile-execution or currency result used. Selected target compilation is not full-profile compilation, API acceptance or CurrencyGreen.

## Cgroup and compiler-input limits

Default before/after path unchanged: `/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/agents.slice/wow-timing-gate-default-check-j12-20261010t041634z.service`. CPU cap **`1200000 100000` (12 CPU quota)** and memory cap **`17179869184` bytes (16 GiB)** unchanged. Recorded memory peak: **49,770,496 → 5,730,381,824 bytes**; after memory.current **1,143,042,048 bytes**. After cpu.stat: usage **56,093,107 µs**, periods **652**, throttled periods/time **0/0**. Lifetime cgroup observations, not isolated compiler RSS or controlled performance evidence. No speedup claim.

Submission excludes **external path dependencies, inherited environment, uncaptured data outside tracked src/tests/build/config scope, runtime cache/addon files and untracked index**. Source-map equality does not establish full compiler-input closure, exact inherited flags/toolchain/dependency identities, or every instantaneous input during compilation. Feature identity comes from Cargo artifacts; revision identity is recorded submission metadata.

## Outputs and exclusions

`aggregate.json` preserves counts, source bindings, exact arguments/features/warnings and receipts. `hashmanifest.json` SHA256-seals all consumed evidence plus report and aggregate, excluding itself to avoid circular hashing. Original evidence unchanged. No current-HEAD/API/CurrencyGreen/native/full-profile/full-suite/runtime acceptance or speedup claims.
