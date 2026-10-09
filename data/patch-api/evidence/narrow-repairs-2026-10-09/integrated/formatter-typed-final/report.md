# Independent bounded typed-formatter proof

Canonical `/home/osso/Projects/wow/wow-ui-sim`, revision `941140fd6fa52133966a8f28a022f49d3538df50`.

**Formatter PASS; separately requested aura reproduction FAIL. No native or parent closure.**

## Execution results

| Proof | Result | Evidence |
|---|---|---|
| One serialized integration compile | PASS, exit0 | `compile.json`, separate `compile.stdout` / `compile.stderr` |
| Formatter listing, once | PASS, exit0; 7 cases; changed duration-binding case exactly once | `formatter-list.*` |
| Formatter module batch, once, timeout90 | PASS, exit0; 7 passed, 0 failed, 10691 filtered; 0.13s | `formatter-batch.*` |
| Exact aura listing, once | PASS, exit0; exactly 1 case | `aura-list.*` |
| Exact aura reproduction, once, timeout90 | FAIL, exit101; 0 passed, 1 failed, 10697 filtered; 0.41s | `aura-red.*` |

The compile command was exactly argv `["/home/osso/.worktrees/build-lock.sh", "cargo", "test", "--offline", "--locked", "--jobs", "4", "--test", "integration", "--no-run", "--message-format=json"]`, canonical cwd, no timeout. Wrapper acquires fd9 flock before Cargo. Stderr records waiting15:00 / acquired15:02. Compile UTC 2026-10-09T20:00:01.715898+00:00 through 20:02:52.849408+00:00. Wrapper released its OS lock on exit; main notified immediately after batch and after both bounded executions for peer reservation release.

All 738 Cargo JSON messages parsed; one integration executable selected, `fresh:false`, build-finished success:true, zero compiler-message records. Full stderr reviewed: **six manifest deprecation warnings**, not suppressed, from `iced-wgpu-patched/Cargo.toml`: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. Cargo reports six manifest warnings and compile completion. No rerun/background incident.

Fresh executable `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`; SHA256 `73b728b4a6dace587225d0b88313c1060943614db7444922655011c13ccc0d6f`. Filename alone was not reused as provenance: newly emitted Cargo metadata/hash recorded in `artifact.json`. Features: aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound. Artifact hash unchanged through both executions.

## Semantic and readability audit

Pinned `integrated/formatter-policy/DurationTextBindingObjectAPIDocumentation.lua` SHA256 `4c3f6bb476f6f2d4e05f4f15dfb107a25f49e3dfa9be2d75e9425902ab832e17` matches capture metadata: SetDuration requires non-nil LuaDurationObject; SetFormatter requires non-nil NumericFormatter. Metadata typing is not native execution proof.

`tests/numeric_rule_formatter.rs:154-189` creates actual duration objects, binds explicit shared manual clock0, and sets duration start0. Executed assertions cover formatted2, FontString2, FontString9, copied formatter output8, custom-table rejection, and retained configured output8. The rejected table is tested transactionally through public behavior, not source-shape assertions.

`src/c_api/duration_text_binding.rs:280-310` authenticates and validates modern setter input before invoking original setter. Its NumericFormatter validation prevents the legacy function/table callback paths from becoming modern setter compatibility. Registration at339-353 passes profile/version flags. `docs/specs/numeric-rule-formatter.md` and `docs/specs/duration-text-binding.md` explicitly separate legacy callback reachability from modern typing and disclose native/older-profile gaps. No runtime patch or compatibility expansion was introduced by this fixture correction.

Changed Rust lines in HEAD (embedded Lua fixture) independently read against rust-readability checklist: **no violations found**. No new branches, opaque accumulated state, deep nesting, warning suppressions, duplication, or hidden side effects. No automated complexity command executed. Format receipt `/tmp/formatter-typed-final/format.json` reused byte-for-byte as `format-reused.json`: exact revision, exit0, UTC19:59:07.210262–19:59:17.355548, equal changed-file hashes. No format rerun.

## Separate aura RED

Observed stderr: `Dirty flags were not fully cleared during update pass (remaining flags: 18)`; native traceback includes MixinUtil.lua:433 ProcessDirtyFlags and Blizzard_ManagedAuraContainer.lua:91. Test panics at `tests/on_update_modes.rs:186:6`, unwrap of Lua assertion failed. Full stdout/stderr retained separately. This reproduces the failure, **not its cause**. Prior `/tmp/aura-phase-fixture-decision.md` attributes it to overwritten native phase registration; current fixture at164-168 does replace dirtyPhases with a synthetic flag1 handler. That separate diagnosis is not promoted to fresh causal proof; no remediation performed.

## Provenance and exclusions

`ledger.json` records all argv, UTC times, structured exits, stream bytes/hashes. Compile and execution receipts capture revision and pre/post per-file SHA256 maps for all files under src/, tests/, docs/specs/ in the same evaluation as each command: all equal. This source hash scope does not claim entire-repository/build-input equivalence. Lists have UTC/exit receipts; their cwd was canonical as shown in evaluation, not originally stored in list JSON. Streams remained separate; no combined_to_file use.

Original scalar RED and 4142 six-of-seven failure proof were not edited; new evidence lives only in this /tmp directory. No tracked edits, rebuild after initial compile, other tests/CVar/profile/check/full-suite/startup commands, network, delegation, or operational changes. No older-profile/Forever/native parity or broader completion claim.
