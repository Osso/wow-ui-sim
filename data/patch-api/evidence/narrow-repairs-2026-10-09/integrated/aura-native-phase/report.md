# Independent bounded native-phase simulator proof

Canonical `/home/osso/Projects/wow/wow-ui-sim`, revision `dcc101d85aece04cf66804c2059b3a7d3da0bfbd` throughout. **PASS within assigned simulator boundary. No native-client or parent closure.**

## Fresh execution

| Check | Actual result | Receipt |
|---|---|---|
| cargo fmt --check, once | exit 0; empty stdout/stderr; 12.563s | format.json / format.stdout / format.stderr |
| One locked compile | exit 0; 243.125s including lock wait | compile.json / compile.stdout / compile.stderr |
| on_update_modes:: listing, once | exit 0; 5 tests, 0 benchmarks; changed case exactly 1 | module-list.* / case-counts.json |
| One module batch bounded by timeout 90 | exit 0; 5 passed, 0 failed, 0 ignored, 0 measured, 10693 filtered; harness 0.27s, command 0.286s | module-batch.* |

Compile argv exactly `["/home/osso/.worktrees/build-lock.sh","cargo","test","--offline","--locked","--jobs","4","--test","integration","--no-run","--message-format=json"]`. No compile timeout. UTC 2026-10-09T20:13:51.042238 through 20:17:54.167049. Shared wrapper stderr: waiting15:13, acquired15:17; Cargo finished in49.11s. OS lock autoreleased on wrapper exit; parent notified immediately after completed batch. No bypass, parallel heavy build, rerun, or background job.

All738 Cargo JSON records read/parsed: 663 compiler-artifact,74 build-script-executed,1 build-finished(success:true); zero compiler-message records. Separate full streams retained. Full stderr reviewed: six iced-wgpu-patched/Cargo.toml manifest deprecations: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. Each diagnostic recommends underscore spelling and warns about future-edition behavior. No suppression or remediation.

New Cargo-emitted integration artifact, fresh:false: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`, SHA256 `6d93fda8fc392c78201bffeafe8d8ec309983802df58f51187ac8661e15110ce`. Metadata/profile/features retained in artifact.json, not inferred from reused filename. Features: aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound. Artifact SHA256 unchanged after batch.

Exact fresh batch stdout:

```text
running 5 tests
test on_update_modes::on_update_modes_reset_before_handlers_and_preserve_rearming ... ok
test on_update_modes::on_update_modes_obey_ancestor_visibility_and_one_shots ... ok
test on_update_modes::on_update_modes_publish_numeric_contract_and_default ... ok
test on_update_modes::on_update_modes_xml_names_select_numeric_modes ... ok
test on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10693 filtered out; finished in 0.27s
```

Full batch stderr3194bytes reviewed: simulator startup timings only, retained untruncated in module-batch.stderr.

## Independent spec and manual readability audit

Read verify and rust-readability skills, corrected tests/on_update_modes.rs, both requested specs, /tmp/aura-phase-fixture-decision.md, and historical integrated/formatter-typed-final/report.md. Read actual cached producer/registration and template/DirtyPhaseMixin boundary.

| Requirement | Evidence and proof level |
|---|---|
| Preserve actual six phases | Cached Blizzard_ManagedAuraContainer.lua:63–74 initializes and registers ParseAuras, ResetAuraFrames, RefreshAuraFrames, RefreshAuraFrameDisplay, RebuildLayoutGroups, ApplyLayout. Corrected fixture never replaces phases/callbacks; static audit plus fresh real-template execution. |
| Real public producer while hidden | tests/on_update_modes.rs:163–167 clears initial flags, hides, calls public UpdateAllAuras(), asserts mode2/dirty. Cached producer:45–56 marks FullAuraRebuild and handles item-enchantment work; not magic MarkDirty(1). Fresh execution PASS. |
| Hidden tick retains armed/dirty state | fixture:170–177 dispatches hidden tick, then explicitly asserts mode2 and IsDirty before Show. Fresh execution PASS. |
| Visible tick clears work/disables | fixture:179–187 dispatches then asserts clean/mode0. Native OnUpdate:90–92 calls inherited ProcessDirtyFlags; OnDirtyChanged:94–98 arms visible-once. CustomAuraContainer.xml:4–13 binds native private mixin and OnUpdate. MixinUtil.lua:413–435 consumes phases and reports residual flags. Fresh execution PASS. |
| Next tick stays clean/disabled; LuaErrorsEmpty | fixture:188–202 ticks again, asserts clean/mode0, then asserts lua_errors.is_empty() with diagnostic payload. Fresh execution PASS. |

Manual changed-Rust readability audit: no violations found. Test is linear setup → hidden dispatch → visible dispatch → following dispatch, with explicit public mutations and observable state assertions. No synthetic counter, phase replacement, new warning suppression, deep nesting, opaque conditionals, parameter overload, or new state accumulation. Numeric modes0/2 are literal specified enum contracts; 0.016 is the existing tick interval. No automated complexity command or code changes.

Boundary limitation: this fixture has no explicitly configured aura-group/frame output; fresh proof establishes native registered dirty-chain consumption and scheduling for this actual initialized template, not visual aura mutation or every native phase being exercised with nonempty aura data. Six phases are retained; the proof does not count callbacks.

## Historical evidence remains separate

Historical revision941140fd6fa52133966a8f28a022f49d3538df50 report records exact aura RED0/1, residual18, and formatter7/7. Those failed-fixture counts are not fresh causal proof. This report credits only corrected execution above. Decision memo's causal interpretation remains source-grounded prior diagnosis, not native parity or reproduction of a controlled causal experiment.

Previous formatter7/7 remains valid on unchanged formatter/runtime paths. Git comparison of941→dcc for src/tests/Cargo/build inputs lists only tests/on_update_modes.rs (prior-formatter-scope.*). No formatter execution repeated. Changed Rust invalidated prior fmt proof; fresh fmt ran once as requested.

## Immutability and receipts

Each command's JSON receipt persisted separate stdout/stderr, exit, UTC start/end, elapsed seconds, revision, per-file source/cache SHA256 maps and scoped hashes in the SAME evaluation as execution; no reliance on persistent Pyrun locals. All before/after maps equal. Compile/list/batch cover src/, tests/, docs/specs/, active retail vendor AddOns cache, UI manifests, root Cargo.toml/Cargo.lock/build.rs and .cargo files. Fmt covers src/tests/specs/cache. This is bounded scope, not whole-repository or every external build-dependency equivalence.

Scoped hashes after batch:

```text
source       9d0f7b03486f6c771e64fcee64a236ba278dd9b2067e9a319af297842aa117c2
tests        8c6ee02af2ac11eac173f8a5aacb94c5f61f07fea8543ba754d46987dd733267
specs        863e2a0c57a27d62c7123cf35541ce9d762795e229658af6ad8f7c0584de75ad
vendor-cache 5e1c88a89bae2b66412222611646a1d6716789468bce2d93471ed9b2a469e723
ui-manifests 04fbe8868e6dd53a0a4d55c3931dcbd64db36068a28eff3ff3a441e9dc81cd8d
build-inputs 18e0bfadec36be8c3bfd8329bdc8b451d608a4a17751080c903ec34f67058566
```

Git status before and after: only pre-existing untracked `.code-index.db`. No tracked edits, vendor/runtime writes, delegation, network, operations, formatter/CVar/profile/full-suite/check runs, native claims, or parent closure. New proof artifacts only in /tmp/aura-native-phase-independent/.
