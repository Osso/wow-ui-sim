# Round 10 integration
Goal: only rows 137/145; behavioral RED/GREEN, narrow regressions, fmt/check, commit. Exclude rendering, blocked rows, wiki/coverage/vendor/cache changes, agents, push/merge/deploy/PR.
Step 1: clean p1207-r10 at 55eb92602a379496a6de9d4fa4d90215c97c2f7d (master identical).
Step 2: all E01/E02/E03 OLD anchors match exactly once; no master adaptation needed. No new state required. Row145 is cfg retail-12-1-5; default feature availability being checked. Tool grep/find unavailable; used bounded Python path/text inspection.
Step 3: copied N02/N03/N04 and applied E03; E01/E02/N01 withheld. Default client-retail enables retail-12-1-0, not retail-12-1-5: four row145 tests excluded by cfg; no alternate features authorized. Six GUID tests ready for RED.
Step 4: RED invocation completed; log /home/osso-test/.cache/wow-ui-sim-audit/r10-RED-p1207_remaining_model_unit_guid.log; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration p1207_remaining_model_unit_guid -- --test-threads=1; exact scope base 55eb92602 + N02/N03/N04/E03, producer withheld. Output inspection next.
RED inspected: p1207_remaining_model_unit_guid 0 passed / 6 failed (exit 101), compiled successfully with no warnings; missing getter causes nil calls and secret-denial message assertion failure. Behavioral RED confirmed, not compile errors. Proof is pre-producer only.
Step 5: formatted and committed RED tests/spec/fixture as 69325810f712de739124d5d2bb0aa50cbd9acd88. Producer still withheld.
Step 6: applied exact E01/E02/N01; getter rejects every secret before native actor validation and resolves existing last_unit against current host identity. No new state, rendering or timeline producer edits. Spec adaptation: replace obsolete author-only verification prohibition and pinned proposal intro with integration scope/default-feature limitation.
Step 7: producer formatted and committed as dffda5a2bbe8e9710d6f8cf12cddbd5646c55016 before GREEN verification. RED evidence remains valid only for withheld-producer scope; GREEN required for new revision.
Step 8: GREEN module invocation at dffda5a2bbe8e9710d6f8cf12cddbd5646c55016 completed exit 0; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-p1207_remaining_model_unit_guid.log; same RED command with producer enabled. Read saved output next.
GREEN output inspected: p1207_remaining_model_unit_guid 6 passed / 0 failed at dffda5a2; no warnings. Required default-Retail behavioral scope proven. Narrow intersecting regressions outstanding; no source changes since producer commit.
Step 9 regression: dffda5a2; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration p1207_remaining_timeline_color_event -- --test-threads=1; exit 0; test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10454 filtered out; finished in 0.00s; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-p1207_remaining_timeline_color_event.log. No later source changes.
Step 9 regression: dffda5a2; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration cast_events_identity::model_identity:: -- --test-threads=1; exit 0; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10448 filtered out; finished in 0.73s; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-cast_events_identity-model_identity.log. No later source changes.
Step 9 regression: dffda5a2; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration model_set_unit_identity_followups:: -- --test-threads=1; exit 0; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10448 filtered out; finished in 0.55s; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-model_set_unit_identity_followups.log. No later source changes.
Step 9 regression: dffda5a2; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration encounter_events:: -- --test-threads=1; exit 0; test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10450 filtered out; finished in 0.37s; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-encounter_events.log. No later source changes.
Step 10: updated startup fixture at dffda5a2; command python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --lib test_patch_12_0_7_widget_compatibility_surface -- --test-threads=1; exit 0; saved /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-test_patch_12_0_7_widget_compatibility_surface.log; output inspection next.
Step 11: saved regression logs read and confirmed: model identity 6/0, model unit followups 6/0, legacy encounter events 4/0, updated startup fixture 1/0; new later timeline module 0/0 (cfg-disabled, NOT proof). All at dffda5a2; no warnings. Manual Rust readability audit of changed producer/registration/tests/fixture: no new suppression, deep nesting, opaque protocol or duplicated state. Existing permanent 3D gap preserved.
Step 12: spec now marks only seven tested GUID behavior bullets complete; feature/older-profile bullet remains unverified, all timeline bullets remain unverified. Four timeline tests authored but cfg-disabled. No source changes, so existing dffda5a2 proof remains valid.
Step 13: proof annotations committed as 4d6cc14762b157cc91303cbe0919eaf60bca2d9c; documentation-only delta, behavioral proof not invalidated.
Step 14 fmt: command CARGO_BUILD_JOBS=4 cargo fmt; explicit cwd /home/osso-test/.worktrees/wow-ui-sim-p1207-r10; revision 4d6cc14762b157cc91303cbe0919eaf60bca2d9c; exit 0; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-fmt.log; output inspection pending.
Step 14 fmt-check: command CARGO_BUILD_JOBS=4 cargo fmt --check; explicit cwd /home/osso-test/.worktrees/wow-ui-sim-p1207-r10; revision 4d6cc14762b157cc91303cbe0919eaf60bca2d9c; exit 0; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-fmt-check.log; output inspection pending.
Step 14 cargo-check: command CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --jobs 4; explicit cwd /home/osso-test/.worktrees/wow-ui-sim-p1207-r10; revision 4d6cc14762b157cc91303cbe0919eaf60bca2d9c; exit 0; log /home/osso-test/.cache/wow-ui-sim-audit/r10-GREEN-cargo-check.log; output inspection pending.

Step 15: final saved fmt/check logs inspected: cargo fmt and cargo fmt --check exit 0; cargo check exit 0 in 48.97s, no warnings. Final diff --check exit 0; clean p1207-r10. Only six authorized tracked files changed. No source changes after dffda5a2; documentation-only final commit does not invalidate behavioral proof. No undefined-symbol failures; no cargo clean needed.

