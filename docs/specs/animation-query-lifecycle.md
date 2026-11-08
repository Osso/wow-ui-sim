# Animation query lifecycle

Ordinary simulator query behavior for `SimpleAnim` and `SimpleAnimGroup`, exercised through real playback and ticks. The pinned PTR register adds `QueryAnimationProgress` access checks to sixteen query occurrences; these tests do not establish that security contract or native WoW equivalence. See [event dispatch](../event-system.md).

## What it must do

- [x] Animation `IsPlaying`, `IsPaused`, and `IsDone` follow their owning group through playback, pause/resume, restart, stop, and completion; an independent idle group stays unchanged.
- [x] Pausing freezes elapsed time. Restart and Stop reset elapsed/progress. Explicit Stop of a playing group dispatches its `OnStop` scripts after modeled state, flipbook, and action-bar state settle; callback cleanup and reentrant Play remain effective. Stop does not dispatch `OnFinished`. The modeled animation `IsStopped` is true whenever its owner is not playing, including pause.
- [x] `Finish` remains pending until a tick; completion callbacks observe settled state and fire once. Natural completion after restart also settles owner/child queries.
- [x] Reverse playback and repeating loops update elapsed/progress while `IsReverse` and `GetLoopState` identify direction and configured loop mode. Paused `Play(true/false)` changes direction without resetting elapsed; omitted direction preserves paused direction. Bounce callbacks observe direction changes at actual reached boundaries. The [current six-case checkpoint](#current-bounded-saved-execution--2026-10-13) covers these asserted simulator behaviors, not native geometry.
- [x] Delayed animations expose local active elapsed/progress independently of the group's total timeline; animation progress clamps during end delay. `GetSmoothProgress` currently equals unsmoothed progress even with `IN` smoothing.
- [x] `IsDelaying` identifies the animation's start-delay interval on its ordered timeline: inclusive at the order's start, exclusive at active playback's start. Earlier orders contribute their longest total duration, including end delay; parallel animations are not summed. Waiting for an earlier order and the queried animation's own end delay return false. Reverse playback queries the same timeline interval; no extra phase state is introduced. Frozen `ccd526c6c` fails paused explicit-reverse traversal; the corrected exact ordered-delay test passes at `c81f10375`.

These are simulator-model requirements, not native lifecycle or smoothing claims.

## How it works

- [Frame/script data flow](../frame-data-flow.md)
- [Event and update dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs`: shared FrameRef method registration; animation objects can call group-state query methods.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations.rs`: owner-state and local elapsed/progress queries.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/runtime.rs`: playback advancement and completion/loop callbacks.

## Tests asserting this spec

`tests/animation_query_lifecycle.rs`, auto-discovered into the existing `integration` Cargo target:

| Test | Queries/boundary |
|---|---|
| `owner_queries_follow_pause_resume_restart_and_stop` | Group/animation `IsPlaying`, `IsPaused`, `IsDone`; animation `IsStopped`, `GetElapsed`, `GetProgress`, `GetSmoothProgress`; group `GetElapsed`, `GetProgress`; paused ticks and independent owner |
| `pending_finish_queries_are_settled_before_callbacks` | Above state/progress queries plus group `IsPendingFinish`; explicit Finish and natural completion callback visibility |
| `reverse_and_repeat_queries_follow_playback_direction` | Group `IsReverse`, `GetLoopState`, elapsed/progress; child elapsed/progress across reverse and repeat |
| `bounce_loop_callback_observes_direction_change` | Loop/direction/state queries inside real `OnLoop` callbacks |
| `delayed_animation_queries_use_local_active_elapsed` | Animation `IsDelaying`, elapsed/progress/smooth progress; exact start-delay boundary and completion across start/end delays |
| `delaying_uses_order_start_and_longest_parallel_duration` | Earlier-order wait, parallel maximum including end delay, exact delay boundaries, own end delay, and reverse traversal |

Existing complementary coverage: `tests/animation_group_state.rs`, `tests/animation_group.rs` (explicit playing-group Stop callback state, visible cleanup, reentrant Play, error routing, and no OnFinished), and `tests/animation_anim.rs`.

The exact playing-Stop case is recorded in `/tmp/cross-version-animation-stop-proof.md`: targeted `gui,client-wrath` was RED 0/1 at `b8596d6dc`, then GREEN 1/1 at `af1b54f2b`. At `43025255d`, `animation_group_stop` obtains ordered group `OnStop` handlers, invokes each through `protected_lua_pcall_state`, and sends errors to `call_error_handler_state` after all modeled Stop state has settled. The error-path regression was RED 0/1 before correction; independent verification at `d6d7a9078` passes all 30 group, 13 group-state, and six lifecycle cases, including error routing and reentrant Play. Format/default-feature check pass; exact proof is `/tmp/cross-version-text-animation-proof.md`. No new Cargo target or API-publication absence assertions are needed.

At test commit `9bc06cfe2`, all five new tests and 59 existing tests passed under each of `client-ptr` and `client-retail`:

```text
cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- animation_anim:: animation_group:: animation_group_state:: animation_query_lifecycle:: --nocapture
```

Logs: `/tmp/animation-query-lifecycle-9bc06cfe2-{ptr,retail}.log`. Proof ledger: `/tmp/animation-query-lifecycle-ledger.json`.

Correction `23bdf9fe6` first reproduced two failures: the exact active-start boundary and an animation incorrectly delaying while an earlier order was still running. After correction, the same bounded suite above passed **65 tests per profile** (six lifecycle tests plus 59 existing animation tests). Logs: `/tmp/animation-delay-23bdf9fe6-{ptr,retail}.log`; RED: `/tmp/animation-delay-red.log`; ledger: `/tmp/animation-delay-proof-ledger.json`. No check, readability, artifact, or broad-suite gates were run.

## Current bounded saved execution — 2026-10-13

[Public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md) links the independent saved report: at `c81f10375`, exact `delaying_uses_order_start_and_longest_parallel_duration` **1 PASS / 0 FAIL, wait exit0** and exact `bounce_loop_callback_observes_direction_change` **1 PASS / 0 FAIL, wait exit0**. Ordered/parallel forward/reverse delay boundaries and paused omitted/explicit-false resume assertions are executed; bounce reaches two OnLoop callbacks with updates1.25 and1.0. These c81 two controls are historical and remain separate from the later six-case execution.

The [current six-case public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/animation-lifecycle-six-2026-10-13/README.md) owns exact sealed revision, execution counts, wait status, timing, diagnostics and retained-text gate; its [independent report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/animation-lifecycle-six-2026-10-13/independent/report.md) and [proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/animation-lifecycle-six-2026-10-13/independent/proof.json) establish the named cases passing together. Historical suites and frozen CCD remain unchanged. No current PTR, native, broad-suite, geometry, security, smoothing or parent acceptance is established.

## Known gaps (current cycle)

- [ ] Frozen `ccd526c6c` reproduces paused `Play(true)` retaining forward direction: after elapsed2.75 and delta1.25, observed3.0 versus expected1.5. Current producer distinguishes omitted direction from explicit true/false, keeps elapsed origin and commits requested direction; exact ordered-delay execution now passes. Added omitted/explicit-false assertions execute in that PASS, not frozen RED. Reverse/repeat now executes in the sealed a8 six-case PASS, independently inspected by3775. Check and full Blizzard startup remain pending for this scope; retention gate PENDING.
- [ ] Bounce fixture corrected second tick from0.5 to1.0: a one-second path at elapsed0.75/reverse after the first1.25 tick must cross zero to reach its second boundary. Frozen failure is the final post-tick assertion, not evidence of an OnLoop callback error or multi-boundary dispatch bug. No callback-dispatch redesign; corrected exact bounce runtime passes at `c81f10375`. All six lifecycle cases now pass together at sealed a8; independent3775 PASS. Retention gate and wider lifecycle acceptance remain pending.

- [x] Correct the reproduced double subtraction of start delay: group time `0.75` with start delay `0.5` now yields local elapsed/progress `0.25` and `IsDelaying() == false`. The getter derives time within the animation's order rather than reinterpreting clamped active elapsed.
- [x] The model root now commits playback/cache state, releases the simulation-state borrow, then dispatches existing resolved-group `OnPlay` handlers. This supports the unmodified Blizzard EncounterWarnings consumer; it does not change credit for these sixteen query rows or establish native callback/security semantics.
- [ ] Native owner-versus-child state, delay boundaries, smoothing, reverse/bounce geometry, and callback timing are not established by this model coverage. Unchanged Blizzard UI consumers source the explicit playing-group Stop callback, but no native execution probe establishes it; paused/inactive Stop, hidden-subtree Stop, child-animation `OnStop`, natural finish, Pause, and other callbacks remain unverified.
- [x] Explicit playing-group Stop reports handler errors without aborting its caller; the post-correction regression passes in the independent 49-case animation batch.

## Out of scope

- `SetParent` and `CreateAnimation` mutation contracts: unchanged by this query correction.
- Forbidden-aspect, protected, taint, and secret enforcement: ordinary lifecycle tests do not exercise those boundaries.
- Audit manifest/checklist changes, broad suites, rendering proof, and final verification gates: separate work.
