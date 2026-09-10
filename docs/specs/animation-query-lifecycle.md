# Animation query lifecycle

Ordinary simulator query behavior for `SimpleAnim` and `SimpleAnimGroup`, exercised through real playback and ticks. The pinned PTR register adds `QueryAnimationProgress` access checks to sixteen query occurrences; these tests do not establish that security contract or native WoW equivalence. See [event dispatch](../event-system.md).

## What it must do

- [x] Animation `IsPlaying`, `IsPaused`, and `IsDone` follow their owning group through playback, pause/resume, restart, stop, and completion; an independent idle group stays unchanged.
- [x] Pausing freezes elapsed time. Restart and Stop reset elapsed/progress. The modeled animation `IsStopped` is true whenever its owner is not playing, including pause.
- [x] `Finish` remains pending until a tick; completion callbacks observe settled state and fire once. Natural completion after restart also settles owner/child queries.
- [x] Reverse playback and repeating loops update elapsed/progress while `IsReverse` and `GetLoopState` identify direction and configured loop mode. Bounce callbacks observe direction changes.
- [x] Delayed animations expose local active elapsed/progress independently of the group's total timeline; animation progress clamps during end delay. `GetSmoothProgress` currently equals unsmoothed progress even with `IN` smoothing.
- [x] `IsDelaying` identifies the animation's start-delay interval on its ordered timeline: inclusive at the order's start, exclusive at active playback's start. Earlier orders contribute their longest total duration, including end delay; parallel animations are not summed. Waiting for an earlier order and the queried animation's own end delay return false. Reverse playback queries the same timeline interval; no extra phase state is introduced.

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

Existing complementary coverage: `tests/animation_group_state.rs`, `tests/animation_group.rs`, and `tests/animation_anim.rs`. No new Cargo target or API-publication absence assertions are needed.

At test commit `9bc06cfe2`, all five new tests and 59 existing tests passed under each of `client-ptr` and `client-retail`:

```text
cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- animation_anim:: animation_group:: animation_group_state:: animation_query_lifecycle:: --nocapture
```

Logs: `/tmp/animation-query-lifecycle-9bc06cfe2-{ptr,retail}.log`. Proof ledger: `/tmp/animation-query-lifecycle-ledger.json`.

Correction `23bdf9fe6` first reproduced two failures: the exact active-start boundary and an animation incorrectly delaying while an earlier order was still running. After correction, the same bounded suite above passed **65 tests per profile** (six lifecycle tests plus 59 existing animation tests). Logs: `/tmp/animation-delay-23bdf9fe6-{ptr,retail}.log`; RED: `/tmp/animation-delay-red.log`; ledger: `/tmp/animation-delay-proof-ledger.json`. No check, readability, artifact, or broad-suite gates were run.

## Known gaps (current cycle)

- [x] Correct the reproduced double subtraction of start delay: group time `0.75` with start delay `0.5` now yields local elapsed/progress `0.25` and `IsDelaying() == false`. The getter derives time within the animation's order rather than reinterpreting clamped active elapsed.
- [ ] Native owner-versus-child state, delay boundaries, smoothing, reverse/bounce geometry, and callback timing are not established by this model coverage.

## Out of scope

- `SetParent` and `CreateAnimation` mutation contracts: unchanged by this query correction.
- Forbidden-aspect, protected, taint, and secret enforcement: ordinary lifecycle tests do not exercise those boundaries.
- Audit manifest/checklist changes, broad suites, rendering proof, and final verification gates: separate work.