## Final result — 2026-10-04

Worktree: `/home/osso-test/.worktrees/wow-ui-sim-p1207-r10`, branch `p1207-r10`.
Base: `55eb92602a379496a6de9d4fa4d90215c97c2f7d`.
Commits, in order:
- `69325810f712de739124d5d2bb0aa50cbd9acd88` — tests/spec/fixture; producer withheld.
- `dffda5a2bbe8e9710d6f8cf12cddbd5646c55016` — gated live host-state GUID producer.
- `4d6cc14762b157cc91303cbe0919eaf60bca2d9c` — bounded proof annotations.
All messages end with requested Claude Opus 5.5 co-author line. No push, merge, deploy or PR.

### Changed files
- `docs/specs/retail-12-0-7-remaining-audit.md`
- `src/loader/tests/wow_api_globals/startup_globals.rs`
- `src/lua_api/frame/methods/widgets/model.rs`
- `src/lua_api/frame/methods/widgets/model/model_unit_guid.rs`
- `tests/p1207_remaining_model_unit_guid.rs`
- `tests/p1207_remaining_timeline_color_event.rs`

### Behavioral proof ledger
All commands used explicit worktree cwd, CARGO_BUILD_JOBS=4, local build host, required BUILD_HOST_SCRIPTS, exclusive b100 target, one filter per invocation and no concurrent builds. Exact commands and saved log paths appear above. Runner prints a generic worktree/target banner, but executable paths in logs confirm b100 was used.

| Filter / target | RED pass/fail | GREEN pass/fail | Scope |
|---|---|---|---|
| p1207_remaining_model_unit_guid / integration | 0/6 | 6/0 | Six public API behaviors, producer absent then present |
| p1207_remaining_timeline_color_event / integration | Not run | 0/0 | Four tests cfg-disabled; ZERO lifecycle proof |
| cast_events_identity::model_identity:: / integration | Not run | 6/0 | Existing identity denial, binding and isolation |
| model_set_unit_identity_followups:: / integration | Not run | 6/0 | Existing model-unit binding contracts |
| encounter_events:: / integration | Not run | 4/0 | Existing legacy encounter catalog/color/sound behavior, not timeline lifecycle |
| test_patch_12_0_7_widget_compatibility_surface / lib | Not run | 1/0 | Updated default getter return/arity fixture |

RED scope: base plus N02/N03/N04/E03, before producer; successful compilation and six behavioral failures. GREEN scope: dffda5a2; final HEAD changes only spec annotations. Total executed GREEN: 23 passed, 0 failed. Ten authored new tests; six executed, four excluded. Default-feature compiler proof does NOT type-check the cfg-disabled timeline test bodies.
Final format/compiler proof: 4d6cc1476, default Retail only; no warnings. No unfiltered suites, alternate features or startup CLI run.

### Per-row coverage and recommended status

| Row | Proven scope | Unproved scope | Recommended status |
|---|---|---|---|
| widgets-ModelSceneActorBase-GetModelUnitGUID-137 | Genuine native actor; live Rust last_unit/host GUID lookup; target/focus mutation and missing identity; actor/environment isolation; one public empty default result; public output despite restricted UnitGUID for secure/tainted callers; forged receiver rejection; secret receiver/all extras rejected before validation for both callers, without taint changes | Native defaults, token-vs-snapshot parity and input policy remain INFERRED in spec/producer. ClearModel and lifecycle parity, older-feature execution, strict 12.0.7-only execution, broader unit taxonomy and rendering are not proven | bounded-coverage — only tested default-Retail non-rendering getter contract; not full native-row conformance |
| events-ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED-145 | Four proving tests authored against existing retail-12-1-5 producer; legacy encounter regressions pass, but supply no timeline proof | All four new lifecycle tests unrun because default client-retail enables only retail-12-1-0. Strict-12.0.7 has event admission but no notification lifecycle. Threshold, payload, cancellation, environment/taint checks, UniqueEvent coalescing, color-setter/alpha behavior and native historical parity remain unproved | audit-pending — retain later-epoch and strict-epoch gap; no promotion |

### Adaptations and exclusions
- E01/E02/E03 OLD anchors each matched exactly once at supplied base; no master-move adaptations. rustfmt reordered module declarations without changing the feature gate.
- Spec updated obsolete authoring-only intro/verification prohibition and marked only backed GUID behaviors passing. Older-feature gate requirement remains unchecked; all timeline requirements remain unchecked.
- Existing explicit PlayerModelState.last_unit and host target/focus identity reused; no new host state required. No setter behavior or timeline producer changed.
- Sixteen blocked rows unchanged: 111–118, 048, 158–162, prose013/prose014. No factories, race DTOs, club identity model, input simulation or debugger history invented. Wiki and coverage JSON untouched.
- Wowless, WowlessData, Interface/BlizzardUI and cached Blizzard Lua untouched. No rendering, shims, fallback providers, source-substring behavioral assertions, agents/models or operational actions.

### Merge risk
Small default-Retail mechanical change; all 23 executed targeted tests and compiler/format checks pass. Public GUID exposure is intentional per ConditionalSecret removal, including identities otherwise restricted by UnitGUID. Semantic risk remains in explicitly INFERRED empty default, live-token identity and all-caller secret-input policy; native parity is not established. Four timeline tests are inactive and not type-checked/executed by this proof; merging does not close row145 or add strict-12.0.7 lifecycle support. No broad-suite, startup or alternate-profile claims. Worktree clean and retained; integration task finished within authorized verification limits.
